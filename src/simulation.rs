use crate::actions::ActionFlattened as Action;
use crate::agent::{self, Agent, AgentType, CrusoeAgent};
use crate::config::{Config, RLConfig};
use crate::goods::Good;
use crate::learning::agent_state::{DiscrRep, LevelPair};
use crate::learning::history::{History, SAR};
use crate::learning::tabular_rl::SARSAModel;
use crate::stock::Stock;
use crate::{Model, UInt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::vec::Vec;

// TODO: add RL algorithm
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Simulation {
    pub time: UInt,
    pub agents: Vec<AgentType>,
    pub config: Config,
    pub agent_hist: BTreeMap<u32, History<Stock, Good, LevelPair, Action>>,
}

impl Simulation {
    pub fn new() -> Self {
        let config = Config::default();
        // TODO: add n_agents to config
        // let num_agents = 10;
        // let multi_policy = false;
        // let model = SARSAModel::new(
        //     (0..num_agents).map(|n| n.into()).collect(),
        //     Good::iter().collect::<Vec<Good>>(),
        //     LevelPair::iter().collect::<Vec<LevelPair>>(),
        //     Action::iter().collect::<Vec<Action>>(),
        //     multi_policy,
        // );
        let mut agent_hist = BTreeMap::new();
        agent_hist.insert(0, History::new());
        Simulation {
            time: 0,
            agents: vec![AgentType::Crusoe(CrusoeAgent::new(0))], // Initialize with one Crusoe agent
            config, // Default value, can be overridden
            agent_hist,
        }
    }

    pub fn step_forward(&mut self, model: &Model) {
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
        println!("{:#?}", self);
        for agent in self.agents.iter_mut() {
            // Check agent is alive
            if !agent.is_alive() {
                continue; // Skip dead agents
            }
            agent.step_forward(model);
            self.agent_hist
                .entry(1)
                .or_insert_with(History::new)
                .push(SAR::new(
                    agent.stock().clone(),
                    *agent.action_history().last().unwrap(),
                    *agent.reward_history().last().unwrap(),
                ))
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_initialization() {
        let sim = Simulation::new();
        assert_eq!(sim.time, 0);
        assert!(!sim.agents.is_empty());
        assert_eq!(sim.agents.len(), 1);
        assert_eq!(sim.config.max_time, 100);

        println!(">>>>> {:?}", sim);
    }
}
