#![deny(rust_2018_idioms, unused, unused_import_braces, unused_qualifications, warnings)]

use {
    std::collections::HashSet,
    collect_mac::collect,
    pyo3::prelude::*,
    semver::Version,
    crate::{
        access::{
            Expr,
            Rule,
        },
        checks::CheckExt as _,
    },
};
pub use crate::{
    check::Check,
    item::Item,
    knowledge::Knowledge,
    ram::Ram,
    rando_info::Rando,
    region::Region,
    save::Save,
};

mod access;
mod check;
pub mod checks;
pub mod info_tables;
mod item;
mod item_ids;
pub mod knowledge;
pub mod model;
#[cfg(not(target_arch = "wasm32"))] pub mod proto;
pub mod ram;
mod rando_info;
pub mod region;
pub mod save;
mod scene;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ModelState {
    pub knowledge: Knowledge,
    pub ram: Ram,
}

impl ModelState {
    /// If access depends on other checks (including an event or the value of an unknown setting), those checks are returned.
    pub(crate) fn can_access<'a>(&self, py: Python<'_>, rando: &Rando, rule: &'a Rule) -> Result<bool, HashSet<Check>> {
        Ok(match rule {
            Rule::All(rules) => {
                let mut deps = HashSet::default();
                for rule in rules {
                    match self.can_access(py, rando, rule) {
                        Ok(true) => {}
                        Ok(false) => return Ok(false),
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { true } else { return Err(deps) }
            }
            Rule::Any(rules) => {
                let mut deps = HashSet::default();
                for rule in rules {
                    match self.can_access(py, rando, rule) {
                        Ok(true) => return Ok(true),
                        Ok(false) => {}
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { false } else { return Err(deps) }
            }
            Rule::AnonymousEvent(at_check, id) => Check::AnonymousEvent(Box::new(at_check.clone()), *id).checked(self).expect(&format!("unimplemented anonymous event check: {} for {}", id, at_check)),
            Rule::Eq(left, right) => self.access_exprs_eq(py, rando, left, right)?,
            Rule::Event(event) => Check::Event(event.clone()).checked(self).expect(&format!("unimplemented event check: {}", event)),
            Rule::HasStones(count) => self.ram.save.quest_items.num_stones() >= *count,
            Rule::Item(item, count) => self.ram.save.amount_of_item(item) >= *count,
            Rule::Not(inner) => !self.can_access(py, rando, inner)?,
            Rule::Setting(setting) => if let Some(&setting_value) = self.knowledge.bool_settings.get(setting) {
                setting_value
            } else {
                return Err(collect![Check::Setting(setting.clone())])
            },
            Rule::TrialActive(trial) => if let Some(&trial_active) = self.knowledge.active_trials.get(trial) {
                trial_active
            } else {
                return Err(collect![Check::TrialActive(trial.clone())]) //TODO remove clone call once `trial` is a `Medallion`
            },
            Rule::Trick(trick) => if let Some(trick_value) = self.knowledge.tricks.as_ref().map_or(Some(false), |tricks| tricks.get(trick).copied()) {
                trick_value
            } else {
                return Err(collect![Check::Trick(trick.clone())])
            },
            Rule::Time(range) => self.ram.save.time_of_day.matches(*range), //TODO take location of check into account, as well as available ways to pass time
            Rule::True => true,
        })
    }

    fn access_exprs_eq<'a>(&self, py: Python<'_>, rando: &Rando, left: &'a Expr, right: &'a Expr) -> Result<bool, HashSet<Check>> {
        Ok(match (left, right) {
            (Expr::All(exprs), expr) | (expr, Expr::All(exprs)) => {
                let mut deps = HashSet::default();
                for other in exprs {
                    match self.access_exprs_eq(py, rando, expr, other) {
                        Ok(true) => {}
                        Ok(false) => return Ok(false),
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { true } else { return Err(deps) }
            }
            (Expr::Any(exprs), expr) | (expr, Expr::Any(exprs)) => {
                let mut deps = HashSet::default();
                for other in exprs {
                    match self.access_exprs_eq(py, rando, expr, other) {
                        Ok(true) => return Ok(true),
                        Ok(false) => {}
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { false } else { return Err(deps) }
            }
            (Expr::Age, Expr::LitStr(s)) if s == "child" => !self.ram.save.is_adult,
            (Expr::Age, Expr::LitStr(s)) if s == "adult" => self.ram.save.is_adult,
            (Expr::Age, Expr::StartingAge) => true, // we always assume that we started as the current age, since going to the other age requires finding the Temple of Time first
            (Expr::Item(item1, count1), Expr::Item(item2, count2)) => item1 == item2 && count1 == count2,
            (Expr::Item(item, 1), Expr::LitStr(s)) |
            (Expr::LitStr(s), Expr::Item(item, 1)) => *item == Item::from_str(py, rando, s).expect(&format!("tried to compare item with non-item string literal {}", s)),
            (Expr::Item(_, _), Expr::LitStr(_)) |
            (Expr::LitStr(_), Expr::Item(_, _)) => false, // multiple items are never the same as another single item
            (Expr::LitStr(s1), Expr::LitStr(s2)) => s1 == s2,
            (Expr::Setting(setting), Expr::LitStr(_)) => return Err(collect![Check::Setting(setting.clone())]), //TODO check knowledge
            (_, _) => unimplemented!("comparison of access expressions {:?} and {:?}", left, right),
        })
    }
}

pub fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, oottracker_derive::version!());
    version
}
