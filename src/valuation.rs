// use rand::{SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    Model, UInt,
    actions::Action,
    agent::Agent,
    goods::{Good, GoodsUnit, PartialGoodsUnit, Productivity},
    learning::reward::Reward,
    stock::Stock,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalAgent {
    id: u64,
    stock: Stock,
    is_alive: bool,
    action_history: Vec<Action>,
    stock_history: Vec<Stock>,
    reward_history: Vec<Reward>,
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
            reward_history: vec![],
            daily_nutrition,
        }
    }

    /// Returns the marginal benefit to the agent of the product (output) of the specified action,
    /// given the existing stock.
    ///
    /// We define the marginal benefit of an action to produce a consumer good $g$, given existing
    /// stock $S$, as the (discounted) sum of the marginal values of the additional units.
    fn marginal_benefit_of_action(&self, action: &Action) -> f32 {
        // IMP TODO: Must take into account the fact that action to produce delayed-productivity
        // capital goods is only beneficial if the agent's stock already contains sufficient units
        // of consumer goods to complete the production of the capital good.

        // TODO: include naive discounting in the case of delayed-production higher-order goods.
        // i.e. disount over the interval of production (but nott the intervals between uses).

        let good = match action {
            Action::ProduceGood(good) => Some(good),
            Action::Leisure => None,
        };
        match good {
            Some(good) => match good.is_consumer() {
                true => self.marginal_benefit_of_producing_consumer_goods(good),
                false => self.marginal_benefit_of_producing_capital_goods(good),
            },
            None => 0.0,
        }
    }

    fn next_missing_input(&self, good: &Good) -> Option<Good> {
        let required_inputs = good.required_inputs();

        let productivity_per_unit_time = match self.productivity(good).per_unit_time() {
            Some(x) => x,
            None => return None,
        };
        let production_interval: u32 = ((1 as f32) / productivity_per_unit_time) as u32;

        for required_input in required_inputs.clone() {
            if required_input.is_material() {
                if self.stock().count_material_units(&required_input) < production_interval {
                    return Some(required_input);
                }
            }
        }
        required_inputs.into_iter().next()
    }

    /// Is this good producible with the existing stock?
    fn is_producible(&self, good: &Good) -> bool {
        if good.is_consumer() {
            return true;
        }
        let productivity_per_unit_time = match self.productivity(good).per_unit_time() {
            Some(x) => x,
            None => return false,
        };
        // Capital good is not producible unless there are enough saved consumer goods in the
        // stock to last through the interval of production.
        let production_interval: u32 = ((1 as f32) / productivity_per_unit_time) as u32;
        if self.count_timesteps_till_death(None) < production_interval {
            return false;
        }

        // Capital good is not producible unless there are enough saved units of material in the
        // stock to last through the interval of production.
        let required_inputs = good.required_inputs();
        for required_input in required_inputs {
            if required_input.is_material() {
                if !self.stock().contains(&required_input) {
                    return false;
                }
                // One unit of material is required for each timestep in the production interval.
                if self.stock().count_material_units(&required_input) < production_interval {
                    return false;
                }
            }
        }
        true
    }

    /// Returns the marginal benefit to the agent of producing a capital good,
    /// given the existing stock.
    fn marginal_benefit_of_producing_capital_goods(&self, good: &Good) -> f32 {
        if good.is_consumer() {
            panic!("Expected capital good.")
        }

        let productivity_per_unit_time = match self.productivity(good).per_unit_time() {
            Some(x) => x,
            None => return 0.0,
        };

        // // Marginal benefit is *zero* unless there is enough stock to finish production.
        // let production_interval: u32 = ((1 as f32) / productivity_per_unit_time) as u32;
        // if self.count_timesteps_till_death(None) < production_interval {
        //     return 0.0;
        // }
        if !self.is_producible(good) {
            return 0.0;
        }

        productivity_per_unit_time * self.marginal_unit_value_of_capital_good(good)
    }

    /// Returns the marginal value of a unit of a capital good, given the existing stock.
    fn marginal_unit_value_of_capital_good(&self, good: &Good) -> f32 {
        if good.is_consumer() {
            panic!("Expected capital good.")
        }
        // Note the marginal value is the maximum (not the sum!) over the values generated in
        // producing all lower-order goods.

        // Return the maximum value of the capital good at all orders (some capital
        // goods may be multiple-order).
        Good::iter()
            // .inspect(|x| println!("before filter: {:?}", x))
            .filter(|g| g.is_produced_using(good))
            // .inspect(|x| println!("after filter: {:?}", x))
            .map(|lower_order_good| {
                self.value_generated_by_higher_order_good(good, &lower_order_good)
            })
            // .inspect(|x| println!("after map: {:?}", x))
            .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            // .inspect(|x| println!("after max_by: {:?}", x))
            .unwrap()
    }

    /// Returns the value generated by a higher-order capital good in producing a particular
    /// lower-order good.
    fn value_generated_by_higher_order_good(
        &self,
        higher_order_good: &Good,
        lower_order_good: &Good,
    ) -> f32 {
        self.validate_higher_and_lower_order_goods(higher_order_good, lower_order_good);
        // TODO: include discounting (see comment in value_generated_by_first_order_capital_good).

        // println!("higher-order good: {:?}", higher_order_good);
        // println!("lower-order good: {:?}", lower_order_good);

        if lower_order_good.is_consumer() {
            return self
                .value_generated_by_first_order_capital_good(higher_order_good, lower_order_good);
        }

        let higher_order_goods_unit = GoodsUnit::new(higher_order_good);

        // Value of a higher order capital good (ignoring discounting) in producing a lower-order
        // capital good is the marginal value of the lower-order good multiplied by the lifetime
        // (number of uses) of the higher-order good. Except in the case of a material, where the
        // lifetime denotes its time before expiry (like a consumer good). In the case of materials
        // only a single use is possible.
        let mut factor = higher_order_goods_unit.remaining_lifetime as f32;
        if higher_order_good.is_material() {
            factor = 1.0;
        }

        // Note: the following results in a recursive call to this method.
        factor * self.marginal_unit_value_of_capital_good(lower_order_good)
    }

    /// Returns the value generated by a capital good in producing a consumer good.
    fn value_generated_by_first_order_capital_good(
        &self,
        capital_good: &Good,
        consumer_good: &Good,
    ) -> f32 {
        self.validate_consumer_and_first_order_capital_good(capital_good, consumer_good);

        // Currently there are no first-order capital goods that are materials. The only material
        // is timber, a second-order capital good. In general, however, first order capital goods
        // may be materials.
        if capital_good.is_material() {
            unreachable!() // Will become reachable if first-order materials are introduced.
        }
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

        // match consumer_good.is_produced_using(capital_good) {
        //     true => {
        dummy_agent.value_of_first_order_productivity(capital_good, consumer_good, factor)
        // }
        // false => {
        //     dummy_agent.value_of_first_order_improvement(capital_good, consumer_good, factor)
        // }
        // }
    }

    fn value_of_first_order_productivity(
        &self,
        capital_good: &Good,
        consumer_good: &Good,
        factor: f32, // Multiplicative factor to take into account existing units of the cap good.
    ) -> f32 {
        if !consumer_good.is_produced_using(capital_good) {
            panic!("Expected first-order producer.")
        }
        let capital_goods_unit = GoodsUnit::new(capital_good);
        let mut dummy_agent = self.clone();

        // Check that the agent does *not* already have the capital good.
        assert!(!dummy_agent.stock().contains(capital_good));

        // Get the productivity of the consumer good with and without the capital good.
        let productivity_sans = match dummy_agent.productivity(consumer_good) {
            Productivity::Immediate(quantity) => quantity,
            Productivity::None => 0,
            Productivity::Delayed(_) => unreachable!("Consumer goods have immediate productivity"),
        };
        dummy_agent.acquire(capital_goods_unit, 1);
        let productivity_with = match dummy_agent.productivity(consumer_good) {
            Productivity::Immediate(quantity) => quantity,
            Productivity::None => 0,
            Productivity::Delayed(_) => unreachable!("Consumer goods have immediate productivity"),
        };
        // Remove the capital good again.
        dummy_agent
            .stock_mut()
            .remove(&capital_goods_unit, 1)
            .expect("Sufficient stock guaranteed");

        if productivity_with == 0 {
            return 0.0;
        }

        // Check that the productivity with the capital good exceeds that without.
        if productivity_with == productivity_sans {
            println!("valuing capital good: {:?}", capital_good);
            println!("for use producing consumer good: {:?}", consumer_good);
            println!("productivity_with: {:?}", productivity_with);
            println!("productivity_sans: {:?}", productivity_sans);
            println!("stock: {:?}", dummy_agent.stock());
        }
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

    // fn value_of_first_order_improvement(
    //     &self,
    //     capital_good: &Good,
    //     consumer_good: &Good,
    //     factor: f32, // Multiplicative factor to take into account existing units of the cap good.
    // ) -> f32 {
    //     if !consumer_good.is_improved_using(capital_good) {
    //         panic!("Expected first-order improver.")
    //     }
    //     let capital_goods_unit = GoodsUnit::new(capital_good);
    //     let mut dummy_agent = self.clone();

    //     // TODO NEXT.
    //     // Get the additional of the consumer good with and without the capital good.
    //     // Using similar methodology to productivity case, but with additional longevity instead
    //     // of productivity.

    //     // Temporary workaround: hard-code a Smoker valuation:
    //     1.5
    // }

    // fn times_of_most_productive_first_order_use(&self, capital_good: &Good, consumer_good: &Good) ->  {
    //     self.validate_consumer_and_first_order_capital_good(capital_good, consumer_good);
    // }

    fn validate_higher_and_lower_order_goods(
        &self,
        higher_order_good: &Good,
        lower_order_good: &Good,
    ) {
        if higher_order_good.is_consumer() {
            panic!("Expected capital good.")
        }
        if !lower_order_good.is_produced_using(higher_order_good) {
            panic!("Invalid higher- and lower-order pair of goods.")
        }
    }

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
            Productivity::Immediate(quantity) => quantity,
            Productivity::None => return 0.0,
            Productivity::Delayed(_) => {
                panic!("All consumer goods have immediate productivity.")
            }
        };

        // // temp:
        // println!("productivity: {:?}", productivity);

        let mut sum: f32 = 0.0;
        let mut count = 0;
        let mut dummy_agent = self.clone();
        while count != productivity {
            // TODO: discounting.
            sum = sum + dummy_agent.marginal_unit_value_of_consumer_good(good);
            // println!("sum: {:?}", sum);
            dummy_agent.acquire(GoodsUnit::new(good), 1);
            count = count + 1;
        }
        sum
    }

    // PROBLEM: Discrete jumps when daily nutritional levels are reached violate the law
    // of diminishing returns. To fix this we need to work in fractional days throughout.

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
        // 1. Count additional days of sustenance from 1 additional unit of the good.
        let additional_sustenance = self.additional_sustenance(good);

        // TODO: consider improving this (we don't want discrete jumps or zero valuation for
        // single units) as follows:
        // - if the additional sustenance is zero:
        //  - keep going (adding the good and then calling this method again) until it isn't zero
        // - then divide by the number of units added in total (including the marginal one)

        // If the additional sustenance is zero, the value of the marginal unit is also zero.
        // (In practice, goods will typically be considered in quantities greater than 1 unit.)
        if additional_sustenance == 0 {
            // Sketch of the improvement suggested above. Note that this code is *not* ready, as
            // it does not take into account the fact that the most efficient route (possibly
            // involving different goods) should be considered when increasing the dummy agent's
            // current stock.
            //
            // // If the additional sustenance is zero, recursively call this method until it isn't
            // // zero, then divide by the number of units added in total (including the marginal one).
            // let mut dummy_agent = self.clone();
            // dummy_agent.acquire(GoodsUnit::new(good), 1);
            // let mut count: u32 = 1;
            // let mut cumulative_additional_sustenance = 0;
            // while cumulative_additional_sustenance == 0 {
            //     count += 1; // Increment the count first, for the marginal good.
            //     cumulative_additional_sustenance += dummy_agent.additional_sustenance(good);
            //     dummy_agent.acquire(GoodsUnit::new(good), 1);
            // }
            // return (cumulative_additional_sustenance as f32) / (count as f32);

            return 0.0;
        }

        // 2. Initialise the minimum time to produce equivalent sustenance, to the value for
        // Berries (as a default which is known to always have a non-zero productivity).
        let productivity_per_unit_time = self.productivity(&Good::Berries).per_unit_time();

        // Time to produce 1 unit of the good is (1 / amount produced in one day's production).
        let mut min_equiv = (1 as f32) / productivity_per_unit_time.unwrap();

        // 3. For every consumer good, compute the time taken to produce the same number of
        // days of sustenance.
        for alt_good in Good::iter() {
            // Ignore Berries as we already initialised min_equiv for it.
            if alt_good == Good::Berries {
                continue;
            }
            if let Some(t) =
                self.time_to_equiv_sustenance(alt_good, additional_sustenance, min_equiv)
            {
                // println!("good: {:?}", good);
                // println!("alt good: {:?}", alt_good);
                // println!("t: {:?}", t);
                if t < min_equiv {
                    min_equiv = t;
                }
            }
        }
        // 3. Return the minimum equivalent.
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
                match self.productivity(&alt_good).per_unit_time() {
                    Some(_) => {
                        // Acquire one unit at a time until the additional survival time
                        // reaches the target time and count the quantity acquired.
                        // Then call time_to_produce_units on that quantity.
                        let mut count_units = 0;
                        loop {
                            // Acquire one extra unit of the alternative good.
                            dummy_agent.acquire(GoodsUnit::new(&alt_good), 1);
                            count_units += 1;

                            // Compute the new survival time with the extra units of the alternative good.
                            let new_survival_time = dummy_agent.count_timesteps_till_death(None);

                            // If the additional sustenance exceeds the target sustenance, return
                            // the equivalent day count necessary to produce the additional goods.
                            let additional_survival = new_survival_time - survival_time;
                            let t = self.time_to_produce_units(&alt_good, count_units);
                            if additional_survival >= target_sustenance {
                                return t;
                            }
                            // If the max limit is already exceeded, return None.
                            if !t.is_none() && t.unwrap() > max {
                                return None;
                            }
                        }
                    }
                    None => None, // Return None if marginal productivity of the alt_good is None.
                }
            }
            false => None,
        }
    }

    /// Returns the (decimal) number of time units required to produce a given quantity of a given
    /// good (given the existing stock), taking into account productivity.
    fn time_to_produce_units(&self, good: &Good, quantity: UInt) -> Option<f32> {
        if quantity == 0 {
            return Some(0.0);
        }
        let prior_qty = self.stock().count_units(good);
        let mut count = 0;
        let mut dummy_agent = self.clone();
        loop {
            let productivity = dummy_agent.productivity(good);
            if productivity == Productivity::None {
                return None;
            }
            dummy_agent.act(Action::ProduceGood(*good));
            let produced_qty = dummy_agent.stock().count_units(good) - prior_qty;
            count += 1;
            if produced_qty >= quantity {
                // Amount produced on the last day is 1 / productivity per unit time.
                let final_day_production = productivity.per_unit_time().unwrap();
                let excess = produced_qty - quantity;
                let final_day_required_production = final_day_production - excess as f32;
                // Time taken to produce the desired quantity is the number of full days of
                // productivity, plus part of the final day.
                let part_day = final_day_required_production / final_day_production;
                return Some((count - 1) as f32 + part_day);
            }
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
            if !dummy_agent.consume(self.daily_nutrition) {
                break; // Break out as soon as death happens.
            }
            dummy_agent.set_stock(dummy_agent.stock().step_forward(Action::Leisure));
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
        let mut max_benefit = 0.0;
        let mut best_good = Good::Berries; // arbitrary initial good.
        let mut best_downstream_good: Option<Good> = None;

        // IF A PARTIALLY-COMPLETED GOOD IS IN THE STOCK, ALWAYS COMPLETE IT.
        for partial in self.stock().partial_stock.iter() {
            return Action::ProduceGood(partial.good);
        }
        // if self.stock().partial_stock.len() > 0 {
        //     let partial = &self.stock().partial_stock.pop().unwrap();
        // }

        // TODO NEXT: UPDATE BENEFIT/VALUE BIT SO THAT CAPITAL GOODS ARE ONLY STARTED IF ENOUGH
        // FOOD *AND* ENOUGH MATERIALS ARE AVAILABLE.

        for good in Good::iter() {
            let benefit = self.marginal_benefit_of_action(&Action::ProduceGood(good));

            // println!(
            //     "Marginal benefit of action to produce {:?}: {:?}",
            //     good, benefit
            // );

            if benefit > max_benefit {
                best_good = good;
                max_benefit = benefit;
            }
            // If we have a capital good we should use it!
            if !good.is_consumer() {
                if self.stock().contains(&good) {
                    let produces = good.produces();
                    let mut max_productivity = 0;
                    for downsteam_good in produces {
                        if downsteam_good.is_consumer() {
                            let productivity =
                                self.productivity(&downsteam_good).per_unit_time().unwrap() as u32;
                            // NOTE: we assume all consumer goods are measured in equivalent units here.
                            // i.e. 1 unit of berries provides equivalent sustenance to 1 unit of fish.
                            if productivity > max_productivity {
                                max_productivity = productivity;
                                best_downstream_good = Some(downsteam_good);
                            }
                        } else {
                            // TODO. Need to handle downstream capital goods.
                            // TEMP SIMPLE ATTEMPT (WILL FAIL FOR TIMBER => Smoker vs Boat)
                            best_downstream_good = Some(downsteam_good);
                        }
                    }
                }
            }
        }
        let mut action = Action::ProduceGood(best_good);

        // An available capital good trumps simpler production.
        if let Some(downstream_good) = best_downstream_good {
            // If required inputs for the downstream good are not alredy in the stock,
            // produce them first.
            if !self.is_producible(&downstream_good) {
                if let Some(missing_input) = self.next_missing_input(&downstream_good) {
                    return Action::ProduceGood(missing_input);
                }
            }
            action = Action::ProduceGood(downstream_good);
            // println!("USE OF CAPITAL GOOD TO PRODUCE: {:?}", downstream_good);
        }

        // Choose leisure sometimes.
        // IMP TODO: make this more rational - at least fix magic number here
        // as this makes Smoker or Boat construction impossible.
        if self.count_timesteps_till_death(None) > 12 {
            action = Action::Leisure;
        }
        // MOVED action_history update to choose_action_with_model:
        // self.action_history.push(action);
        action
    }

    fn choose_action_with_model(&mut self, model: &Model) -> Action {
        // // TMP DEBUGGING:
        // if self.stock().contains(&Good::Axe) {
        //     println!("Have Axe");
        // }
        // let timber_in_stock = self.stock().material_units(&Good::Timber);
        // if timber_in_stock > 3 {
        //     println!(
        //         "Timber in stock: {:?}",
        //         self.stock().material_units(&Good::Timber)
        //     );
        //     for good in Good::iter() {
        //         let benefit = self.marginal_benefit_of_action(&Action::ProduceGood(good));
        //         println!(
        //             "benefit of action to produce good {:?}: {:?}",
        //             good, benefit
        //         );
        //     }
        // }
        if self.stock().contains(&Good::Boat) {
            println!("Have Boat!!");
        }
        let action = self.choose_action(); // Rational agent ignores the RL model.
        // println!("action: {:?}", action);

        self.action_history.push(action);
        action
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goods::{Good, GoodsUnit};

    #[test]
    fn test_is_producible() {
        // TEMP: this belongs in stock.rs
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        assert!(agent.is_producible(&Good::Berries));
        assert!(agent.is_producible(&Good::Fish));
        assert!(!agent.is_producible(&Good::Basket));

        agent.acquire(GoodsUnit::new(&Good::Berries), 4);
        assert!(agent.is_producible(&Good::Basket));

        assert!(!agent.is_producible(&Good::Smoker));
        agent.acquire(GoodsUnit::new(&Good::Timber), 2);
        assert!(!agent.is_producible(&Good::Smoker));

        agent.acquire(GoodsUnit::new(&Good::Berries), 10);
        assert!(!agent.is_producible(&Good::Smoker));

        agent.acquire(GoodsUnit::new(&Good::Timber), 4);
        assert!(agent.is_producible(&Good::Smoker));
    }

    #[test]
    fn test_valuations_and_benefits() {
        // TODO NEXT:
        // call marginal_benefit_of_action for each action, with an empty stock,
        // and try to work out why only baskets and axes are produced.e
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        agent.acquire(GoodsUnit::new(&Good::Berries), 10);
        agent.acquire(GoodsUnit::new(&Good::Fish), 10);
        agent.acquire(GoodsUnit::new(&Good::Axe), 1);
        // agent.acquire(GoodsUnit::new(&Good::Spear), 1);
        for good in Good::iter() {
            let benefit = agent.marginal_benefit_of_action(&Action::ProduceGood(good));
            println!(
                "benefit of action to produce good {:?}: {:?}",
                good, benefit
            );
        }
        println!();
        agent.acquire(GoodsUnit::new(&Good::Spear), 1);
        // agent.acquire(GoodsUnit::new(&Good::Spear), 1);
        for good in Good::iter() {
            let benefit = agent.marginal_benefit_of_action(&Action::ProduceGood(good));
            println!(
                "benefit of action to produce good {:?}: {:?}",
                good, benefit
            );
        }
        // TODO: consider favouring capital goods that are downstream of already acquired capital goods.
    }

    #[test]
    fn test_choose_action() {
        // TODO NEXT:
        // call marginal_benefit_of_action for each action, with an empty stock,
        // and try to work out why only baskets and axes are produced.
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        let action = agent.choose_action();
        assert_eq!(action, Action::ProduceGood(Good::Berries));

        agent.acquire(GoodsUnit::new(&Good::Basket), 1);
        let action = agent.choose_action();
        assert_eq!(action, Action::ProduceGood(Good::Berries));

        agent.acquire(GoodsUnit::new(&Good::Spear), 1);
        let action = agent.choose_action();
        assert_eq!(action, Action::ProduceGood(Good::Fish));

        agent.acquire(GoodsUnit::new(&Good::Boat), 1);
        let action = agent.choose_action();
        assert_eq!(action, Action::ProduceGood(Good::Fish));

        // // TODO. Re-include these tests.
        // // New agent.
        // let mut agent = RationalAgent::new(1, daily_nutrition);
        // agent.acquire(GoodsUnit::new(&Good::Timber), 3);
        // agent.acquire(GoodsUnit::new(&Good::Fish), 10);

        // let action = agent.choose_action();
        // println!("{:?}", action); // EXPECT: produce smoker
        // assert_eq!(action, Action::ProduceGood(Good::Smoker));

        // agent.acquire(GoodsUnit::new(&Good::Fish), 60);
        // agent.acquire(GoodsUnit::new(&Good::Smoker), 1);
        // agent.step_forward(Some(Action::Leisure));

        // let action = agent.choose_action();
        // println!("{:?}", action); // EXPECT: produce boat
        // assert_eq!(action, Action::ProduceGood(Good::Boat));
    }

    #[test]
    fn test_marginal_benefit_of_action() {
        let daily_nutrition = 3;
        let agent = RationalAgent::new(1, daily_nutrition);

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

        // Test when capital goods are available:
        // Start again with empty stock.
        let mut agent = RationalAgent::new(1, daily_nutrition);

        agent.acquire(GoodsUnit::new(&Good::Spear), 1);
        let action = Action::ProduceGood(Good::Fish);

        // Given an initial stock consisting of only a spear, the marginal value of the first two
        // units of fish is zero. The third unit has a marginal value of 1/10. The fourth & fifth
        // units have a marginal value of zero. The sixth unit has a marginal value of 1/10.
        // Further units all have a marginal value of zero because they will expire before they can
        // be consumed. So the marginal benefit of the action to produce (ten) fish is 2/10.
        assert_eq!(agent.marginal_benefit_of_action(&action), 0.2);
    }

    #[test]
    fn test_productivity() {
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        assert!(agent.productivity(&Good::Fish) == Productivity::Immediate(2));
        agent.acquire(GoodsUnit::new(&Good::Boat), 1);
        assert!(agent.productivity(&Good::Fish) == Productivity::Immediate(20));
    }

    #[test]
    fn test_value_generated_by_higher_order_good() {
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        // Test when the lower-order good is a consumer good.
        let higher_order_good = Good::Basket;
        let lower_order_good = Good::Berries;

        let result =
            agent.value_generated_by_higher_order_good(&higher_order_good, &lower_order_good);

        // Result should be the same as the value_generated_by_first_order_capital_good (see
        // other unit test case).
        assert_eq!(result, 2.5);

        // Test when the lower-order good is a capital good and the higher-order good is a material.
        let higher_order_good = Good::Timber;
        let lower_order_good = Good::Boat;

        let result =
            agent.value_generated_by_higher_order_good(&higher_order_good, &lower_order_good);
        assert!(result == 10.0);

        // Test when the lower-order good is a capital good (and a material).
        let higher_order_good = Good::Axe;
        let lower_order_good = Good::Timber;

        let result =
            agent.value_generated_by_higher_order_good(&higher_order_good, &lower_order_good);

        // TODO. This value depends on the temporary workaround hard-coded value for the smoker.
        assert!(result == 50.0);
    }

    #[test]
    fn test_value_generated_by_first_order_capital_good() {
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        let capital_good = Good::Basket;
        let consumer_good = Good::Berries;

        let result =
            agent.value_generated_by_first_order_capital_good(&capital_good, &consumer_good);

        // Marginal value of a basket, given otherwise empty stock, is the lifetime of the basket
        // (10 uses) multiplied by the marginal value of each additional unit of berries afforded
        // by the basket during those uses (which is 0.25). So the value is 2.5.
        assert_eq!(result, 2.5);
    }

    #[test]
    fn test_marginal_unit_value_of_consumer_good() {
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

        // Start again with an empty stock.
        let mut agent = RationalAgent::new(1, daily_nutrition);

        agent.acquire(GoodsUnit::new(&Good::Spear), 1);
        agent.acquire(GoodsUnit::new(&Good::Berries), 2);
        assert_eq!(agent.productivity(&Good::Fish), Productivity::Immediate(10));
        // With a spear, the marginal unit value of both fish and berries falls to 0.1
        // because 10 fish can be producted in 1 day.
        assert_eq!(agent.marginal_unit_value_of_consumer_good(&Good::Fish), 0.1);
        assert_eq!(
            agent.marginal_unit_value_of_consumer_good(&Good::Berries),
            0.1
        );

        // Start again with an empty stock.
        let mut agent = RationalAgent::new(1, daily_nutrition);

        // With four fish, the marginal unit value of one additional unit of fish is zero,
        // because it would not increase the agent's survival.
        agent.acquire(GoodsUnit::new(&Good::Fish), 4);
        assert_eq!(agent.marginal_unit_value_of_consumer_good(&Good::Fish), 0.0);

        agent.acquire(GoodsUnit::new(&Good::Fish), 1);
        // With five fish, the marginal unit value of one additional unit of fish is equal to
        // the minimum possible time to produce an equivalent sustenance, which is by producing
        // berries at a rate of 4 per day.
        assert_eq!(agent.stock().count_units(&Good::Fish), 5);

        assert_eq!(
            agent.marginal_unit_value_of_consumer_good(&Good::Fish),
            0.25
        );

        // Start again with an empty stock except for one spear.
        let mut agent = RationalAgent::new(1, daily_nutrition);
        agent.acquire(GoodsUnit::new(&Good::Spear), 1);

        // With four fish, the marginal unit value of one additional unit of fish is zero,
        // because it would not increase the agent's survival.
        agent.acquire(GoodsUnit::new(&Good::Fish), 4);
        assert_eq!(agent.marginal_unit_value_of_consumer_good(&Good::Fish), 0.0);

        agent.acquire(GoodsUnit::new(&Good::Fish), 1);
        // With five fish, the marginal unit value of one additional unit of fish is equal to
        // the minimum possible time to produce an equivalent sustenance, which is by producing
        // fish at a rate of 10 per day (using the spear).
        assert_eq!(agent.stock().count_units(&Good::Fish), 5);
        assert_eq!(agent.marginal_unit_value_of_consumer_good(&Good::Fish), 0.1);
    }

    #[test]
    fn test_time_to_equiv_sustenance() {
        let daily_nutrition = 3;
        let agent = RationalAgent::new(1, daily_nutrition);

        // When the agent has no capital goods, producing fish will take two days to produce
        // (just over) one day's sustenance. Four units will have been produced, which is 4/3 of
        // the target sustenance. So it took 3/2 days to produce one day's sustenance.
        let result = agent.time_to_equiv_sustenance(Good::Fish, 1, 2.0);
        assert_eq!(result, Some(1.5));

        // Producing berries extends their lifetime by 1 day and the productivity for berries is
        // four. So the time taken to produce 1 day's sustenance is 3/4.
        let result = agent.time_to_equiv_sustenance(Good::Berries, 1, 1.0);
        assert_eq!(result, Some(0.75));

        // agent.acquire(GoodsUnit::new(&Good::Berries), 2);
        // assert_eq!(result, Some(0.25));

        let mut agent = RationalAgent::new(1, daily_nutrition);

        // With a spear, 10 units of fish can be produced per day, so it takes 3/10 of a day to
        // produce one day's sustenance.
        agent.acquire(GoodsUnit::new(&Good::Spear), 1);
        let result = agent.time_to_equiv_sustenance(Good::Fish, 1, 0.5);
        assert_eq!(result, Some(0.3));
        // The spear does not change the agent's productivity for berries.
        let result = agent.time_to_equiv_sustenance(Good::Berries, 1, 0.5);
        assert_eq!(result, Some(0.75));
    }

    #[test]
    fn test_time_to_produce_units() {
        let daily_nutrition = 3;
        let mut agent = RationalAgent::new(1, daily_nutrition);

        // With no capital goods, the time to produce 3 units of berries is 3/4 days.
        assert_eq!(agent.time_to_produce_units(&Good::Berries, 3), Some(0.75));

        // With no capital goods, the time to produce 3 units of fish is 3/2 days.
        assert_eq!(agent.time_to_produce_units(&Good::Fish, 3), Some(1.5));

        // With no capital goods, the time to produce 13 units of fish is 13/2 days.
        assert_eq!(agent.time_to_produce_units(&Good::Fish, 13), Some(6.5));

        // With no capital goods, units of smoked fish cannot be produced.
        assert!(agent.time_to_produce_units(&Good::SmokedFish, 13).is_none());

        // With a new spear, the time to produce 13 units of fish is 1 + 3/10 days.
        agent.acquire(GoodsUnit::new(&Good::Spear), 1);
        assert_eq!(agent.time_to_produce_units(&Good::Fish, 13), Some(1.3));

        let mut agent = RationalAgent::new(1, daily_nutrition);
        // With an old spear, the time to produce 13 units of fish is 1 + 3/2 days.
        agent.acquire(
            GoodsUnit {
                good: Good::Spear,
                remaining_lifetime: 1,
            },
            1,
        );
        assert_eq!(agent.time_to_produce_units(&Good::Fish, 13), Some(2.5));

        // Test with production of capital goods.
        let mut agent = RationalAgent::new(1, daily_nutrition);
        assert_eq!(agent.time_to_produce_units(&Good::Spear, 1), Some(1.0));

        assert_eq!(agent.time_to_produce_units(&Good::Timber, 5), None);
        assert_eq!(agent.time_to_produce_units(&Good::Smoker, 1), None);
        assert_eq!(agent.time_to_produce_units(&Good::Boat, 1), None);

        assert_eq!(agent.time_to_produce_units(&Good::Axe, 1), Some(2.0));

        agent.acquire(GoodsUnit::new(&Good::Axe), 1);
        // With an axe, it takes 5/2 days to produce 5 units of timber.
        assert_eq!(agent.time_to_produce_units(&Good::Timber, 5), Some(2.5));
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

        // Start again.
        let mut agent = RationalAgent::new(1, daily_nutrition);

        // With 4 fish in stock, one additional unit of fish does not increase the survival time.
        agent.acquire(fish_unit, 4);
        assert_eq!(agent.count_timesteps_till_death(Some(&Good::Fish)), 1);

        // With 5 fish in stock, one additional unit of fish increases the survival time by one day.
        agent.acquire(fish_unit, 1);
        assert_eq!(agent.stock().count_units(&Good::Fish), 5);
        assert_eq!(agent.count_timesteps_till_death(Some(&Good::Fish)), 2);
    }
}
