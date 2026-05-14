pub mod file_repository;
#[cfg(test)]
pub mod memory_repository;
pub mod postgres_repository;
pub mod static_user_repository;

use crate::adapters::persistence::file_repository::FileSessionRepository;
use crate::adapters::persistence::postgres_repository::PostgresRepository;
use crate::domain::repository::SessionRepository;
use crate::domain::session::Session;
use chrono::{DateTime, Utc};
use std::error::Error;

#[derive(Clone)]
pub enum CombinedRepository {
    File(FileSessionRepository),
    Postgres(PostgresRepository),
}

use std::future::Future;
use std::pin::Pin;

impl SessionRepository for CombinedRepository {
    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        match self {
            CombinedRepository::File(r) => r.save(session),
            CombinedRepository::Postgres(r) => r.save(session),
        }
    }

    fn find_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>> {
        match self {
            CombinedRepository::File(r) => r.find_all(),
            CombinedRepository::Postgres(r) => r.find_all(),
        }
    }

    fn find_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>> {
        match self {
            CombinedRepository::File(r) => r.find_in_range(start, end),
            CombinedRepository::Postgres(r) => r.find_in_range(start, end),
        }
    }

    fn init_storage(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        match self {
            CombinedRepository::File(r) => r.init_storage(),
            CombinedRepository::Postgres(r) => r.init_storage(),
        }
    }
}
