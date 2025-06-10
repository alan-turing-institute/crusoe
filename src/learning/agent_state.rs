use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    goods::Good,
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
impl DiscrRep<Good, LevelPair> for Stock {
    fn representation(&self) -> Vec<(Good, LevelPair)> {
        let hm: HashMap<Good, LevelPair> = self
            .discretise()
            .stock
            .into_iter()
            .map(|(good, level)| (good.good, LevelPair::from((good.remaining_lifetime, level))))
            .collect();
        Good::iter()
            .map(|good| {
                (
                    good,
                    hm.get(&good)
                        .cloned()
                        .unwrap_or(LevelPair(RemainingLevel::Critical, InvLevel::Critical)),
                )
            })
            .collect()
    }
}

impl LevelPair {
    /// Iterate over all possible combinations of InvLevel for LevelPair
    pub fn iter() -> impl Iterator<Item = LevelPair> {
        InvLevel::iter().flat_map(|a| RemainingLevel::iter().map(move |b| LevelPair::from((b, a))))
    }
}
