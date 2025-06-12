use std::collections::HashMap;

use crate::{
    goods::GoodsUnitLevel,
    stock::{InvLevel, Stock},
};

pub trait DiscrRep<S, L> {
    fn representation(&self) -> Vec<(S, L)>;
}

impl DiscrRep<GoodsUnitLevel, InvLevel> for Stock {
    fn representation(&self) -> Vec<(GoodsUnitLevel, InvLevel)> {
        let hm: HashMap<GoodsUnitLevel, InvLevel> = self.discretise().stock.into_iter().collect();
        GoodsUnitLevel::iter()
            .map(|good_unit_level| {
                (
                    good_unit_level,
                    hm.get(&good_unit_level)
                        .cloned()
                        // .unwrap_or(InvLevel::Critical),
                        .unwrap_or(InvLevel::Low),
                )
            })
            .collect()
    }
}
