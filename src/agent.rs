use enum_dispatch::enum_dispatch;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::actions::{Action, ActionFlattened};
use crate::goods::{Good, GoodsUnit, PartialGoodsUnit, Productivity};
use crate::learning::agent_state::DiscrRep;
use crate::learning::reward::Reward;
use crate::stock::Stock;
use crate::{Model, UInt};

#[enum_dispatch]
pub trait Agent {
    fn get_id(&self) -> u64;
    fn get_name(&self) -> &str;
    /// The stock of goods currently held by the agent.
    fn stock(&self) -> &Stock;
    /// The marginal productivity of the agent, given their current stock.
    fn stock_mut(&mut self) -> &mut Stock;
    fn set_stock(&mut self, stock: Stock);
    fn acquire(&mut self, goods_unit: GoodsUnit, quantity: UInt);
    fn acquire_partial(&mut self, partial_goods_unit: PartialGoodsUnit);
    fn get_partial(&self, good: Good) -> Option<PartialGoodsUnit>;
    /// Returns the number of units of the good produced per day,
    /// given the agent's existing stock.
    fn productivity(&self, good: Good) -> Productivity {
        // TODO: make configurable.
        // Note: can modify default productivity for different agents (for specialisation).
        good.default_productivity(&self.stock())
    }
    // fn productivity(&self, good: Good) -> (UInt, bool);
    /// The agent's choice of action in the next time step.
    fn choose_action(&mut self) -> Action;
    /// The agent's choice of action in the next time step.
    fn choose_action_with_model(&mut self, model: &Model) -> Action;
    /// Consume nutritional units for one time step and return false if insufficient were unavailable.
    fn consume(&mut self, nutritional_units: UInt) -> bool {
        let consumables = self.stock().next_consumables();
        if consumables.is_empty() {
            // println!("DEBUG: consume - no consumables available");
            return false;
        }

        let mut outstanding_nutritional_units = nutritional_units;
        let mut stock_change: Vec<_> = vec![];
        for (good, qty) in consumables {
            if *qty > outstanding_nutritional_units {
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
            self.stock_mut().remove(&good, qty);
        }
        // Returns false if the agent dies from lack of nutrients
        if outstanding_nutritional_units > 0 {
            return false;
        }
        true
    }

    /// Get the complete history of agent actions.
    fn action_history(&self) -> Vec<Action>;
    /// Get the complete history of agent stocks.
    fn stock_history(&self) -> Vec<Stock>;
    /// Get the reward history.
    fn reward_history(&self) -> Vec<Reward>;
    /// Return true if the agent is still alive.
    fn update_stock_history(&mut self);
    fn update_reward_history(&mut self, action: Action, is_alive: bool);
    fn is_alive(&self) -> bool;
    fn set_liveness(&mut self, value: bool);
    /// Execture the given action.
    // fn act(&mut self, action: Action);
    fn act(&mut self, action: Action) {
        match action {
            Action::ProduceGood(good) => {
                let productivity = self.productivity(good);
                match productivity {
                    Productivity::Immediate(qty) => self.acquire(GoodsUnit::new(&good), qty),
                    Productivity::Delayed(_) => {
                        if let Some(mut partial_good) = self.get_partial(good) {
                            // If a partial good already exists, do the next step of production.
                            partial_good.increment_production();
                        } else {
                            // Otherwise create a new partial good.
                            self.acquire_partial(PartialGoodsUnit::new(&good).expect(
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
    /// Step the agent forward by one time step.
    // fn step_forward(&mut self);
    fn step_forward(&mut self, action: Option<Action>) {
        // Select action if not given.
        let action = match action {
            Some(a) => a,
            None => self.choose_action(),
        };
        // Perform action, which updates the agent's stock
        self.act(action);
        // Consume stock, which updates whether the agent is alive
        // TODO: make required nutritional_units per time unit configurable.
        let is_alive = self.consume(1);
        // TODO: removed set_liveness for now
        // self.set_liveness(is_alive);
        // Degrade the agent's stock.
        // self.stock_history.push(self.stock.clone());
        // self.append_to_stock_history(self.stock().clone());
        self.update_stock_history();
        self.update_reward_history(action, is_alive);
        // self.stock = self.stock.step_forward(action);
        self.set_stock(self.stock().step_forward(action));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrusoeAgent {
    pub id: u64,
    pub stock: Stock,
    pub is_alive: bool,
    pub action_history: Vec<Action>,
    stock_history: Vec<Stock>,
    pub reward_history: Vec<Reward>,
}

impl CrusoeAgent {
    pub fn new(id: u64) -> Self {
        CrusoeAgent {
            id,
            stock: Stock::default(),
            is_alive: true,
            action_history: vec![],
            stock_history: vec![],
            reward_history: vec![],
        }
    }
}

impl Agent for CrusoeAgent {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_name(&self) -> &str {
        "Crusoe"
    }

    fn stock(&self) -> &Stock {
        &self.stock
    }

    fn stock_mut(&mut self) -> &mut Stock {
        &mut self.stock
    }

    fn set_stock(&mut self, stock: Stock) {
        self.stock = stock;
    }

    // // TODO: Some capital goods take multiple time units to produce.
    // /// Returns the number of units of the good produced per day, given the
    // /// agent's existing stock. The boolean flag indicates whether the
    // /// integer in the first slot should be interpreted as a fraction.
    // /// e.g. (5, false) indicates  5 units can be produced in one day,
    // /// while (5, true) indicates that a single unit takes 5 days to produce.
    // fn productivity(&self, good: Good) -> Productivity {
    //     // TODO: make configurable.
    //     // TODO: can modify default productivity for different agents (for specialisation).
    //     good.default_productivity(&self.stock)
    // }

    // TODO: consider moving teh action_history update into act method, so
    // self can be immutable here.
    fn choose_action(&mut self) -> Action {
        // let action = Action::random_weighted(&mut StdRng::from_os_rng(), 0.5);
        let action = Action::random(&mut StdRng::from_os_rng());
        // let action = Action::random(&mut StdRng::seed_from_u64(self.id));
        self.action_history.push(action);
        action
    }

    // TODO: consider moving teh action_history update into act method, so
    // self can be immutable here.
    fn choose_action_with_model(&mut self, model: &Model) -> Action {
        let action =
            model.sample_action_by_id(0, &self.stock.representation(), &mut StdRng::from_os_rng());
        self.action_history.push(action.into());
        action.into()
    }

    fn action_history(&self) -> Vec<Action> {
        self.action_history.clone()
    }
    fn stock_history(&self) -> Vec<Stock> {
        self.stock_history.clone()
    }
    fn reward_history(&self) -> Vec<Reward> {
        self.reward_history.clone()
    }

    fn update_stock_history(&mut self) {
        self.stock_history.push(self.stock().clone());
    }

    fn update_reward_history(&mut self, action: Action, is_alive: bool) {
        let reward = match (action, is_alive) {
            (Action::ProduceGood(_), true) => Reward::new(0),
            (Action::Leisure, true) => Reward::new(1),
            (_, false) => Reward::new(-1),
        };
        self.reward_history.push(reward);
    }

    fn is_alive(&self) -> bool {
        self.is_alive
    }

    fn set_liveness(&mut self, value: bool) {
        self.is_alive = value;
    }

    fn acquire(&mut self, goods_unit: GoodsUnit, quantity: UInt) {
        self.stock.add(goods_unit, quantity);
    }

    fn acquire_partial(&mut self, partial_goods_unit: PartialGoodsUnit) {
        self.stock.add_partial(partial_goods_unit);
    }

    fn get_partial(&self, good: Good) -> Option<PartialGoodsUnit> {
        self.stock.get_partial(good)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[enum_dispatch(Agent)]
pub enum AgentType {
    Crusoe(CrusoeAgent),
}

impl AgentType {
    pub fn action_history(&self) -> Vec<ActionFlattened> {
        match self {
            AgentType::Crusoe(agent) => agent.action_history.iter().map(|a| (*a).into()).collect(),
        }
    }

    pub fn reward_history(&self) -> Vec<Reward> {
        match self {
            AgentType::Crusoe(agent) => agent.reward_history().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import the functions from the parent module

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
        agent.consume(2);
        // Expected stock after consumption of the remaining 2 units
        // of berries is empty.
        assert!(agent.stock.stock.is_empty());
    }

    #[test]
    fn test_step_forward() {
        let mut agent = CrusoeAgent::new(1);
        agent.stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            5,
        );
        agent.step_forward(Some(Action::Leisure));
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
