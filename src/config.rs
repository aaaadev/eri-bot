use super::constants::CONFIG_FILE;
use chrono::NaiveTime;
use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Channels {
    pub console: u64,
    pub private: u64,
    pub log: u64,
}

impl Default for Channels {
    fn default() -> Self {
        Self {
            console: 0,
            private: 0,
            log: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub discord_token: String,
    pub main_guild: u64,
    pub verified_role: u64,
    pub reset_time: NaiveTime,
    pub admins: Vec<u64>,
    pub channels: Channels,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            discord_token: String::from("YOUR_TOKEN_HERE"),
            main_guild: 0,
            verified_role: 0,
            reset_time: NaiveTime::from_str("06:00:00").unwrap(),
            admins: vec![],
            channels: Channels::default(),
        }
    }
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(CONFIG_FILE))
            .build()?;
        s.try_deserialize()
    }
}
