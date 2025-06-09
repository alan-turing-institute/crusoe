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
    /// Add a quantify of a good to the stock.
    pub fn add(&mut self, good: GoodsUnit, quantity: UInt) {
        if let Some(existing_qty) = &self.stock.insert(good, quantity) {
            let _ = &self.stock.insert(good, quantity + *existing_qty);
        }
    }

    /// Remove a quantity of a good from the stock.
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
            // If the good exists in the stock and is a consumer good, degrade it.
            // If the remaining_lifetime is 0, remove it from the stock.
            match goods_unit.good {
                Good::Berries => {
                    // TODO: use is_consumer_good() to do this for any consumer good.
                    // If the good is berries, degrade its remaining lifetime.
                    // If the new remaining lifetime is zero, remove it from the stock.
                    if goods_unit.remaining_lifetime > 1 {
                        let new_good = GoodsUnit {
                            good: Good::Berries,
                            remaining_lifetime: goods_unit.remaining_lifetime - 1,
                        };
                        new_stock.stock.insert(new_good, *quantity);
                    }
                }
                Good::Basket => {
                    // TODO: use !is_consumer_good() to do this for any capital good.
                    match action {
                        Action::ProduceGood(good) => {
                            if good.is_produced_using(Good::Basket) {
                                // If the good is a basket and the action uses the basket,
                                // degrade its remaining lifetime.
                                // If the new remaining lifetime is zero, remove it from the stock.
                                if goods_unit.remaining_lifetime > 1 {
                                    let new_good = GoodsUnit {
                                        good: Good::Basket,
                                        remaining_lifetime: goods_unit.remaining_lifetime - 1,
                                    };
                                    new_stock.stock.insert(new_good, *quantity);
                                } // Handle other goods similarly...
                            }
                        }
                        Action::Leisure => {}
                    }
                }
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
        stock.insert(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            5,
        );
        let stock = Stock { stock: stock };

        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10
            }),
            Some(&5)
        );
        let stock = stock.step_forward(Action::ProduceGood(Good::Berries));
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
    }
}
