use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PersistenceMode {
    File,
    Postgres,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PomodoroConfig {
    pub pomodoro_session_dir: String,
    #[serde(default = "default_persistence_mode")]
    pub persistence_mode: PersistenceMode,
    pub database_url: Option<String>,
}

fn default_persistence_mode() -> PersistenceMode {
    PersistenceMode::File
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub pomodoro_config: PomodoroConfig,
}
