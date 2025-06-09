use crate::UInt;
use crate::agent::{Agent, AgentType};
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug)]
pub struct Simulation {
    pub time: UInt,
    pub agents: Vec<AgentType>,
    pub config: Config,
}

impl Default for Simulation {
    fn default() -> Self {
        Simulation {
            time: 0,
            agents: Vec::new(),
            config: Config { max_time: 100 }, // Default value, can be overridden
        }
    }
}

impl Simulation {
    pub fn new() -> Self {
        Simulation::default()
    }

    fn step_forward(&mut self) {
        // Step forward each agent.
        // Per day:
        // - Start the day
        // - Check if agent is alive
        // - Agent selects an action
        // - Agent performs the action
        // - Agent updates its stock
        // - Consume stock
        // - Update whether agent is alive
        // - End the day
        for agent in self.agents.iter_mut() {
            // Check agent is alive
            if !agent.is_alive() {
                continue; // Skip dead agents
            }

            // Select action
            let action = agent.choose_action();

            // Perform action, which updates the agent's stock
            agent.act(action);

            // Consume stock, which updates whether the agent is alive
            agent.consume();
        }
        self.after_step();
    }

    // Trade happens in here.
    fn after_step(&mut self) {
        // Shuffle the vector of agents.
        // for &mut agent in self.agents().shuffle() {
        // Identify the best bilateral trade for this agent.

        // Execute that trade by updating the stocks of the two agents involved.
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_initialization() {
        let sim = Simulation::new();
        assert_eq!(sim.time, 0);
        assert!(sim.agents.is_empty());
        assert_eq!(sim.config.max_time, 100);

        println!(">>>>> {:?}", sim);
    }
}

//     fn init(&mut self);

//     // Trade happens in here.
//     fn after_step(&mut self) {
//         // Shuffle the vector of agents.
//         for &mut agent in self.agents().shuffle() {
//             // Identify the best bilateral trade for this agent.

//             // Execute that trade by updating the stocks of the two agents involved.
//         }
//     }

//     fn update(&mut self) {
//         // Step forward each agent.
//         for &mut agent in self.agents() {
//             let action = agent.select_action()
//             agent.step(action)
//         }
//         self.after_step()
//     }
// }
