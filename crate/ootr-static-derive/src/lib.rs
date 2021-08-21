#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        collections::HashMap,
        fs::{
            self,
            File,
        },
        io::{
            self,
            Cursor,
            prelude::*,
        },
        sync::Arc,
    },
    convert_case::{
        Case,
        Casing as _,
    },
    derive_more::From,
    directories::ProjectDirs,
    graphql_client::{
        GraphQLQuery,
        Response,
    },
    itertools::Itertools as _,
    proc_macro::TokenStream,
    proc_macro2::Span,
    pyo3::prelude::*,
    quote::quote,
    quote_value::QuoteValue,
    syn::{
        DeriveInput,
        Ident,
        parse_macro_input,
    },
    zip::{
        ZipArchive,
        result::ZipError,
    },
    ootr::Rando as _,
};

#[proc_macro]
pub fn version(_: TokenStream) -> TokenStream {
    let version = env!("CARGO_PKG_VERSION");
    TokenStream::from(quote! {
        ::semver::Version::parse(#version).expect("failed to parse current version")
    })
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../assets/graphql/github-schema.graphql",
    query_path = "../../assets/graphql/github-devr-version.graphql",
    response_derives = "Debug",
)]
struct DevRVersionQuery;

#[derive(Debug, From)]
enum Error {
    EmptyResponse,
    Io(io::Error),
    MissingHomeDir,
    MissingRepo,
    MissingVersionPy,
    MissingVersionText,
    Rando(ootr_dynamic::RandoErr),
    Reqwest(reqwest::Error),
    Zip(ZipError),
}

/// A wrapper type around `Expr<ootr_dynamic::Rando>` that's quoted as if it were an `Expr<ootr_static::Rando>`
struct AccessExprWrapper<'a>(&'a ootr::access::Expr<ootr_dynamic::Rando<'a>>);

impl<'a> QuoteValue for AccessExprWrapper<'a> {
    fn quote(&self) -> proc_macro2::TokenStream {
        match self.0 {
            ootr::access::Expr::All(ref exprs) => {
                let exprs = exprs.iter().map(AccessExprWrapper).collect_vec().quote();
                quote!(::ootr::access::Expr::All(#exprs))
            }
            ootr::access::Expr::Any(ref exprs) => {
                let exprs = exprs.iter().map(AccessExprWrapper).collect_vec().quote();
                quote!(::ootr::access::Expr::Any(#exprs))
            }
            ootr::access::Expr::AnonymousEvent(ref at_check, id) => {
                let at_check = CheckWrapper(at_check).quote();
                let id = id.quote();
                quote!(::ootr::access::Expr::AnonymousEvent(#at_check, #id))
            }
            ootr::access::Expr::Eq(ref lhs, ref rhs) => {
                let lhs = Box::new(AccessExprWrapper(lhs)).quote();
                let rhs = Box::new(AccessExprWrapper(rhs)).quote();
                quote!(::ootr::access::Expr::Eq(#lhs, #rhs))
            }
            ootr::access::Expr::HasDungeonRewards(ref n) => {
                let n = Box::new(AccessExprWrapper(n)).quote();
                quote!(::ootr::access::Expr::HasDungeonRewards(#n))
            }
            ootr::access::Expr::HasMedallions(ref n) => {
                let n = Box::new(AccessExprWrapper(n)).quote();
                quote!(::ootr::access::Expr::HasMedallions(#n))
            }
            ootr::access::Expr::HasStones(ref n) => {
                let n = Box::new(AccessExprWrapper(n)).quote();
                quote!(::ootr::access::Expr::HasStones(#n))
            }
            ootr::access::Expr::Item(ref item, ref count) => {
                let item = item.quote();
                let count = Box::new(AccessExprWrapper(count)).quote();
                quote!(::ootr::access::Expr::Item(#item, #count))
            }
            ootr::access::Expr::LogicHelper(ref helper_name, ref exprs) => {
                let helper_name = helper_name.quote();
                let exprs = exprs.iter().map(AccessExprWrapper).collect_vec().quote();
                quote!(::ootr::access::Expr::LogicHelper(#helper_name, #exprs))
            }
            ootr::access::Expr::Not(ref expr) => {
                let expr = Box::new(AccessExprWrapper(expr)).quote();
                quote!(::ootr::access::Expr::Not(#expr))
            }
            _ => self.0.quote(),
        }
    }
}

/// A wrapper type around `Check<ootr_dynamic::Rando>` that's quoted as if it were a `Check<ootr_static::Rando>`
struct CheckWrapper<'a>(&'a ootr::check::Check<ootr_dynamic::Rando<'a>>);

impl<'a> QuoteValue for CheckWrapper<'a> {
    fn quote(&self) -> proc_macro2::TokenStream {
        match self.0 {
            ootr::check::Check::AnonymousEvent(ref at_check, id) => {
                let at_check = CheckWrapper(at_check).quote();
                let id = id.quote();
                quote!(::ootr::check::Check::AnonymousEvent(#at_check, #id))
            }
            ootr::check::Check::Exit { ref from, from_mq, ref to } => {
                let from = (&from[..]).quote(); // quote as &'static str
                let from_mq = from_mq.quote();
                let to = (&to[..]).quote(); // quote as &'static str
                quote!(::ootr::check::Check::Exit {
                    from: #from,
                    from_mq: #from_mq,
                    to: #to,
                })
            }
            _ => self.0.quote(),
        }
    }
}

/// A wrapper type around `(Vec<String>, Expr<ootr_dynamic::Rando>)` that's quoted as if it were a `(Vec<String>, Expr<ootr_static::Rando>)`
struct LogicHelperWrapper<'a>(&'a (Vec<String>, ootr::access::Expr<ootr_dynamic::Rando<'a>>));

impl<'a> QuoteValue for LogicHelperWrapper<'a> {
    fn quote(&self) -> proc_macro2::TokenStream {
        let (params, expr) = self.0;
        let params = params.quote();
        let expr = AccessExprWrapper(expr).quote();
        quote! { (#params, #expr) }
    }
}

/// A wrapper type around `Arc<Region<ootr_dynamic::Rando>>` that's quoted as if it were an `Arc<Region<ootr_static::Rando>>`
struct RegionWrapper<'a>(&'a Arc<ootr::region::Region<ootr_dynamic::Rando<'a>>>);

impl<'a> QuoteValue for RegionWrapper<'a> {
    fn quote(&self) -> proc_macro2::TokenStream {
        let ootr::region::Region { ref name, ref dungeon, ref scene, ref hint, ref time_passes, ref events, ref locations, ref exits } = **self.0;
        let name = (&name[..]).quote(); // quote as &'static str
        let dungeon = dungeon.quote();
        let scene = scene.quote();
        let hint = hint.quote();
        let time_passes = time_passes.quote();
        let events = events.iter().map(|(name, rule)| (name.clone(), AccessExprWrapper(rule))).collect::<HashMap<_, _>>().quote();
        let locations = locations.iter().map(|(name, rule)| (name.clone(), AccessExprWrapper(rule))).collect::<HashMap<_, _>>().quote();
        let exits = exits.iter().map(|(name, rule)| (&name[..], AccessExprWrapper(rule))).collect::<HashMap<_, _>>().quote();
        quote! {
            ::std::sync::Arc::new(
                ::ootr::region::Region {
                    name: #name,
                    dungeon: #dungeon,
                    scene: #scene,
                    hint: #hint,
                    time_passes: #time_passes,
                    events: #events,
                    locations: #locations,
                    exits: #exits,
                }
            )
        }
    }
}

#[proc_macro_derive(Rando)]
pub fn derive_rando(input: TokenStream) -> TokenStream {
    let DeriveInput { ident: ty, .. } = parse_macro_input!(input);
    TokenStream::from(derive_rando_inner(ty).unwrap_or_else(|e| {
        let text = format!("failed to generate code for derive(Rando): {:?}", e);
        quote!(compile_error!(#text);)
    }))
}

fn derive_rando_inner(ty: Ident) -> Result<proc_macro2::TokenStream, Error> {
    let project_dirs = ProjectDirs::from("net", "Fenhl", "RSL").ok_or(Error::MissingHomeDir)?; // re-use rando copy from https://github.com/fenhl/plando-random-settings/tree/riir
    let cache_dir = project_dirs.cache_dir();
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("oottracker/", env!("CARGO_PKG_VERSION")))
        .http2_prior_knowledge()
        .use_rustls_tls()
        .https_only(true)
        .build()?;
    // ensure the correct randomizer version is installed
    let remote_version_string = match client.post("https://api.github.com/graphql")
        .bearer_auth(include_str!("../../../assets/release-token"))
        .json(&DevRVersionQuery::build_query(dev_r_version_query::Variables {}))
        .send()?
        .error_for_status()?
        .json::<Response<dev_r_version_query::ResponseData>>()?
        .data.ok_or(Error::EmptyResponse)?
        .repository.ok_or(Error::MissingRepo)?
        .object.ok_or(Error::MissingVersionPy)?
    {
        dev_r_version_query::DevRVersionQueryRepositoryObject::Blob(blob) => blob.text.ok_or(Error::MissingVersionText)?,
        on => panic!("unexpected GraphQL interface: {:?}", on),
    };
    let rando_path = cache_dir.join("ootr-latest");
    if rando_path.join("version.py").exists() {
        let mut local_version_string = String::default();
        File::open(rando_path.join("version.py"))?.read_to_string(&mut local_version_string)?;
        if remote_version_string.trim() != local_version_string.trim() {
            fs::remove_dir_all(&rando_path)?;
        }
    }
    if !rando_path.exists() {
        let rando_download = client.get("https://github.com/Roman971/OoT-Randomizer/archive/Dev-R.zip")
            .send()?
            .error_for_status()?
            .bytes()?;
        ZipArchive::new(Cursor::new(rando_download))?.extract(&cache_dir)?;
        fs::rename(cache_dir.join("OoT-Randomizer-Dev-R"), &rando_path)?;
    }
    let data = Python::with_gil(|py| {
        let rando = ootr_dynamic::Rando::new(py, rando_path);
        let data = vec![
            ("escaped_items", quote!(HashMap<String, Item>), rando.escaped_items()?.quote()),
            ("item_table", quote!(HashMap<String, Item>), rando.item_table()?.quote()),
            ("logic_helpers", quote!(HashMap<String, (Vec<String>, Expr<#ty>)>), Arc::new(rando.logic_helpers()?.iter().map(|(k, v)| (k.clone(), LogicHelperWrapper(v))).collect::<HashMap<_, _>>()).quote()),
            ("logic_tricks", quote!(HashSet<String>), rando.logic_tricks()?.quote()),
            ("regions", quote!(Vec<Arc<Region<#ty>>>), Arc::new(rando.regions()?.iter().map(RegionWrapper).collect_vec()).quote()),
            ("setting_infos", quote!(HashSet<String>), rando.setting_infos()?.quote()),
        ];
        Ok::<_, Error>(data)
    })?;
    let screaming_idents = data.iter()
        .map(|(name, _, _)| Ident::new(&name.to_case(Case::ScreamingSnake), Span::call_site()))
        .collect_vec();
    let lazy_statics = screaming_idents.iter().zip(&data)
        .map(|(screaming_ident, (_, ty, value))| quote!(static #screaming_ident: Lazy<Arc<#ty>> = Lazy::new(|| #value);));
    let trait_fns = screaming_idents.iter().zip(&data)
        .map(|(screaming_ident, (name, ty, _))| {
            let ident = Ident::new(name, Span::call_site());
            quote! {
                fn #ident<'a>(&'a self) -> Result<Arc<#ty>, RandoErr> {
                    Ok(Arc::clone(&#screaming_ident))
                }
            }
        });
    Ok(quote! {
        #(#lazy_statics)*

        impl ootr::Rando for #ty {
            type Err = RandoErr;
            type RegionName = &'static str;

            fn root() -> &'static str { "Root" }
            #(#trait_fns)*
        }
    })
}
