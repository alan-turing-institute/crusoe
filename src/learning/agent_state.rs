use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    goods::GoodsUnitLevel,
    stock::{InvLevel, RemainingLevel, Stock},
};

pub trait DiscrRep<S, L> {
    fn representation(&self) -> Vec<(S, L)>;
}

// Approximate Q table size
// (4 * 8 * 4) * (leisure + 8[action per good] + trade + credit) = 128 * 11 = 1408
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelPair(pub RemainingLevel, pub InvLevel);

impl From<(RemainingLevel, InvLevel)> for LevelPair {
    fn from(pair: (RemainingLevel, InvLevel)) -> Self {
        LevelPair(pair.0, pair.1)
    }
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
                        .unwrap_or(InvLevel::Critical),
                )
            })
            .collect()
    }
}
