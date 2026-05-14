use crate::domain::repository::{SessionRepository, UserRepository};
use crate::domain::session::Session;
use crate::domain::user::User;
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

#[derive(Clone, Default)]
pub struct InMemoryUserRepository {
    users: Arc<Mutex<Vec<User>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn find_by_username(
        &self,
        username: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<User>, Box<dyn Error>>> + Send + '_>> {
        let username = username.to_string();
        let users = self.users.clone();
        Box::pin(async move {
            let users = users.lock().map_err(|e| e.to_string())?;
            let user = users.iter().find(|u| u.username == username).cloned();
            Ok(user)
        })
    }

    fn save(
        &self,
        user: &User,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        let users = self.users.clone();
        let user = user.clone();
        Box::pin(async move {
            let mut users = users.lock().map_err(|e| e.to_string())?;
            if let Some(existing) = users.iter_mut().find(|u| u.username == user.username) {
                *existing = user;
            } else {
                users.push(user);
            }
            Ok(())
        })
    }
}
