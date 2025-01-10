use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PomodoroConfig {
    pub pomodoro_session_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub pomodoro_config: PomodoroConfig,
}
