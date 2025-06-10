use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reward {
    pub val: i32,
}

impl Reward {
    pub fn new(val: i32) -> Self {
        Reward { val }
    }
}
