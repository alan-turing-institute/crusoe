use crusoe::{config::Config, simulation::Simulation};

fn main() {
    let mut sim = Simulation::new(
        Config {
            max_time: 10,
            daily_nutrition: 3,
        },
        false,
    );
    sim.run();

    // Write sim to disk
    let s = serde_json::to_string(&sim).unwrap();
    println!("{s}");
}
