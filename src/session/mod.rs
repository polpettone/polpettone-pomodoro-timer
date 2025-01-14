use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use std::error::Error;
use std::fs;
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
    pub fn start_session(
        &self,
        description: &str,
        duration_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start = SystemTime::now();
        let start_date: DateTime<Utc> = start.into();

        println!("Using {}", &self.pomodoro_session_dir);

        let session_dir = &self.pomodoro_session_dir;

        let session = Session {
            description: description.to_string(),
            duration: Duration::new(duration_seconds, 0),
            start: start_date,
        };

        serialize_session(&session, session_dir, start_date)?;
        Ok(())
    }

    pub fn load_sessions(&self) -> Result<Vec<Session>, Box<dyn std::error::Error>> {
        let mut sessions = Vec::new();
        let paths = fs::read_dir(&self.pomodoro_session_dir)?;

        for path in paths {
            let path = path?.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let contents = fs::read_to_string(&path)?;

                let session: Session = serde_yaml::from_str(&contents)?;
                sessions.push(session);
            }
        }
        Ok(sessions)
    }

    pub fn find_all_active_sessions(&self) -> Result<Vec<Session>, Box<dyn std::error::Error>> {
        let sessions = self.load_sessions()?;
        let now = Utc::now();
        let active_sessions = sessions
            .into_iter()
            .filter(|session| session.start + session.duration > now)
            .collect();
        Ok(active_sessions)
    }
}

fn serialize_session(
    session: &Session,
    session_dir: &str,
    start_date: DateTime<Utc>,
) -> Result<(), Box<dyn Error>> {
    let filename = format!("{}-session.yaml", start_date.format("%Y%m%d%H%M%S"));
    let filepath = Path::new(session_dir).join(filename);

    let serialized = serde_yaml::to_string(&session)?;
    let mut file = File::create(filepath)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}
