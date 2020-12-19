use pyo3::exceptions::PyValueError;

use {
    pyo3::prelude::*,
    crate::Rando,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item(String);

impl Item {
    pub fn from_str(py: Python<'_>, rando: &Rando, s: &str) -> PyResult<Item> {
        if rando.import(py, "ItemList")?.get("item_table")?.get_item(s)?.get_item(0)?.extract::<&str>()? == "Event" && s != "Scarecrow Song" { // also checks to make sure the item exists //HACK treat Scarecrow Song as not an event since it's not defined as one in any region
            return Err(PyValueError::new_err(format!("The item {:?} is an event", s)))
        }
        Ok(Item(s.to_owned()))
    }

    pub fn name(&self) -> &str { &self.0 }
}
