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
        ffi::OsString,
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
            PathBuf,
            Path,
        },
    },
    itertools::Itertools as _,
    pyo3::prelude::*,
    serde::de::DeserializeOwned,
    crate::{
        Item,
        Region,
    },
};

pub enum RandoInfo {
    //TODO implement static variant
    Dynamic {
        path: PathBuf,
        escaped_items: RefCell<Option<HashMap<String, Item>>>,
        logic_helpers: RefCell<Option<BTreeMap<String, String>>>, //TODO pre-parse?
        logic_tricks: RefCell<Option<HashSet<String>>>,
        region_files: RefCell<Option<HashMap<OsString, Vec<Region>>>>, //TODO glitched support
        setting_infos: RefCell<Option<HashSet<String>>>,
    },
}

impl fmt::Debug for RandoInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RandoInfo::Dynamic { path, .. } => {
                //TODO f.debug_struct("Dynamic").field("path", path).finish_non_exhaustive() (https://github.com/rust-lang/rust/issues/67364)
                write!(f, "Dynamic {{ path: ")?;
                path.fmt(f)?;
                write!(f, ", .. }}")
            }
        }
    }
}

#[derive(Debug)]
pub struct Rando(RandoInfo);

impl Rando {
    pub fn dynamic(path: impl AsRef<Path>) -> Rando {
        Rando(RandoInfo::Dynamic {
            path: path.as_ref().to_owned(),
            escaped_items: RefCell::default(),
            logic_helpers: RefCell::default(),
            logic_tricks: RefCell::default(),
            region_files: RefCell::default(),
            setting_infos: RefCell::default(),
        })
    }

    pub(crate) fn escaped_items(&self) -> PyResult<Ref<'_, HashMap<String, Item>>> {
        match &self.0 {
            RandoInfo::Dynamic { escaped_items, .. } => {
                if escaped_items.borrow().is_none() {
                    *escaped_items.borrow_mut() = Some(Python::with_gil(|py| {
                        let items = self.import(py, "RuleParser")?
                            .get("escaped_items")?
                            .call_method0("items")?
                            .iter()?
                            .map(|elt| elt
                                .and_then(|elt| elt.extract())
                                .and_then(|(esc_name, item_name)| Ok((esc_name, item_name, self.import(py, "ItemList")?.get("item_table")?.get_item(item_name)?.get_item(0)?.extract::<&str>()?)))
                            )
                            .filter_map(|elt| match elt {
                                Ok((esc_name, item_name, kind)) => if kind == "Event" && item_name != "Scarecrow Song" { //HACK treat Scarecrow Song as not an event since it's not defined as one in any region
                                    None
                                } else {
                                    match Item::from_str(py, self, item_name) {
                                        Ok(item) => Some(Ok((esc_name, item))),
                                        Err(e) => Some(Err(e)),
                                    }
                                },
                                Err(e) => Some(Err(e)),
                            })
                            .try_collect()?;
                        PyResult::Ok(items)
                    })?);
                }
                Ok(Ref::map(escaped_items.borrow(), |items| items.as_ref().expect("just inserted")))
            }
        }
    }

    /// Imports and returns the given Python module from the randomizer codebase.
    ///
    /// `rando_path` is the path to the repository of the version of the randomizer you want to use.
    pub(crate) fn import<'p>(&self, py: Python<'p>, module: &str) -> PyResult<&'p PyModule> {
        match &self.0 {
            RandoInfo::Dynamic { path, .. } => {
                let sys = py.import("sys")?;
                sys.get("path")?.call_method1("append", (path.display().to_string(),))?;
                py.import(module)
            }
        }
    }

    pub(crate) fn logic_helpers(&self) -> io::Result<Ref<'_, BTreeMap<String, String>>> {
        match &self.0 {
            RandoInfo::Dynamic { logic_helpers, path, .. } => {
                if logic_helpers.borrow().is_none() {
                    let f = File::open(path.join("data").join("LogicHelpers.json"))?;
                    *logic_helpers.borrow_mut() = Some(read_json_lenient_sync(BufReader::new(f))?);
                }
                Ok(Ref::map(logic_helpers.borrow(), |helpers| helpers.as_ref().expect("just inserted")))
            }
        }
    }

    pub(crate) fn logic_tricks(&self) -> PyResult<Ref<'_, HashSet<String>>> {
        match &self.0 {
            RandoInfo::Dynamic { logic_tricks, .. } => {
                if logic_tricks.borrow().is_none() {
                    let mut tricks = HashSet::default();
                    Python::with_gil(|py| {
                        for trick in self.import(py, "SettingsList")?.get("logic_tricks")?.call_method0("values")?.iter()? {
                            tricks.insert(trick?.get_item("name")?.extract()?);
                        }
                        PyResult::Ok(())
                    })?;
                    *logic_tricks.borrow_mut() = Some(tricks);
                }
                Ok(Ref::map(logic_tricks.borrow(), |tricks| tricks.as_ref().expect("just inserted")))
            }
        }
    }

    pub fn regions(&self) -> io::Result<Ref<'_, HashMap<OsString, Vec<Region>>>> {
        match &self.0 {
            RandoInfo::Dynamic { region_files, path, .. } => {
                if region_files.borrow().is_none() {
                    let world_path = path.join("data").join("World"); //TODO glitched support
                    let mut files = HashMap::default();
                    for region_path in fs::read_dir(world_path)? {
                        let region_path = region_path?;
                        let region_file = File::open(region_path.path())?;
                        files.insert(region_path.file_name(), read_json_lenient_sync(BufReader::new(region_file))?);
                    }
                    *region_files.borrow_mut() = Some(files);
                }
                Ok(Ref::map(region_files.borrow(), |regions| regions.as_ref().expect("just inserted")))
            }
        }
    }

    pub(crate) fn setting_infos(&self) -> PyResult<Ref<'_, HashSet<String>>> {
        match &self.0 {
            RandoInfo::Dynamic { setting_infos, .. } => {
                if setting_infos.borrow().is_none() {
                    let mut settings = HashSet::default();
                    Python::with_gil(|py| {
                        for setting in self.import(py, "SettingsList")?.get("setting_infos")?.iter()? {
                            settings.insert(setting?.getattr("name")?.extract()?);
                        }
                        PyResult::Ok(())
                    })?;
                    *setting_infos.borrow_mut() = Some(settings);
                }
                Ok(Ref::map(setting_infos.borrow(), |settings| settings.as_ref().expect("just inserted")))
            }
        }
    }
}

pub fn read_json_lenient_sync<R: BufRead, T: DeserializeOwned>(mut reader: R) -> io::Result<T> {
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

#[cfg(test)]
mod tests {
    use {
        pin_utils::pin_mut,
        tokio::io::{
            AsyncBufRead,
            AsyncBufReadExt as _,
        },
        super::*,
    };

    async fn read_json_lenient<R: AsyncBufRead, T: DeserializeOwned>(reader: R) -> io::Result<T> {
        pin_mut!(reader);
        let mut buf = String::default();
        let mut line_buf = String::default();
        while reader.read_line(&mut line_buf).await? > 0 {
            buf.push_str(
                &line_buf.split('#')
                    .next().expect("split always yields at least one element")
                    .replace("\r", "")
                    .replace('\n', " ")
            );
            line_buf.clear();
        }
        Ok(serde_json::from_str(&buf)?) //TODO use async-json instead
    }

    #[tokio::test]
    async fn read_world_files() -> io::Result<()> {
        let f = tokio::fs::File::open("C:\\Users\\Fenhl\\git\\github.com\\fenhl\\OoT-Randomizer\\stage\\data\\World\\Overworld.json").await?; //TODO use configurable rando path
        read_json_lenient::<_, Vec<serde_json::Value>>(tokio::io::BufReader::new(f)).await?;
        Ok(())
    }

    #[test]
    fn read_world_files_sync() -> io::Result<()> {
        let f = std::fs::File::open("C:\\Users\\Fenhl\\git\\github.com\\fenhl\\OoT-Randomizer\\stage\\data\\World\\Overworld.json")?; //TODO use configurable rando path
        read_json_lenient_sync::<_, Vec<serde_json::Value>>(std::io::BufReader::new(f))?;
        Ok(())
    }
}
