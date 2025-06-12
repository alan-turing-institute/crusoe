use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    UInt,
    actions::Action,
    config::core_config,
    goods::{Good, GoodsUnit, GoodsUnitLevel, PartialGoodsUnit},
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stock {
    #[serde(serialize_with = "serialize_hm")]
    pub stock: HashMap<GoodsUnit, UInt>,
    pub partial_stock: Vec<PartialGoodsUnit>,
}

fn serialize_hm<S>(hm: &HashMap<GoodsUnit, UInt>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(hm.len()))?;
    for (k, v) in hm.iter() {
        if *v > 0 {
            seq.serialize_element(&(k, v))?;
        }
    }
    seq.end()
}

#[derive(Debug, Copy, Clone, PartialEq, EnumIter, Hash, Eq, Serialize, Deserialize)]
pub enum InvLevel {
    // quantity
    // Critical,
    Low,
    Medium,
    High,
}

#[derive(Debug, Copy, Clone, PartialEq, EnumIter, Hash, Eq, Serialize, Deserialize)]
pub enum RemainingLevel {
    // lifetime
    //Critical,
    Low,
    //Medium,
    // High,
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
                // (
                //     GoodsUnit {
                //         good,
                //         remaining_lifetime,
                //     },
                //     qty,
                // )
                // if *qty < config.agent.inv_level_low => {
                //     ds.insert(
                //         GoodsUnitLevel::new(*good, RemainingLevel::Low),
                //         InvLevel::Critical,
                //     );
                // }
                (
                    GoodsUnit {
                        good,
                        remaining_lifetime,
                    },
                    qty,
                ) if *qty < config.agent.inv_level_med => {
                    ds.insert(
                        GoodsUnitLevel::new(*good, RemainingLevel::Low),
                        InvLevel::Low,
                    );
                }
                (
                    GoodsUnit {
                        good,
                        remaining_lifetime,
                    },
                    qty,
                ) if *qty < config.agent.inv_level_high => {
                    ds.insert(
                        GoodsUnitLevel::new(*good, RemainingLevel::Low),
                        InvLevel::Medium,
                    );
                }
                (
                    GoodsUnit {
                        good,
                        remaining_lifetime,
                    },
                    qty,
                ) => {
                    ds.insert(
                        GoodsUnitLevel::new(*good, RemainingLevel::Low),
                        InvLevel::High,
                    );
                }
            }
        }
        StockDiscrete { stock: ds }
    }
}

impl Stock {
    /// Add units of a good to the stock.
    pub fn add(&mut self, good: GoodsUnit, quantity: UInt) {
        if quantity == 0 {
            panic!("Cannot add a zero quantity of goods to stock.");
        }
        if let Some(existing_qty) = &self.stock.insert(good, quantity) {
            let _ = &self.stock.insert(good, quantity + *existing_qty);
        }
    }

    /// Add a unit of a partially complete good to the stock.
    pub fn add_partial(&mut self, good: PartialGoodsUnit) {
        if let Some(_) = self.get_partial(good.good) {
            panic!("Cannot add multiple partial units of the same good.")
        }
        let _ = &self.partial_stock.push(good);
    }

    /// Remove a units of a good from the stock.
    pub fn remove(&mut self, goods_unit: &GoodsUnit, quantity: UInt) {
        let existing_qty = &self.stock.get(goods_unit);
        match existing_qty.as_ref() {
            Some(&&qty) if qty > quantity => {
                self.stock.insert(*goods_unit, qty - quantity);
            }
            Some(&&qty) if qty == quantity => {
                self.stock.remove(goods_unit);
            }
            Some(_) | None => panic!("Good not found in stock"),
        };

        // if let Some(existing_qty) = &self.stock.insert(good, quantity) {
        //     &self.stock.insert(good, quantity + *existing_qty);
        // }
    }

    /// Remove a units of a good from the stock.
    pub fn remove_partial(&mut self, partial_goods_unit: &PartialGoodsUnit) {
        let index = self
            .partial_stock
            .iter()
            .position(|x| x.good == partial_goods_unit.good)
            .unwrap();
        self.partial_stock.remove(index);
    }

    /// Returns true if the stock contains any units of the given good.
    pub fn contains(&self, good: &Good) -> bool {
        for goods_unit in self.stock.keys() {
            if &goods_unit.good == good {
                return true;
            }
        }
        false
    }

    pub fn goods(&self) -> Vec<Good> {
        Good::iter().filter(|good| self.contains(good)).collect()
    }

    /// Returns a partial unit of the given good, if the stock contains one.
    pub fn get_partial(&self, good: Good) -> Option<PartialGoodsUnit> {
        for partial_unit in &self.partial_stock {
            if partial_unit.good == good {
                return Some(*partial_unit);
            }
        }
        None
    }

    /// Takes in the current action of the agent and updates the stock accordingly.
    pub fn step_forward(&self, action: Action) -> Stock {
        let mut new_stock = Stock::default();
        // Degrade all goods by 1 time unit.
        for (goods_unit, quantity) in &self.stock {
            if let Some(new_goods_unit) = goods_unit.step_forward(action) {
                let mut new_quantity = *quantity;
                // If the goods unit is a material *and* is used by this action,
                // remove one unit of it.
                if goods_unit.good.is_material() {
                    match action {
                        Action::ProduceGood(good) => {
                            if good.is_produced_using(&goods_unit.good) {
                                new_quantity = new_quantity - 1;
                            }
                        }
                        _ => {}
                    }
                }
                // Handle improving capital goods by increasing the remaining lifetime of the
                // goods_unit.
                // TODO: for simplicity, improved goods never expire (as long as the improving
                // capital good is available).
                let mut is_improved_good = false;
                for g in Good::iter() {
                    if goods_unit.good.is_improved_using(&g) && self.contains(&g) {
                        let lifetime_increment = g.lifetime_improvement_increment(&goods_unit.good);
                        let max_lifetime = GoodsUnit::new(&goods_unit.good).remaining_lifetime
                            + lifetime_increment;
                        let new_lifetime = u32::min(
                            goods_unit.remaining_lifetime + lifetime_increment,
                            max_lifetime,
                        );
                        let improved_goods_unit = GoodsUnit {
                            good: goods_unit.good,
                            remaining_lifetime: new_lifetime,
                        };
                        new_stock.stock.insert(improved_goods_unit, new_quantity);
                        is_improved_good = true;
                        break;
                    }
                }
                if !is_improved_good {
                    new_stock.stock.insert(new_goods_unit, new_quantity);
                }
            }
        }
        // Degrade all partial goods by 1 time unit, unless they're finished!
        for partial_goods_unit in &self.partial_stock {
            if let Some(new_partial_goods_unit) = partial_goods_unit.step_forward(action) {
                new_stock.partial_stock.push(new_partial_goods_unit);
            }
        }
        new_stock
    }

    /// Returns a vector of units of consumer goods, ordered by their remaining lifetime.
    pub fn next_consumables(&self) -> Vec<(&GoodsUnit, &u32)> {
        self.stock
            .iter()
            .filter(|(good, _)| good.good.is_consumer())
            .sorted_by_key(|(good, _)| good.remaining_lifetime)
            .collect()
    }

    /// Returns a vector of units of consumer goods, ordered by their remaining lifetime.
    pub fn next_capital_goods_units(&self, capital_good: &Good) -> Vec<(&GoodsUnit, &u32)> {
        self.stock
            .iter()
            .filter(|(good, _)| !good.good.is_consumer())
            .filter(|(good, _)| good.good == *capital_good)
            .sorted_by_key(|(good, _)| good.remaining_lifetime)
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
    fn test_next_consumables() {
        let mut stock = HashMap::<GoodsUnit, UInt>::new();
        // Start with 2 units of berries and 1 unit of fish.
        stock.insert(GoodsUnit::new(&Good::Berries), 2);
        stock.insert(GoodsUnit::new(&Good::Fish), 1);

        let stock = Stock {
            stock: stock,
            partial_stock: vec![],
        };

        let mut result = stock.next_consumables();
        assert_eq!(result.len(), 2);

        let first = result.pop().unwrap();
        assert_eq!(first.0, &GoodsUnit::new(&Good::Berries));
        assert_eq!(first.1, &2);

        let second = result.pop().unwrap();
        assert_eq!(second.0, &GoodsUnit::new(&Good::Fish));
        assert_eq!(second.1, &1);
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
        let stock = Stock {
            stock: stock,
            partial_stock: vec![],
        };

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

        // Test with fish, with and without a smoker.
        // Without smoker:
        let mut stock = HashMap::<GoodsUnit, UInt>::new();
        stock.insert(
            GoodsUnit {
                good: Good::Fish,
                remaining_lifetime: 2,
            },
            5,
        );
        let mut stock = Stock {
            stock: stock,
            partial_stock: vec![],
        };

        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Fish,
                remaining_lifetime: 2
            }),
            Some(&5)
        );

        stock = stock.step_forward(Action::Leisure);

        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Fish,
                remaining_lifetime: 1
            }),
            Some(&5)
        );

        // With smoker:
        let mut stock = HashMap::<GoodsUnit, UInt>::new();
        stock.insert(
            GoodsUnit {
                good: Good::Fish,
                remaining_lifetime: 2,
            },
            5,
        );
        stock.insert(
            GoodsUnit {
                good: Good::Smoker,
                remaining_lifetime: 5,
            },
            1,
        );
        let mut stock = Stock {
            stock: stock,
            partial_stock: vec![],
        };

        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Fish,
                remaining_lifetime: 2
            }),
            Some(&5)
        );

        stock = stock.step_forward(Action::Leisure);

        println!("{:?}", stock);

        // Lifetime of smoked fish is improved by 60 time units.
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Fish,
                remaining_lifetime: 61
            }),
            Some(&5)
        );
    }
}
