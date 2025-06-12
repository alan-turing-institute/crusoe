use crate::{
    actions::ActionFlattened as Action,
    goods::GoodsUnitLevel,
    learning::tabular_rl::SARSAModel,
    stock::{InvLevel, Stock},
};

pub mod actions;
pub mod agent;
pub mod config;
pub mod goods;
pub mod learning;
pub mod simulation;
pub mod stock;
pub mod valuation;

pub type Int = i32;
pub type UInt = u32;

pub type Model = SARSAModel<Stock, GoodsUnitLevel, InvLevel, Action>;

pub const POSITIVE_REWARD: Int = 1;
pub const NEGATIVE_REWARD: Int = -100000;
