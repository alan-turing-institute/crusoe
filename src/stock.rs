use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{UInt, actions::Action, goods::Good};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stock {
    pub stock: HashMap<Good, UInt>,
}

impl Stock {
    /// Add a quantify of a good to the stock.
    pub fn add(&mut self, good: Good, quantity: UInt) {
        if let Some(existing_qty) = &self.stock.insert(good, quantity) {
            let _ = &self.stock.insert(good, quantity + *existing_qty);
        }
    }

    /// Remove a quantity of a good from the stock.
    pub fn remove(&mut self, good: &Good, quantity: UInt) {
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

    /// Takes in the current action of the agent and updates the stock accordingly.
    fn step_forward(&self, action: Action) -> Stock {
        let mut new_stock = Stock::default();
        // Degrade all consumer goods by 1 time unit.
        for (good, quantity) in &self.stock {
            // If the good exists in the stock and is a consumer good, degrade it.
            // If the remaining_lifetime is 0, remove it from the stock.
            match good {
                Good::Berries { remaining_lifetime } => {
                    // If the good is berries, degrade its remaining lifetime.
                    if *remaining_lifetime > 0 {
                        let new_good = Good::Berries {
                            remaining_lifetime: *remaining_lifetime - 1,
                        };
                        new_stock.stock.insert(new_good, *quantity);
                    } // Handle other goods similarly...
                }
            }
            // TODO: Reduce remaining uses of any capital goods involved in the action.
        }
        new_stock
    }

    pub fn next_consumable(&self) -> Option<(&Good, &u32)> {
        self.stock
            .iter()
            .filter(|(good, _)| {
                matches!(
                    **good,
                    Good::Berries {
                        remaining_lifetime: _
                    }
                )
            })
            .sorted_by_key(|(good, _)| {
                if let Good::Berries { remaining_lifetime } = good {
                    *remaining_lifetime
                } else {
                    unreachable!()
                }
            })
            .next()
    }

    /// Returns a vector of units of consumer goods, ordered by their remaining lifetime.
    pub fn next_consumables(&self) -> Vec<(&Good, &u32)> {
        self.stock
            .iter()
            .filter(|(good, _)| {
                matches!(
                    **good,
                    Good::Berries {
                        remaining_lifetime: _
                    }
                )
            })
            .sorted_by_key(|(good, _)| {
                if let Good::Berries { remaining_lifetime } = good {
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

    #[test]
    fn test_add() {
        let mut stock = Stock::default();
        assert_eq!(
            stock.stock.get(&Good::Berries {
                remaining_lifetime: 10
            }),
            None
        );
        stock.add(
            Good::Berries {
                remaining_lifetime: 10,
            },
            2,
        );
        assert_eq!(
            stock.stock.get(&Good::Berries {
                remaining_lifetime: 10
            }),
            Some(&2)
        );
        assert_eq!(
            stock.stock.get(&Good::Berries {
                remaining_lifetime: 9
            }),
            None
        );
        stock.add(
            Good::Berries {
                remaining_lifetime: 10,
            },
            3,
        );
        assert_eq!(
            stock.stock.get(&Good::Berries {
                remaining_lifetime: 10
            }),
            Some(&5)
        );
        assert_eq!(
            stock.stock.get(&Good::Berries {
                remaining_lifetime: 9
            }),
            None
        );
        stock.add(
            Good::Berries {
                remaining_lifetime: 9,
            },
            1,
        );
        assert_eq!(
            stock.stock.get(&Good::Berries {
                remaining_lifetime: 9
            }),
            Some(&1)
        );
    }
}
