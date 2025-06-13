use serde::{Deserialize, Serialize};

use crate::UInt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub max_time: UInt,
    pub daily_nutrition: UInt, // Number of units (of any consumer good) required per day.
    pub agent: AgentConfig,
    pub rl: RLConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    pub inv_level_low: UInt,
    pub inv_level_med: UInt,
    pub inv_level_high: UInt,
    // pub remaining_level_high: UInt,
}

impl Default for AgentConfig {
    fn default() -> Self {
        AgentConfig {
            inv_level_low: 0,
            // inv_level_med: 3,
            inv_level_med: 6,
            inv_level_high: 80000,
            // inv_level_low: 20000,
            // inv_level_med: 40000,
            // inv_level_high: 80000,
            // remaining_level_high: 5,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_time: 100,
            daily_nutrition: 3,
            rl: RLConfig::default(),
            agent: AgentConfig::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct RLConfig {
    pub init_q_value: f32,
    pub sarsa_n: u8,
    pub gamma: f32,
    pub alpha: f32,
    pub epsilon: f32,
    pub multi_policy: bool,
    // pub save_model: bool,
    // pub load_model: bool,
    // pub model_checkpoint_file: Option<String>,
}

impl Default for RLConfig {
    fn default() -> Self {
        RLConfig {
            init_q_value: 0.0,
            // sarsa_n: 50,
            sarsa_n: 2,
            gamma: 0.9,
            alpha: 0.1,
            // epsilon: 0.999,
            // epsilon: 0.1,
            // epsilon: 0.5,
            epsilon: 0.1,
            multi_policy: false,
            // save_model: false,
            // load_model: false,
            // model_checkpoint_file: None,
        }
    }
}

pub fn core_config() -> Config {
    Config::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization_toml() {
        let config = Config {
            max_time: 100,
            daily_nutrition: 3,
            rl: RLConfig::default(),
            agent: AgentConfig::default(),
        };
        let serialized = toml::to_string(&config).unwrap();

        // assert_eq!(serialized, "max_time = 100\ndaily_nutrition = 3\n");

        let deserialized: Config = toml::from_str(&serialized).unwrap();
        // assert_eq!(deserialized, config);
    }

    #[test]
    fn test_read_from_file() {
        std::fs::read_to_string("./crusoe.toml").expect("Failed to read the file");
    }
}
