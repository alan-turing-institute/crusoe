use std::collections::HashMap;

use enum_dispatch::enum_dispatch;
use rand::SeedableRng;
use rand::rngs::{OsRng, StdRng};
use serde::{Deserialize, Serialize};

use crate::actions::Action;
use crate::goods::Good;
use crate::{Int, UInt};

#[enum_dispatch]
pub trait Agent {
    fn get_id(&self) -> Int;
    fn get_name(&self) -> &str;
    /// The stock of goods currently held by the agent.
    fn stock(&self) -> &HashMap<Good, Int>;
    /// The marginal productivity of the agent, given their current stock.
    fn productivity(&self, good: Good) -> Int;
    /// The agent's choice of action in the next time step.
    fn choose_action(&self) -> Action;
    /// Consume nutritional units for one time step and return false if insufficient were unavailable.
    fn consume(&mut self) -> bool;
    /// Get the complete history of agent actions.
    fn history(&self) -> Vec<Action>;
    /// Return true if the agent is still alive.
    fn is_alive(&self) -> bool;
    /// Update
    fn act(&mut self, action: Action);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrusoeAgent {
    id: Int,
    stock: HashMap<Good, UInt>,
}

impl Agent for CrusoeAgent {
    fn get_id(&self) -> Int {
        todo!()
    }

    fn get_name(&self) -> &str {
        "Crusoe"
    }

    fn stock(&self) -> &HashMap<Good, Int> {
        todo!()
    }

    fn productivity(&self, good: Good) -> Int {
        // TODO: make configurable.
        match good {
            Good::Berries => 4, // 4 units per day.
        }
    }

    fn choose_action(&self) -> Action {
        Action::random(&mut StdRng::from_os_rng())
    }

    /// Consume the requisite units of food per time unit.
    /// Return false if insufficient stock was available.
    fn consume(&mut self) -> bool {
        // Reduce the berry stock, otherwise change the alive status to false.
        if let Some(berries) = self.stock.get(&Good::Berries) {
            if *berries > 0 {
                self.stock.insert(Good::Berries, berries - 3);
                return true;
            } else {
                return false;
            }
        }
        // If no berries are available, the agent cannot consume and is considered dead.
        false
    }

    fn history(&self) -> Vec<Action> {
        todo!()
    }

    fn is_alive(&self) -> bool {
        todo!()
    }

    fn act(&mut self, action: Action) {
        todo!()
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
}
