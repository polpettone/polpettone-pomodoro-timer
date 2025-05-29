use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::date_time::{deserialize_human_readable, duration_in_minutes, serialize_human_readable};
use std::fs::OpenOptions;
use std::io;

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub description: String,
    pub duration: Duration,
    #[serde(
        serialize_with = "serialize_human_readable",
        deserialize_with = "deserialize_human_readable"
    )]
    pub start: DateTime<Utc>,
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
}

pub struct SessionService {
    pub pomodoro_session_dir: String,
    pub pomodoro_status_path: String,
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

    pub fn update_pomodoro_status(&self) -> Result<(), io::Error> {
        if let Ok(sessions) = self.find_all_active_sessions() {
            if let Some(session) = sessions.get(0) {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(self.pomodoro_status_path.clone())?;

                writeln!(
                    file,
                    "{} - {}/{}",
                    session.description,
                    duration_in_minutes(session.duration),
                    duration_in_minutes(session.elapsed_duration())
                )?;
            }
        }
        Ok(())
    }

    pub fn find_sessions_in_range(
        &self,
        range_start: DateTime<Utc>,
        range_end: DateTime<Utc>,
        search_query: Option<String>,
    ) -> Result<Vec<Session>, Box<dyn std::error::Error>> {
        let sessions = self.load_sessions()?;
        let sessions_in_range = sessions
            .into_iter()
            .filter(|session| {
                let session_end = session.start + session.duration;
                let time_matches = session.start < range_end && session_end > range_start;

                match &search_query {
                    Some(query) => {
                        time_matches
                            && session
                                .description
                                .to_lowercase()
                                .contains(&query.to_lowercase())
                    }
                    None => time_matches,
                }
            })
            .collect();
        Ok(sessions_in_range)
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
