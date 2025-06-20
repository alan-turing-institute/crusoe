use super::{agent_state::DiscrRep, q_table::QKey, reward::Reward};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct History<T, S, L, A>
where
    T: DiscrRep<S, L> + Clone,
    A: Clone,
{
    pub trajectory: Vec<SAR<T, S, L, A>>,
    agent_state_items: PhantomData<S>,
    agent_state_item_levels: PhantomData<L>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SAR<T, S, L, A>
where
    T: DiscrRep<S, L>,
    A: Clone,
{
    pub state: T,
    pub action: A,
    pub reward: Reward,
    agent_state_items: PhantomData<S>,
    agent_state_item_levels: PhantomData<L>,
}

impl<T, S, L, A> History<T, S, L, A>
where
    T: DiscrRep<S, L> + Clone,
    A: Clone,
{
    pub fn new() -> Self {
        Self {
            trajectory: Vec::new(),
            agent_state_items: PhantomData,
            agent_state_item_levels: PhantomData,
        }
    }
    pub fn push(&mut self, sar: SAR<T, S, L, A>) {
        self.trajectory.push(sar);
    }
    pub fn last_state_action(&self) -> Option<(T, A)> {
        let len = self.trajectory.len();
        if len > 0 {
            Some((
                self.trajectory[self.trajectory.len() - 1].state.clone(),
                self.trajectory[self.trajectory.len() - 1].action.clone(),
            ))
        } else {
            None
        }
    }
    pub fn len(&self) -> usize {
        self.trajectory.len()
    }
}

impl<T, S, L, A> SAR<T, S, L, A>
where
    T: DiscrRep<S, L> + Clone,
    A: Clone,
{
    pub fn new(state: T, action: A, reward: Reward) -> Self {
        SAR {
            state,
            action,
            reward,
            agent_state_items: PhantomData,
            agent_state_item_levels: PhantomData,
        }
    }

    pub fn representation(&self) -> QKey<S, L, A> {
        QKey(self.state.representation(), self.action.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AgenteState is now Stock
    // AgentStateItems is now Good
    use crate::actions::ActionFlattened as Action;
    use crate::goods::GoodsUnitLevel as AgentStateItems;
    use crate::stock::{InvLevel, Stock as AgentState};

    fn get_test_history() -> History<AgentState, AgentStateItems, InvLevel, Action> {
        History {
            trajectory: vec![SAR::new(
                AgentState::default(),
                Action::ProduceBerries,
                Reward { val: -1 },
            )],
            agent_state_items: PhantomData,
            agent_state_item_levels: PhantomData,
        }
    }

    #[test]
    fn test_history_push() {
        let mut history = get_test_history();
        let sar = SAR::new(AgentState::default(), Action::Leisure, Reward { val: -1 });
        let sar2 = SAR::new(AgentState::default(), Action::Leisure, Reward { val: -2 });
        history.push(sar.clone());
        assert_eq!(history.len(), 2);
        assert_eq!(history.trajectory.last().unwrap(), &sar);
        assert_ne!(history.trajectory.last().unwrap(), &sar2);
    }

    #[test]
    fn test_last_state_action() {
        assert_eq!(
            get_test_history().last_state_action(),
            Some((AgentState::default(), Action::ProduceBerries))
        )
    }
}
