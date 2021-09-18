#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        borrow::Cow,
        collections::{
            BTreeSet,
            HashMap,
        },
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
    ootr::{
        Rando as _,
        settings::Knowledge as _,
    },
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
    MissingVanillaSetting(String),
    MissingVersionPy,
    MissingVersionText,
    Rando(ootr_dynamic::RandoErr),
    Reqwest(reqwest::Error),
    SettingsKnowledge(ootr::settings::KnowledgeTypeError),
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
                let from = Ident::new(&from.to_case(Case::Pascal), Span::call_site());
                let from_mq = from_mq.quote();
                let to = Ident::new(&to.to_case(Case::Pascal), Span::call_site());
                quote!(::ootr::check::Check::Exit {
                    from: RegionName::#from,
                    from_mq: #from_mq,
                    to: RegionName::#to,
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
        let name = Ident::new(&name.to_case(Case::Pascal), Span::call_site());
        let dungeon = dungeon.quote();
        let scene = scene.quote();
        let hint = hint.quote();
        let time_passes = time_passes.quote();
        let events = events.iter().map(|(name, rule)| (name.clone(), AccessExprWrapper(rule))).collect::<HashMap<_, _>>().quote();
        let locations = locations.iter().map(|(name, rule)| (name.clone(), AccessExprWrapper(rule))).collect::<HashMap<_, _>>().quote();
        let exit_inserts = exits.iter()
            .map(|(name, rule)| {
                let name = Ident::new(&name.to_case(Case::Pascal), Span::call_site());
                let rule = AccessExprWrapper(rule).quote();
                quote!(map.insert(RegionName::#name, #rule);)
            })
            .collect_vec();
        let num_exits = exit_inserts.len();
        quote! {
            ::std::sync::Arc::new(
                ::ootr::region::Region {
                    name: RegionName::#name,
                    dungeon: #dungeon,
                    scene: #scene,
                    hint: #hint,
                    time_passes: #time_passes,
                    events: #events,
                    locations: #locations,
                    exits: {
                        let mut map = HashMap::with_capacity(#num_exits);
                        #(#exit_inserts)*
                        map
                    },
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
    let (region_names, default_settings, vanilla_settings, data) = Python::with_gil(|py| {
        let rando = ootr_dynamic::Rando::new(py, rando_path);
        let region_names = rando.regions()?.iter().map(|region| region.name.clone()).collect::<BTreeSet<_>>();
        let default_settings = ootr_dynamic::settings::Knowledge::default(&rando)?;
        let vanilla_settings = ootr_dynamic::settings::Knowledge::vanilla(&rando);
        let data = vec![
            ("escaped_items", quote!(HashMap<String, Item>), rando.escaped_items()?.quote()),
            ("item_table", quote!(HashMap<String, Item>), rando.item_table()?.quote()),
            ("logic_helpers", quote!(HashMap<String, (Vec<String>, Expr<#ty>)>), Arc::new(rando.logic_helpers()?.iter().map(|(k, v)| (k.clone(), LogicHelperWrapper(v))).collect::<HashMap<_, _>>()).quote()),
            ("logic_tricks", quote!(HashSet<String>), rando.logic_tricks()?.quote()),
            ("regions", quote!(Vec<Arc<Region<#ty>>>), Arc::new(rando.regions()?.iter().map(RegionWrapper).collect_vec()).quote()),
            ("setting_names", quote!(HashMap<String, String>), rando.setting_names()?.quote()),
        ];
        Ok::<_, Error>((region_names, default_settings, vanilla_settings, data))
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
    let (region_name_from_str_arms, region_name_as_str_arms, region_name_variants) = region_names.iter()
        .map(|region_name| {
            let ident = Ident::new(&region_name.to_case(Case::Pascal), Span::call_site());
            (quote!(#region_name => Ok(Self::#ident),), quote!(Self::#ident => #region_name,), ident)
        })
        .multiunzip::<(Vec<_>, Vec<_>, Vec<_>)>();
    let (
        settings_knowledge_types,
        settings_knowledge_fields,
        settings_knowledge_defaults,
        settings_knowledge_vanilla,
        settings_knowledge_get_arms,
        settings_knowledge_update_arms,
        settings_knowledge_remove_arms,
    ) = default_settings.0.into_iter()
        .map(|(name, value)| {
            let name_ident = Ident::new(&name, Span::call_site());
            Ok::<_, Error>(match value {
                ootr::settings::KnowledgeValue::Bool(_) => {
                    let quoted_vanilla = vanilla_settings.get::<bool>(&name)?
                        .ok_or_else(|| Error::MissingVanillaSetting(name.to_string()))?
                        .quote();
                    (
                        quote!(),
                        quote!(#name_ident: Option<bool>),
                        quote!(#name_ident: None),
                        quote!(#name_ident: Some(#quoted_vanilla)),
                        quote!(#name => Some(T::from_bool(self.#name_ident)?)),
                        quote!(#name => self.#name_ident = value.into_bool()?),
                        quote!(#name => self.#name_ident = None),
                    )
                }
                ootr::settings::KnowledgeValue::Int(range) => {
                    let quoted_range = range.quote();
                    let quoted_vanilla = vanilla_settings.get::<u8>(&name)?
                        .ok_or_else(|| Error::MissingVanillaSetting(name.to_string()))?
                        .quote();
                    (
                        quote!(),
                        quote!(#name_ident: RangeInclusive<u8>), //TODO bounds checks?
                        quote!(#name_ident: #quoted_range),
                        quote!(#name_ident: #quoted_vanilla..=#quoted_vanilla),
                        quote!(#name => Some(T::from_int(self.#name_ident.clone())?)),
                        quote!(#name => self.#name_ident = value.into_int()?),
                        quote!(#name => self.#name_ident = #quoted_range),
                    )
                }
                ootr::settings::KnowledgeValue::String(choices) => {
                    let vanilla_choice = vanilla_settings.get::<Cow<'static, str>>(&name)?
                        .ok_or_else(|| Error::MissingVanillaSetting(name.to_string()))?;
                    let type_ident = Ident::new(&format!("{}Knowledge", name.to_case(Case::Pascal)), Span::call_site());
                    let (choice_idents, vanilla_choices, choices) = choices.into_iter()
                        .map(|choice| (
                            Ident::new(match &*choice {
                                "0" => "zero",
                                "1" => "one",
                                "2" => "two",
                                "3" => "three",
                                "4" => "four",
                                "witching-hour" => "witching_hour",
                                _ => &choice,
                            }, Span::call_site()),
                            (*choice == vanilla_choice).quote(),
                            choice,
                        ))
                        .multiunzip::<(Vec<_>, Vec<_>, Vec<_>)>();
                    (
                        quote! {
                            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                            pub struct #type_ident {
                                #(#choice_idents: bool,)*
                            }

                            impl Default for #type_ident {
                                fn default() -> Self {
                                    Self {
                                        #(#choice_idents: true,)*
                                    }
                                }
                            }

                            impl TryFrom<HashSet<Cow<'static, str>>> for #type_ident {
                                type Error = ootr::settings::KnowledgeTypeError;

                                fn try_from(mut set: HashSet<Cow<'static, str>>) -> Result<Self, ootr::settings::KnowledgeTypeError> {
                                    let choices = Self {
                                        #(#choice_idents: set.remove(#choices),)*
                                    };
                                    if set.is_empty() {
                                        Ok(choices)
                                    } else {
                                        Err(ootr::settings::KnowledgeTypeError::StrInvalid)
                                    }
                                }
                            }

                            impl From<#type_ident> for HashSet<Cow<'static, str>> {
                                fn from(choices: #type_ident) -> Self {
                                    let mut set = HashSet::default();
                                    #(
                                        if choices.#choice_idents { set.insert(Cow::Borrowed(#choices)); }
                                    )*
                                    set
                                }
                            }
                        },
                        quote!(#name_ident: #type_ident),
                        quote!(#name_ident: Default::default()),
                        quote!(#name_ident: #type_ident { #(#choice_idents: #vanilla_choices,)* }),
                        quote!(#name => Some(T::from_string(&self.#name_ident.into())?)),
                        quote!(#name => self.#name_ident = #type_ident::try_from(value.into_string()?)?),
                        quote!(#name => self.#name_ident = Default::default()),
                    )
                }
                ootr::settings::KnowledgeValue::List(_) => {
                    let quoted_vanilla = vanilla_settings.get::<HashMap<Cow<'static, str>, bool>>(&name)?
                        .ok_or_else(|| Error::MissingVanillaSetting(name.to_string()))?
                        .quote();
                    (
                        quote!(),
                        quote!(#name_ident: HashMap<Cow<'static, str>, bool>), //TODO use structs with Option<bool> fields for known lists?
                        quote!(#name_ident: HashMap::default()),
                        quote!(#name_ident: #quoted_vanilla),
                        quote!(#name => Some(T::from_list(&self.#name_ident)?)),
                        quote!(#name => self.#name_ident = value.into_list()?),
                        quote!(#name => self.#name_ident = HashMap::default()),
                    )
                }
            })
        })
        .try_collect::<_, Vec<_>, _>()?
        .into_iter()
        .multiunzip::<(Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>)>();
    Ok(quote! {
        #(#lazy_statics)*

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum RegionName {
            #(#region_name_variants,)*
        }

        impl FromStr for RegionName {
            /// An error is returned if there is no region with the given name.
            type Err = ();

            fn from_str(s: &str) -> Result<Self, ()> {
                match s {
                    #(#region_name_from_str_arms)*
                    _ => Err(()),
                }
            }
        }

        impl AsRef<str> for RegionName {
            fn as_ref(&self) -> &str {
                match self {
                    #(#region_name_as_str_arms)*
                }
            }
        }

        impl fmt::Display for RegionName {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.as_ref().fmt(f)
            }
        }

        impl<'a> PartialEq<&'a str> for RegionName {
            fn eq(&self, rhs: &&str) -> bool {
                self.as_ref() == *rhs
            }
        }

        impl Protocol for RegionName {
            fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
                Box::pin(async move {
                    Self::from_str(&String::read(stream).await?).map_err(|()| ReadError::Custom(format!("unknown region name")))
                })
            }

            fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
                Box::pin(async move {
                    self.to_string().write(sink).await?;
                    Ok(())
                })
            }

            fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
                self.to_string().write_sync(sink)?;
                Ok(())
            }
        }

        #(#settings_knowledge_types)*

        pub struct SettingsKnowledge {
            #(#settings_knowledge_fields,)*
        }

        impl Default for SettingsKnowledge {
            fn default() -> Self {
                Self {
                    #(#settings_knowledge_defaults,)*
                }
            }
        }

        impl ootr::settings::Knowledge<#ty> for SettingsKnowledge {
            fn default(_: &#ty) -> Result<Self, RandoErr> {
                Ok(<Self as Default>::default())
            }

            fn vanilla(_: &#ty) -> Self {
                Self {
                    #(#settings_knowledge_vanilla,)*
                }
            }

            fn get<T: ootr::settings::KnowledgeType>(&self, setting: &str) -> Result<Option<T>, ootr::settings::KnowledgeTypeError> {
                Ok(match setting {
                    #(#settings_knowledge_get_arms,)*
                    _ => return Err(ootr::settings::KnowledgeTypeError::UnknownSetting),
                })
            }

            fn update<T: ootr::settings::KnowledgeType>(&mut self, setting: &str, value: T) -> Result<(), ootr::settings::KnowledgeTypeError> {
                match setting {
                    #(#settings_knowledge_update_arms,)*
                    _ => return Err(ootr::settings::KnowledgeTypeError::UnknownSetting),
                }
                Ok(())
            }

            fn remove(&mut self, setting: &str) {
                match setting {
                    #(#settings_knowledge_remove_arms,)*
                    _ => {} //TODO return error?
                }
            }
        }

        impl ootr::Rando for #ty {
            type Err = RandoErr;
            type RegionName = RegionName;
            type SettingsKnowledge = SettingsKnowledge;

            fn root() -> RegionName { RegionName::Root }
            #(#trait_fns)*
        }
    })
}
