use crusoe::{
    actions::ActionFlattened as Action,
    goods::Good,
    learning::{agent_state::LevelPair, tabular_rl::SARSAModel},
    simulation::Simulation,
    stock::Stock,
};
use strum::IntoEnumIterator;

fn main() {
    let mut sim = Simulation::new();

    let num_agents = 10u32;
    let multi_policy = false;
    let mut model: SARSAModel<Stock, _, _, _> = SARSAModel::new(
        (0..num_agents).collect(),
        Good::iter().collect::<Vec<Good>>(),
        LevelPair::iter().collect::<Vec<LevelPair>>(),
        Action::iter().collect::<Vec<Action>>(),
        multi_policy,
    );

    while sim.time < sim.config.max_time {
        sim.step_forward(&model);
        println!("Time: {}, Agents: {}", sim.time, sim.agents.len());
        println!("Actions:  {0:#?}", sim.agents[0]);
        sim.time += 1;

        // Update model given agent history
        model.step(sim.time as i32, &sim.agent_hist);
    }
}
