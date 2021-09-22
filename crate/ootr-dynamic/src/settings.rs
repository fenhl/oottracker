use {
    std::{
        borrow::Cow,
        collections::hash_map::{
            self,
            HashMap,
        },
    },
    collect_mac::collect,
    pyo3::{
        AsPyPointer as _,
        types::{
            PyBool,
            PyDict,
            PyInt,
            PyList,
            PyString,
            PyType,
        },
    },
    ootr::settings::{
        KnowledgeType,
        KnowledgeTypeError,
        KnowledgeValue
    },
    crate::PyResultExt as _,
};

pub struct Knowledge(pub HashMap<Cow<'static, str>, KnowledgeValue>);

impl<'p> ootr::settings::Knowledge<crate::Rando<'p>> for Knowledge {
    fn default(rando: &crate::Rando<'p>) -> Result<Self, crate::RandoErr> {
        if rando.setting_infos.borrow().is_none() {
            let settings_list = rando.py.import("SettingsList").at("SettingsList")?;
            let mut settings = HashMap::default();
            *rando.setting_infos.borrow_mut() = Some(settings);
        }
        Ok(Self(rando.setting_infos.borrow().as_ref().expect("just inserted").clone()))
    }

    fn get<T: KnowledgeType>(&self, setting: &str) -> Result<Option<T>, KnowledgeTypeError> {
        Ok(if let Some(val) = self.0.get(setting) {
            Some(T::from_any(val)?)
        } else {
            None
        })
    }

    fn update<T: KnowledgeType>(&mut self, setting: &str, value: T) -> Result<(), KnowledgeTypeError> {
        match self.0.entry(Cow::Owned(setting.to_owned())) {
            hash_map::Entry::Occupied(mut entry) => { entry.insert((entry.get().clone() & value.into_any())?); }
            hash_map::Entry::Vacant(entry) => { entry.insert(value.into_any()); }
        }
        Ok(())
    }

    fn remove(&mut self, setting: &str) {
        self.0.remove(setting);
    }
}
