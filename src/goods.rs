use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{UInt, actions::Action, learning::agent_state::InvLevel};

// A good in the abstract (as opposed to particular units of a good).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum Good {
    Berries,
    Fish,
    Basket,
    Spear,
    // Smoker,
    // Boat,
    // Timber,
    // Axe,
}

// For units of goods, each has a lifetime remaining value before it is destroyed.
// For capital goods, (e.g. spear, timber), each has a number of uses remaining before it is destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoodsUnit {
    pub good: Good,
    pub remaining_lifetime: UInt, // interpreted as remaining uses for capital goods.
}

// For units of goods, each has a lifetime remaining value before it is destroyed.
// For capital goods, (e.g. spear, timber), each has a number of uses remaining before it is destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoodsUnitLevel {
    pub good: Good,
    pub remaining_lifetime: InvLevel,
}

impl GoodsUnitLevel {
    pub fn new(good: Good, remaining_lifetime: InvLevel) -> Self {
        GoodsUnitLevel {
            good,
            remaining_lifetime,
        }
    }
}

impl GoodsUnit {
    /// Returns a newly-produced unit of the given good.
    pub fn new(good: &Good) -> Self {
        match good {
            Good::Berries => GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            Good::Fish => GoodsUnit {
                good: Good::Fish,
                remaining_lifetime: 1,
            },
            Good::Basket => GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: 10,
            },
            Good::Spear => GoodsUnit {
                good: Good::Spear,
                remaining_lifetime: 5,
            },
        }
    }

    pub fn step_forward(&self, action: Action) -> Option<Self> {
        match self.good.is_consumer() {
            // If the good exists in the stock and is a consumer good, degrade it.
            true => {
                if self.remaining_lifetime > 1 {
                    return Some(GoodsUnit {
                        good: self.good,
                        remaining_lifetime: self.remaining_lifetime - 1,
                    });
                }
                // If the remaining_lifetime is 0 (after the step), return None.
                None
            }
            // If the good is a capital good and the action makes use of it,
            // degrade its remaining lifetime. Otherwise return it unchanged.
            false => match action {
                Action::ProduceGood(produced_good) => {
                    if produced_good.is_produced_using(self.good) {
                        if self.remaining_lifetime > 1 {
                            return Some(GoodsUnit {
                                good: self.good,
                                remaining_lifetime: self.remaining_lifetime - 1,
                            });
                        }
                        // If the remaining_lifetime is 0 (after the step), return None.
                        return None;
                    }
                    Some(self.clone())
                }
                Action::Leisure => Some(self.clone()),
            },
        }
    }
}

impl Good {
    pub fn is_consumer(&self) -> bool {
        match self {
            Good::Berries => true,
            Good::Fish => true,
            Good::Basket => false,
            Good::Spear => false,
        }
    }

    /// Returns true if this good is produced using the given (higher order) good.
    pub fn is_produced_using(&self, good: Good) -> bool {
        match self {
            Good::Berries => match good {
                Good::Basket => true,
                _ => false,
            },
            Good::Fish => match good {
                Good::Spear => true,
                _ => false,
            },
            Good::Basket => false,
            Good::Spear => false,
        }
    }
}
