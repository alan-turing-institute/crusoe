use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    ProduceBerries,
    Leisure,
}

impl Action {
    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.random_range(0..=1) {
            0 => Action::ProduceBerries,
            1 => Action::Leisure,
            _ => unreachable!(),
        }
    }

    pub fn random_weighted<R: Rng + ?Sized>(rng: &mut R, prob_produce_berries: f64) -> Self {
        if rng.random::<f64>() < prob_produce_berries {
            Action::ProduceBerries
        } else {
            Action::Leisure
        }
    }
}
