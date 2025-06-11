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

pub type Int = i32;
pub type UInt = u32;

type Model = SARSAModel<Stock, GoodsUnitLevel, InvLevel, Action>;
