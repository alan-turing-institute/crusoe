use crusoe::simulation::Simulation;

fn main() {
    let mut sim = Simulation::new();

    while sim.time < sim.config.max_time {
        sim.step_forward();
        println!("Time: {}, Agents: {}", sim.time, sim.agents.len());
        println!("Actions:  {0:#?}", sim.agents[0]);
        sim.time += 1;
    }
}
