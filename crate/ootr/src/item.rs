use {
    quote_value::QuoteValue,
    serde::{
        Deserialize,
        Serialize,
    },
    crate::{
        Rando,
        RandoErr as _,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, QuoteValue)]
#[serde(transparent)]
pub struct Item(pub String);

impl Item {
    pub fn from_str<R: Rando>(rando: &R, s: &str) -> Result<Item, R::Err> {
        rando.item_table()?.get(s).cloned().ok_or(R::Err::ITEM_NOT_FOUND)
    }

    pub fn name(&self) -> &str { &self.0 }
}
