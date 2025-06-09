use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::goods::Good;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    ProduceGood(Good),
    Leisure,
}

impl Action {
    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.random_range(0..=1) {
            0 => Action::ProduceGood(Good::Berries),
            1 => Action::Leisure,
            _ => unreachable!(),
        }
    }

    pub fn random_weighted<R: Rng + ?Sized>(rng: &mut R, prob_produce_berries: f64) -> Self {
        if rng.random::<f64>() < prob_produce_berries {
            Action::ProduceGood(Good::Berries)
        } else {
            Action::Leisure
        }
    }
}
