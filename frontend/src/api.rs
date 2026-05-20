use crate::models::{Session, SessionRatings, User};
use gloo_net::http::Request;
use serde::Serialize;

pub const API_BASE_URL: &str = match option_env!("API_URL") {
    Some(url) => url,
    None => "http://127.0.0.1:3000",
};

pub async fn fetch_active_sessions(token: String) -> Result<Vec<Session>, String> {
    let resp = Request::get(&format!("{}/sessions/active", API_BASE_URL))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        if resp.status() == 401 {
            return Err("Nicht autorisiert. Bitte erneut anmelden.".to_string());
        }
        return Err(format!("Fehler beim Laden: {}", resp.status()));
    }

    resp.json::<Vec<Session>>().await.map_err(|e| e.to_string())
}

pub async fn fetch_current_user(token: String) -> Result<User, String> {
    let resp = Request::get(&format!("{}/me", API_BASE_URL))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!(
            "Fehler beim Laden des Benutzers: {}",
            resp.status()
        ));
    }

    resp.json::<User>().await.map_err(|e| e.to_string())
}

pub async fn register(username: String, password: String) -> Result<(), String> {
    let payload = serde_json::json!({
        "username": username,
        "password": password,
    });

    let resp = Request::post(&format!("{}/register", API_BASE_URL))
        .json(&payload)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let error_msg = resp
            .text()
            .await
            .unwrap_or_else(|_| "Registrierung fehlgeschlagen".to_string());
        return Err(error_msg);
    }

    Ok(())
}

pub async fn fetch_all_sessions(token: String) -> Result<Vec<Session>, String> {
    let resp = Request::get(&format!(
        "{}/sessions?start=2000-01-01%2000:00:00&end=2099-12-31%2023:59:59",
        API_BASE_URL
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await
    .map_err(|e| e.to_string())?;

    if !resp.ok() {
        if resp.status() == 401 {
            return Err("Nicht autorisiert.".to_string());
        }
        return Err(format!("Fehler beim Laden der Historie: {}", resp.status()));
    }

    let mut sessions = resp
        .json::<Vec<Session>>()
        .await
        .map_err(|e| e.to_string())?;

    sessions.sort_by(|a, b| b.start.cmp(&a.start));

    Ok(sessions)
}

#[derive(Serialize)]
pub struct StartSessionRequest {
    pub description: String,
    pub duration_minutes: u64,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub ratings: Option<SessionRatings>,
}

pub async fn start_session(token: String, payload: StartSessionRequest) -> Result<(), String> {
    let resp = Request::post(&format!("{}/sessions/start", API_BASE_URL))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&payload)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Fehler beim Starten: {}", resp.status()));
    }

    Ok(())
}

#[derive(Serialize)]
pub struct UpdateSessionRequest {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub ratings: Option<SessionRatings>,
    pub state: Option<String>,
}

pub async fn update_session(
    token: String,
    id: uuid::Uuid,
    payload: UpdateSessionRequest,
) -> Result<Session, String> {
    let resp = Request::post(&format!("{}/sessions/{}", API_BASE_URL, id))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&payload)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Fehler beim Aktualisieren: {}", resp.status()));
    }

    resp.json::<Session>().await.map_err(|e| e.to_string())
}

pub async fn delete_session(token: String, id: uuid::Uuid) -> Result<(), String> {
    let resp = Request::delete(&format!("{}/sessions/{}", API_BASE_URL, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Fehler beim Löschen: {}", resp.status()));
    }

    Ok(())
}
