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

use std::future::Future;
use std::pin::Pin;

impl SessionRepository for FileSessionRepository {
    fn save(&self, session: &Session) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        let session = session.clone();
        let session_dir = self.session_dir.clone();
        Box::pin(async move {
            let filename = format!("{}-session.yaml", session.start.format("%Y%m%d%H%M%S"));
            let filepath = Path::new(&session_dir).join(filename);

            let serialized = serde_yaml::to_string(&session)?;
            let mut file = File::create(filepath)?;
            file.write_all(serialized.as_bytes())?;
            Ok(())
        })
    }

    fn find_all(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>> {
        let session_dir = self.session_dir.clone();
        Box::pin(async move {
            let mut sessions = Vec::new();
            let paths = fs::read_dir(&session_dir)?;

            for path in paths {
                let path = path?.path();

                if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    let contents = fs::read_to_string(&path)?;
                    let session: Session = serde_yaml::from_str(&contents)?;
                    sessions.push(session);
                }
            }
            Ok(sessions)
        })
    }

    fn find_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>> {
        Box::pin(async move {
            let sessions = self.find_all().await?;
            let sessions_in_range = sessions
                .into_iter()
                .filter(|session| {
                    let session_end = session.start + session.duration;
                    session.start < end && session_end > start
                })
                .collect();
            Ok(sessions_in_range)
        })
    }

    fn init_storage(&self) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        let session_dir = self.session_dir.clone();
        Box::pin(async move {
            fs::create_dir_all(&session_dir)?;
            Ok(())
        })
    }
}
