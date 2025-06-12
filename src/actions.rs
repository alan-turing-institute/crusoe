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
            ActionFlattened::ProduceSmoker => Action::ProduceGood(Good::Smoker),
            ActionFlattened::ProduceBoat => Action::ProduceGood(Good::Boat),
            ActionFlattened::ProduceTimber => Action::ProduceGood(Good::Timber),
            ActionFlattened::ProduceAxe => Action::ProduceGood(Good::Axe),
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
            Action::ProduceGood(Good::Smoker) => ActionFlattened::ProduceSmoker,
            Action::ProduceGood(Good::Boat) => ActionFlattened::ProduceBoat,
            Action::ProduceGood(Good::Timber) => ActionFlattened::ProduceTimber,
            Action::ProduceGood(Good::Axe) => ActionFlattened::ProduceAxe,
            Action::Leisure => ActionFlattened::Leisure,
        }
    }
}

impl Action {
    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.random_range(0..=8) {
            0 => Action::Leisure,
            1 => Action::ProduceGood(Good::Berries),
            2 => Action::ProduceGood(Good::Fish),
            3 => Action::ProduceGood(Good::Basket),
            4 => Action::ProduceGood(Good::Spear),
            5 => Action::ProduceGood(Good::Smoker),
            6 => Action::ProduceGood(Good::Boat),
            7 => Action::ProduceGood(Good::Timber),
            8 => Action::ProduceGood(Good::Axe),
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
