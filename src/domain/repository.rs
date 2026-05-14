use crate::domain::session::Session;
use crate::domain::user::User;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

pub trait SessionRepository {
    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;
    fn find_all(
        &self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>>;
    fn find_in_range(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>>;
    fn init_storage(&self)
        -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;
}

pub trait UserRepository {
    fn find_by_username(
        &self,
        username: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<User>, Box<dyn Error>>> + Send + '_>>;
    fn save(
        &self,
        user: &User,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;
}
