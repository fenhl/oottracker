#![deny(rust_2018_idioms, unused, unused_import_braces, unused_qualifications, warnings)]

use {
    convert_case::{
        Case,
        Casing as _,
    },
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::quote,
    syn::{
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

enum FlagName {
    Ident(Ident),
    Lit(LitStr),
}

impl FlagName {
    fn to_ident(&self) -> Ident {
        match self {
            FlagName::Ident(ident) => ident.clone(),
            FlagName::Lit(lit) => Ident::new(&lit.value().to_case(Case::ScreamingSnake), lit.span()),
        }
    }
}

impl Parse for FlagName {
    fn parse(input: ParseStream<'_>) -> Result<FlagName> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            input.parse().map(FlagName::Ident)
        } else if lookahead.peek(LitStr) {
            input.parse().map(FlagName::Lit)
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
    let mut all_fields = (0..num_fields).map(|_| None).collect::<Vec<_>>();
    for Flags { idx, fields } in fields {
        let idx = match idx.base10_parse::<usize>() {
            Ok(n) => n,
            Err(e) => return e.to_compile_error().into(),
        };
        all_fields[idx] = Some(fields);
    }
    let fields_tys = (0..num_fields).map(|i|
        Ident::new(&format!("{}{}", name, i), Span::call_site())
    ).collect::<Vec<_>>();
    let contents = all_fields.iter().zip(&fields_tys).map(|(fields, fields_ty)| {
        if fields.is_some() { quote!(#vis #fields_ty) } else { quote!(#fields_ty) }
    }).collect::<Vec<_>>();
    let tup_idxs = (0..num_fields).map(Index::from).collect::<Vec<_>>();
    let checks = all_fields.iter()
        .zip(&tup_idxs)
        .zip(&fields_tys)
        .filter_map(|((fields, idx), fields_ty)| fields.as_ref().map(|fields| (idx, fields, fields_ty)))
        .flat_map(|(idx, fields, fields_ty)|
            fields.iter()
                .filter_map(move |Flag { name, .. }| if let FlagName::Lit(name_lit) = name.clone() {
                    let name_ident = name.to_ident();
                    Some(quote!(#name_lit => Some(self.#idx.contains(#fields_ty::#name_ident))))
                } else {
                    None
                })
        );
    let start_idxs = (0..num_fields).map(|i| i * field_ty_size);
    let end_idxs = (1..=num_fields).map(|i| i * field_ty_size);
    let decls = all_fields.iter().zip(&fields_tys).map(|(fields, fields_ty)|
        if let Some(fields) = fields {
            let fields = fields.iter().map(|Flag { name, value }| {
                let name_ident = name.to_ident();
                quote!(const #name_ident = #value;)
            });
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
    ).collect::<Vec<_>>();
    TokenStream::from(quote! {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        #vis #struct_token #name(#(#contents,)*);

        impl #name {
            pub(crate) fn checked(&self, loc: &str) -> Option<bool> {
                match loc {
                    #(#checks,)*
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

    fn ty(&self, scene_name: String) -> Ident {
        Ident::new(&format!("{}{}", scene_name.to_case(Case::Pascal), match self {
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

impl Parse for SceneFieldsKind {
    fn parse(input: ParseStream<'_>) -> Result<SceneFieldsKind> {
        let ident = input.parse::<Ident>()?;
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

struct SceneFields {
    kind: SceneFieldsKind,
    fields: Punctuated<Flag, Token![,]>,
}

impl Parse for SceneFields {
    fn parse(input: ParseStream<'_>) -> Result<SceneFields> {
        let kind = input.parse()?;
        input.parse::<Token![:]>()?;
        let content;
        braced!(content in input);
        let fields = content.parse_terminated(Flag::parse)?;
        Ok(SceneFields { kind, fields })
    }
}

struct Scene {
    idx: LitInt,
    name: LitStr,
    fields: Punctuated<SceneFields, Token![,]>,
}

impl Parse for Scene {
    fn parse(input: ParseStream<'_>) -> Result<Scene> {
        let idx = input.parse()?;
        input.parse::<Token![:]>()?;
        let name = input.parse()?;
        let content;
        braced!(content in input);
        let fields = content.parse_terminated(SceneFields::parse)?;
        Ok(Scene { idx, name, fields })
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
        let scene_field = Ident::new(&name.value().to_case(Case::Snake), name.span());
        let scene_ty = Ident::new(&name.value().to_case(Case::Pascal), name.span());
        quote!(#vis #scene_field: #scene_ty)
    }).collect::<Vec<_>>();
    let checks = scenes.iter()
        .flat_map(|Scene { name: scene_name, fields, .. }| {
            let scene_field = Ident::new(&scene_name.value().to_case(Case::Snake), name.span());
            fields.iter()
                .flat_map(move |SceneFields { kind, fields }| {
                    let kind = kind.clone();
                    let scene_field = scene_field.clone();
                    fields.iter()
                        .filter_map(move |Flag { name, .. }| if let FlagName::Lit(name_lit) = name.clone() {
                            let fields_ty = kind.ty(scene_name.value());
                            let name_ident = name.to_ident();
                            Some(quote!(#name_lit => Some(self.#scene_field.#kind.contains(#fields_ty::#name_ident))))
                        } else {
                            None
                        })
                })
        });
    let try_from_items = scenes.iter()
        .map(|Scene { idx, name, .. }| {
            let scene_field = Ident::new(&name.value().to_case(Case::Snake), name.span());
            let scene_ty = Ident::new(&name.value().to_case(Case::Pascal), name.span());
            let start_idx = idx.base10_parse::<usize>().expect("failed to parse scene index") * scene_size;
            let end_idx = start_idx + scene_size;
            quote!(#scene_field: #scene_ty::try_from(&raw_data[#start_idx..#end_idx]).map_err(|()| raw_data.clone())?)
        });
    let into_items = scenes.iter()
        .map(|Scene { idx, name, .. }| {
            let scene_field = Ident::new(&name.value().to_case(Case::Snake), name.span());
            let start_idx = idx.base10_parse::<usize>().expect("failed to parse scene index") * scene_size;
            let end_idx = start_idx + scene_size;
            quote!(buf.splice(#start_idx..#end_idx, Vec::from(value.#scene_field));)
        });
    let decls = scenes.iter().map(|Scene { name, fields, .. }| {
        let scene_ty = Ident::new(&name.value().to_case(Case::Pascal), name.span());
        let struct_fields = fields.iter().map(|SceneFields { kind, .. }| {
            let fields_ty = kind.ty(name.value());
            quote!(#vis #kind: #fields_ty)
        }).collect::<Vec<_>>();
        let try_from_items = fields.iter()
            .map(|SceneFields { kind, .. }| {
                let fields_ty = kind.ty(name.value());
                let start_idx = kind.start_idx();
                let end_idx = kind.end_idx();
                quote!(#kind: #fields_ty::try_from(&raw_data[#start_idx..#end_idx])?)
            });
        let into_items = fields.iter()
            .map(|SceneFields { kind, .. }| {
                let start_idx = kind.start_idx();
                let end_idx = kind.end_idx();
                quote!(buf.splice(#start_idx..#end_idx, Vec::from(value.#kind));)
            });
        let subdecls = fields.iter().map(|SceneFields { kind, fields }| {
            let fields_ty = kind.ty(name.value());
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
        }).collect::<Vec<_>>();
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

            #(#subdecls)*
        }
    }).collect::<Vec<_>>();
    TokenStream::from(quote! {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        #vis #struct_token #name {
            #(#contents,)*
        }

        impl #name {
            pub(crate) fn checked(&self, loc: &str) -> Option<bool> {
                match loc {
                    #(#checks,)*
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
    })
}
