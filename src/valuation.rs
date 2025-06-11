use std::ops::Deref;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    UInt,
    actions::Action,
    agent::Agent,
    goods::{Good, GoodsUnit, PartialGoodsUnit},
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

    /// Returns the marginal valuation of the goods unit, given the stock.
    ///
    /// We define the marginal unit value of a consumer good $g$, given existing stock $S$, as
    /// the min amount of time required to produce equivalent additional sustenance to 1 additional
    /// unit of g (given stock S).
    fn marginal_unit_value(self, goods_unit: &GoodsUnit) -> f32 {
        // 1. Count additional days of sustenance from 1 additional unit of g
        let additional_sustenance = self.additional_sustenance(goods_unit);

        // 2. Initialise the minimum time to produce equivalent sustenance, to the value for
        // this good. Return zero value if the agent's productivity for this good is None.
        let productivity_per_unit_time = self.productivity(goods_unit.good).per_unit_time();
        if productivity_per_unit_time.is_none() {
            return 0.0;
        }
        // Time to produce 1 unit of the good is (1 / amount produced in one day's production).
        let mut min_equiv = (1 as f32) / productivity_per_unit_time.unwrap();

        // 3. For every conumer good, compute the time taken to produce the same number of
        // days of sustenance.
        for alt_good in Good::iter() {
            // Ignore the good itself as we alreday initialised min_equiv for it.
            if alt_good == goods_unit.good {
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

    // Returns None if alt_good is a capital good of if the max limit is exceeded.
    fn time_to_equiv_sustenance(
        &self,
        alt_good: Good,
        target_sustenance: u32,
        max: f32,
    ) -> Option<f32> {
        let mut dummy_agent = self.clone();
        let survival_time = self.count_timesteps_till_death(None);
        match alt_good.is_consumer() {
            true => {
                if let Some(productivity) = self.productivity(alt_good).per_unit_time() {
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
                            return Some(
                                (count_days as f32)
                                    / (additional_survival as f32 / target_sustenance as f32),
                            );
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

    /// Counts the number of additional days of survival provided by the additional goods unit.
    fn additional_sustenance(&self, goods_unit: &GoodsUnit) -> u32 {
        let survival_days = self.count_timesteps_till_death(None);
        // println!("Survival days: {:?}", survival_days);
        let additional_survival_days = &self.count_timesteps_till_death(Some(goods_unit));
        // println!("Additional survival days: {:?}", additional_survival_days);
        additional_survival_days - survival_days
        // let extra_count = additional_survival_days - survival_days;
        // if extra_count == 0 {
        //     return 0.0;
        // }
        // (extra_count as f32) / (self.daily_nutrition as f32)
    }

    /// Counts the number of timesteps that the agent can survive with the current
    /// stock, plus one unit of an optional additional good, assuming only consumption
    /// (i.e. no production/acquision of new goods).
    fn count_timesteps_till_death(&self, additional_goods_unit: Option<&GoodsUnit>) -> UInt {
        let mut dummy_agent = self.clone();
        if let Some(goods_unit) = additional_goods_unit {
            dummy_agent.acquire(goods_unit.clone(), 1);
        }
        let mut count = 0;
        // while dummy_agent.is_alive() {
        loop {
            let action = Action::Leisure;
            // let is_alive = dummy_agent.consume(self.daily_nutrition);
            if !dummy_agent.consume(self.daily_nutrition) {
                break; // Break out as soon as death happens.
            }
            // dummy_agent.set_liveness(is_alive);
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
    fn test_marginal_unit_value() {
        // Test marginal unit value of berries, given zero stock.
        let daily_nutrition = 3;
        let agent = RationalAgent::new(1, daily_nutrition);

        let goods_unit = GoodsUnit::new(&Good::Berries);

        // Minimum time required to produce sustanance equivalent to 1 unit of berries is 1/4 days
        // (by producing berries):
        let expected: f32 = 0.25;
        assert_eq!(agent.marginal_unit_value(&goods_unit), expected);

        // Start again with an empty stock.
        let agent = RationalAgent::new(1, daily_nutrition);

        let goods_unit = GoodsUnit::new(&Good::Fish);

        // Minimum time required to produce sustanance equivalent to 1 unit of fish is 1/4 days
        // (by producing berries):
        let expected: f32 = 0.25;

        assert_eq!(agent.marginal_unit_value(&goods_unit), expected);
    }

    #[test]
    fn test_additional_sustenance() {
        // Test additional sustenance from berries.
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);
        let berries_unit = GoodsUnit::new(&Good::Berries);

        // No additional sustenance from 1 unit of berries (when starting from none).
        let expected = 0;
        assert_eq!(agent.additional_sustenance(&berries_unit), expected);

        // No additional sustenance from 1 unit of berries (when starting from 1 unit).
        agent.acquire(berries_unit, 1);
        let expected = 0;
        assert_eq!(agent.additional_sustenance(&berries_unit), expected);

        // One additional day's sustenance from 1 unit of berries (when starting from 2 units).
        agent.acquire(berries_unit, 1);

        let expected = 1;
        assert_eq!(agent.additional_sustenance(&berries_unit), expected);

        // One additional day's sustenance from 1 unit of fish (when starting from 2 units).
        let fish_unit = GoodsUnit::new(&Good::Fish);
        assert_eq!(agent.additional_sustenance(&fish_unit), expected);
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
            agent.count_timesteps_till_death(Some(&berries_unit)),
            expected
        );
        // With two units of berries the timesteps till death
        // *with one additional unit of fish* is one.
        let expected = 1;
        assert_eq!(agent.count_timesteps_till_death(Some(&fish_unit)), expected);

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
            agent.count_timesteps_till_death(Some(&berries_unit)),
            expected
        );

        agent.acquire(berries_unit, 1);

        // With six units of berries the timesteps till death is two.
        let expected = 2;
        assert_eq!(agent.count_timesteps_till_death(None), expected);
    }
}
