use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use chrono::{DateTime, Utc};
use crate::date_time::{deserialize_human_readable, serialize_human_readable};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SessionState {
    Running,
    Done,
    Deleted,
    Canceled,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SessionRatings {
    pub mental_energy: u8,
    pub physical_energy: u8,
    pub cognitive_load: u8,
    #[serde(default)]
    pub motivation: u8,
}

fn default_state() -> SessionState {
    SessionState::Done
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub description: String,
    pub duration: Duration,
    #[serde(
        serialize_with = "serialize_human_readable",
        deserialize_with = "deserialize_human_readable"
    )]
    pub start: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "default_state")]
    pub state: SessionState,
    #[serde(default)]
    pub ratings: Option<SessionRatings>,
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {} minutes - {}",
            self.start.format("%Y-%m-%d"),
            self.duration.as_secs() / 60,
            self.description
        )
    }
}

impl Session {
    pub fn elapsed_duration(&self) -> Duration {
        let now = Utc::now();
        let duration_since_start = now.signed_duration_since(self.start);
        Duration::new(
            duration_since_start.num_seconds() as u64,
            duration_since_start.num_nanoseconds().unwrap_or(0) as u32,
        )
    }


    pub fn remaining_duration(&self) -> Duration {
        let now = Utc::now();
        let end = self.start + self.duration;
        if end > now {
            (end - now).to_std().unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(0)
        }
    }
}
