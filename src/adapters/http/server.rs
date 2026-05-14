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
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub struct AppState<R: SessionRepository> {
    pub service: SessionService<R>,
    pub auth_service: AuthService,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct UpdateSessionRequest {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub ratings: Option<crate::domain::session::SessionRatings>,
    pub state: Option<SessionState>,
}

pub fn create_router<R: SessionRepository + Send + Sync + 'static>(
    service: SessionService<R>,
    auth_service: AuthService,
) -> Router {
    let state = Arc::new(AppState {
        service,
        auth_service,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/login", post(login::<R>))
        .route("/register", post(register::<R>))
        .route("/sessions/start", post(start_session::<R>))
        .route("/sessions/active", get(get_active_sessions::<R>))
        .route("/sessions", get(get_sessions::<R>))
        .route("/sessions/:id", post(update_session::<R>))
        .route("/sessions/:id", axum::routing::delete(delete_session::<R>))
        .route("/sessions/init", post(init_session_dir::<R>))
        .route("/sessions/generate", post(generate_test_data::<R>))
        .layer(cors)
        .with_state(state)
}

pub async fn run_server<R: SessionRepository + Send + Sync + 'static>(
    service: SessionService<R>,
    auth_service: AuthService,
    host: String,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(service, auth_service);

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
) -> Result<crate::domain::user::User, (StatusCode, String)> {
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
    let user = check_auth(&state, &headers).await?;
    state
        .service
        .start_session(user.id, &payload.description, payload.duration_minutes * 60)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json("Session started".to_string()))
}

async fn get_active_sessions<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    headers: HeaderMap,
) -> Result<Json<Vec<Session>>, (StatusCode, String)> {
    let user = check_auth(&state, &headers).await?;
    let sessions = state
        .service
        .find_all_active_sessions(user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(sessions))
}

async fn update_session<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    headers: HeaderMap,
    Json(payload): Json<UpdateSessionRequest>,
) -> Result<Json<Session>, (StatusCode, String)> {
    let user = check_auth(&state, &headers).await?;
    let mut sessions = state
        .service
        .load_sessions(user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session = sessions
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or((StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    if let Some(desc) = payload.description {
        session.description = desc;
    }
    if let Some(tags) = payload.tags {
        session.tags = tags;
    }
    if let Some(notes) = payload.notes {
        session.notes = notes;
    }
    if let Some(ratings) = payload.ratings {
        session.ratings = Some(ratings);
    }
    if let Some(s) = payload.state {
        session.state = s;
    }

    state
        .service
        .save_session(session)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(session.clone()))
}

async fn delete_session<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, String)> {
    let user = check_auth(&state, &headers).await?;
    let mut sessions = state
        .service
        .load_sessions(user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session = sessions
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or((StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    session.state = SessionState::Deleted;

    state
        .service
        .save_session(session)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_sessions<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    headers: HeaderMap,
    Query(params): Query<FindSessionsRequest>,
) -> Result<Json<Vec<Session>>, (StatusCode, String)> {
    let user = check_auth(&state, &headers).await?;
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
        .find_sessions_in_range(user.id, start, end, params.query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(sessions))
}

async fn init_session_dir<R: SessionRepository + Send + Sync + 'static>(
    State(state): State<Arc<AppState<R>>>,
    headers: HeaderMap,
) -> Result<Json<String>, (StatusCode, String)> {
    let _user = check_auth(&state, &headers).await?;
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
    let user = check_auth(&state, &headers).await?;
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
                id: uuid::Uuid::new_v4(),
                user_id: user.id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::persistence::memory_repository::{
        InMemorySessionRepository, InMemoryUserRepository,
    };
    use crate::domain::user::{LoginRequest, LoginResponse, RegisterRequest};
    use axum_test::TestServer;

    async fn setup_test_server() -> TestServer {
        let session_repo = InMemorySessionRepository::new();
        let user_repo = Arc::new(InMemoryUserRepository::new());
        let session_service = SessionService::new(session_repo, "test_dir".to_string());
        let auth_service = AuthService::new(user_repo);

        let app = create_router(session_service, auth_service);
        TestServer::new(app).expect("Failed to create test server")
    }

    #[tokio::test]
    async fn test_register_and_login_flow() {
        let server = setup_test_server().await;

        // 1. Register a new user
        let register_payload = RegisterRequest {
            username: "testuser".to_string(),
            password: "securepassword".to_string(),
        };

        let register_response = server.post("/register").json(&register_payload).await;

        register_response.assert_status_success();
        assert_eq!(register_response.status_code(), StatusCode::CREATED);

        // 2. Try to register the same user again (should fail)
        let duplicate_response = server.post("/register").json(&register_payload).await;

        duplicate_response.assert_status_bad_request();

        // 3. Login with the new user
        let login_payload = LoginRequest {
            username: "testuser".to_string(),
            password: "securepassword".to_string(),
        };

        let login_response = server.post("/login").json(&login_payload).await;

        login_response.assert_status_success();
        let body: LoginResponse = login_response.json();
        assert!(!body.token.is_empty());
    }

    #[tokio::test]
    async fn test_login_with_invalid_credentials() {
        let server = setup_test_server().await;

        let login_payload = LoginRequest {
            username: "nonexistent".to_string(),
            password: "wrongpassword".to_string(),
        };

        let response = server.post("/login").json(&login_payload).await;

        response.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn test_user_scoped_sessions() {
        let server = setup_test_server().await;

        // 1. Register and login User A
        let user_a = RegisterRequest {
            username: "user_a".to_string(),
            password: "password_a".to_string(),
        };
        server
            .post("/register")
            .json(&user_a)
            .await
            .assert_status_success();
        let login_a: LoginResponse = server.post("/login").json(&user_a).await.json();

        // 2. Register and login User B
        let user_b = RegisterRequest {
            username: "user_b".to_string(),
            password: "password_b".to_string(),
        };
        server
            .post("/register")
            .json(&user_b)
            .await
            .assert_status_success();
        let login_b: LoginResponse = server.post("/login").json(&user_b).await.json();

        // 3. User A starts a session
        let session_req = StartSessionRequest {
            description: "User A session".to_string(),
            duration_minutes: 25,
        };
        server
            .post("/sessions/start")
            .add_header("Authorization", format!("Bearer {}", login_a.token))
            .json(&session_req)
            .await
            .assert_status_success();

        // 4. User B checks active sessions (should be empty)
        let active_b_res = server
            .get("/sessions/active")
            .add_header("Authorization", format!("Bearer {}", login_b.token))
            .await;
        active_b_res.assert_status_success();
        let sessions_b: Vec<Session> = active_b_res.json();
        assert_eq!(sessions_b.len(), 0);

        // 5. User A checks active sessions (should have 1)
        let active_a_res = server
            .get("/sessions/active")
            .add_header("Authorization", format!("Bearer {}", login_a.token))
            .await;
        active_a_res.assert_status_success();
        let sessions_a: Vec<Session> = active_a_res.json();
        assert_eq!(sessions_a.len(), 1);
        assert_eq!(sessions_a[0].description, "User A session");
    }

    #[tokio::test]
    async fn test_update_session() {
        let server = setup_test_server().await;

        let user = RegisterRequest {
            username: "update_user".to_string(),
            password: "password".to_string(),
        };
        server
            .post("/register")
            .json(&user)
            .await
            .assert_status_success();
        let login: LoginResponse = server.post("/login").json(&user).await.json();

        // Start session
        let start_req = StartSessionRequest {
            description: "Initial description".to_string(),
            duration_minutes: 25,
        };
        server
            .post("/sessions/start")
            .add_header("Authorization", format!("Bearer {}", login.token))
            .json(&start_req)
            .await
            .assert_status_success();

        // Get session
        let sessions_res = server
            .get("/sessions/active")
            .add_header("Authorization", format!("Bearer {}", login.token))
            .await;
        let sessions: Vec<Session> = sessions_res.json();
        let session_id = sessions[0].id;

        // Update session
        let update_req = UpdateSessionRequest {
            description: Some("Updated description".to_string()),
            tags: Some(vec!["rust".to_string()]),
            notes: Some("New notes".to_string()),
            ratings: None,
            state: None,
        };

        let update_res = server
            .post(&format!("/sessions/{}", session_id))
            .add_header("Authorization", format!("Bearer {}", login.token))
            .json(&update_req)
            .await;
        update_res.assert_status_success();

        let updated_session: Session = update_res.json();
        assert_eq!(updated_session.description, "Updated description");
        assert_eq!(updated_session.tags, vec!["rust".to_string()]);
        assert_eq!(updated_session.notes, "New notes");
    }
}
