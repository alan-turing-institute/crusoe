use crate::{UInt, actions::Action, stock::Stock};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::stock::RemainingLevel;
use strum::IntoEnumIterator;

type Quantity = UInt;
type Interval = UInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum Good {
    Berries,
    Fish,
    SmokedFish,
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
            Good::SmokedFish => true,
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

    /// Gets the default productivity.
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
                // Note: highest productivity must come first!
                if stock.contains(&Good::Boat) {
                    return Productivity::Immediate(20);
                }
                if stock.contains(&Good::Spear) {
                    return Productivity::Immediate(10);
                }
                Productivity::Immediate(2)
            }
            Good::SmokedFish => match stock.contains(&Good::Smoker) {
                true => {
                    let stock_of_fish = stock.count_units(&Good::Fish);
                    match stock_of_fish > 0 {
                        // Smoked fish are unusual in that productivity is given by
                        // the existing stock of fish.
                        true => Productivity::Immediate(stock_of_fish),
                        false => Productivity::None,
                    }
                }
                false => Productivity::None,
            },
            Good::Spear => Productivity::Immediate(1),
            Good::Smoker => {
                // Productivity of a Smoker is dependent on access to sufficient Timber.
                if stock.count_units(&Good::Timber) >= 3 {
                    return Productivity::Delayed(3);
                }
                Productivity::None
            }
            Good::Boat => {
                // Productivity of a Boat is dependent on access to sufficient Timber.
                if stock.count_units(&Good::Timber) >= 10 {
                    return Productivity::Delayed(10);
                }
                Productivity::None
            }
            Good::Timber => {
                // Productivity of timber is dependent on access to an axe.
                if stock.contains(&Good::Axe) {
                    return Productivity::Immediate(2);
                }
                Productivity::None
            }
            Good::Axe => return Productivity::Delayed(2),
        }
    }

    /// Returns true if this good is produced using the given (higher order) good.
    pub fn is_produced_using(&self, good: &Good) -> bool {
        match self {
            Good::Berries => matches!(good, Good::Basket),
            Good::Fish => match good {
                Good::Spear => true,
                Good::Boat => true,
                _ => false,
            },
            Good::SmokedFish => matches!(good, Good::Smoker),
            Good::Basket => false,
            Good::Spear => false,
            Good::Smoker => matches!(good, Good::Timber),
            Good::Boat => matches!(good, Good::Timber),
            Good::Timber => matches!(good, Good::Axe),
            Good::Axe => false,
        }
    }

    pub fn produces(&self) -> Vec<Good> {
        match self {
            Good::Berries => Vec::new(),
            Good::Fish => Vec::new(),
            Good::SmokedFish => Vec::new(),
            Good::Basket => vec![Good::Berries],
            Good::Spear => vec![Good::Fish],
            Good::Smoker => vec![Good::SmokedFish],
            Good::Boat => vec![Good::Fish],
            Good::Timber => vec![Good::Smoker, Good::Boat],
            Good::Axe => vec![Good::Timber],
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
            Good::SmokedFish => vec![Good::Smoker, Good::Fish],
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
            Good::SmokedFish => None,
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

// For units of goods, each has a lifetime remaining value before it is destroyed.
// For capital goods, (e.g. spear, timber), each has a number of uses remaining before it is destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoodsUnitLevel {
    pub good: Good,
    pub remaining_lifetime: RemainingLevel,
}

impl GoodsUnitLevel {
    pub fn new(good: Good, remaining_lifetime: RemainingLevel) -> Self {
        GoodsUnitLevel {
            good,
            remaining_lifetime,
        }
    }

    pub fn iter() -> impl Iterator<Item = GoodsUnitLevel> {
        Good::iter().flat_map(|good| {
            RemainingLevel::iter().map(move |remaining_lifetime| GoodsUnitLevel {
                good,
                remaining_lifetime,
            })
        })
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
                remaining_lifetime: 2, // Lifetime: day of production plus one more day.
            },
            Good::SmokedFish => GoodsUnit {
                good: Good::SmokedFish,
                remaining_lifetime: 30,
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
                remaining_lifetime: 20,
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
    pub fn step_forward(&self) -> Option<Self> {
        // Degrade the lifetime of consumer goods and materials.
        match self.good.is_consumer() || self.good.is_material() {
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
            // If this is a capital good, return it unchanged.
            false => Some(self.clone()),
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
                time_to_completion: time - 1, // One action has alredy been taken!
            }),
            None => {
                panic!("Expected multiple timesteps to complete a partial good.")
            }
        }
    }

    /// Increments production of this partial goods unit by one timestep and returns
    /// the new partial goods unit with reduced time to completion, or None if
    /// production of the good has been completed.
    pub fn increment_production(mut self) -> Option<PartialGoodsUnit> {
        self.time_to_completion = self.time_to_completion - 1;
        println!("new self.time_to_completion {:?}", self.time_to_completion);
        match self.time_to_completion == 0 {
            true => None,
            false => Some(self),
        }
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

        // If the time to complete has increased to its maximum,
        // remove the partial good altogether.
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

        let good = good.step_forward().unwrap();
        assert_eq!(good.remaining_lifetime, 2);

        let good = good.step_forward().unwrap();
        assert_eq!(good.remaining_lifetime, 1);

        let good = good.step_forward();
        assert!(good.is_none());

        // Test with a material.
        let good = GoodsUnit {
            good: Good::Timber,
            remaining_lifetime: 99,
        };
        assert_eq!(good.remaining_lifetime, 99);

        // Expect "overnight" degradation (similar to consumer goods).
        let good = good.step_forward().unwrap();
        assert_eq!(good.remaining_lifetime, 98);

        let good = good.step_forward().unwrap();
        assert_eq!(good.remaining_lifetime, 97);

        // Test with a non-material capital good.
        let good = GoodsUnit {
            good: Good::Spear,
            remaining_lifetime: 5,
        };
        assert_eq!(good.remaining_lifetime, 5);

        // Expect no "overnight" degradation.
        let good = good.step_forward().unwrap();
        assert_eq!(good.remaining_lifetime, 5);
    }

    #[test]
    fn test_productivity_per_unit_time() {
        let good = Good::Spear;
        let stock = Stock::default();
        assert_eq!(good.default_productivity(&stock).per_unit_time(), Some(1.0));
    }
}
