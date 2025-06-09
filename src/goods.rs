use serde::{Deserialize, Serialize};

use crate::UInt;

// A good in the abstract (as opposed to particular units of a good).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Good {
    Berries,
    Basket,
}

// For units of goods, each has a lifetime remaining value before it is destroyed.
// For capital goods, (e.g. spear, timber), each has a number of uses remaining before it is destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GoodsUnit {
    Berries { remaining_lifetime: UInt },
    // Fish{ remaining_lifetime: UInt },
    Basket { remaining_uses: UInt },
    // Spear{ remaining_uses: UInt },
    // Smoker{ remaining_uses: UInt },
    // Boat{ remaining_uses: UInt },
    // Timber,
    // Axe,
}

impl GoodsUnit {
    /// Returns a newly-produced unit of the given good.
    pub fn new(good: &Good) -> Self {
        match good {
            Good::Berries => GoodsUnit::Berries {
                remaining_lifetime: 10,
            },
            Good::Basket => GoodsUnit::Basket { remaining_uses: 10 },
        }
    }
}

impl Good {
    pub fn is_consumer(&self) -> bool {
        match self {
            Good::Berries => true,
            Good::Basket => false,
        }
    }

    /// Returns true if this good is produced using the given (higher order) good.
    pub fn is_produced_using(&self, good: Good) -> bool {
        match self {
            Good::Berries => match good {
                Good::Basket => true,
                _ => false,
            },
            Good::Basket => false,
        }
    }
}
