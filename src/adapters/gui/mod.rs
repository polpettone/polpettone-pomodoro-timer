use crate::application::service::SessionService;
use crate::domain::repository::SessionRepository;
use std::error::Error;
use uuid::Uuid;

pub mod components;
pub mod styles;

pub mod app;

pub fn run<R: SessionRepository + Send + 'static>(
    user_id: Uuid,
    session_service: SessionService<R>,
) -> Result<(), Box<dyn Error>> {
    app::run(user_id, session_service)
}
