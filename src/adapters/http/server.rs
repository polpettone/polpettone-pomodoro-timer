use axum::{
    routing::{get, post},
    extract::{State, Query},
    Json, Router,
};
use crate::application::service::SessionService;
use crate::domain::repository::SessionRepository;
use serde::Deserialize;
use std::sync::Arc;
use std::net::SocketAddr;
use chrono::{Utc, NaiveDateTime};
use rand::Rng;
use chrono::Duration as ChronoDuration;
use crate::domain::session::{Session, SessionState};
use std::time::Duration;

pub struct AppState<R: SessionRepository> {
    pub service: SessionService<R>,
}

#[derive(Deserialize)]
pub struct StartSessionRequest {
    pub description: String,
    pub duration_minutes: u64,
}

#[derive(Deserialize)]
pub struct FindSessionsRequest {
    pub start: Option<String>,
    pub end: Option<String>,
    pub query: Option<String>,
}

#[derive(Deserialize)]
pub struct GenerateRequest {
    pub number: u32,
}

pub async fn run_server<R: SessionRepository + Send + Sync + 'static>(
    service: SessionService<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState { service });

    let app = Router::new()
        .route("/sessions/start", post(start_session::<R>))
        .route("/sessions/active", get(get_active_sessions::<R>))
        .route("/sessions", get(get_sessions::<R>))
        .route("/sessions/init", post(init_session_dir::<R>))
        .route("/sessions/generate", post(generate_test_data::<R>))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn start_session<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    Json(payload): Json<StartSessionRequest>,
) -> Result<Json<String>, String> {
    state.service.start_session(&payload.description, payload.duration_minutes * 60)
        .map_err(|e| e.to_string())?;
    Ok(Json("Session started".to_string()))
}

async fn get_active_sessions<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
) -> Result<Json<Vec<Session>>, String> {
    let sessions = state.service.find_all_active_sessions()
        .map_err(|e| e.to_string())?;
    Ok(Json(sessions))
}

async fn get_sessions<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    Query(params): Query<FindSessionsRequest>,
) -> Result<Json<Vec<Session>>, String> {
    let now = Utc::now();
    let start = if let Some(s) = params.start {
        NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc())
            .map_err(|_| "Invalid start date. Use YYYY-MM-DD HH:MM:SS".to_string())?
    } else {
        now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()
    };

    let end = if let Some(e) = params.end {
        NaiveDateTime::parse_from_str(&e, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc())
            .map_err(|_| "Invalid end date. Use YYYY-MM-DD HH:MM:SS".to_string())?
    } else {
        now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc()
    };

    let sessions = state.service.find_sessions_in_range(start, end, params.query)
        .map_err(|e| e.to_string())?;
    Ok(Json(sessions))
}

async fn init_session_dir<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
) -> Result<Json<String>, String> {
    state.service.init_session_dir()
        .map_err(|e| e.to_string())?;
    Ok(Json("Session directory initialized".to_string()))
}

async fn generate_test_data<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    Json(payload): Json<GenerateRequest>,
) -> Result<Json<String>, String> {
    let mut rng = rand::rng();
    let now = Utc::now();
    let descriptions = vec![
        "Implement feature X", "Fix bug Y", "Code review", "Planning meeting",
        "Refactoring", "Documentation", "Learning Rust", "Setup environment",
    ];

    for _ in 0..payload.number {
        let days_ago = rng.random_range(0..30);
        let hours_ago = rng.random_range(0..24);
        let minutes_ago = rng.random_range(0..60);

        let start_time = now
            - ChronoDuration::days(days_ago)
            - ChronoDuration::hours(hours_ago)
            - ChronoDuration::minutes(minutes_ago);

        let session = Session {
            description: descriptions[rng.random_range(0..descriptions.len())].to_string(),
            duration: Duration::from_secs(25 * 60),
            start: start_time,
            tags: vec![],
            notes: String::new(),
            state: SessionState::Done,
            ratings: None,
        };
        state.service.save_session(&session).map_err(|e| e.to_string())?;
    }

    Ok(Json(format!("Generated {} test sessions", payload.number)))
}
