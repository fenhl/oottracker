use {
    std::{
        io,
        sync::Arc,
    },
    derivative::Derivative,
    ootr::Rando,
    crate::{
        Check,
        ModelState,
        region::RegionLookupError,
    },
};

pub trait CheckExt {
    fn checked(&self, model: &ModelState) -> Option<bool>; //TODO change return type to bool once all used checks are implemented
}

impl CheckExt for Check {
    fn checked(&self, model: &ModelState) -> Option<bool> {
        // event and location lists from Dev-R as of commit b670183e9aff520c20ac2ee65aa55e3740c5f4b4
        if let Some(checked) = model.ram.save.gold_skulltulas.checked(self) { return Some(checked) }
        if let Some(checked) = model.ram.scene_flags().checked(self) { return Some(checked) }
        if let Some(checked) = model.ram.save.event_chk_inf.checked(self) { return Some(checked) }
        if let Some(checked) = model.ram.save.item_get_inf.checked(self) { return Some(checked) }
        if let Some(checked) = model.ram.save.inf_table.checked(self) { return Some(checked) }
        match self {
            Check::Location(loc) => panic!("unknown location name: {loc}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckStatus {
    Checked,
    Reachable,
    NotYetReachable, //TODO split into definitely/possibly/not reachable later in order to determine ALR setting
}

#[derive(Derivative, thiserror::Error)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
pub enum CheckStatusError<R: Rando> {
    #[error("I/O error: {0}")]
    Io(#[from] Arc<io::Error>),
    #[error(transparent)] RegionLookup(#[from] RegionLookupError<R>),
}

impl<R: Rando> From<io::Error> for CheckStatusError<R> { //TODO add support for generics to FromArc derive macro
    fn from(e: io::Error) -> CheckStatusError<R> {
        CheckStatusError::Io(Arc::new(e))
    }
}
