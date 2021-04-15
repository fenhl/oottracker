#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::collections::HashSet,
    collect_mac::collect,
    enum_iterator::IntoEnumIterator as _,
    itertools::Itertools as _,
    semver::Version,
    ootr::{
        Rando,
        access,
        check::Check,
        item::Item,
        model::{
            DungeonReward,
            DungeonRewardLocation,
        },
    },
    crate::{
        checks::CheckExt as _,
        save::GameMode,
    },
};
pub use crate::{
    knowledge::Knowledge,
    ram::Ram,
    save::Save,
};

pub mod checks;
pub mod firebase;
pub mod info_tables;
mod item_ids;
pub mod knowledge;
pub mod net;
pub mod proto;
pub mod ram;
pub mod region;
pub mod save;
mod scene;
pub mod ui;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ModelState {
    pub knowledge: Knowledge,
    pub ram: Ram,
}

impl ModelState {
    pub fn update_knowledge(&mut self) {
        if self.ram.save.game_mode != GameMode::Gameplay { return } //TODO read knowledge from inventory preview on file select?
        if let Ok(reward) = DungeonReward::into_enum_iter().filter(|reward| self.ram.save.quest_items.has(reward)).exactly_one() {
            self.knowledge.dungeon_reward_locations.insert(reward, DungeonRewardLocation::LinksPocket);
        }
    }

    /// If access depends on other checks (including an event or the value of an unknown setting), those checks are returned.
    pub(crate) fn can_access<'a, R: Rando>(&self, rando: &R, rule: &'a access::Expr<R>) -> Result<bool, HashSet<Check<R>>> {
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
            access::Expr::Event(event) | access::Expr::LitStr(event) => Check::<R>::Event(event.clone()).checked(self).expect(&format!("unimplemented event check: {}", event)),
            access::Expr::HasStones(count) => self.access_expr_le_val(count, self.ram.save.quest_items.num_stones())?,
            access::Expr::Item(item, count) => self.access_expr_le_val(count, self.ram.save.amount_of_item(item))?,
            access::Expr::LogicHelper(helper_name, args) => {
                let helpers = rando.logic_helpers().expect("failed to load logic helpers");
                let (params, helper) = helpers.get(helper_name).expect("no such logic helper");
                self.can_access(rando, &helper.resolve_args(params, args))?
            }
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
            _ => unimplemented!("can_access for {:?}", rule),
        })
    }

    fn access_exprs_eq<'a, R: Rando>(&self, rando: &R, left: &'a access::Expr<R>, right: &'a access::Expr<R>) -> Result<bool, HashSet<Check<R>>> {
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
            (access::Expr::Item(item1, count1), access::Expr::Item(item2, count2)) => item1 == item2 && self.access_exprs_eq(rando, count1, count2)?,
            (access::Expr::Item(item, count), access::Expr::LitStr(s)) |
            (access::Expr::LitStr(s), access::Expr::Item(item, count)) => if self.access_expr_eq_val(count, 1)? {
                *item == Item::from_str(rando, s).expect(&format!("tried to compare item with non-item string literal {}", s))
            } else {
                false // multiple items are never the same as another single item
            },
            (access::Expr::LitInt(n1), access::Expr::LitInt(n2)) => n1 == n2,
            (access::Expr::LitStr(s1), access::Expr::LitStr(s2)) => s1 == s2,
            (access::Expr::LogicHelper(helper_name, args), expr) | (expr, access::Expr::LogicHelper(helper_name, args)) => {
                let helpers = rando.logic_helpers().expect("failed to load logic helpers");
                let (params, helper) = helpers.get(helper_name).expect("no such logic helper");
                self.access_exprs_eq(rando, &helper.resolve_args(params, args), expr)?
            }
            (access::Expr::Setting(setting), access::Expr::LitStr(_)) => return Err(collect![Check::Setting(setting.clone())]), //TODO check knowledge
            (_, _) => unimplemented!("comparison of access expressions {:?} and {:?}", left, right),
        })
    }

    fn access_expr_eq_val<R: Rando>(&self, expr: &access::Expr<R>, value: u8) -> Result<bool, HashSet<Check<R>>> {
        Ok(match expr {
            access::Expr::LitInt(n) => *n == value,
            _ => unimplemented!("access expr {:?} == value", expr),
        })
    }

    fn access_expr_le_val<R: Rando>(&self, expr: &access::Expr<R>, value: u8) -> Result<bool, HashSet<Check<R>>> {
        Ok(match expr {
            access::Expr::LitInt(n) => *n <= value,
            _ => unimplemented!("access expr {:?} <= value", expr),
        })
    }
}

pub fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, oottracker_derive::version!());
    version
}
