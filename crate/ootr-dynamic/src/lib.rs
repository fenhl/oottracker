#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        borrow::Cow,
        cell::RefCell,
        collections::{
            BTreeMap,
            HashMap,
            HashSet,
        },
        fmt,
        fs::{
            self,
            File,
        },
        io::{
            self,
            BufRead,
            BufReader,
        },
        path::{
            Path,
            PathBuf,
        },
        sync::Arc,
    },
    derive_more::From,
    itertools::Itertools as _,
    pyo3::{
        PyDowncastError,
        prelude::*,
    },
    semver::Version,
    serde::de::DeserializeOwned,
    wheel::FromArc,
    ootr::{
        check::Check,
        item::Item,
        region::Region,
    },
    crate::{
        access::ExprExt as _,
        region::{
            RawRegion,
            parse_dungeon_info,
        },
    },
};

mod access;
mod region;
pub mod settings;

pub struct Rando<'p> {
    py: Python<'p>,
    path: PathBuf,
    escaped_items: RefCell<Option<Arc<HashMap<String, Item>>>>,
    item_table: RefCell<Option<Arc<HashMap<String, Item>>>>,
    logic_helpers: RefCell<Option<Arc<HashMap<String, (Vec<String>, ootr::access::Expr<Rando<'p>>)>>>>,
    logic_tricks: RefCell<Option<Arc<HashSet<String>>>>,
    regions: RefCell<Option<Arc<Vec<Arc<Region<Self>>>>>>, //TODO glitched support
    setting_names: RefCell<Option<Arc<HashMap<String, String>>>>,
    pub(crate) setting_infos: RefCell<Option<HashMap<Cow<'static, str>, ootr::settings::KnowledgeValue>>>,
}

impl<'p> Rando<'p> {
    pub fn new(py: Python<'p>, path: impl AsRef<Path>) -> Rando<'p> {
        Rando {
            py,
            path: path.as_ref().to_owned(),
            escaped_items: RefCell::default(),
            item_table: RefCell::default(),
            logic_helpers: RefCell::default(),
            logic_tricks: RefCell::default(),
            regions: RefCell::default(),
            setting_names: RefCell::default(),
            setting_infos: RefCell::default(),
        }
    }
}

impl<'p> fmt::Debug for Rando<'p> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //TODO f.debug_struct("Rando").field("path", path).finish_non_exhaustive() (https://github.com/rust-lang/rust/issues/67364)
        write!(f, "Rando {{ path: ")?;
        self.path.fmt(f)?;
        write!(f, ", .. }}")
    }
}

#[derive(Debug, From, FromArc, Clone)]
pub enum RandoErr {
    #[from]
    AccessExprParse(access::ParseError),
    #[from_arc]
    Io(Arc<io::Error>),
    InvalidLogicHelper,
    ItemNotFound,
    MissingIntSettingBound,
    NonJsonRegionFile(String),
    NonUnicodeRegionFilename,
    //#[from_arc]
    //Py(Arc<PyErr>),
    Py(String, Arc<PyErr>),
    UnknownRegionFilename(String),
}

impl From<PyDowncastError<'_>> for RandoErr {
    fn from(e: PyDowncastError<'_>) -> Self {
        Self::Py(format!("downcast"), Arc::new(e.into()))
    }
}

impl fmt::Display for RandoErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RandoErr::AccessExprParse(e) => e.fmt(f),
            RandoErr::Io(e) => write!(f, "I/O error: {}", e),
            RandoErr::InvalidLogicHelper => write!(f, "multiple ( found in logic helper"),
            RandoErr::ItemNotFound => write!(f, "no such item"),
            RandoErr::MissingIntSettingBound => write!(f, "fount a setting of type `int` with no min or no max"),
            RandoErr::NonJsonRegionFile(name) => write!(f, "expected region filename ending in .json but found {}", name),
            RandoErr::NonUnicodeRegionFilename => write!(f, "non-Unicode region filename"),
            RandoErr::Py(loc, e) => write!(f, "Python error in {}: {}", loc, e),
            RandoErr::UnknownRegionFilename(name) => write!(f, "unexpected region filename: {}", name),
        }
    }
}

trait PyResultExt {
    type Ok;

    fn at(self, loc: &str) -> Result<Self::Ok, RandoErr>;
}

impl<T> PyResultExt for PyResult<T> {
    type Ok = T;

    fn at(self, loc: &str) -> Result<T, RandoErr> {
        self.map_err(|e| RandoErr::Py(loc.to_owned(), Arc::new(e)))
    }
}

impl ootr::RandoErr for RandoErr {
    const ITEM_NOT_FOUND: RandoErr = RandoErr::ItemNotFound;
}

impl<'p> ootr::Rando for Rando<'p> {
    type Err = RandoErr;
    type RegionName = String;
    type SettingsKnowledge = settings::Knowledge;

    fn escaped_items(&self) -> Result<Arc<HashMap<String, Item>>, RandoErr> {
        if self.escaped_items.borrow().is_none() {
            let items = self.import("RuleParser").at("RuleParser")?
                .getattr("escaped_items").at("escaped_items")?
                .call_method0("items").at("items")?
                .iter().at("items")?
                .map(|elt| elt
                    .and_then(|elt| elt.extract())
                    .and_then(|(esc_name, item_name)| Ok((esc_name, item_name, self.import("ItemList")?.getattr("item_table")?.get_item(item_name)?.get_item(0)?.extract::<&str>()?)))
                )
                .filter_map(|elt| match elt {
                    Ok((esc_name, item_name, kind)) => if kind == "Event" && item_name != "Scarecrow Song" { //HACK treat Scarecrow Song as not an event since it's not defined as one in any region
                        None
                    } else {
                        match Item::from_str(self, item_name) {
                            Ok(item) => Some(Ok((esc_name, item))),
                            Err(e) => Some(Err(e)),
                        }
                    },
                    Err(e) => Some(Err(e).at("items")),
                })
                .try_collect()?;
            *self.escaped_items.borrow_mut() = Some(Arc::new(items));
        }
        Ok(Arc::clone(self.escaped_items.borrow().as_ref().expect("just inserted")))
    }

    fn item_table(&self) -> Result<Arc<HashMap<String, Item>>, RandoErr> {
        if self.item_table.borrow().is_none() {
            let items = self.import("ItemList").at("ItemList")?
                .getattr("item_table").at("item_table")?
                .call_method0("items").at("items")?
                .iter().at("items")?
                .map(|elt| {
                    let (name, (kind, _, _, _)) = elt?.extract::<(String, (String, &PyAny, &PyAny, &PyAny))>()?;
                    PyResult::Ok((name, kind))
                })
                .try_collect::<_, Vec<_>, _>().at("items")?
                .into_iter()
                .filter_map(|(name, kind)| if kind != "Event" || name == "Scarecrow Song" { //HACK treat Scarecrow Song as not an event since it's not defined as one in any region
                    Some((name.clone(), Item(name)))
                } else {
                    None
                })
                .collect();
            *self.item_table.borrow_mut() = Some(Arc::new(items));
        }
        Ok(Arc::clone(self.item_table.borrow().as_ref().expect("just inserted")))
    }

    fn logic_helpers(&self) -> Result<Arc<HashMap<String, (Vec<String>, ootr::access::Expr<Rando<'p>>)>>, RandoErr> {
        if self.logic_helpers.borrow().is_none() {
            let f = File::open(self.path.join("data").join("LogicHelpers.json"))?;
            let raw_helpers = read_json_lenient_sync::<_, BTreeMap<String, String>>(BufReader::new(f))?;
            let mut helper_headers = HashMap::new();
            for (fn_def, fn_body) in &raw_helpers {
                let (fn_name, fn_params) = if fn_def.contains('(') {
                    fn_def[..fn_def.len() - 1].split('(').collect_tuple().ok_or(RandoErr::InvalidLogicHelper)?
                } else {
                    (&**fn_def, "")
                };
                let fn_params = if fn_params.is_empty() {
                    Vec::default()
                } else {
                    fn_params.split(',').map(str::to_owned).collect_vec()
                };
                helper_headers.insert(fn_name.to_owned(), (fn_params, fn_body));
            }
            let arities = helper_headers.iter().map(|(fn_name, (fn_params, _))| (&**fn_name, fn_params.len())).collect();
            let mut helpers = HashMap::default();
            for (fn_name, (fn_params, fn_body)) in &helper_headers {
                let ctx = Check::LogicHelper(fn_name.to_owned());
                let expr = ootr::access::Expr::parse_helper(self, &ctx, &arities, &fn_params, &fn_body)?;
                helpers.insert(fn_name.to_owned(), (fn_params.clone(), expr));
            }
            *self.logic_helpers.borrow_mut() = Some(Arc::new(helpers));
        }
        Ok(Arc::clone(self.logic_helpers.borrow().as_ref().expect("just inserted")))
    }

    fn logic_tricks(&self) -> Result<Arc<HashSet<String>>, RandoErr> {
        if self.logic_tricks.borrow().is_none() {
            let mut tricks = HashSet::default();
            for trick in self.import("SettingsList").at("SettingsList")?.getattr("logic_tricks").at("logic_tricks")?.call_method0("values").at("values")?.iter().at("values")? {
                tricks.insert(trick.at("values")?.get_item("name").at("name")?.extract().at("name")?);
            }
            *self.logic_tricks.borrow_mut() = Some(Arc::new(tricks));
        }
        Ok(Arc::clone(self.logic_tricks.borrow().as_ref().expect("just inserted")))
    }

    fn setting_names(&self) -> Result<Arc<HashMap<String, String>>, RandoErr> {
        if self.setting_names.borrow().is_none() {
            let mut settings = HashMap::default();
            for setting in self.import("SettingsList").at("SettingsList")?.getattr("setting_infos").at("setting_infos")?.iter().at("setting_infos")? {
                let setting = setting.at("setting_infos")?;
                let name = setting.getattr("name").at("name")?.extract::<String>().at("name")?;
                if settings::ignore_setting(&name) { continue }
                settings.insert(name.clone(), setting.getattr("gui_text").at("gui_text getattr")?.extract().at(&format!("gui_text extract for {}", name))?);
            }
            *self.setting_names.borrow_mut() = Some(Arc::new(settings));
        }
        Ok(Arc::clone(self.setting_names.borrow().as_ref().expect("just inserted")))
    }
}

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}

#[test]
fn load_rando_data() -> Result<(), RandoErr> {
    use ootr::Rando as _;

    Python::with_gil(|py| {
        let rando = Rando::new(py, "C:\\Users\\fenhl\\AppData\\Local\\Fenhl\\RSL\\cache\\ootr-latest");
        rando.escaped_items()?;
        rando.item_table()?;
        rando.logic_helpers()?;
        rando.logic_tricks()?;
        rando.regions()?;
        rando.setting_names()?;
        Ok(())
    })
}
