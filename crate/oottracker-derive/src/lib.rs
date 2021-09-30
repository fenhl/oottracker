#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        fs::{
            self,
            File,
        },
        io::{
            self,
            Cursor,
            prelude::*,
        },
        path::{
            Path,
            PathBuf,
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
    once_cell::sync::Lazy,
    proc_macro::TokenStream,
    proc_macro2::{
        Literal,
        Span,
    },
    pyo3::prelude::*,
    quote::quote,
    syn::{
        Ident,
        LitStr,
        parse_macro_input,
    },
    wheel::FromArc,
    zip::{
        ZipArchive,
        result::ZipError,
    },
};

mod access;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../assets/graphql/github-schema.graphql",
    query_path = "../../assets/graphql/github-devr-version.graphql",
    response_derives = "Debug",
)]
struct DevRVersionQuery;

#[derive(Debug, From, FromArc, Clone)]
enum Error {
    #[from]
    AccessParse(access::ParseError),
    EmptyResponse,
    ExitToUnknownRegion,
    InvalidLogicHelper,
    #[from_arc]
    Io(Arc<io::Error>),
    MissingHomeDir,
    MissingIntSettingBound,
    MissingRepo,
    MissingVersionPy,
    MissingVersionText,
    NonJsonRegionFile(String),
    NonUnicodeRegionFilename,
    #[from_arc]
    Py(Arc<PyErr>),
    #[from_arc]
    Reqwest(Arc<reqwest::Error>),
    UnknownDungeon(String),
    UnknownSettingType(String),
    UnknownStringSetting(String),
    #[from_arc]
    Zip(Arc<ZipError>),
}

impl Error {
    fn to_compile_error(&self) -> proc_macro2::TokenStream {
        let msg = format!("{:?}", self);
        quote!(compile_error!(#msg);)
    }
}

impl<'a> From<pyo3::PyDowncastError<'a>> for Error {
    fn from(e: pyo3::PyDowncastError<'_>) -> Self {
        Self::Py(Arc::new(e.into()))
    }
}

static RANDO_PATH: Lazy<Result<PathBuf, Error>> = Lazy::new(|| {
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
    Ok(PathBuf::from(rando_path))
});

/// Imports the given Python module from the randomizer codebase and runs the given function on it.
fn rando_import<T>(mod_name: &str, f: impl for<'p> FnOnce(&'p PyModule) -> Result<T, Error>) -> Result<T, Error> {
    let rando_path = RANDO_PATH.clone()?;
    Python::with_gil(|py| {
        let sys = py.import("sys")?;
        sys.getattr("path")?.call_method1("append", (rando_path.display().to_string(),))?;
        f(py.import(mod_name)?)
    })
}

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
    let img_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("crate has no parent").parent().expect("crates dir has no parent").join(img_path);
    let name = Ident::new(&img_path.file_name().expect("empty filename").to_string_lossy().split('.').next().expect("empty filename").to_case(Case::Snake), Span::call_site());
    let mut buf = Vec::default();
    File::open(img_path).expect("failed to open image to embed").read_to_end(&mut buf).expect("failed to read image to embed");
    let contents_lit = Literal::byte_string(&buf);
    TokenStream::from(quote! {
        pub fn #name<T: FromEmbeddedImage>() -> T {
            T::from_embedded_image(#contents_lit)
        }
    })
}

#[proc_macro]
pub fn embed_images(input: TokenStream) -> TokenStream {
    let dir_path = parse_macro_input!(input as LitStr).value();
    let dir_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("crate has no parent").parent().expect("crates dir has no parent").join(dir_path);
    let name = Ident::new(&dir_path.file_name().expect("empty filename").to_string_lossy().to_case(Case::Snake), Span::call_site());
    let name_all = Ident::new(&format!("{}_all", name), Span::call_site());
    let img_consts = fs::read_dir(dir_path).expect("failed to open images dir") //TODO compile error instead of panic
        .filter_map(|img_path| match img_path {
            Ok(img_path) => if img_path.file_name().to_str().map_or(false, |file_name| file_name.starts_with('.')) { None } else { Some(Ok(img_path)) },
            Err(e) => Some(Err(e)),
        })
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
        pub fn #name<T: FromEmbeddedImage>(name: &str) -> T {
            static IMG_CONSTS: ::once_cell::sync::Lazy<::std::collections::HashMap<&'static str, &'static [u8]>> = ::once_cell::sync::Lazy::new(|| {
                let mut consts = ::std::collections::HashMap::<&'static str, &'static [u8]>::default();
                #(#img_consts)*
                consts
            });

            T::from_embedded_image(IMG_CONSTS[name])
        }

        pub fn #name_all<T: FromEmbeddedImage>() -> impl Iterator<Item = T> {
            static IMG_CONSTS: ::once_cell::sync::Lazy<::std::collections::HashMap<&'static str, &'static [u8]>> = ::once_cell::sync::Lazy::new(|| {
                let mut consts = ::std::collections::HashMap::<&'static str, &'static [u8]>::default();
                #(#img_consts)*
                consts
            });

            IMG_CONSTS.values().map(|contents| T::from_embedded_image(contents))
        }
    })
}

mod flags;

#[proc_macro]
pub fn flags_list(input: TokenStream) -> TokenStream {
    flags::flags_list(parse_macro_input!(input as flags::FlagsList)).into()
}

#[proc_macro]
pub fn scene_flags(input: TokenStream) -> TokenStream {
    flags::scene_flags(parse_macro_input!(input as flags::SceneFlags)).into()
}

mod item;

#[proc_macro]
pub fn item(input: TokenStream) -> TokenStream {
    if input.is_empty() {
        match rando_import("ItemList", |item_list| item::item(item_list)) {
            Ok(output) => output,
            Err(e) => e.to_compile_error(),
        }
    } else {
        quote!(compile_error!("item!() takes no arguments");)
    }.into()
}

mod region;

#[proc_macro]
pub fn region(input: TokenStream) -> TokenStream {
    if input.is_empty() {
        match RANDO_PATH.clone().and_then(|rando_path| Python::with_gil(|py| region::region(py, &rando_path))) {
            Ok(output) => output,
            Err(e) => e.to_compile_error(),
        }
    } else {
        quote!(compile_error!("region!() takes no arguments");)
    }.into()
}

mod settings;

#[proc_macro]
pub fn settings(input: TokenStream) -> TokenStream {
    if input.is_empty() {
        match rando_import("SettingsList", |settings_list| settings::settings(settings_list)) {
            Ok(output) => output,
            Err(e) => e.to_compile_error(),
        }
    } else {
        quote!(compile_error!("settings!() takes no arguments");)
    }.into()
}
