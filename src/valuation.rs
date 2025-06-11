use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    UInt,
    actions::Action,
    agent::Agent,
    goods::{Good, GoodsUnit, PartialGoodsUnit, Productivity},
    stock::Stock,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalAgent {
    id: u64,
    stock: Stock,
    is_alive: bool,
    action_history: Vec<Action>,
    stock_history: Vec<Stock>,
    daily_nutrition: UInt,
}

impl RationalAgent {
    pub fn new(id: u64, daily_nutrition: UInt) -> Self {
        RationalAgent {
            id,
            stock: Stock::default(),
            is_alive: true,
            action_history: vec![],
            stock_history: vec![],
            daily_nutrition: daily_nutrition,
        }
    }

    /// Returns the marginal benefit to the agent of the product (output) of the specified action,
    /// given the existing stock.
    ///
    /// We define the marginal benefit of an action to produce a consumer good $g$, given existing
    /// stock $S$, as the (discounted) sum of the marginal values of the additional units.
    fn marginal_benefit_of_action(&self, action: &Action) -> f32 {
        let good = match action {
            Action::ProduceGood(good) => Some(good),
            Action::Leisure => None,
        };
        match good {
            Some(good) => match good.is_consumer() {
                true => self.marginal_benefit_of_producing_consumer_goods(good),
                false => todo!(),
            },
            None => 0.0,
        }
    }

    /// Returns the marginal value of a unit of a capital good, given the existing stock.
    fn marginal_unit_value_of_capital_good(&self, good: &Good) -> f32 {
        if good.is_consumer() {
            panic!("Expected capital good.")
        }

        // First-order capital goods.
        for alt_good in Good::iter() {}
        let result: f32 = Good::iter()
            .filter(|g| g.is_consumer())
            .filter(|g| g.is_produced_using(good))
            .map(|consumer_good| {
                self.value_generated_by_first_order_capital_good(good, &consumer_good)
            })
            .sum();

        todo!()
    }

    fn value_generated_by_first_order_capital_good(
        &self,
        capital_good: &Good,
        consumer_good: &Good,
    ) -> f32 {
        self.validate_consumer_and_first_order_capital_good(capital_good, consumer_good);
        // TODO: include discounting, which requires finding the times of most productive
        // use of the capital good in producing the consumer good and the number of days taken
        // to produce the capital good. For simplicity, we currently ignore discounting.

        let capital_goods_unit = GoodsUnit::new(capital_good);
        let mut dummy_agent = self.clone();

        // Take into account the possibility that the stock may already contain the capital good.
        let mut factor = 1.0;
        if self.stock().contains(capital_good) {
            let capital_goods_in_stock = &self.stock().next_capital_goods_units(capital_good);
            let usable_days: u32 = capital_goods_in_stock
                .iter()
                .map(|(goods_unit, qty)| *qty * goods_unit.remaining_lifetime)
                .sum();
            // Reduce the result by a factor equal to the lifetime of the new capital goods unit
            // divided by the number of days of use already available from the existing stock.
            factor = (capital_goods_unit.remaining_lifetime as f32) / (usable_days as f32);

            // Remove any units of the capital good from the dummy agent's stock.
            for goods_unit_in_stock in capital_goods_in_stock {
                dummy_agent
                    .stock_mut()
                    .remove(goods_unit_in_stock.0, *goods_unit_in_stock.1);
            }
        }
        // Check that the dummy agent's stock does not contain the capital good.
        assert!(!dummy_agent.stock().contains(capital_good));

        // Get the productivity of the consumer good with and without the capital good.
        let productivity_sans = match dummy_agent.productivity(consumer_good) {
            Productivity::Immediate(quantity) => quantity,
            Productivity::Delayed(_) => unreachable!("Consumer goods have immediate productivity"),
            Productivity::None => unreachable!("Consumer goods have immediate productivity"),
        };
        dummy_agent.acquire(capital_goods_unit, 1);
        let productivity_with = match dummy_agent.productivity(consumer_good) {
            Productivity::Immediate(quantity) => quantity,
            Productivity::Delayed(_) => unreachable!("Consumer goods have immediate productivity"),
            Productivity::None => unreachable!("Consumer goods have immediate productivity"),
        };
        // Check that the productivity with the capital good exceeds that without.
        assert!(productivity_with > productivity_sans);

        let mut sum: f32 = 0.0;
        let mut count = 0;
        while count + productivity_sans != productivity_with {
            // TODO: discounting.

            // Add the marginal value of one unit of the consumer good, given a stock
            // that contains `count` additional units of the consumer good.
            sum = sum + dummy_agent.marginal_unit_value_of_consumer_good(consumer_good);
            dummy_agent.acquire(GoodsUnit::new(consumer_good), 1);

            count = count + 1;
        }

        factor * (capital_goods_unit.remaining_lifetime as f32) * sum
    }

    // fn times_of_most_productive_first_order_use(&self, capital_good: &Good, consumer_good: &Good) ->  {
    //     self.validate_consumer_and_first_order_capital_good(capital_good, consumer_good);
    // }

    fn validate_consumer_and_first_order_capital_good(
        &self,
        capital_good: &Good,
        consumer_good: &Good,
    ) {
        if capital_good.is_consumer() {
            panic!("Expected capital good.")
        }
        if !consumer_good.is_consumer() {
            panic!("Expected consumer good.")
        }
        if !consumer_good.is_produced_using(capital_good) {
            panic!("Invalid higher- and lower-order pair of goods.")
        }
    }

    /// Returns the marginal benefit to the agent of producing a consumer good,
    /// given the existing stock.
    fn marginal_benefit_of_producing_consumer_goods(&self, good: &Good) -> f32 {
        if !good.is_consumer() {
            panic!("Expected consumer good.")
        }
        let productivity = match self.productivity(good) {
            crate::goods::Productivity::Immediate(quantity) => quantity,
            _ => {
                panic!("All consumer goods have immediate productivity.")
            }
        };
        let mut sum: f32 = 0.0;
        let mut count = 0;
        let mut dummy_agent = self.clone();
        while count != productivity {
            // TODO: discounting.
            sum = sum + dummy_agent.marginal_unit_value_of_consumer_good(good);
            dummy_agent.acquire(GoodsUnit::new(good), 1);
            count = count + 1;
        }
        sum
    }

    /// Returns the marginal value of a unit of a consumer good, given the existing stock.
    ///
    /// We define the marginal unit value of a consumer good $g$, given existing stock $S$, as
    /// the min amount of time required to produce equivalent additional sustenance to 1 additional
    /// unit of g (given stock S). If 1 additional unit of g (given stock S) produces no additional
    /// sustenance, it's marginal unit value is zero.
    fn marginal_unit_value_of_consumer_good(&self, good: &Good) -> f32 {
        if !good.is_consumer() {
            panic!("Expected consumer good.")
        }
        // 1. Count additional days of sustenance from 1 additional unit of g
        let additional_sustenance = self.additional_sustenance(good);
        // If the additional sustenance is zero, the value of the marginal unit is also zero.
        // (In practice, goods will typically be considered in quantities greater than 1 unit.)
        if additional_sustenance == 0 {
            return 0.0;
        }

        // 2. Initialise the minimum time to produce equivalent sustenance, to the value for
        // this good. Return zero value if the agent's productivity for this good is None.
        let productivity_per_unit_time = self.productivity(good).per_unit_time();
        if productivity_per_unit_time.is_none() {
            return 0.0;
        }
        // Time to produce 1 unit of the good is (1 / amount produced in one day's production).
        let mut min_equiv = (1 as f32) / productivity_per_unit_time.unwrap();

        // 3. For every consumer good, compute the time taken to produce the same number of
        // days of sustenance.
        for alt_good in Good::iter() {
            // Ignore the good itself as we alreday initialised min_equiv for it.
            if alt_good == *good {
                continue;
            }
            if let Some(t) =
                self.time_to_equiv_sustenance(alt_good, additional_sustenance, min_equiv)
            {
                if t < min_equiv {
                    min_equiv = t;
                }
            }
        }
        // 3. Return the minium equivalent.
        min_equiv
    }

    /// Returns the time taken to produce enough of an alternative good to reach a target measure
    /// of sustenance. Returns None if alt_good is a capital good or if the max limit is exceeded.
    fn time_to_equiv_sustenance(
        &self,
        alt_good: Good,
        target_sustenance: u32,
        max: f32,
    ) -> Option<f32> {
        if target_sustenance == 0 {
            panic!("ERROR: target sustenance must be greater than zero.");
        }
        let mut dummy_agent = self.clone();
        let survival_time = self.count_timesteps_till_death(None);
        match alt_good.is_consumer() {
            true => {
                if let Some(productivity) = self.productivity(&alt_good).per_unit_time() {
                    let mut count_days = 0;
                    loop {
                        // Simulate one day of action to produce the alternative good.
                        // NB: we truncate productivity as this will be an integer for Productivity::Immediate consumer goods.
                        dummy_agent.acquire(GoodsUnit::new(&alt_good), productivity.trunc() as u32);
                        count_days += 1;

                        // Compute the new survival time with the extra units of the alternative goods.
                        let new_survival_time = dummy_agent.count_timesteps_till_death(None);

                        // If the additional sustenance exceeds the target sustenance, return
                        // the equivalent day count necessasry to produce the additional goods.
                        let additional_survival = new_survival_time - survival_time;
                        if additional_survival > target_sustenance {
                            return Some((count_days as f32) / productivity);
                        }
                        // If the max limit is already exceeded, return None
                        if ((count_days as f32) / productivity) > max {
                            return None;
                        }
                    }
                }
                // If the marginal productivity of the alt_good is None, return None
                None
            }
            false => None,
        }
    }

    /// Counts the number of additional days of survival provided by one additional unit of a good.
    fn additional_sustenance(&self, good: &Good) -> u32 {
        let survival_days = self.count_timesteps_till_death(None);
        let additional_survival_days = &self.count_timesteps_till_death(Some(&good));
        additional_survival_days - survival_days
    }

    /// Counts the number of timesteps that the agent can survive with the current
    /// stock, plus one unit of an optional additional good, assuming only consumption
    /// (i.e. no production/acquision of new goods).
    fn count_timesteps_till_death(&self, additional_good: Option<&Good>) -> UInt {
        let mut dummy_agent = self.clone();
        if let Some(good) = additional_good {
            dummy_agent.acquire(GoodsUnit::new(good), 1);
        }
        let mut count = 0;
        loop {
            let action = Action::Leisure;
            if !dummy_agent.consume(self.daily_nutrition) {
                break; // Break out as soon as death happens.
            }
            dummy_agent.set_stock(dummy_agent.stock().step_forward(action));
            count += 1;
        }
        count
    }
}

impl Agent for RationalAgent {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_name(&self) -> &str {
        "Rational"
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

    fn choose_action(&mut self) -> Action {
        todo!()
    }

    fn action_history(&self) -> Vec<Action> {
        self.action_history.clone()
    }

    fn stock_history(&self) -> Vec<Stock> {
        self.stock_history.clone()
    }

    fn update_stock_history(&mut self) {
        self.stock_history.push(self.stock().clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goods::{Good, GoodsUnit};

    #[test]
    fn test_marginal_benefit_of_action() {
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        let action = Action::ProduceGood(Good::Berries);

        // Given an initially empty stock, the marginal value of the first two units of berries
        // is zero. The third unit has a marginal value of 1/4. The fourth unit has a marginal
        // value of zero. So the marginal benefit of the action to produce berries is 1/4.
        assert_eq!(agent.marginal_benefit_of_action(&action), 0.25);

        // Start again with empty stock.
        let mut agent = RationalAgent::new(1, daily_nutrition);

        let action = Action::ProduceGood(Good::Fish);

        // Given an initially empty stock, the marginal value of the first two units of fish
        // is zero. So the marginal benefit of the action to produce fish is 0.
        assert_eq!(agent.marginal_benefit_of_action(&action), 0.0);

        agent.acquire(GoodsUnit::new(&Good::Berries), 1);

        // Given an initial stock of 1 unit of berries, the marginal value of the first unit
        // of fish is zero but the value of the second is 1/4. So the marginal benefit of the
        // action to produce fish is 1/4.
        assert_eq!(agent.marginal_benefit_of_action(&action), 0.25);
    }

    #[test]
    fn test_marginal_unit_value() {
        // Test marginal unit value of berries, given zero stock.
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        let berries_unit = GoodsUnit::new(&Good::Berries);

        // 1 additional unit of berries provides no additional sustenance when stock is empty.
        // So the marginal value of 1 unit of berries is zero.
        assert_eq!(
            agent.marginal_unit_value_of_consumer_good(&Good::Berries),
            0.0
        );

        agent.acquire(berries_unit, 1);
        // 1 additional unit of berries provides no additional sustenance when stock is 1 unit of berries.
        // So the marginal value of 1 unit of berries is zero.
        assert_eq!(
            agent.marginal_unit_value_of_consumer_good(&Good::Berries),
            0.0
        );

        agent.acquire(berries_unit, 1);

        // 1 additional unit of berries provides 1 day of additional sustenance when
        // stock is 2 units of berries. Minimum time required to produce sustanance
        // equivalent to additional 1 unit of berries is 1/4 days (by producing berries).
        // So the marginal value of 1 unit of berries is 1/4.
        assert_eq!(
            agent.marginal_unit_value_of_consumer_good(&Good::Berries),
            0.25
        );

        // Start again with an empty stock.
        let mut agent = RationalAgent::new(1, daily_nutrition);

        let fish_unit = GoodsUnit::new(&Good::Fish);

        // 1 additional unit of fish provides no additional sustenance when stock is empty.
        // So the marginal value of 1 unit of fish is zero.
        assert_eq!(agent.marginal_unit_value_of_consumer_good(&Good::Fish), 0.0);

        agent.acquire(fish_unit, 1);
        // 1 additional unit of fish provides no additional sustenance when stock is 1 unit of fish.
        // So the marginal value of 1 unit of fish is zero.
        assert_eq!(agent.marginal_unit_value_of_consumer_good(&Good::Fish), 0.0);

        agent.acquire(fish_unit, 1);
        // 1 additional unit of fish provides 1 day of additional sustenance when
        // stock is 2 units of fish. Minimum time required to produce sustanance
        // equivalent to additional 1 unit of fish is 1/4 days (by producing berries, not fish!).
        // So the marginal value of 1 unit of fish is 1/4.
        assert_eq!(
            agent.marginal_unit_value_of_consumer_good(&Good::Fish),
            0.25
        );
        assert_eq!(
            agent.marginal_unit_value_of_consumer_good(&Good::Berries),
            0.25
        );
    }

    #[test]
    fn test_additional_sustenance() {
        // Test additional sustenance from berries.
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);
        let berries_unit = GoodsUnit::new(&Good::Berries);

        // No additional sustenance from 1 unit of berries (when starting from none).
        let expected = 0;
        assert_eq!(agent.additional_sustenance(&Good::Berries), expected);

        // No additional sustenance from 1 unit of berries (when starting from 1 unit).
        agent.acquire(berries_unit, 1);
        let expected = 0;
        assert_eq!(agent.additional_sustenance(&Good::Berries), expected);

        agent.acquire(berries_unit, 1);

        let expected = 1;
        // One additional day's sustenance from 1 unit of berries (when starting from 2 units).
        assert_eq!(agent.additional_sustenance(&Good::Berries), expected);
        // One additional day's sustenance from 1 unit of fish (when starting from 2 units of berries).
        assert_eq!(agent.additional_sustenance(&Good::Fish), expected);
    }

    #[test]
    fn test_count_timesteps_till_death() {
        // Test additional sustenance from berries.
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        // With zero stock the timesteps till death is zero.
        let expected = 0;
        assert_eq!(agent.count_timesteps_till_death(None), expected);

        let berries_unit = GoodsUnit::new(&Good::Berries);
        let fish_unit = GoodsUnit::new(&Good::Fish);

        agent.acquire(berries_unit, 1);

        // With one unit of berries the timesteps till death is zero.
        let expected = 0;
        assert_eq!(agent.count_timesteps_till_death(None), expected);

        agent.acquire(berries_unit, 1);

        // With two units of berries the timesteps till death is zero.
        let expected = 0;
        assert_eq!(agent.count_timesteps_till_death(None), expected);

        // With two units of berries the timesteps till death
        // *with one additional unit of berries* is one.
        let expected = 1;
        assert_eq!(
            agent.count_timesteps_till_death(Some(&Good::Berries)),
            expected
        );
        // With two units of berries the timesteps till death
        // *with one additional unit of fish* is one.
        let expected = 1;
        assert_eq!(
            agent.count_timesteps_till_death(Some(&Good::Fish)),
            expected
        );

        agent.acquire(berries_unit, 1);

        // With three units of berries the timesteps till death is one.
        let expected = 1;
        assert_eq!(agent.count_timesteps_till_death(None), expected);

        agent.acquire(berries_unit, 1);

        // With four units of berries the timesteps till death is one.
        let expected = 1;
        assert_eq!(agent.count_timesteps_till_death(None), expected);

        agent.acquire(berries_unit, 1);

        // With five units of berries the timesteps till death is one.
        let expected = 1;
        assert_eq!(agent.count_timesteps_till_death(None), expected);

        // With five units of berries the timesteps till death
        // *with one additional unit of berries* is two.
        let expected = 2;
        assert_eq!(
            agent.count_timesteps_till_death(Some(&Good::Berries)),
            expected
        );

        agent.acquire(berries_unit, 1);

        // With six units of berries the timesteps till death is two.
        let expected = 2;
        assert_eq!(agent.count_timesteps_till_death(None), expected);

        agent.acquire(fish_unit, 2);

        // With 6 units of berries & 2 units of fish the timesteps till death is two.
        let expected = 2;
        assert_eq!(agent.count_timesteps_till_death(None), expected);

        // With 6 units of berries & 2 units of fish the timesteps
        // till death *with one additional unit of fish* is three.
        let expected = 3;
        assert_eq!(
            agent.count_timesteps_till_death(Some(&Good::Fish)),
            expected
        );
    }
}
