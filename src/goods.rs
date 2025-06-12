use crate::{UInt, actions::Action, stock::Stock};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

type Quantity = UInt;
type Interval = UInt;

pub enum Productivity {
    Immediate(Quantity),
    Delayed(Interval),
    None,
}

impl Productivity {
    /// Returns the productivity per unit time.
    pub fn per_unit_time(&self) -> Option<f32> {
        match self {
            Productivity::Immediate(quantity) => Some(*quantity as f32),
            Productivity::Delayed(interval) => Some((1 as f32) / (*interval as f32)),
            Productivity::None => None,
        }
    }
}

// A good in the abstract (as opposed to particular units of a good).
#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
                    if !stock.contains(&required_input) {
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
                if stock.contains(&Good::Basket) {
                    return Productivity::Immediate(8);
                }
                Productivity::Immediate(4)
            }
            Good::Basket => return Productivity::Immediate(1),
            Good::Fish => {
                // Productivity of fish is increased by access to a spear or a boat.
                if stock.contains(&Good::Spear) {
                    return Productivity::Immediate(10);
                }
                if stock.contains(&Good::Boat) {
                    return Productivity::Immediate(20);
                }
                Productivity::Immediate(2)
            }
            Good::Spear => Productivity::Immediate(1),
            Good::Smoker => panic!("Smoker takes multiple timesteps to complete"),
            Good::Boat => panic!("Boat takes multiple timesteps to complete"),
            Good::Timber => {
                // Productivity of timber is dependent on access to an axe.
                if stock.contains(&Good::Axe) {
                    return Productivity::Immediate(2);
                }
                return Productivity::None;
            }
            Good::Axe => panic!("Axe takes multiple timesteps to complete"),
        }
    }

    /// Returns true if this good is produced using the given (higher order) good.
    pub fn is_produced_using(&self, good: &Good) -> bool {
        match self {
            Good::Berries => match good {
                Good::Basket => true,
                _ => false,
            },
            Good::Fish => match good {
                Good::Spear => true,
                Good::Boat => true,
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

    pub fn is_improved_using(&self, good: &Good) -> bool {
        match self {
            Good::Fish => match good {
                Good::Smoker => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_downsteam_of(&self, good: &Good) -> bool {
        self.is_produced_using(good) || self.is_improved_using(good)
    }

    pub fn lifetime_improvement_increment(&self, improved_good: &Good) -> u32 {
        match self {
            // Smoker increases lifetime by 20 time units.
            Good::Smoker => match improved_good {
                Good::Fish => 20,
                _ => 0,
            },
            _ => 0,
        }
    }

    /// Returns the capital goods that are *required* at every timestep to produce this good.
    /// This is distinct from the `is_produced_using` method in that the inputs reported
    /// here are required for production to take place, as opposed to merely productivity-enhancing.
    /// However there is some overlap.
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

    // Hopefully this isn't needed if we adjust the productivity instead.
    // i.e. every consumer good is measured in the same units as the config
    // parameter `daily_nutrition`.
    //
    // // TODO: make sustanance levels configurable.
    // /// Returns the number of units required for 1 day's sustanance.
    // pub fn sustanance(&self) -> Option<UInt> {
    //     match self {
    //         Good::Berries => Some(3),
    //         Good::Fish => Some(1),
    //         Good::Basket => None,
    //         Good::Spear => None,
    //         Good::Smoker => None,
    //         Good::Boat => None,
    //         Good::Timber => None,
    //         Good::Axe => None,
    //     }
    // }
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

    /// Degrade this good by one time step.
    pub fn step_forward(&self, action: Action) -> Option<Self> {
        // Note: handles materials differently. They are capital goods but can only be used once.
        // The remaining_lifetime of a material is its time before expiry (like a consumer good).

        match self.good.is_consumer() {
            // If this good exists in the stock and is a consumer good, degrade it.
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
            // If this is a capital good and the action makes use of it, degrade its
            // remaining lifetime. Otherwise return it unchanged.
            false => match action {
                Action::ProduceGood(produced_good) => {
                    if produced_good.is_produced_using(&self.good) {
                        match &self.good.is_material() {
                            true => {
                                // If the good is a material, and is used in production, it cannot
                                // be used again, so return None.
                                return None;
                            }
                            false => {
                                if self.remaining_lifetime > 1 {
                                    return Some(GoodsUnit {
                                        good: self.good,
                                        remaining_lifetime: self.remaining_lifetime - 1,
                                    });
                                }
                                // If the remaining_lifetime is 0 (after the step), return None.
                                return None;
                            }
                        }
                    }
                    // If the good is a material but is *not* used in production, reduce its
                    // remaining lifetime (as if it were a consumer good).
                    if self.good.is_material() {
                        return Some(GoodsUnit {
                            good: self.good,
                            remaining_lifetime: self.remaining_lifetime - 1,
                        });
                    }
                    // If the capital good is not used in production (as is not a material),
                    // it is unchanged.
                    Some(self.clone())
                }
                Action::Leisure => {
                    match self.good.is_material() {
                        true => {
                            // If the good is a material but is *not* used in production, reduce its
                            // remaining lifetime (as if it were a consumer good).
                            return Some(GoodsUnit {
                                good: self.good,
                                remaining_lifetime: self.remaining_lifetime - 1,
                            });
                        }
                        false => Some(self.clone()),
                    }
                }
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

    // Step forward this partially complete goods unit and penalise any
    // discontinuity in the production process.
    pub fn step_forward(&self, action: Action) -> Option<PartialGoodsUnit> {
        // If the action is to continue production, return the partial good
        // unchanged (as production was incremented when the agent acted).
        match action {
            Action::ProduceGood(good) => {
                if good == self.good {
                    return Some(*self);
                }
            }
            _ => {}
        }
        // If the action is *not* to continue production, extend
        // the remaining time to completion by 1 time unit.
        let time_to_completion = self.time_to_completion + 1;
        let max_time_to_completion = self
            .good
            .multiple_timesteps_to_complete()
            .expect("PartialGoodsUnit must take multiple timesteps to complete");

        if self.time_to_completion == max_time_to_completion {
            return None;
        }
        Some(PartialGoodsUnit {
            good: self.good,
            time_to_completion: time_to_completion,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goods::{Good, GoodsUnit};

    #[test]
    fn test_step_forward() {
        // Test with a consumer good.
        let good = GoodsUnit {
            good: Good::Berries,
            remaining_lifetime: 3,
        };
        assert_eq!(good.remaining_lifetime, 3);

        let action = Action::Leisure;
        let good = good.step_forward(action).unwrap();
        assert_eq!(good.remaining_lifetime, 2);

        let action = Action::ProduceGood(Good::Berries);
        let good = good.step_forward(action).unwrap();
        assert_eq!(good.remaining_lifetime, 1);

        let good = good.step_forward(action);
        assert!(good.is_none());

        // Test with a material.
        let good = GoodsUnit {
            good: Good::Timber,
            remaining_lifetime: 99,
        };
        assert_eq!(good.remaining_lifetime, 99);

        let action = Action::Leisure;
        let good = good.step_forward(action).unwrap();
        assert_eq!(good.remaining_lifetime, 98);

        let action = Action::ProduceGood(Good::Berries);
        let good = good.step_forward(action).unwrap();
        assert_eq!(good.remaining_lifetime, 97);

        let action = Action::ProduceGood(Good::Boat);
        let good = good.step_forward(action);
        assert!(good.is_none());

        // Test with a non-material capital good.
        let good = GoodsUnit {
            good: Good::Spear,
            remaining_lifetime: 5,
        };
        assert_eq!(good.remaining_lifetime, 5);

        let action = Action::Leisure;
        let good = good.step_forward(action).unwrap();
        assert_eq!(good.remaining_lifetime, 5);

        let action = Action::ProduceGood(Good::Berries);
        let good = good.step_forward(action).unwrap();
        assert_eq!(good.remaining_lifetime, 5);

        let action = Action::ProduceGood(Good::Fish);
        let good = good.step_forward(action).unwrap();
        assert_eq!(good.remaining_lifetime, 4);
    }
}
