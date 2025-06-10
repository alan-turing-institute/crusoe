use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    UInt,
    actions::Action,
    config::core_config,
    goods::{Good, GoodsUnit, GoodsUnitLevel},
    learning::agent_state::InvLevel,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stock {
    pub stock: HashMap<GoodsUnit, UInt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockDiscrete {
    pub stock: HashMap<GoodsUnitLevel, InvLevel>,
}

impl Stock {
    pub fn discretise(&self) -> StockDiscrete {
        let mut ds = HashMap::new();
        let config = core_config();
        for (goods_unit, quantity) in &self.stock {
            match (goods_unit, quantity) {
                (
                    GoodsUnit {
                        good,
                        remaining_lifetime,
                    },
                    qty,
                ) if *qty < config.agent.inv_level_low => match *remaining_lifetime {
                    x if x < config.agent.inv_level_low => {
                        ds.insert(
                            GoodsUnitLevel::new(*good, InvLevel::Critical),
                            InvLevel::Critical,
                        );
                    }
                    x if x >= config.agent.inv_level_low && x < config.agent.inv_level_med => {
                        ds.insert(
                            GoodsUnitLevel::new(*good, InvLevel::Low),
                            InvLevel::Critical,
                        );
                    }
                    x if x >= config.agent.inv_level_med && x < config.agent.inv_level_high => {
                        ds.insert(
                            GoodsUnitLevel::new(*good, InvLevel::Medium),
                            InvLevel::Critical,
                        );
                    }
                    x if x >= config.agent.inv_level_high => {
                        ds.insert(
                            GoodsUnitLevel::new(*good, InvLevel::High),
                            InvLevel::Critical,
                        );
                    }
                    _ => unreachable!("No other lifetime levels to handle"),
                },
                (
                    GoodsUnit {
                        good,
                        remaining_lifetime,
                    },
                    qty,
                ) if *qty >= config.agent.inv_level_low && *qty < config.agent.inv_level_med => {
                    match *remaining_lifetime {
                        x if x < config.agent.inv_level_low => {
                            ds.insert(
                                GoodsUnitLevel::new(*good, InvLevel::Critical),
                                InvLevel::Low,
                            );
                        }
                        x if x >= config.agent.inv_level_low && x < config.agent.inv_level_med => {
                            ds.insert(GoodsUnitLevel::new(*good, InvLevel::Low), InvLevel::Low);
                        }
                        x if x >= config.agent.inv_level_med && x < config.agent.inv_level_high => {
                            ds.insert(GoodsUnitLevel::new(*good, InvLevel::Medium), InvLevel::Low);
                        }
                        x if x >= config.agent.inv_level_high => {
                            ds.insert(GoodsUnitLevel::new(*good, InvLevel::High), InvLevel::Low);
                        }
                        _ => unreachable!("No other lifetime levels to handle"),
                    }
                }
                (
                    GoodsUnit {
                        good,
                        remaining_lifetime,
                    },
                    qty,
                ) if *qty >= config.agent.inv_level_med && *qty < config.agent.inv_level_high => {
                    match *remaining_lifetime {
                        x if x < config.agent.inv_level_low => {
                            ds.insert(
                                GoodsUnitLevel::new(*good, InvLevel::Critical),
                                InvLevel::Medium,
                            );
                        }
                        x if x >= config.agent.inv_level_low && x < config.agent.inv_level_med => {
                            ds.insert(GoodsUnitLevel::new(*good, InvLevel::Low), InvLevel::Medium);
                        }
                        x if x >= config.agent.inv_level_med && x < config.agent.inv_level_high => {
                            ds.insert(
                                GoodsUnitLevel::new(*good, InvLevel::Medium),
                                InvLevel::Medium,
                            );
                        }
                        x if x >= config.agent.inv_level_high => {
                            ds.insert(GoodsUnitLevel::new(*good, InvLevel::High), InvLevel::Medium);
                        }

                        _ => unreachable!("No other lifetime levels to handle"),
                    }
                }
                (
                    GoodsUnit {
                        good,
                        remaining_lifetime,
                    },
                    qty,
                ) if *qty >= config.agent.inv_level_high => match *remaining_lifetime {
                    x if x < config.agent.inv_level_low => {
                        ds.insert(
                            GoodsUnitLevel::new(*good, InvLevel::Critical),
                            InvLevel::High,
                        );
                    }
                    x if x >= config.agent.inv_level_low && x < config.agent.inv_level_med => {
                        ds.insert(GoodsUnitLevel::new(*good, InvLevel::Low), InvLevel::High);
                    }
                    x if x >= config.agent.inv_level_med && x < config.agent.inv_level_high => {
                        ds.insert(GoodsUnitLevel::new(*good, InvLevel::Medium), InvLevel::High);
                    }
                    x if x >= config.agent.inv_level_high => {
                        ds.insert(GoodsUnitLevel::new(*good, InvLevel::High), InvLevel::High);
                    }
                    _ => unreachable!("No other lifetime levels to handle"),
                },
                _ => unreachable!("No other quantity levels to handle"),
            }
        }
        StockDiscrete { stock: ds }
    }

    // pub fn representation<S, L>(&self) -> ((S, L), (S, L))
    // where
    //     S: std::cmp::Eq + std::hash::Hash + Clone,
    //     L: std::cmp::Eq + std::hash::Hash + Clone,
    // {
    //     let discr = self.discretise();
    //     (
    //         (AgentStateItems::Food, discr.food),
    //         (AgentStateItems::Water, discr.water),
    //     )
    // }
}

impl Stock {
    /// Add units of a good to the stock.
    pub fn add(&mut self, good: GoodsUnit, quantity: UInt) {
        if let Some(existing_qty) = &self.stock.insert(good, quantity) {
            let _ = &self.stock.insert(good, quantity + *existing_qty);
        }
    }

    /// Remove a units of a good from the stock.
    pub fn remove(&mut self, good: &GoodsUnit, quantity: UInt) {
        let existing_qty = &self.stock.get(good);
        match existing_qty.as_ref() {
            Some(&&qty) if qty >= quantity => {
                self.stock.insert(*good, qty - quantity);
            }
            Some(_) | None => panic!("Good not found in stock"),
        };

        // if let Some(existing_qty) = &self.stock.insert(good, quantity) {
        //     &self.stock.insert(good, quantity + *existing_qty);
        // }
    }

    /// Returns true if the stock contains any units of the given good.
    pub fn contains(&self, good: Good) -> bool {
        for (goods_unit, _) in &self.stock {
            if goods_unit.good == good {
                return true;
            }
        }
        false
    }

    /// Takes in the current action of the agent and updates the stock accordingly.
    pub fn step_forward(&self, action: Action) -> Stock {
        let mut new_stock = Stock::default();
        // Degrade all consumer goods by 1 time unit.
        for (goods_unit, quantity) in &self.stock {
            if let Some(new_goods_unit) = goods_unit.step_forward(action) {
                new_stock.stock.insert(new_goods_unit, *quantity);
            }
        }
        new_stock
    }

    /// Returns a vector of units of consumer goods, ordered by their remaining lifetime.
    pub fn next_consumables(&self) -> Vec<(&GoodsUnit, &u32)> {
        self.stock
            .iter()
            .filter(|(good, _)| {
                matches!(
                    **good,
                    GoodsUnit {
                        good: Good::Berries,
                        remaining_lifetime: _
                    }
                )
            })
            .sorted_by_key(|(good, _)| {
                if let GoodsUnit {
                    good: Good::Berries,
                    remaining_lifetime,
                } = good
                {
                    *remaining_lifetime
                } else {
                    unreachable!()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        UInt,
        actions::Action,
        goods::{Good, GoodsUnit},
    };

    #[test]
    fn test_add() {
        let mut stock = Stock::default();
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10
            }),
            None
        );
        stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            2,
        );
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10
            }),
            Some(&2)
        );
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 9
            }),
            None
        );
        stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            3,
        );
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10
            }),
            Some(&5)
        );
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 9
            }),
            None
        );
        stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 9,
            },
            1,
        );
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 9
            }),
            Some(&1)
        );
    }

    #[test]
    fn test_step_forward() {
        let mut stock = HashMap::<GoodsUnit, UInt>::new();
        // Start with 5 units of berries (lifetime 10) and one basket (lifetime 5).
        stock.insert(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            5,
        );
        stock.insert(
            GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: 5,
            },
            1,
        );
        let stock = Stock { stock: stock };

        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10
            }),
            Some(&5)
        );
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: 5
            }),
            Some(&1)
        );
        let stock = stock.step_forward(Action::ProduceGood(Good::Berries));
        // Check the berries have lost one unit of lifetime.
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10
            }),
            None
        );
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 9
            }),
            Some(&5)
        );
        // Check the basket has been used once.
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: 5
            }),
            None
        );
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: 4
            }),
            Some(&1)
        );
        // Test that the basket is not degraded if it is not used in production (e.g. of fish).
        let stock = stock.step_forward(Action::ProduceGood(Good::Fish));
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: 4
            }),
            Some(&1)
        );
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: 3
            }),
            None
        );
    }
}
