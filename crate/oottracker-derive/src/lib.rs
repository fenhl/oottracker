#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        convert::TryFrom,
        fs::{
            self,
            File,
        },
        io::prelude::*,
        path::Path,
    },
    convert_case::{
        Case,
        Casing as _,
    },
    itertools::Itertools as _,
    proc_macro::TokenStream,
    proc_macro2::{
        Literal,
        Span,
    },
    quote::quote,
    syn::{
        Expr,
        Ident,
        Index,
        LitInt,
        LitStr,
        Token,
        Visibility,
        braced,
        bracketed,
        parse::{
            Parse,
            ParseStream,
            Result,
        },
        parse_macro_input,
        punctuated::Punctuated,
    },
};

#[proc_macro]
pub fn version(_: TokenStream) -> TokenStream {
    let version = env!("CARGO_PKG_VERSION");
    TokenStream::from(quote! {
        ::semver::Version::parse(#version).expect("failed to parse current version")
    })
}

#[proc_macro]
pub fn embed_image(input: TokenStream) -> TokenStream {
    let img_path = parse_macro_input!(input as LitStr).value();
    let img_path = Path::new(&img_path);
    let name = Ident::new(&img_path.file_name().expect("empty filename").to_string_lossy().split('.').next().expect("empty filename").to_case(Case::Snake), Span::call_site());
    let path_lit = img_path.to_str().expect("filename is not valid UTF-8");
    let mut buf = Vec::default();
    File::open(img_path).expect("failed to open image to embed").read_to_end(&mut buf).expect("failed to read image to embed");
    let contents_lit = Literal::byte_string(&buf);
    TokenStream::from(quote! {
        pub fn #name<T: FromEmbeddedImage>() -> T {
            T::from_embedded_image(::std::path::Path::new(#path_lit), #contents_lit)
        }
    })
}

#[proc_macro]
pub fn embed_images(input: TokenStream) -> TokenStream {
    let dir_path = parse_macro_input!(input as LitStr).value();
    let dir_path = Path::new(&dir_path);
    let name = Ident::new(&dir_path.file_name().expect("empty filename").to_string_lossy().to_case(Case::Snake), Span::call_site());
    let name_all = Ident::new(&format!("{}_all", name), Span::call_site());
    let path_lit = dir_path.to_str().expect("filename is not valid UTF-8");
    let img_consts = fs::read_dir(dir_path).expect("failed to open images dir") //TODO compile error instead of panic
        .map(|img_path| img_path.and_then(|img_path| Ok({
            let name = img_path.file_name();
            let name = name.to_string_lossy();
            let name = name.split('.').next().expect("empty filename");
            let mut buf = Vec::default();
            File::open(img_path.path())?.read_to_end(&mut buf)?;
            let lit = Literal::byte_string(&buf);
            quote!(consts.insert(#name, #lit);)
        })))
        .try_collect::<_, Vec<_>, _>().expect("failed to read images"); //TODO compile error instead of panic
    TokenStream::from(quote! {
        pub fn #name<T: FromEmbeddedImage>(name: &str, ext: &str) -> T {
            ::lazy_static::lazy_static! {
                static ref IMG_CONSTS: ::std::collections::HashMap<&'static str, &'static [u8]> = {
                    let mut consts = ::std::collections::HashMap::<&'static str, &'static [u8]>::default();
                    #(#img_consts)*
                    consts
                };
            }

            T::from_embedded_image(&::std::path::Path::new(#path_lit).join(format!("{}.{}", name, ext)), IMG_CONSTS[name])
        }

        pub fn #name_all<T: FromEmbeddedImage>(ext: &'static str) -> impl Iterator<Item = T> {
            ::lazy_static::lazy_static! {
                static ref IMG_CONSTS: ::std::collections::HashMap<&'static str, &'static [u8]> = {
                    let mut consts = ::std::collections::HashMap::<&'static str, &'static [u8]>::default();
                    #(#img_consts)*
                    consts
                };
            }

            IMG_CONSTS.iter().map(move |(name, contents)| T::from_embedded_image(&::std::path::Path::new(#path_lit).join(format!("{}.{}", name, ext)), contents))
        }
    })
}

enum FlagName {
    Event(LitStr),
    Ident(Ident),
    Lit(LitStr),
    Entrance(LitStr, LitStr),
    Prereq(LitInt, Box<FlagName>),
}

impl FlagName {
    fn to_ident(&self) -> Ident {
        match self {
            FlagName::Event(lit) | FlagName::Lit(lit) => Ident::new(&lit.value().replace('&', "AND").to_case(Case::ScreamingSnake), lit.span()),
            FlagName::Ident(ident) => ident.clone(),
            FlagName::Entrance(from, to) => Ident::new(&format!("ENTRANCE_{}_TO_{}", from.value().to_case(Case::ScreamingSnake), to.value().to_case(Case::ScreamingSnake)), to.span()),
            FlagName::Prereq(id, at_check) => Ident::new(&format!("REQ_{}_FOR_{}", id, at_check.to_ident()), id.span()),
        }
    }
}

impl Parse for FlagName {
    fn parse(input: ParseStream<'_>) -> Result<FlagName> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let ident = input.parse::<Ident>()?;
            if ident.to_string() == "event" {
                input.parse().map(FlagName::Event)
            } else {
                Ok(FlagName::Ident(ident))
            }
        } else if lookahead.peek(LitStr) {
            let lit = input.parse()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![->]) {
                input.parse::<Token![->]>()?;
                let to = input.parse()?;
                Ok(FlagName::Entrance(lit, to))
            } else if lookahead.peek(Token![=]) {
                Ok(FlagName::Lit(lit))
            } else {
                Err(lookahead.error())
            }
        } else if lookahead.peek(LitInt) {
            let lit = input.parse()?;
            input.parse::<Token![for]>()?;
            Ok(FlagName::Prereq(lit, Box::new(input.parse()?)))
        } else {
            Err(lookahead.error())
        }
    }
}

struct Flag {
    name: FlagName,
    value: LitInt,
}

impl Parse for Flag {
    fn parse(input: ParseStream<'_>) -> Result<Flag> {
        let name = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Flag { name, value })
    }
}

struct Flags {
    idx: LitInt,
    fields: Punctuated<Flag, Token![,]>,
}

impl Parse for Flags {
    fn parse(input: ParseStream<'_>) -> Result<Flags> {
        let idx = input.parse()?;
        input.parse::<Token![:]>()?;
        let content;
        braced!(content in input);
        let fields = content.parse_terminated(Flag::parse)?;
        Ok(Flags { idx, fields })
    }
}

struct FlagsList {
    vis: Visibility,
    struct_token: Token![struct],
    name: Ident,
    field_ty: Ident,
    num_fields: LitInt,
    fields: Punctuated<Flags, Token![,]>,
}

impl Parse for FlagsList {
    fn parse(input: ParseStream<'_>) -> Result<FlagsList> {
        let vis = input.parse()?;
        let struct_token = input.parse()?;
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let content;
        bracketed!(content in input);
        let field_ty = content.parse()?;
        content.parse::<Token![;]>()?;
        let num_fields = content.parse()?;
        let content;
        braced!(content in input);
        let fields = content.parse_terminated(Flags::parse)?;
        Ok(FlagsList { vis, struct_token, name, field_ty, num_fields, fields })
    }
}

#[proc_macro]
pub fn flags_list(input: TokenStream) -> TokenStream {
    let FlagsList { vis, struct_token, name, field_ty, num_fields, fields } = parse_macro_input!(input as FlagsList);
    let field_ty_size = match &field_ty.to_string()[..] {
        "i8" | "u8" => 1,
        "i16" | "u16" => 2,
        "i32" | "u32" => 4,
        "i64" | "u64" => 8,
        _ => return quote!(compile_error!("unsupported field type: {}", field_ty)).into(),
    };
    let num_fields = match num_fields.base10_parse() {
        Ok(n) => n,
        Err(e) => return e.to_compile_error().into(),
    };
    let mut all_fields = (0..num_fields).map(|_| None).collect_vec();
    for Flags { idx, fields } in fields {
        let idx = match idx.base10_parse::<usize>() {
            Ok(n) => n,
            Err(e) => return e.to_compile_error().into(),
        };
        all_fields[idx] = Some(fields);
    }
    let fields_tys = (0..num_fields).map(|i|
        Ident::new(&format!("{}{}", name, i), Span::call_site())
    ).collect_vec();
    let contents = all_fields.iter().zip(&fields_tys).map(|(fields, fields_ty)| {
        if fields.is_some() { quote!(#vis #fields_ty) } else { quote!(#fields_ty) }
    }).collect_vec();
    let tup_idxs = (0..num_fields).map(Index::from).collect_vec();
    let mut entrance_prereqs = Vec::default();
    let mut event_checks = Vec::default();
    let mut location_checks = Vec::default();
    for ((fields, idx), fields_ty) in all_fields.iter().zip(&tup_idxs).zip(&fields_tys) {
        if let Some(fields) = fields {
            for Flag { name, .. } in fields {
                let name_ident = name.to_ident();
                match name {
                    FlagName::Event(event_name_lit) => {
                        event_checks.push(quote!(#event_name_lit => Some(self.#idx.contains(#fields_ty::#name_ident))));
                    }
                    FlagName::Ident(_) => {} // internal use only, don't auto-generate check logic
                    FlagName::Lit(name_lit) => {
                        location_checks.push(quote!(#name_lit => Some(self.#idx.contains(#fields_ty::#name_ident))));
                    }
                    FlagName::Entrance(_, _) => unreachable!("entrance checks aren't saved in RAM"), //TODO replace with compile error
                    FlagName::Prereq(id, at_check) => match &**at_check {
                        FlagName::Entrance(from, to) => entrance_prereqs.push(quote!((#id, (#from, #to)) => Some(self.#idx.contains(#fields_ty::#name_ident)))),
                        _ => unimplemented!("prereqs for non-entrance checks"),
                    },
                }
            }
        }
    }
    let start_idxs = (0..num_fields).map(|i| i * field_ty_size);
    let end_idxs = (1..=num_fields).map(|i| i * field_ty_size);
    let decls = all_fields.iter().zip(&fields_tys).map(|(fields, fields_ty)|
        if let Some(fields) = fields {
            let fields = fields.iter().map(|Flag { name, value }| {
                let name_ident = name.to_ident();
                quote!(const #name_ident = #value;)
            });
            let read_field = if matches!(&field_ty.to_string()[..], "u8" | "i8") {
                quote!(raw_data[0] as #field_ty)
            } else {
                let read_field_ty = Ident::new(&format!("read_{}", field_ty), Span::call_site());
                quote!(<::byteorder::BigEndian as ::byteorder::ByteOrder>::#read_field_ty(&raw_data))
            };
            quote! {
                ::bitflags::bitflags! {
                    #[derive(Default)]
                    #vis struct #fields_ty: #field_ty {
                        #(#fields)*
                    }
                }

                impl<'a> ::std::convert::TryFrom<&'a [u8]> for #fields_ty {
                    type Error = ();

                    fn try_from(raw_data: &[u8]) -> Result<#fields_ty, ()> {
                        if raw_data.len() != #field_ty_size { return Err(()) }
                        Ok(#fields_ty::from_bits_truncate(#read_field))
                    }
                }

                impl From<#fields_ty> for Vec<u8> {
                    fn from(value: #fields_ty) -> Vec<u8> {
                        value.bits().to_be_bytes().into()
                    }
                }
            }
        } else {
            quote! {
                #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
                struct #fields_ty;

                impl<'a> ::std::convert::TryFrom<&'a [u8]> for #fields_ty {
                    type Error = ();

                    fn try_from(raw_data: &[u8]) -> Result<#fields_ty, ()> {
                        if raw_data.len() != #field_ty_size { return Err(()) }
                        Ok(#fields_ty)
                    }
                }

                impl From<#fields_ty> for Vec<u8> {
                    fn from(_: #fields_ty) -> Vec<u8> {
                        vec![0; #field_ty_size]
                    }
                }
            }
        }
    ).collect_vec();
    TokenStream::from(quote! {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        #vis #struct_token #name(#(#contents,)*);

        impl #name {
            pub(crate) fn checked(&self, check: &ootr::check::Check) -> Option<bool> {
                match check {
                    ootr::check::Check::AnonymousEvent(at_check, id) => match &**at_check {
                        ootr::check::Check::Exit { from, to, .. } => match (id, (&**from, &**to)) {
                            #(#entrance_prereqs,)*
                            _ => None,
                        },
                        _ => None,
                    },
                    ootr::check::Check::Event(event) => match &event[..] {
                        #(#event_checks,)*
                        _ => None,
                    }
                    ootr::check::Check::Location(loc) => match &loc[..] {
                        #(#location_checks,)*
                        _ => None,
                    },
                    _ => None,
                }
            }
        }

        impl ::std::convert::TryFrom<Vec<u8>> for #name {
            type Error = Vec<u8>;

            fn try_from(raw_data: Vec<u8>) -> Result<#name, Vec<u8>> {
                if raw_data.len() != #num_fields * #field_ty_size { return Err(raw_data) }
                Ok(#name(
                    #(#fields_tys::try_from(&raw_data[#start_idxs..#end_idxs]).map_err(|()| raw_data.clone())?,)*
                ))
            }
        }

        impl<'a> From<&'a #name> for Vec<u8> {
            fn from(value: &#name) -> Vec<u8> {
                ::std::iter::empty()
                    #(.chain(Vec::from(value.#tup_idxs)))*
                    .collect()
            }
        }

        #(#decls)*
    })
}

enum SceneName {
    Ident(Ident),
    Lit(LitStr),
}

impl SceneName {
    fn to_field(&self) -> Ident {
        match self {
            SceneName::Ident(ident) => Ident::new(&ident.to_string().to_case(Case::Snake), ident.span()),
            SceneName::Lit(lit) => Ident::new(&lit.value().to_case(Case::Snake), lit.span()),
        }
    }

    fn to_lit(&self) -> LitStr {
        match self {
            SceneName::Ident(ident) => LitStr::new(&ident.to_string(), ident.span()),
            SceneName::Lit(lit) => lit.clone(),
        }
    }

    fn to_type(&self) -> Ident {
        match self {
            SceneName::Ident(ident) => ident.clone(),
            SceneName::Lit(lit) => Ident::new(&lit.value().to_case(Case::Pascal), lit.span()),
        }
    }
}

impl Parse for SceneName {
    fn parse(input: ParseStream<'_>) -> Result<SceneName> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            input.parse().map(SceneName::Ident)
        } else if lookahead.peek(LitStr) {
            input.parse().map(SceneName::Lit)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(PartialEq, Eq)]
enum SceneFieldsKind {
    Chests,
    Switches,
    RoomClear,
    Collectible,
    Unused,
    VisitedRooms,
    VisitedFloors,
}

impl SceneFieldsKind {
    fn start_idx(&self) -> usize {
        match self {
            SceneFieldsKind::Chests => 0x00,
            SceneFieldsKind::Switches => 0x04,
            SceneFieldsKind::RoomClear => 0x08,
            SceneFieldsKind::Collectible => 0x0c,
            SceneFieldsKind::Unused => 0x10,
            SceneFieldsKind::VisitedRooms => 0x14,
            SceneFieldsKind::VisitedFloors => 0x18,
        }
    }

    fn end_idx(&self) -> usize { self.start_idx() + 4 }

    fn ty(&self, scene_name: &SceneName) -> Ident {
        Ident::new(&format!("{}{}", scene_name.to_type(), match self {
            SceneFieldsKind::Chests => "Chests",
            SceneFieldsKind::Switches => "Switches",
            SceneFieldsKind::RoomClear => "RoomClear",
            SceneFieldsKind::Collectible => "Collectible",
            SceneFieldsKind::Unused => "Unused",
            SceneFieldsKind::VisitedRooms => "VisitedRooms",
            SceneFieldsKind::VisitedFloors => "VisitedFloors",
        }), Span::call_site())
    }
}

impl TryFrom<Ident> for SceneFieldsKind {
    type Error = syn::Error;

    fn try_from(ident: Ident) -> Result<SceneFieldsKind> {
        match &ident.to_string()[..] {
            "chests" => Ok(SceneFieldsKind::Chests),
            "switches" => Ok(SceneFieldsKind::Switches),
            "room_clear" => Ok(SceneFieldsKind::RoomClear),
            "collectible" => Ok(SceneFieldsKind::Collectible),
            "unused" => Ok(SceneFieldsKind::Unused),
            "visited_rooms" => Ok(SceneFieldsKind::VisitedRooms),
            "visited_floors" => Ok(SceneFieldsKind::VisitedFloors),
            _ => Err(syn::Error::new(ident.span(), "expected `chests`, `switches`, `room_clear`, `collectible`, `unused`, `visited_rooms`, or `visited_floors`")),
        }
    }
}

impl quote::ToTokens for SceneFieldsKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        Ident::new(match self {
            SceneFieldsKind::Chests => "chests",
            SceneFieldsKind::Switches => "switches",
            SceneFieldsKind::RoomClear => "room_clear",
            SceneFieldsKind::Collectible => "collectible",
            SceneFieldsKind::Unused => "unused",
            SceneFieldsKind::VisitedRooms => "visited_rooms",
            SceneFieldsKind::VisitedFloors => "visited_floors",
        }, Span::call_site()).to_tokens(tokens)
    }
}

enum RegionName {
    One(LitStr),
    Multiple(Expr),
}

impl Parse for RegionName {
    fn parse(input: ParseStream<'_>) -> Result<RegionName> {
        if input.peek(LitStr) {
            input.parse().map(RegionName::One)
        } else {
            input.parse().map(RegionName::Multiple)
        }
    }
}

enum SceneData {
    RegionName(RegionName),
    Fields {
        kind: SceneFieldsKind,
        fields: Punctuated<Flag, Token![,]>,
    },
}

impl Parse for SceneData {
    fn parse(input: ParseStream<'_>) -> Result<SceneData> {
        let ident = input.parse::<Ident>()?;
        Ok(match &*ident.to_string() {
            "region_name" => {
                input.parse::<Token![:]>()?;
                SceneData::RegionName(input.parse()?)
            }
            _ => {
                input.parse::<Token![:]>()?;
                let content;
                braced!(content in input);
                let fields = content.parse_terminated(Flag::parse)?;
                SceneData::Fields {
                    kind: SceneFieldsKind::try_from(ident)?,
                    fields,
                }
            }
        })
    }
}

struct Scene {
    idx: LitInt,
    name: SceneName,
    data: Punctuated<SceneData, Token![,]>,
}

impl Scene {
    fn fields(&self) -> impl Iterator<Item = (&SceneFieldsKind, &Punctuated<Flag, Token![,]>)> {
        self.data.iter().filter_map(|data| if let SceneData::Fields { kind, fields } = data { Some((kind, fields)) } else { None })
    }
}

impl Parse for Scene {
    fn parse(input: ParseStream<'_>) -> Result<Scene> {
        let idx = input.parse()?;
        input.parse::<Token![:]>()?;
        let name = input.parse()?;
        let content;
        braced!(content in input);
        let data = content.parse_terminated(SceneData::parse)?;
        Ok(Scene { idx, name, data })
    }
}

struct SceneFlags {
    vis: Visibility,
    struct_token: Token![struct],
    name: Ident,
    scenes: Punctuated<Scene, Token![,]>,
}

impl Parse for SceneFlags {
    fn parse(input: ParseStream<'_>) -> Result<SceneFlags> {
        let vis = input.parse()?;
        let struct_token = input.parse()?;
        let name = input.parse()?;
        let content;
        braced!(content in input);
        let scenes = content.parse_terminated(Scene::parse)?;
        Ok(SceneFlags { vis, struct_token, name, scenes })
    }
}

#[proc_macro]
pub fn scene_flags(input: TokenStream) -> TokenStream {
    let SceneFlags { vis, struct_token, name, scenes } = parse_macro_input!(input as SceneFlags);
    let scene_size = 0x1c;
    let num_scenes = 0x65usize;
    let contents = scenes.iter().map(|Scene { name, .. }| {
        let scene_field = name.to_field();
        let scene_ty = name.to_type();
        quote!(#vis #scene_field: #scene_ty)
    }).collect_vec();
    let mut entrance_prereqs = Vec::default();
    let mut event_checks = Vec::default();
    let mut location_checks = Vec::default();
    for scene in &scenes {
        let scene_field = scene.name.to_field();
        for (kind, fields) in scene.fields() {
            for Flag { name, .. } in fields {
                let fields_ty = kind.ty(&scene.name);
                let name_ident = name.to_ident();
                match name {
                    FlagName::Event(event_name_lit) => {
                        event_checks.push(quote!(#event_name_lit => Some(self.#scene_field.#kind.contains(#fields_ty::#name_ident))));
                    }
                    FlagName::Ident(_) => {} // internal use only, don't auto-generate check logic
                    FlagName::Lit(name_lit) => {
                        location_checks.push(quote!(#name_lit => Some(self.#scene_field.#kind.contains(#fields_ty::#name_ident))));
                    }
                    FlagName::Entrance(_, _) => unreachable!("entrance checks aren't saved in RAM"), //TODO replace with compile error
                    FlagName::Prereq(id, at_check) => match &**at_check {
                        FlagName::Entrance(from, to) => entrance_prereqs.push(quote!((#id, (#from, #to)) => Some(self.#scene_field.#kind.contains(#fields_ty::#name_ident)))),
                        _ => unimplemented!("prereqs for non-entrance checks"),
                    },
                }
            }
        }
    }
    let get_mut_items = scenes.iter()
        .map(|Scene { name, .. }| {
            let name_lit = name.to_lit();
            let scene_field = name.to_field();
            quote!(#name_lit => Some(&mut self.#scene_field))
        });
    let try_from_items = scenes.iter()
        .map(|Scene { idx, name, .. }| {
            let scene_field = name.to_field();
            let scene_ty = name.to_type();
            let start_idx = idx.base10_parse::<usize>().expect("failed to parse scene index") * scene_size;
            let end_idx = start_idx + scene_size;
            quote!(#scene_field: #scene_ty::try_from(&raw_data[#start_idx..#end_idx]).map_err(|()| raw_data.clone())?)
        });
    let into_items = scenes.iter()
        .map(|Scene { idx, name, .. }| {
            let scene_field = name.to_field();
            let start_idx = idx.base10_parse::<usize>().expect("failed to parse scene index") * scene_size;
            let end_idx = start_idx + scene_size;
            quote!(buf.splice(#start_idx..#end_idx, Vec::from(value.#scene_field));)
        });
    let decls = scenes.iter().map(|scene| {
        let scene_ty = scene.name.to_type();
        let struct_fields = scene.fields().map(|(kind, _)| {
            let fields_ty = kind.ty(&scene.name);
            quote!(#vis #kind: #fields_ty)
        }).collect_vec();
        let try_from_items = scene.fields()
            .map(|(kind, _)| {
                let fields_ty = kind.ty(&scene.name);
                let start_idx = kind.start_idx();
                let end_idx = kind.end_idx();
                quote!(#kind: #fields_ty::try_from(&raw_data[#start_idx..#end_idx])?)
            });
        let into_items = scene.fields()
            .map(|(kind, _)| {
                let start_idx = kind.start_idx();
                let end_idx = kind.end_idx();
                quote!(buf.splice(#start_idx..#end_idx, Vec::from(value.#kind));)
            });
        let set_chests = if let Some((kind, _)) = scene.fields().find(|(kind, _)| **kind == SceneFieldsKind::Chests) {
            let fields_ty = kind.ty(&scene.name);
            quote!(fn set_chests(&mut self, chests: u32) {
                self.#kind = #fields_ty::from_bits_truncate(chests);
            })
        } else {
            quote!(fn set_chests(&mut self, _: u32) {})
        };
        let set_switches = if let Some((kind, _)) = scene.fields().find(|(kind, _)| **kind == SceneFieldsKind::Switches) {
            let fields_ty = kind.ty(&scene.name);
            quote!(fn set_switches(&mut self, switches: u32) {
                self.#kind = #fields_ty::from_bits_truncate(switches);
            })
        } else {
            quote!(fn set_switches(&mut self, _: u32) {})
        };
        let set_room_clear = if let Some((kind, _)) = scene.fields().find(|(kind, _)| **kind == SceneFieldsKind::RoomClear) {
            let fields_ty = kind.ty(&scene.name);
            quote!(fn set_room_clear(&mut self, room_clear: u32) {
                self.#kind = #fields_ty::from_bits_truncate(room_clear);
            })
        } else {
            quote!(fn set_room_clear(&mut self, _: u32) {})
        };
        let subdecls = scene.fields().map(|(kind, fields)| {
            let fields_ty = kind.ty(&scene.name);
            let fields = fields.iter().map(|Flag { name, value }| {
                let name_ident = name.to_ident();
                quote!(const #name_ident = #value;)
            });
            let field_ty = Ident::new("u32", Span::call_site());
            let field_ty_size = 4usize;
            let read_field_ty = Ident::new(&format!("read_{}", field_ty), Span::call_site());
            quote! {
                ::bitflags::bitflags! {
                    #[derive(Default)]
                    #vis struct #fields_ty: #field_ty {
                        #(#fields)*
                    }
                }

                impl<'a> ::std::convert::TryFrom<&'a [u8]> for #fields_ty {
                    type Error = ();

                    fn try_from(raw_data: &[u8]) -> Result<#fields_ty, ()> {
                        if raw_data.len() != #field_ty_size { return Err(()) }
                        Ok(#fields_ty::from_bits_truncate(<::byteorder::BigEndian as ::byteorder::ByteOrder>::#read_field_ty(&raw_data)))
                    }
                }

                impl From<#fields_ty> for Vec<u8> {
                    fn from(value: #fields_ty) -> Vec<u8> {
                        value.bits().to_be_bytes().into()
                    }
                }
            }
        }).collect_vec();
        quote! {
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
            #vis struct #scene_ty {
                #(#struct_fields,)*
            }

            impl<'a> ::std::convert::TryFrom<&'a [u8]> for #scene_ty {
                type Error = ();

                fn try_from(raw_data: &[u8]) -> Result<#scene_ty, ()> {
                    if raw_data.len() != #scene_size { return Err(()) }
                    Ok(#scene_ty {
                        #(#try_from_items,)*
                    })
                }
            }

            impl From<#scene_ty> for Vec<u8> {
                fn from(value: #scene_ty) -> Vec<u8> {
                    let mut buf = vec![0; #scene_size];
                    #(#into_items)*
                    buf
                }
            }

            impl FlagsScene for #scene_ty {
                #set_chests
                #set_switches
                #set_room_clear
            }

            #(#subdecls)*
        }
    }).collect_vec();
    let from_id_arms = scenes.iter().map(|Scene { idx, name, .. }| {
        let name_lit = name.to_lit();
        quote!(#idx => #name_lit)
    });
    let region_arms = scenes.iter().filter_map(|Scene { name, data, .. }| data.iter().filter_map(|data| {
        if let SceneData::RegionName(region_name) = data {
            let scene_name = name.to_lit();
            Some(match region_name {
                RegionName::One(region_name) => quote!(#scene_name => Ok(Region::new(rando, #region_name)?)),
                RegionName::Multiple(f) => quote! {
                    #scene_name => {
                        let f: Box<dyn Fn(&Ram) -> &str + Send + Sync> = Box::new(#f);
                        Ok(Region::new(rando, f(ram))?)
                    }
                },
            })
        } else {
            None
        }
    }).next());
    TokenStream::from(quote! {
        use itertools::Itertools as _;
        use crate::region::RegionLookup;

        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        #vis #struct_token #name {
            #(#contents,)*
        }

        impl #name {
            pub(crate) fn checked(&self, check: &ootr::check::Check) -> Option<bool> {
                match check {
                    ootr::check::Check::AnonymousEvent(at_check, id) => match &**at_check {
                        ootr::check::Check::Exit { from, to, .. } => match (id, (&**from, &**to)) {
                            #(#entrance_prereqs,)*
                            _ => None,
                        },
                        _ => None,
                    },
                    ootr::check::Check::Event(event) => match &event[..] {
                        #(#event_checks,)*
                        _ => None,
                    }
                    ootr::check::Check::Location(loc) => match &loc[..] {
                        #(#location_checks,)*
                        _ => None,
                    },
                    _ => None,
                }
            }

            pub(crate) fn get_mut(&mut self, scene: Scene) -> Option<&mut dyn FlagsScene> {
                match &scene.0[..] {
                    #(#get_mut_items,)*
                    _ => None,
                }
            }
        }

        impl ::std::convert::TryFrom<Vec<u8>> for #name {
            type Error = Vec<u8>;

            fn try_from(raw_data: Vec<u8>) -> Result<#name, Vec<u8>> {
                if raw_data.len() != #num_scenes * #scene_size { return Err(raw_data) }
                Ok(#name {
                    #(#try_from_items,)*
                })
            }
        }

        impl<'a> From<&'a #name> for Vec<u8> {
            fn from(value: &#name) -> Vec<u8> {
                let mut buf = vec![0; #scene_size * #num_scenes];
                #(#into_items)*
                buf
            }
        }

        #(#decls)*

        impl Scene {
            fn from_id(scene_id: u8) -> Scene {
                Scene(match scene_id {
                    #(#from_id_arms,)*
                    _ => panic!("unknown scene ID: {}", scene_id),
                })
            }

            pub(crate) fn region<R: Rando>(&self, rando: &R, ram: &Ram) -> Result<RegionLookup, RegionLookupError<R>> {
                match self.0 {
                    #(#region_arms,)*
                    _ => RegionLookup::new(self.regions(rando)?),
                }
            }
        }
    })
}
