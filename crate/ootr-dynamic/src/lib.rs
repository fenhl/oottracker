#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        cell::{
            Ref,
            RefCell,
        },
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
        ops::Deref,
        path::{
            Path,
            PathBuf,
        },
        sync::Arc,
    },
    derive_more::From,
    itertools::Itertools as _,
    pyo3::prelude::*,
    serde::de::DeserializeOwned,
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

pub struct Rando<'p> {
    py: Python<'p>,
    path: PathBuf,
    escaped_items: RefCell<Option<HashMap<String, Item>>>,
    item_table: RefCell<Option<HashMap<String, Item>>>,
    logic_helpers: RefCell<Option<HashMap<String, (Vec<String>, ootr::access::Expr)>>>,
    logic_tricks: RefCell<Option<HashSet<String>>>,
    regions: RefCell<Option<Vec<Region>>>, //TODO glitched support
    setting_infos: RefCell<Option<HashSet<String>>>,
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
            setting_infos: RefCell::default(),
        }
    }

    /// Imports and returns the given Python module from the randomizer codebase.
    fn import(&self, module: &str) -> PyResult<&'p PyModule> {
        let sys = self.py.import("sys")?;
        sys.get("path")?.call_method1("append", (self.path.display().to_string(),))?;
        self.py.import(module)
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

#[derive(Debug, From, Clone)]
pub enum RandoErr {
    #[from]
    AccessExprParse(access::ParseError),
    Io(Arc<io::Error>),
    InvalidLogicHelper,
    ItemNotFound,
    Py(Arc<PyErr>),
    RegionFilename,
}

impl From<io::Error> for RandoErr {
    fn from(e: io::Error) -> RandoErr {
        RandoErr::Io(Arc::new(e))
    }
}

impl From<PyErr> for RandoErr {
    fn from(e: PyErr) -> RandoErr {
        RandoErr::Py(Arc::new(e))
    }
}

impl fmt::Display for RandoErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RandoErr::AccessExprParse(e) => e.fmt(f),
            RandoErr::Io(e) => write!(f, "I/O error: {}", e),
            RandoErr::InvalidLogicHelper => write!(f, "multiple ( found in logic helper"),
            RandoErr::ItemNotFound => write!(f, "no such item"),
            RandoErr::Py(e) => write!(f, "Python error: {}", e),
            RandoErr::RegionFilename => write!(f, "unexpected region filename"),
        }
    }
}

impl ootr::RandoErr for RandoErr {
    const ITEM_NOT_FOUND: RandoErr = RandoErr::ItemNotFound;
}

impl<'p> ootr::Rando for Rando<'p> {
    type Err = RandoErr;

    fn escaped_items<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashMap<String, Item>> + 'a>, RandoErr> {
        if self.escaped_items.borrow().is_none() {
            let items = self.import("RuleParser")?
                .get("escaped_items")?
                .call_method0("items")?
                .iter()?
                .map(|elt| elt
                    .and_then(|elt| elt.extract())
                    .and_then(|(esc_name, item_name)| Ok((esc_name, item_name, self.import("ItemList")?.get("item_table")?.get_item(item_name)?.get_item(0)?.extract::<&str>()?)))
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
                    Err(e) => Some(Err(e.into())),
                })
                .try_collect()?;
            *self.escaped_items.borrow_mut() = Some(items);
        }
        Ok(Box::new(Ref::map(self.escaped_items.borrow(), |items| items.as_ref().expect("just inserted"))))
    }

    fn item_table<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashMap<String, Item>> + 'a>, RandoErr> {
        if self.item_table.borrow().is_none() {
            let items = self.import("ItemList")?
                .get("item_table")?
                .call_method0("items")?
                .iter()?
                .map(|elt| {
                    let (name, (kind, _, _, _)) = elt?.extract::<(String, (String, &PyAny, &PyAny, &PyAny))>()?;
                    PyResult::Ok((name, kind))
                })
                .try_collect::<_, Vec<_>, _>()?
                .into_iter()
                .filter_map(|(name, kind)| if kind != "Event" || name == "Scarecrow Song" { //HACK treat Scarecrow Song as not an event since it's not defined as one in any region
                    Some((name.clone(), Item(name)))
                } else {
                    None
                })
                .collect();
            *self.item_table.borrow_mut() = Some(items);
        }
        Ok(Box::new(Ref::map(self.item_table.borrow(), |items| items.as_ref().expect("just inserted"))))
    }

    fn logic_helpers<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashMap<String, (Vec<String>, ootr::access::Expr)>> + 'a>, RandoErr> {
        if self.logic_helpers.borrow().is_none() {
            let f = File::open(self.path.join("data").join("LogicHelpers.json"))?;
            let mut helpers = HashMap::default();
            for (fn_def, fn_body) in read_json_lenient_sync::<_, BTreeMap<String, String>>(BufReader::new(f))? {
                let (fn_name, fn_params) = if fn_def.contains('(') {
                    fn_def[..fn_def.len() - 1].split('(').collect_tuple().ok_or(RandoErr::InvalidLogicHelper)?
                } else {
                    (&*fn_def, "")
                };
                let fn_params = fn_params.split(',').map(str::to_owned).collect_vec();
                let ctx = Check::LogicHelper(fn_name.to_owned());
                let expr = ootr::access::Expr::parse_helper(self, &ctx, &fn_params, &fn_body)?;
                helpers.insert(fn_name.to_owned(), (fn_params, expr));
            }
            *self.logic_helpers.borrow_mut() = Some(helpers);
        }
        Ok(Box::new(Ref::map(self.logic_helpers.borrow(), |helpers| helpers.as_ref().expect("just inserted"))))
    }

    fn logic_tricks<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashSet<String>> + 'a>, RandoErr> {
        if self.logic_tricks.borrow().is_none() {
            let mut tricks = HashSet::default();
            for trick in self.import("SettingsList")?.get("logic_tricks")?.call_method0("values")?.iter()? {
                tricks.insert(trick?.get_item("name")?.extract()?);
            }
            *self.logic_tricks.borrow_mut() = Some(tricks);
        }
        Ok(Box::new(Ref::map(self.logic_tricks.borrow(), |tricks| tricks.as_ref().expect("just inserted"))))
    }

    fn regions<'a>(&'a self) -> Result<Box<dyn Deref<Target = Vec<Region>> + 'a>, RandoErr> {
        if self.regions.borrow().is_none() {
            let world_path = self.path.join("data").join("World"); //TODO glitched support
            let mut regions = Vec::default();
            for region_path in fs::read_dir(world_path)? {
                let region_path = region_path?;
                let dungeon = parse_dungeon_info(region_path.file_name().to_str().and_then(|filename| filename.strip_suffix(".json")).ok_or(RandoErr::RegionFilename)?)?;
                let region_file = File::open(region_path.path())?;
                for raw_region in read_json_lenient_sync::<_, Vec<RawRegion>>(BufReader::new(region_file))? {
                    let name = raw_region.region_name.clone();
                    regions.push(Region {
                        dungeon,
                        scene: raw_region.scene,
                        hint: raw_region.hint,
                        time_passes: raw_region.time_passes,
                        events: raw_region.events.into_iter().map(|(event_name, rule_str)| Ok::<_, RandoErr>((event_name.clone(), ootr::access::Expr::parse(self, &Check::Event(event_name), rule_str.trim())?))).try_collect()?,
                        locations: raw_region.locations.into_iter().map(|(loc_name, rule_str)| Ok::<_, RandoErr>((loc_name.clone(), ootr::access::Expr::parse(self, &Check::Location(loc_name), rule_str.trim())?))).try_collect()?,
                        exits: raw_region.exits.into_iter().map(|(to, rule_str)| Ok::<_, RandoErr>((to.clone(), ootr::access::Expr::parse(self, &Check::Exit { to, from: name.clone(), from_mq: dungeon.map(|(_, mq)| mq) }, rule_str.trim())?))).try_collect()?,
                        name,
                    });
                }
            }
            *self.regions.borrow_mut() = Some(regions);
        }
        Ok(Box::new(Ref::map(self.regions.borrow(), |regions| regions.as_ref().expect("just inserted"))))
    }

    fn setting_infos<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashSet<String>> + 'a>, RandoErr> {
        if self.setting_infos.borrow().is_none() {
            let mut settings = HashSet::default();
            for setting in self.import("SettingsList")?.get("setting_infos")?.iter()? {
                settings.insert(setting?.getattr("name")?.extract()?);
            }
            *self.setting_infos.borrow_mut() = Some(settings);
        }
        Ok(Box::new(Ref::map(self.setting_infos.borrow(), |settings| settings.as_ref().expect("just inserted"))))
    }
}

fn read_json_lenient_sync<R: BufRead, T: DeserializeOwned>(mut reader: R) -> io::Result<T> {
    let mut buf = String::default();
    let mut line_buf = String::default();
    while reader.read_line(&mut line_buf)? > 0 {
        buf.push_str(
            &line_buf.split('#')
                .next().expect("split always yields at least one element")
                .replace("\r", "")
                .replace('\n', " ")
        );
        line_buf.clear();
    }
    Ok(serde_json::from_str(&buf)?)
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
        rando.setting_infos()?;
        Ok(())
    })
}
