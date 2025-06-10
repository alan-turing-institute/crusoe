use std::i32;

use enum_dispatch::enum_dispatch;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::actions::{Action, ActionFlattened};
use crate::goods::{Good, GoodsUnit};
use crate::learning::agent_state::DiscrRep;
use crate::learning::reward::Reward;
use crate::stock::Stock;
use crate::{Int, Model, UInt};

#[enum_dispatch]
pub trait Agent {
    fn get_id(&self) -> Int;
    fn get_name(&self) -> &str;
    /// The stock of goods currently held by the agent.
    fn stock(&self) -> &Stock;
    /// The marginal productivity of the agent, given their current stock.
    fn productivity(&self, good: Good) -> UInt;
    /// The agent's choice of action in the next time step.
    fn choose_action(&mut self) -> Action;
    /// The agent's choice of action in the next time step.
    fn choose_action_with_model(&mut self, model: &Model) -> Action;
    /// Consume nutritional units for one time step and return false if insufficient were unavailable.
    fn consume(&mut self, nutritional_units: UInt) -> bool;
    /// Get the complete history of agent actions.
    fn history(&self) -> Vec<Action>;
    /// Get the reward history.
    fn reward_history(&self) -> Vec<Reward>;
    /// Return true if the agent is still alive.
    fn is_alive(&self) -> bool;
    /// Execture the given action.
    fn act(&mut self, action: Action);
    /// Step the agent forward by one time step.
    fn step_forward(&mut self, model: &Model);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrusoeAgent {
    pub id: Int,
    pub stock: Stock,
    pub is_alive: bool,
    pub action_history: Vec<Action>,
    pub reward_history: Vec<Reward>,
}

impl CrusoeAgent {
    pub fn new(id: Int) -> Self {
        CrusoeAgent {
            id,
            stock: Stock::default(),
            is_alive: true,
            action_history: vec![],
            reward_history: vec![],
        }
    }
}

impl Agent for CrusoeAgent {
    fn get_id(&self) -> Int {
        todo!()
    }

    fn get_name(&self) -> &str {
        "Crusoe"
    }

    fn stock(&self) -> &Stock {
        &self.stock
    }

    // TODO: Some capital goods take multiple time units to produce.
    fn productivity(&self, good: Good) -> UInt {
        // TODO: make configurable.
        match good {
            Good::Berries => {
                // Productivity of berries is increased by access to a basket.
                if self.stock.contains(Good::Basket) {
                    return 8;
                }
                return 4;
            }
            Good::Basket => 1,
            Good::Fish => {
                // Productivity of fish is increased by access to a spear.
                if self.stock.contains(Good::Spear) {
                    return 10;
                }
                return 2;
            }
            Good::Spear => 1,
        }
    }

    // TODO: consider moving teh action_history update into act method, so
    // self can be immutable here.
    fn choose_action(&mut self) -> Action {
        let action = Action::random_weighted(&mut StdRng::from_os_rng(), 0.5);
        self.action_history.push(action.into());
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

    fn history(&self) -> Vec<Action> {
        self.action_history.clone()
    }

    fn is_alive(&self) -> bool {
        self.is_alive
    }

    fn act(&mut self, action: Action) {
        match action {
            Action::ProduceGood(good) => {
                let qty = self.productivity(good);
                self.stock.add(GoodsUnit::new(&good), qty);
            }
            Action::Leisure => (),
        }
    }

    fn step_forward(&mut self, model: &Model) {
        // Select action
        let action = self.choose_action_with_model(model);
        // Perform action, which updates the agent's stock
        self.act(action);
        // Degrade the agent's stock.
        self.stock.step_forward(action);
        // Consume stock, which updates whether the agent is alive
        // TODO: make required nutritional_units per time unit configurable.
        self.is_alive = self.consume(1);
        // Update reward history
        match (action, self.is_alive) {
            (Action::ProduceGood(_), true) => {
                self.reward_history.push(Reward::new(0));
            }
            (Action::Leisure, true) => {
                self.reward_history.push(Reward::new(1));
            }
            (_, false) => {
                self.reward_history.push(Reward::new(-100000));
            }
        };
    }

    fn reward_history(&self) -> Vec<Reward> {
        self.reward_history.clone()
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
}
