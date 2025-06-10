use enum_dispatch::enum_dispatch;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::actions::Action;
use crate::goods::{Good, GoodsUnit, PartialGoodsUnit, Productivity};
use crate::stock::Stock;
use crate::{Int, UInt};

#[enum_dispatch]
pub trait Agent {
    fn get_id(&self) -> u64;
    fn get_name(&self) -> &str;
    /// The stock of goods currently held by the agent.
    fn stock(&self) -> &Stock;
    /// The marginal productivity of the agent, given their current stock.
    fn productivity(&self, good: Good) -> Productivity;
    // fn productivity(&self, good: Good) -> (UInt, bool);
    /// The agent's choice of action in the next time step.
    fn choose_action(&mut self) -> Action;
    /// Consume nutritional units for one time step and return false if insufficient were unavailable.
    fn consume(&mut self, nutritional_units: UInt) -> bool;
    /// Get the complete history of agent actions.
    fn action_history(&self) -> Vec<Action>;
    /// Get the complete history of agent stocks.
    fn stock_history(&self) -> Vec<Stock>;
    /// Return true if the agent is still alive.
    fn is_alive(&self) -> bool;
    /// Execture the given action.
    fn act(&mut self, action: Action);
    /// Step the agent forward by one time step.
    fn step_forward(&mut self);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrusoeAgent {
    id: u64,
    stock: Stock,
    is_alive: bool,
    action_history: Vec<Action>,
    stock_history: Vec<Stock>,
}

impl CrusoeAgent {
    pub fn new(id: u64) -> Self {
        CrusoeAgent {
            id,
            stock: Stock::default(),
            is_alive: true,
            action_history: vec![],
            stock_history: vec![],
        }
    }
}

impl Agent for CrusoeAgent {
    fn get_id(&self) -> u64 {
        todo!()
    }

    fn get_name(&self) -> &str {
        "Crusoe"
    }

    fn stock(&self) -> &Stock {
        &self.stock
    }

    // TODO: Some capital goods take multiple time units to produce.
    /// Returns the number of units of the good produced per day, given the
    /// agent's existing stock. The boolean flag indicates whether the
    /// integer in the first slot should be interpreted as a fraction.
    /// e.g. (5, false) indicates  5 units can be produced in one day,
    /// while (5, true) indicates that a single unit takes 5 days to produce.
    fn productivity(&self, good: Good) -> Productivity {
        // TODO: make configurable.
        // TODO: can modify default productivity for different agents (for specialisation).
        good.default_productivity(&self.stock)
    }

    // TODO: consider moving teh action_history update into act method, so
    // self can be immutable here.
    fn choose_action(&mut self) -> Action {
        // let action = Action::random_weighted(&mut StdRng::from_os_rng(), 0.5);
        // let action = Action::random(&mut StdRng::from_os_rng());
        let action = Action::random(&mut StdRng::seed_from_u64(self.id));
        self.action_history.push(action);
        action
    }

    // TODO: initially 1 unit of nutrition required per time unit
    // TODO: in future, make this e.g. 3 units which could be 1 berries + 2 fish.

    /// Consume the requisite units of food per time unit.
    /// Return false if insufficient stock was available.
    fn consume(&mut self, nutritional_units: UInt) -> bool {
        let mut outstanding_nutritional_units = nutritional_units;
        let mut stock_change: Vec<_> = vec![];
        while let Some((good, qty)) = self.stock.next_consumables().into_iter().next() {
            // self.stock.remove(good, *qty);
            // to_remove.push((good.clone(), *qty));
            // If qty_remaining < nutritional_units, recursively call consume()
            if *qty > outstanding_nutritional_units {
                // return self.consume(nutritional_units - qty);
                stock_change.push((good.clone(), outstanding_nutritional_units));
                outstanding_nutritional_units = 0;
                break;
            } else {
                stock_change.push((good.clone(), *qty));
                outstanding_nutritional_units -= *qty;
            }
        }
        // Update stock
        for (good, qty) in stock_change {
            self.stock.remove(&good, qty);
        }
        // Returns false if the agent dies from lack of nutrients
        if outstanding_nutritional_units > 0 {
            return false;
        }
        true
    }

    fn action_history(&self) -> Vec<Action> {
        self.action_history.clone()
    }

    fn stock_history(&self) -> Vec<Stock> {
        self.stock_history.clone()
    }
    fn is_alive(&self) -> bool {
        self.is_alive
    }

    fn act(&mut self, action: Action) {
        match action {
            Action::ProduceGood(good) => {
                let productivity = self.productivity(good);
                match productivity {
                    Productivity::Immediate(qty) => self.stock.add(GoodsUnit::new(&good), qty),
                    Productivity::Delayed(_) => {
                        if let Some(mut partial_good) = self.stock.get_partial(good) {
                            // If a partial good already exists, do the next step of production.
                            partial_good.increment_production();
                        } else {
                            // Otherwise create a new partial good.
                            self.stock.add_partial(PartialGoodsUnit::new(&good).expect(
                                "Delayed productivity implies multiple timesteps to produce.",
                            ))
                        }
                    }
                    Productivity::None => {} // Wasted action.
                }
            }
            Action::Leisure => (),
        }
    }

    fn step_forward(&mut self) {
        // Select action
        let action = self.choose_action();
        // Perform action, which updates the agent's stock
        self.act(action);
        // Consume stock, which updates whether the agent is alive
        // TODO: make required nutritional_units per time unit configurable.
        self.is_alive = self.consume(1);
        // Degrade the agent's stock.
        self.stock_history.push(self.stock.clone());
        self.stock = self.stock.step_forward(action);
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[enum_dispatch(Agent)]
pub enum AgentType {
    Crusoe(CrusoeAgent),
}

/// A simple function that adds two integers.
pub fn some_agent_fn(x: Int, y: Int) -> Int {
    x + y
}

#[cfg(test)]
mod tests {
    use super::*; // Import the functions from the parent module

    #[test]
    fn test_some_agent_fn() {
        assert_eq!(some_agent_fn(2, 3), 5);
        assert_eq!(some_agent_fn(-1, 1), 0);
        assert_eq!(some_agent_fn(0, 0), 0);
    }

    #[test]
    fn test_consume() {
        let mut agent = CrusoeAgent::new(1);
        agent.stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            5,
        );
        agent.consume(3);
        // Expected stock after consumption is 2 units of berries
        // (three units were consumed) with remaining lifetime 10.
        let mut expected = Stock::default();
        expected.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            2,
        );
        assert_eq!(agent.stock, expected);
    }

    #[test]
    fn test_step_forward() {
        // Note: id parameter is the random seed and we assume
        // the first action it chooses does not affect the stock
        let mut agent = CrusoeAgent::new(1);
        agent.stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            5,
        );
        agent.step_forward();
        // Expected stock after one step forward is 4 units of berries
        // (one unit was consumed) with remaining lifetime 9.
        let mut expected = Stock::default();
        expected.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 9,
            },
            4,
        );
        assert_eq!(agent.stock, expected);
    }
}
