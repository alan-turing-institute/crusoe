use crusoe::{
    NEGATIVE_REWARD,
    actions::ActionFlattened as Action,
    config::Config,
    goods::GoodsUnitLevel,
    learning::tabular_rl::SARSAModel,
    simulation::Simulation,
    stock::{InvLevel, Stock},
};
use itertools::Itertools;
use strum::IntoEnumIterator;

fn main() {
    let mut sim = Simulation::new(
        Config {
            max_time: 100000,
            daily_nutrition: 3,
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

        if sim.time % 10 == 0 {
            let n_steps = 100;
            let avg_reward = sim.agents[0]
                .reward_history()
                .iter()
                .rev()
                .take(n_steps)
                .map(|el| el.val as f32)
                .sum::<f32>()
                / n_steps as f32;

            let lifetime = sim.agents[0]
                .reward_history()
                .iter()
                .rev()
                .enumerate()
                .take_while_inclusive(|(_, el)| el.val != NEGATIVE_REWARD)
                .map(|(idx, _)| idx)
                .last()
                .unwrap();

            println!(
                "Time: {}, Avg. Reward: {}, Lifetime: {}",
                sim.time, avg_reward, lifetime
            )
        }
        sim.time += 1;

        // Update model given agent history
        model.step(sim.time as i32, &sim.agent_hist);
    }
    // println!("Actions:  {0:?}", sim.agents[0]);

    // Write sim to disk
    let s = serde_json::to_string(&sim).unwrap();
    println!("{s}");
}
