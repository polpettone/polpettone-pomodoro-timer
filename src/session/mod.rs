use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    description: String,
    duration: Duration,
    #[serde(with = "ts_seconds")]
    start: DateTime<Utc>,
}

pub struct SessionService {
    pub pomodoro_session_dir: String,
}

impl SessionService {
    pub fn start_session(&self, description: &str, duration_seconds: u64) -> Session {
        let start = SystemTime::now();
        let datetime: DateTime<Utc> = start.into();

        println!("Using {}", &self.pomodoro_session_dir);

        Session {
            description: description.to_string(),
            duration: Duration::new(duration_seconds, 0),
            start: datetime,
        }
    }
}
