use serde::{Deserialize, Serialize};

use crate::{UInt, actions::Action, stock::Stock};

type Quantity = UInt;
type Interval = UInt;

pub enum Productivity {
    Immediate(Quantity),
    Delayed(Interval),
    None,
}

// A good in the abstract (as opposed to particular units of a good).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Good {
    Berries,
    Fish,
    Basket,
    Spear,
    Smoker,
    Boat,
    Timber,
    Axe,
}

impl Good {
    pub fn is_consumer(&self) -> bool {
        match self {
            Good::Berries => true,
            Good::Fish => true,
            Good::Basket => false,
            Good::Spear => false,
            Good::Smoker => false,
            Good::Boat => false,
            Good::Timber => false,
            Good::Axe => false,
        }
    }

    /// Returns true if this is a capital good that is a material.
    /// Materials are used up when used to produce lower order goods.
    pub fn is_material(&self) -> bool {
        match self {
            Good::Timber => true,
            _ => false,
        }
    }

    /// Gets the default productivity
    /// **FOR GOODS THAT DO NOT REQUIRE MULTIPLE TIMESTEPS TO COMPLETE**.
    pub fn default_productivity(&self, stock: &Stock) -> Productivity {
        match self.multiple_timesteps_to_complete() {
            Some(time_to_complete) => {
                // If a good takes multiple timesteps to complete and requires inputs
                // at every timestep, the productivity is zero unless they already
                // have enough input materials.
                for required_input in self.required_inputs() {
                    // IMP TODO: check sufficient quantity of inputs.
                    if !stock.contains(required_input) {
                        return Productivity::None;
                    }
                }
                return Productivity::Delayed(time_to_complete);
            }
            None => {} // Do nothing. Contiue to following match statement.
        }
        match self {
            Good::Berries => {
                // Productivity of berries is increased by access to a basket.
                if stock.contains(Good::Basket) {
                    return Productivity::Immediate(8);
                }
                Productivity::Immediate(4)
            }
            Good::Basket => return Productivity::Immediate(1),
            Good::Fish => {
                // Productivity of fish is increased by access to a spear.
                if stock.contains(Good::Spear) {
                    return Productivity::Immediate(10);
                }
                Productivity::Immediate(2)
            }
            Good::Spear => Productivity::Immediate(1),
            Good::Smoker => panic!("Smoker takes multiple timesteps to complete"),
            Good::Boat => panic!("Boat takes multiple timesteps to complete"),
            Good::Timber => {
                // Productivity of timber is dependent on access to an axe.
                if stock.contains(Good::Axe) {
                    return Productivity::Immediate(2);
                }
                return Productivity::None;
            }
            Good::Axe => panic!("Axe takes multiple timesteps to complete"),
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
            Good::Smoker => match good {
                Good::Timber => true,
                _ => false,
            },
            Good::Boat => match good {
                Good::Timber => true,
                _ => false,
            },
            Good::Timber => match good {
                Good::Axe => true,
                _ => false,
            },
            Good::Axe => false,
        }
    }

    /// Returns the capital goods that are required at every timestep to produce this good.
    pub fn required_inputs(&self) -> Vec<Good> {
        match self {
            Good::Berries => Vec::new(),
            Good::Fish => Vec::new(),
            Good::Basket => Vec::new(),
            Good::Spear => Vec::new(),
            Good::Smoker => vec![Good::Timber],
            Good::Boat => vec![Good::Timber],
            Good::Timber => vec![Good::Axe],
            Good::Axe => Vec::new(),
        }
    }

    // TODO: This should be different for each agent,
    // i.e. agents could have different rates of productivity
    // for capital goods, as they do for consumer goods.
    /// Returns the number of timesteps taken to complete production of this good,
    /// or None if one or more units of the good can be produced in one time unit.
    pub fn multiple_timesteps_to_complete(&self) -> Option<UInt> {
        match self {
            Good::Berries => None,
            Good::Fish => None,
            Good::Basket => None,
            Good::Spear => None,
            Good::Smoker => Some(3),
            Good::Boat => Some(10),
            Good::Timber => None,
            Good::Axe => Some(2),
        }
    }
}

// For units of goods, each has a lifetime remaining value before it is destroyed.
// For capital goods, (e.g. spear, timber), each has a number of uses remaining before it is destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoodsUnit {
    pub good: Good,
    pub remaining_lifetime: UInt, // interpreted as remaining uses for capital goods.
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
            Good::Smoker => GoodsUnit {
                good: Good::Smoker,
                remaining_lifetime: 5,
            },
            Good::Boat => GoodsUnit {
                good: Good::Boat,
                remaining_lifetime: 20,
            },
            Good::Timber => GoodsUnit {
                good: Good::Timber,
                remaining_lifetime: 1000,
            },
            Good::Axe => GoodsUnit {
                good: Good::Axe,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartialGoodsUnit {
    pub good: Good,
    pub time_to_completion: UInt, // Number of days required to complete production.
}

impl PartialGoodsUnit {
    /// Returns a partially complete unit of a good.
    pub fn new(good: &Good) -> Option<Self> {
        match good.multiple_timesteps_to_complete() {
            Some(time) => Some(PartialGoodsUnit {
                good: good.clone(),
                time_to_completion: time,
            }),
            None => None,
        }
    }

    pub fn increment_production(&mut self) {
        self.time_to_completion = self.time_to_completion - 1
    }

    // TODO NEXT: Discontinuities in production must be penalised!
    // pub fn step_forward(&mut self, action: Action) -> Option<Self> {}
}
