use crate::domain::session::Session;
use chrono::{DateTime, Utc};
use std::error::Error;

pub trait SessionRepository {
    fn save(&self, session: &Session) -> Result<(), Box<dyn Error>>;
    fn find_all(&self) -> Result<Vec<Session>, Box<dyn Error>>;
    fn find_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Session>, Box<dyn Error>>;
    fn init_storage(&self) -> Result<(), Box<dyn Error>>;
}
