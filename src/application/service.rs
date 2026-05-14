use crate::domain::repository::SessionRepository;
use crate::domain::session::{Session, SessionState};
use chrono::{DateTime, Utc};
use std::error::Error;
use std::time::Duration;
use std::io::Write;
use std::fs::OpenOptions;
use crate::date_time::duration_in_minutes;

pub struct SessionService<R: SessionRepository> {
    repository: R,
    session_dir: String, // Still needed for the status file logic for now
}

impl<R: SessionRepository + Clone> Clone for SessionService<R> {
    fn clone(&self) -> Self {
        Self {
            repository: self.repository.clone(),
            session_dir: self.session_dir.clone(),
        }
    }
}

impl<R: SessionRepository> SessionService<R> {
    pub fn new(repository: R, session_dir: String) -> Self {
        Self { repository, session_dir }
    }

    pub async fn start_session(
        &self,
        description: &str,
        duration_seconds: u64,
    ) -> Result<(), Box<dyn Error>> {
        let start_date = Utc::now();

        let session = Session {
            description: description.to_string(),
            duration: Duration::from_secs(duration_seconds),
            start: start_date,
            tags: Vec::new(),
            notes: String::new(),
            state: SessionState::Running,
            ratings: None,
        };

        self.repository.save(&session).await?;
        Ok(())
    }

    pub async fn init_session_dir(&self) -> Result<(), Box<dyn Error>> {
        self.repository.init_storage().await?;
        Ok(())
    }

    pub async fn load_sessions(&self) -> Result<Vec<Session>, Box<dyn Error>> {
        self.repository.find_all().await
    }

    pub async fn save_session(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        self.repository.save(session).await
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }

    pub fn pomodoro_session_dir_clone(&self) -> String {
        self.session_dir.clone()
    }

    pub async fn find_all_active_sessions(&self) -> Result<Vec<Session>, Box<dyn Error>> {
        let sessions = self.repository.find_all().await?;
        let now = Utc::now();
        let active_sessions = sessions
            .into_iter()
            .filter(|session| session.start + session.duration > now)
            .filter(|session| session.state == SessionState::Running)
            .collect();
        Ok(active_sessions)
    }

    pub async fn update_pomodoro_status(&self) -> Result<(), Box<dyn Error>> {
        if let Ok(sessions) = self.find_all_active_sessions().await {
            if let Some(session) = sessions.get(0) {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(format!("{}/status", self.session_dir))?;

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

    pub async fn find_sessions_in_range(
        &self,
        range_start: DateTime<Utc>,
        range_end: DateTime<Utc>,
        search_query: Option<String>,
    ) -> Result<Vec<Session>, Box<dyn Error>> {
        let sessions = self.repository.find_in_range(range_start, range_end).await?;
        
        let filtered = if let Some(query) = search_query {
            let query = query.to_lowercase();
            sessions.into_iter()
                .filter(|s| s.description.to_lowercase().contains(&query))
                .collect()
        } else {
            sessions
        };
        
        Ok(filtered)
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<(), Box<dyn Error>> {
        let mut sessions = self.repository.find_all().await?;
        let now = Utc::now();
        let mut changed = false;
        for session in sessions.iter_mut() {
            if session.state == SessionState::Running && session.start + session.duration <= now {
                session.state = SessionState::Done;
                self.repository.save(session).await?;
                changed = true;
            }
        }
        if changed {
            self.update_pomodoro_status().await?;
        }
        Ok(())
    }
}
