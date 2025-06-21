use enum_dispatch::enum_dispatch;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::actions::{Action, ActionFlattened};
use crate::goods::{Good, GoodsUnit, PartialGoodsUnit, Productivity};
use crate::learning::agent_state::DiscrRep;
use crate::learning::learning_agent::LearningAgent;
use crate::learning::reward::Reward;
use crate::stock::Stock;
use crate::valuation::RationalAgent;
use crate::{Model, NEGATIVE_REWARD, POSITIVE_REWARD, UInt};

#[enum_dispatch]
pub trait Agent {
    fn get_id(&self) -> u64;

    fn get_name(&self) -> &str;

    /// The stock of goods currently held by the agent.
    fn stock(&self) -> &Stock;

    /// The marginal productivity of the agent, given their current stock.
    fn stock_mut(&mut self) -> &mut Stock;

    fn set_stock(&mut self, stock: Stock);

    fn acquire(&mut self, goods_unit: GoodsUnit, quantity: UInt);

    fn acquire_partial(&mut self, partial_goods_unit: PartialGoodsUnit);

    fn get_partial(&self, good: Good) -> Option<PartialGoodsUnit>;

    /// Returns the number of units of the good produced per day,
    /// given the agent's existing stock.
    fn productivity(&self, good: &Good) -> Productivity {
        // TODO: make configurable.
        // Note: can modify default productivity for different agents (for specialisation).
        good.default_productivity(&self.stock())
    }

    /// The agent's choice of action in the next time step.
    fn choose_action(&mut self) -> Action;

    /// The agent's choice of action in the next time step.
    fn choose_action_with_model(&mut self, model: &Model) -> Action;

    /// Consume nutritional units for one time step and return false if insufficient were unavailable.
    fn consume(&mut self, nutritional_units: UInt) -> bool {
        let consumables = self.stock().next_consumables();
        if consumables.is_empty() {
            return false;
        }

        let mut outstanding_nutritional_units = nutritional_units;
        let mut stock_change: Vec<_> = vec![];
        for (good, qty) in consumables {
            if *qty > outstanding_nutritional_units {
                stock_change.push((good.clone(), outstanding_nutritional_units));
                outstanding_nutritional_units = 0;
                break;
            } else {
                stock_change.push((good.clone(), *qty));
                outstanding_nutritional_units -= *qty;
            }
        }
        // Update stock
        for (good, qty) in stock_change {
            self.stock_mut().remove(&good, qty);
        }
        // Returns false if the agent dies from lack of nutrients
        if outstanding_nutritional_units > 0 {
            return false;
        }
        true
    }

    /// Get the complete history of agent actions.
    fn action_history(&self) -> &[Action];

    /// Get the complete history of agent stocks.
    fn stock_history(&self) -> &[Stock];

    /// Get the reward history.
    fn reward_history(&self) -> &[Reward];

    /// Get the complete history of agent actions.
    fn action_history_mut(&mut self) -> &mut Vec<Action>;

    /// Get the complete history of agent stocks.
    fn stock_history_mut(&mut self) -> &mut Vec<Stock>;

    /// Get the reward history.
    fn reward_history_mut(&mut self) -> &mut Vec<Reward>;

    /// Return true if the agent is still alive.
    fn update_stock_history(&mut self, stock: &Stock) {
        self.stock_history_mut().push(stock.clone());
    }

    fn update_reward_history(&mut self, action: Action, is_alive: bool) {
        let reward = match (action, is_alive) {
            (Action::ProduceGood(_), true) => Reward::new(0),
            (Action::Leisure, true) => Reward::new(POSITIVE_REWARD),
            (_, false) => Reward::new(NEGATIVE_REWARD),
        };
        self.reward_history_mut().push(reward);
    }

    fn is_alive(&self) -> bool;

    fn set_liveness(&mut self, value: bool);

    /// Execute the given action and update the agent's stock with the result.
    fn act(&mut self, action: Action) {
        match action {
            Action::ProduceGood(good) => {
                let productivity = self.productivity(&good);

                // Increase the stock with the new production, taking existing stock into account.
                match productivity {
                    Productivity::Immediate(qty) => self.acquire(GoodsUnit::new(&good), qty),
                    Productivity::Delayed(_) => {
                        if let Some(partial_good) = self.get_partial(good) {
                            // If a partial good already exists, do the next step of production
                            // *and* acquire it if it's finished.
                            self.stock_mut().remove_partial(&partial_good);
                            match partial_good.increment_production() {
                                Some(new_partial_good) => {
                                    // Replace the partial goods unit with the new one.
                                    self.stock_mut().add_partial(new_partial_good);
                                }
                                None => {
                                    // If the production is complete, acquire the completed good.
                                    self.acquire(GoodsUnit::new(&partial_good.good), 1);
                                }
                            }
                        } else {
                            // Otherwise create a new partial good.
                            self.acquire_partial(PartialGoodsUnit::new(&good).expect(
                                "Delayed productivity implies multiple timesteps to produce.",
                            ))
                        }
                    }
                    Productivity::None => {} // Wasted action.
                }

                // Degrade any capital goods used in production (inc. materials).
                self.stock_mut()
                    .degrade_capital_stock(action)
                    .expect("Action choice should respect existing stock.");
            }
            Action::Leisure => (),
        }
    }

    /// Step the agent forward by one time step.
    fn step_forward(&mut self, action: Option<Action>) {
        // Select action if not given.
        let action = match action {
            Some(action) => action,
            None => self.choose_action(),
        };
        // Perform action, which updates the agent's stock
        self.act(action);

        // Consume stock, which updates whether the agent is alive
        // TODO: make required nutritional_units per time unit configurable.
        let survived = self.consume(3);

        // TODO: removed set_liveness for now, can be made configurable later.
        // self.set_liveness(is_alive);

        // Degrade the agent's stock.
        self.update_stock_history(&self.stock().clone());
        self.update_reward_history(action, survived);

        // Update the stock
        match survived {
            true => self.set_stock(self.stock().step_forward(action)),
            false => self.set_stock(Stock::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrusoeAgent {
    pub id: u64,
    pub stock: Stock,
    pub is_alive: bool,
    pub action_history: Vec<Action>,
    stock_history: Vec<Stock>,
    pub reward_history: Vec<Reward>,
}

impl CrusoeAgent {
    pub fn new(id: u64) -> Self {
        CrusoeAgent {
            id,
            stock: Stock::default(),
            is_alive: true,
            action_history: vec![],
            stock_history: vec![],
            reward_history: vec![],
        }
    }
}

impl Agent for CrusoeAgent {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_name(&self) -> &str {
        "Crusoe"
    }

    fn stock(&self) -> &Stock {
        &self.stock
    }

    fn stock_mut(&mut self) -> &mut Stock {
        &mut self.stock
    }

    fn set_stock(&mut self, stock: Stock) {
        self.stock = stock;
    }

    // TODO: consider moving the action_history update into the act method, so
    // self can be immutable here.
    fn choose_action(&mut self) -> Action {
        // let action = Action::random_weighted(&mut StdRng::from_os_rng(), 0.5);
        let action = Action::random(&mut StdRng::from_os_rng());
        // let action = Action::random(&mut StdRng::seed_from_u64(self.id));
        self.action_history.push(action);
        action
    }

    // TODO: consider moving the action_history update into the act method, so
    // self can be immutable here.
    fn choose_action_with_model(&mut self, model: &Model) -> Action {
        let action =
            model.sample_action_by_id(0, &self.stock.representation(), &mut StdRng::from_os_rng());
        self.action_history.push(action.into());
        action.into()
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

    fn is_alive(&self) -> bool {
        self.is_alive
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
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[enum_dispatch(Agent)]
pub enum AgentType {
    Crusoe(CrusoeAgent),
    Rational(RationalAgent),
    Rl(LearningAgent),
}

impl AgentType {
    pub fn action_history(&self) -> Vec<ActionFlattened> {
        match self {
            AgentType::Crusoe(agent) => {
                agent.action_history().iter().map(|a| (*a).into()).collect()
            }
            AgentType::Rl(agent) => agent.action_history().iter().map(|a| (*a).into()).collect(),
            AgentType::Rational(agent) => {
                agent.action_history().iter().map(|a| (*a).into()).collect()
            }
        }
    }

    pub fn reward_history(&self) -> Vec<Reward> {
        match self {
            AgentType::Crusoe(agent) => agent.reward_history().to_vec(),
            AgentType::Rl(agent) => agent.reward_history().to_vec(),
            AgentType::Rational(agent) => agent.reward_history().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import the functions from the parent module

    #[test]
    fn test_act() {
        let mut agent = CrusoeAgent::new(1);

        // Start with 2 units of berries.
        agent.acquire(GoodsUnit::new(&Good::Berries), 2);
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Berries)),
            Some(&2)
        );
        // Check that the stock of berries has not increased after a day of Leisure.
        agent.act(Action::Leisure);
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Berries)),
            Some(&2)
        );
        // Check that the stock of berries *has* increased after a day of production.
        agent.act(Action::ProduceGood(Good::Berries));
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Berries)),
            Some(&6) // Default productivity of berries is 4 units.
        );

        let mut agent = CrusoeAgent::new(1);
        // Start again with 5 units of berries and one basket.
        agent.acquire(GoodsUnit::new(&Good::Berries), 5);
        agent.acquire(GoodsUnit::new(&Good::Basket), 1);

        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Berries)),
            Some(&5)
        );
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Basket)),
            Some(&1)
        );

        // Check that the basket has not been used after a day of Leisure.
        agent.act(Action::Leisure);
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Berries)),
            Some(&5)
        );
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Basket)),
            Some(&1)
        );

        // Check that the basket has not been used after a day of production that does not
        // involve a basket.
        agent.act(Action::ProduceGood(Good::Fish));
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Berries)),
            Some(&5)
        );
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Basket)),
            Some(&1)
        );

        // Check that the basket has been used after a day of producing berries
        // and the stock of berries has increased.
        agent.act(Action::ProduceGood(Good::Berries));
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Berries)),
            Some(&13) // Default productivity of berries with a basket is 8 units.
        );
        let lifetime = GoodsUnit::new(&Good::Basket).remaining_lifetime - 1;
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit {
                good: Good::Basket,
                remaining_lifetime: lifetime
            }),
            Some(&1)
        );
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Basket)),
            None
        );

        let mut agent = CrusoeAgent::new(1);
        // Start again with 5 units of Fish.
        agent.acquire(GoodsUnit::new(&Good::Fish), 5);
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Fish)),
            Some(&5)
        );

        agent.act(Action::Leisure);
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Fish)),
            Some(&5)
        );

        agent.act(Action::ProduceGood(Good::Fish));
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Fish)),
            Some(&7)
        );

        // Test that production of smoked fish requires a smoker & consumes the stock of fish.
        // Without smoker:
        agent.act(Action::ProduceGood(Good::SmokedFish));
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Fish)),
            Some(&7)
        );
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::SmokedFish)),
            None
        );

        // With smoker but no fish:
        let mut agent = CrusoeAgent::new(1);
        agent.acquire(GoodsUnit::new(&Good::Smoker), 1);

        agent.act(Action::ProduceGood(Good::SmokedFish));
        assert_eq!(agent.stock().stock.get(&GoodsUnit::new(&Good::Fish)), None);

        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Smoker)),
            Some(&1)
        );
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::SmokedFish)),
            None
        );

        // With smoker and fish:
        let mut agent = CrusoeAgent::new(1);
        agent.acquire(GoodsUnit::new(&Good::Fish), 7);
        agent.acquire(GoodsUnit::new(&Good::Smoker), 1);

        agent.act(Action::ProduceGood(Good::SmokedFish));

        assert_eq!(agent.stock().stock.get(&GoodsUnit::new(&Good::Fish)), None);
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::SmokedFish)),
            Some(&7)
        );

        let lifetime = GoodsUnit::new(&Good::Smoker).remaining_lifetime - 1;
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Smoker)),
            None
        );
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit {
                good: Good::Smoker,
                remaining_lifetime: lifetime
            }),
            Some(&1)
        );

        // With smoker (having one use left) and fish:
        let mut agent = CrusoeAgent::new(1);
        agent.acquire(GoodsUnit::new(&Good::Fish), 7);
        agent.acquire(
            GoodsUnit {
                good: Good::Smoker,
                remaining_lifetime: 1,
            },
            1,
        );

        agent.act(Action::ProduceGood(Good::SmokedFish));

        assert_eq!(agent.stock().stock.get(&GoodsUnit::new(&Good::Fish)), None);
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::SmokedFish)),
            Some(&7)
        );

        // The smoker has been lost after its final use.
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit::new(&Good::Smoker)),
            None
        );
        assert_eq!(
            agent.stock().stock.get(&GoodsUnit {
                good: Good::Smoker,
                remaining_lifetime: 1
            }),
            None
        );

        // Test with partial goods.

        // Test production of a smoker.
        let mut agent = CrusoeAgent::new(1);
        agent.acquire(GoodsUnit::new(&Good::Timber), 3);

        assert!(!agent.stock().contains(&Good::Smoker));
        assert!(agent.stock().get_partial(Good::Smoker).is_none());
        assert_eq!(agent.stock().count_units(&Good::Timber), 3);

        agent.act(Action::ProduceGood(Good::Smoker));
        assert!(!agent.stock().contains(&Good::Smoker));
        assert_eq!(
            agent
                .stock()
                .get_partial(Good::Smoker)
                .unwrap()
                .time_to_completion,
            2
        );
        assert_eq!(agent.stock().count_units(&Good::Timber), 2);

        agent.act(Action::ProduceGood(Good::Smoker));
        assert!(!agent.stock().contains(&Good::Smoker));
        assert_eq!(
            agent
                .stock()
                .get_partial(Good::Smoker)
                .unwrap()
                .time_to_completion,
            1
        );
        assert_eq!(agent.stock().count_units(&Good::Timber), 1);

        agent.act(Action::ProduceGood(Good::Smoker));
        assert!(agent.stock().contains(&Good::Smoker));
        assert!(!agent.stock().contains(&Good::Timber));
        assert_eq!(agent.stock().count_units(&Good::Timber), 0);
        assert!(agent.stock().get_partial(Good::Smoker).is_none());

        // Test production of an axe.
        let mut agent = CrusoeAgent::new(1);
        agent.acquire(GoodsUnit::new(&Good::Berries), 20);

        agent.act(Action::ProduceGood(Good::Axe));
        assert!(!agent.stock().contains(&Good::Axe));

        assert!(
            agent
                .stock()
                .partial_stock
                .contains(&PartialGoodsUnit::new(&Good::Axe).unwrap())
        );

        // Agent acquires Axe after 2 consecutive days of production:
        agent.act(Action::ProduceGood(Good::Axe));
        assert!(agent.stock().contains(&Good::Axe));
    }

    #[test]
    fn test_consume() {
        let mut agent = CrusoeAgent::new(1);
        agent.stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            5,
        );
        agent.consume(3);
        // Expected stock after consumption is 2 units of berries
        // (three units were consumed) with remaining lifetime 10.
        let mut expected = Stock::default();
        expected.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            2,
        );
        assert_eq!(agent.stock, expected);
        agent.consume(2);
        // Expected stock after consumption of the remaining 2 units
        // of berries is empty.
        assert!(agent.stock.stock.is_empty());
    }

    #[test]
    fn test_consume_different_goods() {
        let mut agent = CrusoeAgent::new(1);
        // Add 2 units of berries and 1 unit of fish.
        agent.acquire(GoodsUnit::new(&Good::Berries), 2);
        agent.acquire(GoodsUnit::new(&Good::Fish), 1);

        // The agent can successfully consume 3 nutritional units.
        assert!(agent.consume(3));
        assert!(agent.stock.stock.is_empty());
    }

    #[test]
    fn test_step_forward() {
        let mut agent = CrusoeAgent::new(1);
        agent.stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            5,
        );
        agent.step_forward(Some(Action::Leisure));

        // Expected stock after one step forward is 2 units of berries
        // (three units were consumed) with remaining lifetime 9.
        let mut expected = Stock::default();
        expected.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 9,
            },
            2,
        );
        assert_eq!(agent.stock, expected);

        // Test production of a good that takes multiple timesteps to complete.
        let mut agent = CrusoeAgent::new(1);

        // Start with sufficient materials and sufficient consumer goods to survive.
        agent.acquire(GoodsUnit::new(&Good::Berries), 15);
        agent.acquire(GoodsUnit::new(&Good::Timber), 3);

        agent.step_forward(Some(Action::ProduceGood(Good::Smoker)));

        // After one action, the Smoker is partially completed with 2 timesteps left to completion.
        // One unit of Timber has been used.
        assert!(!agent.stock().contains(&Good::Smoker));
        assert_eq!(
            agent
                .stock()
                .get_partial(Good::Smoker)
                .unwrap()
                .time_to_completion,
            2
        );
        assert_eq!(agent.stock().count_units(&Good::Timber), 2);

        agent.step_forward(Some(Action::ProduceGood(Good::Smoker)));

        // After two actions, the Smoker is partially completed with 1 timestep left to completion.
        // Two units of Timber have been used.
        assert!(!agent.stock().contains(&Good::Smoker));
        assert_eq!(
            agent
                .stock()
                .get_partial(Good::Smoker)
                .unwrap()
                .time_to_completion,
            1
        );
        assert_eq!(agent.stock().count_units(&Good::Timber), 1);

        // If the agent fails to continue production, the time to completion increases.
        agent.step_forward(Some(Action::Leisure));

        assert!(!agent.stock().contains(&Good::Smoker));
        assert_eq!(
            agent
                .stock()
                .get_partial(Good::Smoker)
                .unwrap()
                .time_to_completion,
            2
        );
        assert_eq!(agent.stock().count_units(&Good::Timber), 1);

        // Now the agent will need more timber to complete production of the Smoker.
        agent.acquire(GoodsUnit::new(&Good::Timber), 1);
        assert_eq!(agent.stock().count_units(&Good::Timber), 2);

        agent.step_forward(Some(Action::ProduceGood(Good::Smoker)));

        // Now the Smoker is partially completed with 1 timestep left to completion.
        // One unit of Timber is left.
        assert!(!agent.stock().contains(&Good::Smoker));
        assert_eq!(
            agent
                .stock()
                .get_partial(Good::Smoker)
                .unwrap()
                .time_to_completion,
            1
        );
        assert_eq!(agent.stock().count_units(&Good::Timber), 1);

        agent.step_forward(Some(Action::ProduceGood(Good::Smoker)));

        // Finally, the Smoker is complete.
        assert!(agent.stock().contains(&Good::Smoker));
        assert!(agent.stock().get_partial(Good::Smoker).is_none());
        assert_eq!(agent.stock().count_units(&Good::Timber), 0);
    }

    #[test]
    fn test_acquire() {
        // Test acquisition of berries.
        let mut agent = CrusoeAgent::new(1);
        let goods_unit = GoodsUnit::new(&Good::Berries);

        let mut stock = Stock::default();
        assert_eq!(agent.stock(), &stock);

        agent.acquire(goods_unit, 2);

        stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            2,
        );
        assert_eq!(agent.stock(), &stock);

        agent.acquire(goods_unit, 1);

        stock.add(
            GoodsUnit {
                good: Good::Berries,
                remaining_lifetime: 10,
            },
            1,
        );
        assert_eq!(agent.stock(), &stock);
    }
}
