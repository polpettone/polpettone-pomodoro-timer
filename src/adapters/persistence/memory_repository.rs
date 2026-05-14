use crate::domain::repository::SessionRepository;
use crate::domain::session::Session;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct InMemorySessionRepository {
    sessions: Arc<Mutex<Vec<Session>>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl SessionRepository for InMemorySessionRepository {
    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        let sessions = self.sessions.clone();
        let session = session.clone();
        Box::pin(async move {
            let mut sessions = sessions.lock().map_err(|e| e.to_string())?;
            sessions.push(session);
            Ok(())
        })
    }

    fn find_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>> {
        let sessions = self.sessions.clone();
        Box::pin(async move {
            let sessions = sessions.lock().map_err(|e| e.to_string())?;
            Ok(sessions.clone())
        })
    }

    fn find_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>> {
        let sessions = self.sessions.clone();
        Box::pin(async move {
            let sessions = sessions.lock().map_err(|e| e.to_string())?;
            let filtered = sessions
                .iter()
                .filter(|s| s.start >= start && s.start <= end)
                .cloned()
                .collect();
            Ok(filtered)
        })
    }

    fn init_storage(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }
}
