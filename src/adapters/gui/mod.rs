use crate::application::service::SessionService;
use crate::domain::repository::SessionRepository;
use std::error::Error;

pub mod components;
pub mod styles;

pub mod app;

pub fn run<R: SessionRepository + Send + 'static>(
    session_service: SessionService<R>,
) -> Result<(), Box<dyn Error>> {
    app::run(session_service)
}
