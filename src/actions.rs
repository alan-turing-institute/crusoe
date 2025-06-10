use rand::Rng;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::goods::Good;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    ProduceGood(Good),
    Leisure,
}

impl From<ActionFlattened> for Action {
    fn from(action: ActionFlattened) -> Self {
        match action {
            ActionFlattened::ProduceBerries => Action::ProduceGood(Good::Berries),
            ActionFlattened::ProduceFish => Action::ProduceGood(Good::Fish),
            ActionFlattened::ProduceBasket => Action::ProduceGood(Good::Basket),
            ActionFlattened::ProduceSpear => Action::ProduceGood(Good::Spear),
            ActionFlattened::ProduceSmoker => Action::Leisure, // Placeholder for future goods
            ActionFlattened::ProduceBoat => Action::Leisure,   // Placeholder for future goods
            ActionFlattened::ProduceTimber => Action::Leisure, // Placeholder for future goods
            ActionFlattened::ProduceAxe => Action::Leisure,    // Placeholder for future goods
            ActionFlattened::Leisure => Action::Leisure,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum ActionFlattened {
    ProduceBerries,
    ProduceFish,
    ProduceBasket,
    ProduceSpear,
    ProduceSmoker,
    ProduceBoat,
    ProduceTimber,
    ProduceAxe,
    Leisure,
}

impl From<Action> for ActionFlattened {
    fn from(action: Action) -> Self {
        match action {
            Action::ProduceGood(Good::Berries) => ActionFlattened::ProduceBerries,
            Action::ProduceGood(Good::Fish) => ActionFlattened::ProduceFish,
            Action::ProduceGood(Good::Basket) => ActionFlattened::ProduceBasket,
            Action::ProduceGood(Good::Spear) => ActionFlattened::ProduceSpear,
            _ => ActionFlattened::Leisure,
        }
    }
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
