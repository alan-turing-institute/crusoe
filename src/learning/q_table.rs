use crate::config::core_config;
use crate::learning::serde_utils;
use itertools::Itertools;
use rand::{Rng, rngs::StdRng};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use strum::IntoEnumIterator;

#[derive(Debug, Serialize, Deserialize)]
pub struct QTable<S, L, A>
where
    S: std::cmp::Eq + std::hash::Hash + Clone + Debug + Serialize + DeserializeOwned,
    L: std::cmp::Eq + std::hash::Hash + Clone + Debug + Serialize + DeserializeOwned,
    A: std::cmp::Eq
        + std::hash::Hash
        + Clone
        + Debug
        + IntoEnumIterator
        + Serialize
        + DeserializeOwned,
{
    // To serialize with a struct as key, custom serialization with a `serde_utils` module can be [used](https://stackoverflow.com/questions/51276896/how-do-i-use-serde-to-serialize-a-hashmap-with-structs-as-keys-to-json)
    #[serde(with = "serde_utils")]
    pub tab: HashMap<QKey<S, L, A>, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct QKey<S, L, A>(pub Vec<(S, L)>, pub A);

impl<S, L, A> QKey<S, L, A> {
    pub fn from_tuple(tup: (Vec<(S, L)>, A)) -> QKey<S, L, A> {
        QKey(tup.0, tup.1)
    }
}

impl<S, L, A> QTable<S, L, A>
where
    S: std::cmp::Eq + std::hash::Hash + Clone + Debug + Serialize + DeserializeOwned,
    L: std::cmp::Eq + std::hash::Hash + Clone + Debug + Serialize + DeserializeOwned,
    A: std::cmp::Eq
        + std::hash::Hash
        + Clone
        + Debug
        + Serialize
        + IntoEnumIterator
        + DeserializeOwned,
{
    pub fn new(state_items: Vec<S>, state_levels: Vec<L>, actions: Vec<A>) -> Self {
        let mut q_tbl = HashMap::new();
        let mut combs_for_all_state_items = Vec::new();
        for s in state_items {
            let mut levels_for_item = Vec::new();
            for l in state_levels.clone() {
                levels_for_item.push((s.clone(), l))
            }
            combs_for_all_state_items.push(levels_for_item);
        }

        let combs = combs_for_all_state_items
            .clone()
            .into_iter()
            .multi_cartesian_product()
            .collect_vec();

        // println!("{:?}", combs);

        // let manual = combs_for_all_state_items[0]
        //     .clone()
        //     .into_iter()
        //     .cartesian_product(combs_for_all_state_items[1].clone())
        //     .into_iter()
        //     .collect_vec();

        let q = combs.into_iter().cartesian_product(actions).collect_vec();

        for el in q {
            // println!("{:?}", el);
            let q_key = QKey(el.0, el.1);
            q_tbl.insert(q_key, core_config().rl.init_q_value);
        }

        QTable { tab: q_tbl }
    }

    pub fn get_tab_mut(&mut self) -> &mut HashMap<QKey<S, L, A>, f32> {
        &mut self.tab
    }
    pub fn get_tab(&self) -> &HashMap<QKey<S, L, A>, f32> {
        &self.tab
    }

    pub fn sample_action(&self, state: &Vec<(S, L)>, rng: &mut StdRng) -> (A, f32) {
        let mut optimal_a: A = self.pick_rnd(rng);
        let mut q_optimal = self
            .get_tab()
            .get(&QKey(state.to_owned(), optimal_a.clone()))
            .unwrap();

        for a in A::iter() {
            let q_a = self
                .get_tab()
                .get(&QKey(state.to_owned(), optimal_a.clone()))
                .unwrap();
            // println!("{:?}, {:?}", a, q_a);
            if q_a > q_optimal {
                optimal_a = a;
                q_optimal = self
                    .get_tab()
                    .get(&QKey(state.to_owned(), optimal_a.clone()))
                    .unwrap();
            }
        }
        let r: f32 = rng.random();
        if r < core_config().rl.epsilon {
            optimal_a = self.pick_rnd(rng);
        }
        (optimal_a, *q_optimal)
    }
    fn pick_rnd(&self, rng: &mut StdRng) -> A {
        let r: f32 = rng.random();
        let mut a_iter = A::iter();
        let a: A;
        if r < 0.3 {
            a = a_iter.next().expect("at least one action in enum");
        } else if r < 0.6 {
            a_iter.next();
            a = a_iter.next().unwrap();
        } else {
            a_iter.next();
            a_iter.next();
            a = a_iter.next().unwrap();
        }
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stock::InvLevel;

    #[test]
    fn test_multi_product() {
        let combs = vec![InvLevel::iter().collect_vec(); 3]
            .clone()
            .into_iter()
            .multi_cartesian_product()
            .collect_vec();
        // Should be: 4 ** 3 with each position taking all possible variants of the enum
        assert_eq!(combs.len(), 64)
    }
}
