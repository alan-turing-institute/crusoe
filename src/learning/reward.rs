use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reward {
    pub val: i32,
}

impl Reward {
    pub fn new(val: i32) -> Self {
        Reward { val }
    }
}
