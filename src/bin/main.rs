use crusoe::{
    NEGATIVE_REWARD,
    actions::ActionFlattened as Action,
    config::Config,
    goods::{Good, GoodsUnitLevel},
    learning::{history::SAR, q_table::QKey, reward::Reward, tabular_rl::SARSAModel},
    simulation::Simulation,
    stock::{InvLevel, Stock},
};
use itertools::Itertools;
use log::{debug, info};
use strum::IntoEnumIterator;

fn main() {
    env_logger::init();
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

    let mut lifetimes = vec![];
    while sim.time < sim.config.max_time {
        sim.step_forward(&model);
        if sim.agents[0].reward_history().last().unwrap().val == NEGATIVE_REWARD {
            let lifetime = sim.agents[0]
                .reward_history()
                .iter()
                .rev()
                .skip(1)
                .enumerate()
                .take_while_inclusive(|(_, el)| el.val != NEGATIVE_REWARD)
                .map(|(idx, _)| idx)
                .last()
                .unwrap_or(0);
            lifetimes.push(lifetime);
        }
        if sim.time % 1000 == 0 {
            let n_steps = 1000;
            let avg_reward = sim.agents[0]
                .reward_history()
                .iter()
                .rev()
                .take(n_steps)
                .map(|el| el.val as f32)
                .sum::<f32>()
                / n_steps as f32;

            let avg_lifetime =
                // lifetimes.iter().map(|el| *el as f32).sum::<f32>() / lifetimes.len() as f32;
                lifetimes[lifetimes.len() /2];
            info!(
                "Time: {}, Avg. Reward: {}, Avg. Lifetime: {}",
                sim.time, avg_reward, avg_lifetime
            );
            // println!("{:?}", lifetimes);
        }
        // Update model given agent history
        model.step(sim.time as i32, &sim.agent_hist);

        debug!(
            "Time: {:5.0} | B(L): {:.1?} | L(L): {:.1?} | B(M): {:.1?} | L(M): {:.1?} | B(H): {:.1?} | L(H): {:.1?} | Alive: {:?} | Action: {:?}",
            sim.time,
            model
                .q_tbls
                .get(&0)
                .unwrap()
                .get_tab()
                .get(&QKey::from_tuple((
                    GoodsUnitLevel::iter()
                        .map(|el| {
                            match el {
                                GoodsUnitLevel {
                                    good: Good::Berries,
                                    remaining_lifetime: _,
                                } => (el, InvLevel::Low),
                                _ => (el, InvLevel::Low),
                            }
                        })
                        .collect_vec(),
                    Action::ProduceBerries
                )))
                .unwrap(),
            model
                .q_tbls
                .get(&0)
                .unwrap()
                .get_tab()
                .get(&QKey::from_tuple((
                    GoodsUnitLevel::iter()
                        .map(|el| {
                            match el {
                                GoodsUnitLevel {
                                    good: Good::Berries,
                                    remaining_lifetime: _,
                                } => (el, InvLevel::Low),
                                _ => (el, InvLevel::Low),
                            }
                        })
                        .collect_vec(),
                    Action::Leisure
                )))
                .unwrap(),
            model
                .q_tbls
                .get(&0)
                .unwrap()
                .get_tab()
                .get(&QKey::from_tuple((
                    GoodsUnitLevel::iter()
                        .map(|el| {
                            match el {
                                GoodsUnitLevel {
                                    good: Good::Berries,
                                    remaining_lifetime: _,
                                } => (el, InvLevel::Medium),
                                _ => (el, InvLevel::Low),
                            }
                        })
                        .collect_vec(),
                    Action::ProduceBerries
                )))
                .unwrap(),
            model
                .q_tbls
                .get(&0)
                .unwrap()
                .get_tab()
                .get(&QKey::from_tuple((
                    GoodsUnitLevel::iter()
                        .map(|el| {
                            match el {
                                GoodsUnitLevel {
                                    good: Good::Berries,
                                    remaining_lifetime: _,
                                } => (el, InvLevel::Medium),
                                _ => (el, InvLevel::Low),
                            }
                        })
                        .collect_vec(),
                    Action::Leisure
                )))
                .unwrap(),
            model
                .q_tbls
                .get(&0)
                .unwrap()
                .get_tab()
                .get(&QKey::from_tuple((
                    GoodsUnitLevel::iter()
                        .map(|el| {
                            match el {
                                GoodsUnitLevel {
                                    good: Good::Berries,
                                    remaining_lifetime: _,
                                } => (el, InvLevel::High),
                                _ => (el, InvLevel::Low),
                            }
                        })
                        .collect_vec(),
                    Action::ProduceBerries
                )))
                .unwrap(),
            model
                .q_tbls
                .get(&0)
                .unwrap()
                .get_tab()
                .get(&QKey::from_tuple((
                    GoodsUnitLevel::iter()
                        .map(|el| {
                            match el {
                                GoodsUnitLevel {
                                    good: Good::Berries,
                                    remaining_lifetime: _,
                                } => (el, InvLevel::High),
                                _ => (el, InvLevel::Low),
                            }
                        })
                        .collect_vec(),
                    Action::Leisure
                )))
                .unwrap(),
            sim.agents[0]
                .reward_history()
                .last()
                .unwrap_or(&Reward::new(0))
                .val
                != NEGATIVE_REWARD,
            sim.agents[0].action_history().last().unwrap(),
        );
        // println!("Reward history: {:?}", sim.agents[0].reward_history());
        sim.time += 1;
    }
    // println!("Actions:  {0:?}", sim.agents[0]);

    // Write sim to disk
    let s = serde_json::to_string(&sim).unwrap();
    // println!("{s}");
}
