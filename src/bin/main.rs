use crusoe::{
    actions::ActionFlattened as Action,
    config::Config,
    goods::{Good, GoodsUnitLevel},
    learning::{agent_state::LevelPair, tabular_rl::SARSAModel},
    simulation::Simulation,
    stock::{InvLevel, Stock},
};
use strum::IntoEnumIterator;

fn main() {
    let mut sim = Simulation::new(
        Config {
            max_time: 100,
            ..Config::default()
        },
        true,
    );
    let num_agents = 1u32;
    let multi_policy = false;
    let mut model: SARSAModel<Stock, _, _, _> = SARSAModel::new(
        (0..num_agents).collect(),
        GoodsUnitLevel::iter().collect::<Vec<GoodsUnitLevel>>(),
        InvLevel::iter().collect::<Vec<InvLevel>>(),
        Action::iter().collect::<Vec<Action>>(),
        multi_policy,
    );
    println!("Model initialized with {} agents", num_agents);

    while sim.time < sim.config.max_time {
        sim.step_forward(&model);
        println!("Time: {}, Agents: {}", sim.time, sim.agents.len());
        sim.time += 1;

        // Update model given agent history
        model.step(sim.time as i32, &sim.agent_hist);
    }
    println!("Actions:  {0:?}", sim.agents[0]);
}
