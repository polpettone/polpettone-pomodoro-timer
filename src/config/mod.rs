use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Line,
    Json,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: LogFormat,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> LogFormat {
    LogFormat::Line
}

#[derive(Debug, Deserialize, Clone)]
pub struct PomodoroConfig {
    pub pomodoro_session_dir: String,

    pub database_url: Option<String>,
    #[serde(default = "default_log_config")]
    pub log_config: LogConfig,
}

fn default_log_config() -> LogConfig {
    LogConfig {
        level: default_log_level(),
        format: default_log_format(),
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub pomodoro_config: PomodoroConfig,
}
