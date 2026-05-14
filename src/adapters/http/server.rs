use crate::application::auth_service::AuthService;
use crate::application::service::SessionService;
use crate::domain::repository::SessionRepository;
use crate::domain::session::{Session, SessionState};
use crate::domain::user::{LoginRequest, LoginResponse, RegisterRequest};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::Duration as ChronoDuration;
use chrono::{NaiveDateTime, Utc};
use rand::Rng;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub struct AppState<R: SessionRepository> {
    pub service: SessionService<R>,
    pub auth_service: AuthService,
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
    auth_service: AuthService,
    host: String,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState {
        service,
        auth_service,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/login", post(login::<R>))
        .route("/register", post(register::<R>))
        .route("/sessions/start", post(start_session::<R>))
        .route("/sessions/active", get(get_active_sessions::<R>))
        .route("/sessions", get(get_sessions::<R>))
        .route("/sessions/init", post(init_session_dir::<R>))
        .route("/sessions/generate", post(generate_test_data::<R>))
        .layer(cors)
        .with_state(state);

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse()?;
    info!("Server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn login<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    state
        .auth_service
        .login(payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))
}

async fn register<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .auth_service
        .register(payload)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn check_auth<R: SessionRepository>(
    state: &AppState<R>,
    headers: &HeaderMap,
) -> Result<String, (StatusCode, String)> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing Authorization header".to_string(),
        ))?;

    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid Authorization header format".to_string(),
        ));
    }

    let token = &auth_header[7..];
    state
        .auth_service
        .validate_token(token)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid token".to_string()))
}

async fn start_session<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    headers: HeaderMap,
    Json(payload): Json<StartSessionRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    check_auth(&state, &headers).await?;
    state
        .service
        .start_session(&payload.description, payload.duration_minutes * 60)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json("Session started".to_string()))
}

async fn get_active_sessions<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    headers: HeaderMap,
) -> Result<Json<Vec<Session>>, (StatusCode, String)> {
    check_auth(&state, &headers).await?;
    let sessions = state
        .service
        .find_all_active_sessions()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(sessions))
}

async fn get_sessions<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    headers: HeaderMap,
    Query(params): Query<FindSessionsRequest>,
) -> Result<Json<Vec<Session>>, (StatusCode, String)> {
    check_auth(&state, &headers).await?;
    let now = Utc::now();
    let start = if let Some(s) = params.start {
        NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc())
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid start date. Use YYYY-MM-DD HH:MM:SS".to_string(),
                )
            })?
    } else {
        now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()
    };

    let end = if let Some(e) = params.end {
        NaiveDateTime::parse_from_str(&e, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc())
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid end date. Use YYYY-MM-DD HH:MM:SS".to_string(),
                )
            })?
    } else {
        now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc()
    };

    let sessions = state
        .service
        .find_sessions_in_range(start, end, params.query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(sessions))
}

async fn init_session_dir<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    headers: HeaderMap,
) -> Result<Json<String>, (StatusCode, String)> {
    check_auth(&state, &headers).await?;
    state
        .service
        .init_session_dir()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json("Session directory initialized".to_string()))
}

async fn generate_test_data<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    headers: HeaderMap,
    Json(payload): Json<GenerateRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    check_auth(&state, &headers).await?;
    let now = Utc::now();
    let descriptions = vec![
        "Implement feature X",
        "Fix bug Y",
        "Code review",
        "Planning meeting",
        "Refactoring",
        "Documentation",
        "Learning Rust",
        "Setup environment",
    ];

    for _ in 0..payload.number {
        let session = {
            let mut rng = rand::rng();
            let days_ago = rng.random_range(0..30);
            let hours_ago = rng.random_range(0..24);
            let minutes_ago = rng.random_range(0..60);

            let start_time = now
                - ChronoDuration::days(days_ago)
                - ChronoDuration::hours(hours_ago)
                - ChronoDuration::minutes(minutes_ago);

            Session {
                description: descriptions[rng.random_range(0..descriptions.len())].to_string(),
                duration: Duration::from_secs(25 * 60),
                start: start_time,
                tags: vec![],
                notes: String::new(),
                state: SessionState::Done,
                ratings: None,
            }
        };
        state
            .service
            .save_session(&session)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(format!("Generated {} test sessions", payload.number)))
}
