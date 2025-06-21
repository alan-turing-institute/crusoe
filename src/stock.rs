use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use thiserror::Error;

use crate::{
    UInt,
    actions::Action,
    config::core_config,
    goods::{Good, GoodsUnit, GoodsUnitLevel, PartialGoodsUnit},
};

#[derive(Error, Debug)]
pub enum StockError {
    #[error("Insufficient stock.")]
    InsufficientStock,
}

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

// TODO: move code relating to tabular RL into a different module.
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

    /// Remove units of a good from the stock.
    pub fn remove(&mut self, goods_unit: &GoodsUnit, quantity: UInt) -> Result<(), StockError> {
        let existing_qty = &self.stock.get(goods_unit);
        match existing_qty.as_ref() {
            Some(&&qty) if qty > quantity => {
                self.stock.insert(*goods_unit, qty - quantity);
                Ok(())
            }
            Some(&&qty) if qty == quantity => {
                self.stock.remove(goods_unit);
                Ok(())
            }
            Some(_) | None => Err(StockError::InsufficientStock),
        }
    }

    /// Remove all existing units of the given good.
    pub fn remove_all(&mut self, good: &Good) {
        self.stock.retain(|k, _| &k.good != good);
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

    pub fn units(&self, good: &Good) -> Vec<GoodsUnit> {
        self.stock
            .iter()
            .filter(|(goods_unit, _)| goods_unit.good == *good)
            .map(|(goods_unit, _)| goods_unit.clone())
            .collect()
    }

    pub fn count_units(&self, good: &Good) -> UInt {
        self.stock
            .iter()
            .filter(|(goods_unit, _)| goods_unit.good == *good)
            .map(|(_, qty)| *qty)
            .sum()
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
            if let Some(new_goods_unit) = goods_unit.step_forward() {
                new_stock.stock.insert(new_goods_unit, *quantity);
            }
        }
        // Degrade all partial goods by 1 time unit, unless they were worked on.
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

    /// Returns a vector of units of capital goods (with quantity held), ordered by their
    /// remaining lifetime.
    pub fn next_capital_goods_units(&self, capital_good: &Good) -> Vec<(&GoodsUnit, &u32)> {
        self.stock
            .iter()
            .filter(|(good, _)| !good.good.is_consumer())
            .filter(|(good, _)| good.good == *capital_good)
            .sorted_by_key(|(good, _)| good.remaining_lifetime)
            .collect()
    }

    pub fn count_material_units(&self, material_good: &Good) -> UInt {
        if !material_good.is_material() {
            return 0;
        }
        self.next_capital_goods_units(material_good)
            .iter()
            .map(|(_, qty)| *qty)
            .sum()
    }

    /// Returns true if the given good is actually used in the given action.
    /// This depends not only on whether the action produces a good that is
    /// produced using the given good, but also on whether all of the other
    /// required inputs are available in the stock to support that production.
    pub fn is_used(&self, good: &Good, action: &Action) -> bool {
        match action {
            Action::ProduceGood(produced_good) => {
                if !produced_good.is_produced_using(good) {
                    return false;
                }
                let required_inputs = produced_good.required_inputs();
                for input in required_inputs {
                    if !self.contains(&input) {
                        return false;
                    }
                }
                true
            }
            Action::Leisure => false,
        }
    }

    /// Consumes units of materials (capital goods) required for the given action by removing them
    /// from the stock. Returns true if sufficient materials were available in the stock to
    /// satisfy the requirements for the action.
    fn consume_material_inputs(&mut self, action: Action) -> Result<(), StockError> {
        // TDOO: Make all of this generic (via good.is_material, etc.) instead of
        // referring to specific goods.
        match action {
            Action::ProduceGood(good) => match good {
                Good::SmokedFish => {
                    // If smoked fish were produced, remove all units of fish from the stock.
                    if self.contains(&Good::Smoker) {
                        self.remove_all(&Good::Fish);
                    }
                    Ok(())
                }
                Good::Smoker | Good::Boat => {
                    // Reduce the stock by 1 unit of Timber (if available).
                    let timber_units = self.next_capital_goods_units(&Good::Timber);
                    let timber_unit = timber_units.iter().next();
                    if timber_unit.is_none() {
                        return Err(StockError::InsufficientStock);
                    }
                    self.remove(&timber_unit.unwrap().0.clone(), 1)?;
                    Ok(())
                }
                _ => Ok(()),
            },
            Action::Leisure => Ok(()),
        }
    }

    // Degrades capital goods used in an action.
    pub fn degrade_capital_stock(&mut self, action: Action) -> Result<(), StockError> {
        // Identify which units of stock were used in production.
        let mut stock_change: Vec<(GoodsUnit, UInt)> = vec![];
        for (&goods_unit, &quantity) in &self.stock {
            let capital_good = goods_unit.good;
            if capital_good.is_consumer() || capital_good.is_material() {
                continue; // Do nothing. Materials were handled above.
            }
            if !self.is_used(&capital_good, &action) {
                continue; // Do nothing. The capital good was not used in the action.
            }
            match action {
                Action::ProduceGood(produced_good) => {
                    match produced_good.is_produced_using(&capital_good) {
                        true => {
                            // Get the next unit of this capital good.
                            let next_units = self.next_capital_goods_units(&capital_good);
                            let next_unit = next_units.iter().next();
                            // If this capital good is a required input but no unit is
                            // available, return an error.
                            match next_unit.is_none() {
                                true => {
                                    if produced_good.required_inputs().contains(&capital_good) {
                                        return Err(StockError::InsufficientStock);
                                    }
                                }
                                false => {} // Do nothing. Production didn't involve this capital good.
                            }
                            let unit = next_unit.unwrap().0.clone();
                            stock_change.push((unit, quantity));
                        }
                        false => {} // Do nothing. Production doesn't depend on this capital good.
                    }
                }
                Action::Leisure => {} // Do nothing.
            }
        }
        // Having determined the changes in the non-material capital stock, consume material
        // inputs. (This must be done *before* actually updating the stock!)
        self.consume_material_inputs(action)?;

        // Update the stock.
        for (goods_unit, qty) in stock_change {
            // Remove the existing capital goods unit.
            self.stock.remove(&goods_unit);
            // If the capital goods unit has non-zero remaining lifetime (after this use), insert a
            // new, single capital goods unit with diminished remaining lifetime.
            if goods_unit.remaining_lifetime > 1 {
                let new_unit = GoodsUnit {
                    good: goods_unit.good,
                    remaining_lifetime: goods_unit.remaining_lifetime - 1,
                };
                self.stock.insert(new_unit, 1);
            }
            // If the quantity was originally greater than one, replace the unused capital goods.
            if qty > 1 {
                self.stock.insert(goods_unit, qty - 1);
            }
        }
        Ok(())
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
        let mut stock = Stock::default();

        // Start with 5 units of berries (lifetime 10) and one basket (lifetime 5).
        stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            5,
        );
        stock.add(
            GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: 5,
            },
            1,
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
        // Check the basket has not been used (this happens in the agent act method).
        assert_eq!(
            stock.stock.get(&GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: 5
            }),
            Some(&1)
        );

        // Test with partial goods.
        let mut stock = Stock::default();

        // Add a smoker with 2 days to completion.
        stock.add_partial(PartialGoodsUnit {
            good: Good::Smoker,
            time_to_completion: 2,
        });
        assert!(!stock.contains(&Good::Smoker));
        assert_eq!(
            stock.get_partial(Good::Smoker).unwrap().time_to_completion,
            2
        );

        // If the action is to continue production of the smoker, the time to completion
        // does not increase overnight.
        stock = stock.step_forward(Action::ProduceGood(Good::Smoker));
        assert!(!stock.contains(&Good::Smoker));
        assert_eq!(
            stock.get_partial(Good::Smoker).unwrap().time_to_completion,
            2
        );

        // If the action does not continue production, the time to completion increases overnight.
        stock = stock.step_forward(Action::Leisure);
        assert!(!stock.contains(&Good::Smoker));
        assert_eq!(
            stock.get_partial(Good::Smoker).unwrap().time_to_completion,
            3
        );

        // The time to completion cannot increase above the amount of time required to produce
        // the good from scratch. Instead it is removed from the stock.
        stock = stock.step_forward(Action::Leisure);
        assert!(!stock.contains(&Good::Smoker));
        assert!(stock.get_partial(Good::Smoker).is_none());
    }

    #[test]
    fn test_material_units() {
        let mut stock = Stock::default();

        let material_good = Good::Timber;
        assert_eq!(stock.count_material_units(&material_good), 0);

        stock.add(GoodsUnit::new(&material_good), 2);
        assert_eq!(stock.count_material_units(&material_good), 2);

        stock.add(GoodsUnit::new(&material_good), 4);
        assert_eq!(stock.count_material_units(&material_good), 6);

        stock.add(
            GoodsUnit {
                good: material_good,
                remaining_lifetime: 1,
            },
            9,
        );
        assert_eq!(stock.count_material_units(&material_good), 15);
    }

    #[test]
    fn test_consume_material_inputs() {
        let mut stock = Stock::default();

        let result = stock.consume_material_inputs(Action::Leisure);
        assert!(result.is_ok());

        let result = stock.consume_material_inputs(Action::ProduceGood(Good::Smoker));
        assert!(result.is_err());

        assert_eq!(stock.count_material_units(&Good::Timber), 0);

        stock.add(GoodsUnit::new(&Good::Timber), 2);

        assert_eq!(stock.count_material_units(&Good::Timber), 2);

        let result = stock.consume_material_inputs(Action::ProduceGood(Good::Smoker));
        assert!(result.is_ok());

        assert_eq!(stock.count_material_units(&Good::Timber), 1);

        let result = stock.consume_material_inputs(Action::ProduceGood(Good::Smoker));
        assert!(result.is_ok());

        assert_eq!(stock.count_material_units(&Good::Timber), 0);

        let result = stock.consume_material_inputs(Action::ProduceGood(Good::Smoker));
        assert!(result.is_err());
    }
}
