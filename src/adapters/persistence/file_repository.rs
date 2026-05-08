use crate::domain::repository::SessionRepository;
use crate::domain::session::Session;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Clone)]
pub struct FileSessionRepository {
    session_dir: String,
}

impl FileSessionRepository {
    pub fn new(session_dir: String) -> Self {
        Self { session_dir }
    }
}

impl SessionRepository for FileSessionRepository {
    fn save(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        let filename = format!("{}-session.yaml", session.start.format("%Y%m%d%H%M%S"));
        let filepath = Path::new(&self.session_dir).join(filename);

        let serialized = serde_yaml::to_string(&session)?;
        let mut file = File::create(filepath)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Session>, Box<dyn Error>> {
        let mut sessions = Vec::new();
        let paths = fs::read_dir(&self.session_dir)?;

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

    fn find_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Session>, Box<dyn Error>> {
        let sessions = self.find_all()?;
        let sessions_in_range = sessions
            .into_iter()
            .filter(|session| {
                let session_end = session.start + session.duration;
                session.start < end && session_end > start
            })
            .collect();
        Ok(sessions_in_range)
    }

    fn init_storage(&self) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(&self.session_dir)?;
        Ok(())
    }
}
