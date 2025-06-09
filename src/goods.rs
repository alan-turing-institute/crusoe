use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Good {
    Berries,
    // Fish,
    // Spear,
    // Smoker,
    // Boat,
    // Timber,
    // Axe,
}
