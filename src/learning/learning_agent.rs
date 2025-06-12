use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::actions::Action;
use crate::agent::Agent;
use crate::goods::{Good, GoodsUnit, PartialGoodsUnit, Productivity};
use crate::learning::agent_state::DiscrRep;
use crate::learning::reward::Reward;
use crate::stock::Stock;
use crate::{Model, NEGATIVE_REWARD, POSITIVE_REWARD, UInt};

// LearningAgent is currently just a clone of CrusoeAgent. The idea would
// be to have each agent type in its own module (or sub-directory)

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningAgent {
    pub id: u64,
    pub stock: Stock,
    pub is_alive: bool,
    pub action_history: Vec<Action>,
    stock_history: Vec<Stock>,
    pub reward_history: Vec<Reward>,
}

impl LearningAgent {
    pub fn new(id: u64) -> Self {
        LearningAgent {
            id,
            stock: Stock::default(),
            is_alive: true,
            action_history: vec![],
            stock_history: vec![],
            reward_history: vec![],
        }
    }
}

impl Agent for LearningAgent {
    fn get_id(&self) -> u64 {
        todo!()
    }

    fn get_name(&self) -> &str {
        "LearningAgent"
    }

    fn stock(&self) -> &Stock {
        &self.stock
    }

    // TODO: Some capital goods take multiple time units to produce.
    /// Returns the number of units of the good produced per day, given the
    /// agent's existing stock. The boolean flag indicates whether the
    /// integer in the first slot should be interpreted as a fraction.
    /// e.g. (5, false) indicates  5 units can be produced in one day,
    /// while (5, true) indicates that a single unit takes 5 days to produce.
    fn productivity(&self, good: Good) -> Productivity {
        // TODO: make configurable.
        // TODO: can modify default productivity for different agents (for specialisation).
        good.default_productivity(&self.stock)
    }

    // TODO: consider moving teh action_history update into act method, so
    // self can be immutable here.
    fn choose_action(&mut self) -> Action {
        // let action = Action::random_weighted(&mut StdRng::from_os_rng(), 0.5);
        let action = Action::random(&mut StdRng::from_os_rng());
        // let action = Action::random(&mut StdRng::seed_from_u64(self.id));
        self.action_history.push(action);
        action
    }

    // TODO: consider moving teh action_history update into act method, so
    // self can be immutable here.
    fn choose_action_with_model(&mut self, model: &Model) -> Action {
        let action =
            model.sample_action_by_id(0, &self.stock.representation(), &mut StdRng::from_os_rng());
        self.action_history.push(action.into());
        action.into()
    }

    // TODO: initially 1 unit of nutrition required per time unit
    // TODO: in future, make this e.g. 3 units which could be 1 berries + 2 fish.

    /// Consume the requisite units of food per time unit.
    /// Return false if insufficient stock was available.
    fn consume(&mut self, nutritional_units: UInt) -> bool {
        let consumables = self.stock.next_consumables();
        if consumables.is_empty() {
            // println!("DEBUG: consume - no consumables available");
            return false;
        }

        let mut outstanding_nutritional_units = nutritional_units;
        let mut stock_change: Vec<_> = vec![];
        for (good, qty) in consumables {
            if *qty > outstanding_nutritional_units {
                // return self.consume(nutritional_units - qty);
                stock_change.push((*good, outstanding_nutritional_units));
                outstanding_nutritional_units = 0;
                break;
            } else {
                stock_change.push((*good, *qty));
                outstanding_nutritional_units -= *qty;
            }
        }
        // Update stock
        for (good, qty) in stock_change {
            self.stock.remove(&good, qty);
        }
        // Returns false if the agent dies from lack of nutrients
        if outstanding_nutritional_units > 0 {
            return false;
        }
        true
    }

    fn is_alive(&self) -> bool {
        self.is_alive
    }

    fn act(&mut self, action: Action) {
        match action {
            Action::ProduceGood(good) => {
                let productivity = self.productivity(good);
                match productivity {
                    Productivity::Immediate(qty) => self.stock.add(GoodsUnit::new(&good), qty),
                    Productivity::Delayed(_) => {
                        if let Some(mut partial_good) = self.stock.get_partial(good) {
                            // If a partial good already exists, do the next step of production.
                            partial_good.increment_production();
                        } else {
                            // Otherwise create a new partial good.
                            self.stock.add_partial(PartialGoodsUnit::new(&good).expect(
                                "Delayed productivity implies multiple timesteps to produce.",
                            ))
                        }
                    }
                    Productivity::None => {} // Wasted action.
                }
            }
            Action::Leisure => (),
        }
    }

    fn step_forward(&mut self, action: Option<Action>) {
        // Select action if not given.
        let action = match action {
            Some(a) => a,
            None => self.choose_action(),
        };
        // Perform action, which updates the agent's stock
        self.act(action);
        // Consume stock, which updates whether the agent is alive
        // TODO: make required nutritional_units per time unit configurable.
        // self.is_alive = self.consume(1);
        let is_alive = self.consume(1);
        // Degrade the agent's stock.
        self.stock_history.push(self.stock.clone());
        self.stock = self.stock.step_forward(action);
        // Update reward history
        match (action, is_alive) {
            (Action::ProduceGood(_), true) => {
                self.reward_history.push(Reward::new(0));
            }
            (Action::Leisure, true) => {
                self.reward_history.push(Reward::new(POSITIVE_REWARD));
            }
            (_, false) => {
                self.reward_history.push(Reward::new(NEGATIVE_REWARD));
            }
        };
    }

    fn action_history(&self) -> &[Action] {
        &self.action_history
    }
    fn stock_history(&self) -> &[Stock] {
        &self.stock_history
    }
    fn reward_history(&self) -> &[Reward] {
        &self.reward_history
    }
    fn action_history_mut(&mut self) -> &mut Vec<Action> {
        &mut self.action_history
    }
    fn stock_history_mut(&mut self) -> &mut Vec<Stock> {
        &mut self.stock_history
    }
    fn reward_history_mut(&mut self) -> &mut Vec<Reward> {
        &mut self.reward_history
    }
    fn set_liveness(&mut self, value: bool) {
        self.is_alive = value;
    }

    fn acquire(&mut self, goods_unit: GoodsUnit, quantity: UInt) {
        self.stock.add(goods_unit, quantity);
    }

    fn acquire_partial(&mut self, partial_goods_unit: PartialGoodsUnit) {
        self.stock.add_partial(partial_goods_unit);
    }

    fn get_partial(&self, good: Good) -> Option<PartialGoodsUnit> {
        self.stock.get_partial(good)
    }

    fn stock_mut(&mut self) -> &mut Stock {
        &mut self.stock
    }

    fn set_stock(&mut self, stock: Stock) {
        self.stock = stock;
    }
}
