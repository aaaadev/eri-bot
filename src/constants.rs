use fast_log::consts::LogSize;
use std::time::Duration;

pub const CONFIG_FILE: &'static str = "App.toml";
pub const CONFIG_EXAMPLE_FILE: &'static str = "App.example.toml";
pub const LOG_FILE_PATH: &'static str = "logs/";
pub const LOG_FILE_SIZE: LogSize = LogSize::MB(1);
pub const TIMER_INTERVAL: Duration = Duration::from_secs(5);
