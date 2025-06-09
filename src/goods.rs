use serde::{Deserialize, Serialize};

use crate::UInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]

// For goods, each has a lifetime remaining value before it is destroyed.
// For capital goods, (e.g. spear, timber), each has a number of uses remaining before it is destroyed.

pub enum Good {
    Berries { remaining_lifetime: UInt },
    // Fish{ remaining_lifetime: UInt },
    Basket { remaining_uses: UInt },
    // Spear{ remaining_uses: UInt },
    // Smoker{ remaining_uses: UInt },
    // Boat{ remaining_uses: UInt },
    // Timber,
    // Axe,
}

impl Good {
    pub fn is_consumer(&self) -> bool {
        match self {
            Good::Berries {
                remaining_lifetime: _,
            } => true,
            Good::Basket { remaining_uses: _ } => false,
        }
    }
}
