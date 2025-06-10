use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{goods::Good, stock::Stock};

pub trait DiscrRep<S, L> {
    fn representation(&self) -> Vec<(S, L)>;
}

// Approximate Q table size
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
        self.discretise()
            .stock
            .into_iter()
            .map(|(good, level)| (good.good, LevelPair::from((good.remaining_lifetime, level))))
            .collect()
    }
}

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
