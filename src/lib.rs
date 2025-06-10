use crate::{
    actions::ActionFlattened as Action,
    goods::Good,
    learning::{agent_state::LevelPair, tabular_rl::SARSAModel},
    stock::Stock,
};

pub mod actions;
pub mod agent;
pub mod config;
pub mod goods;
pub mod learning;
pub mod simulation;
pub mod stock;

pub type Int = i32;
pub type UInt = u32;

type Model = SARSAModel<Stock, Good, LevelPair, Action>;
