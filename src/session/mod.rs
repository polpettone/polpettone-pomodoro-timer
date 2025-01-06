use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use std::fs::File;
use std::io::Write;
use std::path::Path;

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
    pub fn start_session(&self, description: &str, duration_seconds: u64) {
        let start = SystemTime::now();
        let start_date: DateTime<Utc> = start.into();

        println!("Using {}", &self.pomodoro_session_dir);

        let session_dir = &self.pomodoro_session_dir;

        let session = Session {
            description: description.to_string(),
            duration: Duration::new(duration_seconds, 0),
            start: start_date,
        };

        serialize_session(session, session_dir.to_string(), start_date);
    }
}

fn serialize_session(session: Session, session_dir: String, start_date: DateTime<Utc>) {
    let filename = format!("{}-session.yaml", start_date.format("%Y%m%d%H%M%S"));
    let filepath = Path::new(&session_dir).join(filename);

    let serialized = serde_yaml::to_string(&session).expect("Failed to serialize session");
    let mut file = File::create(filepath).expect("Failed to create file");
    file.write_all(serialized.as_bytes())
        .expect("Failed to write to file");
}
