use crate::UInt;
use crate::agent::{Agent, AgentType, CrusoeAgent};
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug)]
pub struct Simulation {
    pub time: UInt,
    pub agents: Vec<AgentType>,
    pub config: Config,
    pub verbose: bool,
}

impl Default for Simulation {
    fn default() -> Self {
        Simulation {
            time: 0,
            agents: Vec::new(),
            config: Config { max_time: 100 },
            verbose: true,
        }
    }
}

impl Simulation {
    pub fn new(config: Config, verbose: bool) -> Self {
        Simulation {
            time: 0,
            agents: vec![AgentType::Crusoe(CrusoeAgent::new(1))], // Initialize with one Crusoe agent
            config,
            verbose,
        }
    }

    pub fn step_forward(&mut self) {
        // Step forward each agent.
        // Per day:
        // - Start the day
        // - Check if agent is alive
        // - Agent selects an action
        // - Agent performs the action
        // - Agent updates its stock
        // - Consume stock
        // - Update whether agent is alive
        // - Degrade the agent's stock
        // - End the day
        for agent in self.agents.iter_mut() {
            // Check agent is alive
            if !agent.is_alive() {
                continue; // Skip dead agents
            }
            agent.step_forward();
        }
        self.after_step();
    }

    // Trade happens in here.
    pub fn after_step(&mut self) {
        // Shuffle the vector of agents.
        // for &mut agent in self.agents().shuffle() {
        // Identify the best bilateral trade for this agent.

        // Execute that trade by updating the stocks of the two agents involved.
    }

    // Run simulation
    pub fn run(&mut self) {
        while self.time < self.config.max_time {
            self.step_forward();
            if self.verbose {
                println!("Time: {}, Agents: {}", self.time, self.agents.len());
                println!("Actions:  {0:#?}", self.agents[0]);
            }
            self.time += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_initialization() {
        let sim = Simulation::new(Config { max_time: 100 }, true);
        assert_eq!(sim.time, 0);
        assert!(!sim.agents.is_empty());
        assert_eq!(sim.agents.len(), 1);
        assert_eq!(sim.config.max_time, 100);

        println!(">>>>> {:?}", sim);
    }
}
