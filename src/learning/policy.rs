use crate::stock::StockDiscrete;

use crate::actions::ActionFlattened as Action;

pub trait Policy {
    fn chose_action(&self, agent_state: &StockDiscrete) -> Action;
}
