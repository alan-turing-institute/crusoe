use serde::{Deserialize, Serialize};

use crate::UInt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub max_time: UInt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization_toml() {
        let config = Config { max_time: 100 };
        let serialized = toml::to_string(&config).unwrap();

        assert_eq!(serialized, "max_time = 100\n");

        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, config);
    }

    #[test]
    fn test_read_from_file() {
        std::fs::read_to_string("./crusoe.toml").expect("Failed to read the file");
    }
}
