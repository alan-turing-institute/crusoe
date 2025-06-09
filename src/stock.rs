use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    UInt,
    actions::Action,
    goods::{Good, GoodsUnit},
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stock {
    pub stock: HashMap<GoodsUnit, UInt>,
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
            match goods_unit.good.is_consumer() {
                // If the good exists in the stock and is a consumer good, degrade it.
                // If the remaining_lifetime is 0 (after the step), remove it from the stock.
                true => {
                    if goods_unit.remaining_lifetime > 1 {
                        let new_good = GoodsUnit {
                            good: goods_unit.good,
                            remaining_lifetime: goods_unit.remaining_lifetime - 1,
                        };
                        new_stock.stock.insert(new_good, *quantity);
                    }
                }
                // If the good is a capital good and the action makes use of it,
                // degrade its remaining lifetime.
                // If the new remaining lifetime is zero, remove it from the stock.
                false => match action {
                    Action::ProduceGood(action_good) => {
                        if action_good.is_produced_using(goods_unit.good) {
                            if goods_unit.remaining_lifetime > 1 {
                                let new_good = GoodsUnit {
                                    good: goods_unit.good,
                                    remaining_lifetime: goods_unit.remaining_lifetime - 1,
                                };
                                new_stock.stock.insert(new_good, *quantity);
                            }
                        }
                    }
                    Action::Leisure => {}
                },
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
    }
}
