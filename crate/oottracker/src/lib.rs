#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::collections::HashSet,
    collect_mac::collect,
    semver::Version,
    ootr::{
        Rando,
        access,
        check::Check,
        item::Item,
    },
    crate::checks::CheckExt as _,
};
pub use crate::{
    knowledge::Knowledge,
    ram::Ram,
    save::Save,
};

pub mod checks;
pub mod info_tables;
mod item_ids;
pub mod knowledge;
#[cfg(not(target_arch = "wasm32"))] pub mod proto;
pub mod ram;
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
    pub(crate) fn can_access<'a>(&self, rando: &impl Rando, rule: &'a access::Expr) -> Result<bool, HashSet<Check>> {
        Ok(match rule {
            access::Expr::All(rules) => {
                let mut deps = HashSet::default();
                for rule in rules {
                    match self.can_access(rando, rule) {
                        Ok(true) => {}
                        Ok(false) => return Ok(false),
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { true } else { return Err(deps) }
            }
            access::Expr::Any(rules) => {
                let mut deps = HashSet::default();
                for rule in rules {
                    match self.can_access(rando, rule) {
                        Ok(true) => return Ok(true),
                        Ok(false) => {}
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { false } else { return Err(deps) }
            }
            access::Expr::AnonymousEvent(at_check, id) => Check::AnonymousEvent(Box::new(at_check.clone()), *id).checked(self).expect(&format!("unimplemented anonymous event check: {} for {}", id, at_check)),
            access::Expr::Eq(left, right) => self.access_exprs_eq(rando, left, right)?,
            access::Expr::Event(event) => Check::Event(event.clone()).checked(self).expect(&format!("unimplemented event check: {}", event)),
            access::Expr::HasStones(count) => self.ram.save.quest_items.num_stones() >= *count,
            access::Expr::Item(item, count) => self.ram.save.amount_of_item(item) >= *count,
            access::Expr::Not(inner) => !self.can_access(rando, inner)?,
            access::Expr::Setting(setting) => if let Some(&setting_value) = self.knowledge.bool_settings.get(setting) {
                setting_value
            } else {
                return Err(collect![Check::Setting(setting.clone())])
            },
            access::Expr::TrialActive(trial) => if let Some(&trial_active) = self.knowledge.active_trials.get(trial) {
                trial_active
            } else {
                return Err(collect![Check::TrialActive(trial.clone())]) //TODO remove clone call once `trial` is a `Medallion`
            },
            access::Expr::Trick(trick) => if let Some(trick_value) = self.knowledge.tricks.as_ref().map_or(Some(false), |tricks| tricks.get(trick).copied()) {
                trick_value
            } else {
                return Err(collect![Check::Trick(trick.clone())])
            },
            access::Expr::Time(range) => self.ram.save.time_of_day.matches(*range), //TODO take location of check into account, as well as available ways to pass time
            access::Expr::True => true,
        })
    }

    fn access_exprs_eq<'a>(&self, rando: &impl Rando, left: &'a access::Expr, right: &'a access::Expr) -> Result<bool, HashSet<Check>> {
        Ok(match (left, right) {
            (access::Expr::All(exprs), expr) | (expr, access::Expr::All(exprs)) => {
                let mut deps = HashSet::default();
                for other in exprs {
                    match self.access_exprs_eq(rando, expr, other) {
                        Ok(true) => {}
                        Ok(false) => return Ok(false),
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { true } else { return Err(deps) }
            }
            (access::Expr::Any(exprs), expr) | (expr, access::Expr::Any(exprs)) => {
                let mut deps = HashSet::default();
                for other in exprs {
                    match self.access_exprs_eq(rando, expr, other) {
                        Ok(true) => return Ok(true),
                        Ok(false) => {}
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { false } else { return Err(deps) }
            }
            (access::Expr::Age, access::Expr::LitStr(s)) if s == "child" => !self.ram.save.is_adult,
            (access::Expr::Age, access::Expr::LitStr(s)) if s == "adult" => self.ram.save.is_adult,
            (access::Expr::Age, access::Expr::StartingAge) => true, // we always assume that we started as the current age, since going to the other age requires finding the Temple of Time first
            (access::Expr::ForAge(age1), access::Expr::ForAge(age2)) => age1 == age2,
            (access::Expr::Item(item1, count1), access::Expr::Item(item2, count2)) => item1 == item2 && count1 == count2,
            (access::Expr::Item(item, 1), access::Expr::LitStr(s)) |
            (access::Expr::LitStr(s), access::Expr::Item(item, 1)) => *item == Item::from_str(rando, s).expect(&format!("tried to compare item with non-item string literal {}", s)),
            (access::Expr::Item(_, _), access::Expr::LitStr(_)) |
            (access::Expr::LitStr(_), access::Expr::Item(_, _)) => false, // multiple items are never the same as another single item
            (access::Expr::LitStr(s1), access::Expr::LitStr(s2)) => s1 == s2,
            (access::Expr::Setting(setting), access::Expr::LitStr(_)) => return Err(collect![Check::Setting(setting.clone())]), //TODO check knowledge
            (_, _) => unimplemented!("comparison of access expressions {:?} and {:?}", left, right),
        })
    }
}

pub fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, oottracker_derive::version!());
    version
}
