// // TODO
// #[derive(Serialize, Deserialize, Debug)]
// pub struct Simulation {
//     pub time: usize,
//     pub agents: Vec<RefCell<Agent>>,
// }

// impl Simulation {
//     fn init(&mut self);

//     // Trade happens in here.
//     fn after_step(&mut self) {
//         // Shuffle the vector of agents.
//         for &mut agent in self.agents().shuffle() {
//             // Identify the best bilateral trade for this agent.

//             // Execute that trade by updating the stocks of the two agents involved.
//         }
//     }

//     fn update(&mut self) {
//         // Step forward each agent.
//         for &mut agent in self.agents() {
//             let action = agent.select_action()
//             agent.step(action)
//         }
//         self.after_step()
//     }
// }
