use serde::{Deserialize, Serialize};
use std::time::Duration;

use chrono::{DateTime, Utc};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::date_time::{deserialize_human_readable, duration_in_minutes, serialize_human_readable};
use std::fs::OpenOptions;
use std::io;
use uuid::Uuid;
use serde_with::{serde_as, DisplayFromStr};

fn default_difficulty() -> u8 {
    3
}

fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}

pub trait SessionRepository {
    fn save_session(&self, session: &Session) -> Result<(), Box<dyn Error>>;
    fn load_sessions(&self) -> Result<Vec<Session>, Box<dyn Error>>;
    fn init_session_dir(&self) -> Result<(), Box<dyn Error>>;
    fn get_status_file_path(&self) -> PathBuf;
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "generate_uuid")]
    pub id: Uuid,
    pub description: String,
    pub duration: Duration,
    // difficulty from 1 to 5
    #[serde(default = "default_difficulty")]
    pub difficulty: u8,
    #[serde(
        serialize_with = "serialize_human_readable",
        deserialize_with = "deserialize_human_readable"
    )]
    pub start: DateTime<Utc>,
}

impl Session {
    pub fn new(
        description: String,
        duration: Duration,
        difficulty: u8,
    ) -> Self {
        Session {
            id: Uuid::new_v4(),
            description,
            duration,
            difficulty,
            start: Utc::now(),
        }
    }

    pub fn elapsed_duration(&self) -> Duration {
        let now = Utc::now();
        let duration_since_start = now.signed_duration_since(self.start);
        Duration::new(
            duration_since_start.num_seconds() as u64,
            duration_since_start.num_nanoseconds().unwrap_or(0) as u32,
        )
    }
}

use std::path::PathBuf;

pub struct FileSystemSessionRepository {
    pub pomodoro_session_dir: String,
}

impl SessionRepository for FileSystemSessionRepository {
    fn save_session(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        let filename = format!("{}.yaml", session.id);
        let filepath = Path::new(&self.pomodoro_session_dir).join(filename);

        let serialized = serde_yaml::to_string(&session)?;
        let mut file = File::create(filepath)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn load_sessions(&self) -> Result<Vec<Session>, Box<dyn Error>> {
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

    fn init_session_dir(&self) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(&self.pomodoro_session_dir)?;
        Ok(())
    }

    fn get_status_file_path(&self) -> PathBuf {
        PathBuf::from(&self.pomodoro_session_dir).join("status")
    }
}

pub struct SessionService {
    repository: Box<dyn SessionRepository>,
}

impl SessionService {
    pub fn new(repository: Box<dyn SessionRepository>) -> Self {
        SessionService { repository }
    }

    pub fn start_session(
        &self,
        description: &str,
        duration_seconds: u64,
        difficulty: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let session = Session::new(
            description.to_string(),
            Duration::new(duration_seconds, 0),
            difficulty,
        );

        self.repository.save_session(&session)?;
        Ok(())
    }

    pub fn init_session_dir(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.repository.init_session_dir()?;
        Ok(())
    }

    pub fn load_sessions(&self) -> Result<Vec<Session>, Box<dyn std::error::Error>> {
        self.repository.load_sessions()
    }

    pub fn find_all_active_sessions(&self) -> Result<Vec<Session>, Box<dyn std::error::Error>> {
        let sessions = self.repository.load_sessions()?;
        let now = Utc::now();
        let mut active_sessions: Vec<Session> = sessions
            .into_iter()
            .filter(|session| session.start + session.duration > now)
            .collect();

        active_sessions.sort_by(|a, b| b.start.cmp(&a.start));

        Ok(active_sessions)
    }

    pub fn update_pomodoro_status(&self) -> Result<(), io::Error> {
        if let Ok(sessions) = self.find_all_active_sessions() {
            if let Some(session) = sessions.get(0) {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(self.repository.get_status_file_path())?;

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
        let sessions = self.repository.load_sessions()?;
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
