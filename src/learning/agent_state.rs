use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::map::Iter;
use strum_macros::EnumIter;

use crate::stock::Stock as AgentState;
use crate::stock::StockDiscrete as AgentStateDiscrete;

use crate::{
    UInt,
    config::core_config,
    goods::{Good, GoodsUnit, GoodsUnitLevel},
    stock::Stock,
};
use strum::IntoEnumIterator;

pub trait DiscrRep<S, L> {
    fn representation(&self) -> Vec<(S, L)>;
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct AgentState {
//     pub food: i32,
//     pub water: i32,
//     pub min_steps_to_food: Option<u32>,
//     pub min_steps_to_water: Option<u32>,
//     pub min_steps_to_trader: Option<u32>,
// }

// #[derive(Debug, Clone, PartialEq, EnumIter, Hash, Eq, Serialize, Deserialize)]
// pub enum AgentStateItems {
//     Food,
//     Water,
//     MinStepsToFood,
//     MinStepsToWater,
//     MinStepsToTrader,
// }

// TODO: add GoodsUnitLevel enum to represent different goods

// TODO: add conversion from Stock to StockDiscrete
// TODO: add the unnested good

// (4 * 8 * 4) * (leisure + 8[action per good] + trade + credit) = 128 * 11 = 1408

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelPair(pub InvLevel, pub InvLevel);

impl From<(InvLevel, InvLevel)> for LevelPair {
    fn from(pair: (InvLevel, InvLevel)) -> Self {
        LevelPair(pair.0, pair.1)
    }
}
impl DiscrRep<Good, LevelPair> for Stock {
    fn representation(&self) -> Vec<(Good, LevelPair)> {
        let discr = self.discretise();

        let mut v = vec![];
        for (good, level) in discr.stock {
            let new_level = LevelPair::from((level, good.remaining_lifetime));
            v.push((good.good, new_level));
        }
        v
    }
}

// impl AgentState {
//     pub fn discretise(&self) -> AgentStateDiscrete {
//         let f: InvLevel;
//         let w: InvLevel;
//         let m_s_f: InvLevel;
//         let m_s_w: InvLevel;
//         let m_s_t: InvLevel;

//         if self.food < core_config().agent.INVENTORY_LEVEL_CRITICAL_LOW {
//             f = InvLevel::Critical
//         } else if self.food < core_config().agent.INVENTORY_LEVEL_LOW_MEDIUM {
//             f = InvLevel::Low
//         } else if self.food < core_config().agent.INVENTORY_LEVEL_MEDIUM_HIGH {
//             f = InvLevel::Medium
//         } else {
//             f = InvLevel::High
//         }

//         if self.water < core_config().agent.INVENTORY_LEVEL_CRITICAL_LOW {
//             w = InvLevel::Critical
//         } else if self.water < core_config().agent.INVENTORY_LEVEL_LOW_MEDIUM {
//             w = InvLevel::Low
//         } else if self.water < core_config().agent.INVENTORY_LEVEL_MEDIUM_HIGH {
//             w = InvLevel::Medium
//         } else {
//             w = InvLevel::High
//         }

//         if let Some(dist) = self.min_steps_to_food {
//             if dist < core_config().agent.DISTANCE_LEVEL_CRITICAL_LOW {
//                 m_s_f = InvLevel::Critical
//             } else if dist < core_config().agent.DISTANCE_LEVEL_LOW_MEDIUM {
//                 m_s_f = InvLevel::Low
//             } else if dist < core_config().agent.DISTANCE_LEVEL_MEDIUM_HIGH {
//                 m_s_f = InvLevel::Medium
//             } else {
//                 m_s_f = InvLevel::High
//             }
//         } else {
//             m_s_f = InvLevel::High
//         }

//         if let Some(dist) = self.min_steps_to_water {
//             if dist < core_config().agent.DISTANCE_LEVEL_CRITICAL_LOW {
//                 m_s_w = InvLevel::Critical
//             } else if dist < core_config().agent.DISTANCE_LEVEL_LOW_MEDIUM {
//                 m_s_w = InvLevel::Low
//             } else if dist < core_config().agent.DISTANCE_LEVEL_MEDIUM_HIGH {
//                 m_s_w = InvLevel::Medium
//             } else {
//                 m_s_w = InvLevel::High
//             }
//         } else {
//             m_s_w = InvLevel::High
//         }

//         if let Some(dist) = self.min_steps_to_trader {
//             if dist < core_config().agent.DISTANCE_LEVEL_CRITICAL_LOW {
//                 m_s_t = InvLevel::Critical
//             } else if dist < core_config().agent.DISTANCE_LEVEL_LOW_MEDIUM {
//                 m_s_t = InvLevel::Low
//             } else if dist < core_config().agent.DISTANCE_LEVEL_MEDIUM_HIGH {
//                 m_s_t = InvLevel::Medium
//             } else {
//                 m_s_t = InvLevel::High
//             }
//         } else {
//             m_s_t = InvLevel::High
//         }

//         AgentStateDiscrete {
//             food: f,
//             water: w,
//             min_steps_to_food: m_s_f,
//             min_steps_to_water: m_s_w,
//             min_steps_to_trader: m_s_t,
//         }
//     }

//     // pub fn representation<S, L>(&self) -> ((S, L), (S, L))
//     // where
//     //     S: std::cmp::Eq + std::hash::Hash + Clone,
//     //     L: std::cmp::Eq + std::hash::Hash + Clone,
//     // {
//     //     let discr = self.discretise();
//     //     (
//     //         (AgentStateItems::Food, discr.food),
//     //         (AgentStateItems::Water, discr.water),
//     //     )
//     // }
// }

#[derive(Debug, Copy, Clone, PartialEq, EnumIter, Hash, Eq, Serialize, Deserialize)]
pub enum InvLevel {
    Critical,
    Low,
    Medium,
    High,
}

impl LevelPair {
    /// Iterate over all possible combinations of InvLevel for LevelPair
    pub fn iter() -> impl Iterator<Item = LevelPair> {
        InvLevel::iter().flat_map(|a| InvLevel::iter().map(move |b| LevelPair::from((a, b))))
    }
}
