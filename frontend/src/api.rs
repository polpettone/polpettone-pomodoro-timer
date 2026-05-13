use gloo_net::http::Request;
use crate::models::Session;

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

    resp.json::<Vec<Session>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_all_sessions(token: String) -> Result<Vec<Session>, String> {
    let resp = Request::get(&format!("{}/sessions?start=2000-01-01%2000:00:00&end=2099-12-31%2023:59:59", API_BASE_URL))
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

    let mut sessions = resp.json::<Vec<Session>>()
        .await
        .map_err(|e| e.to_string())?;
    
    sessions.sort_by(|a, b| b.start.cmp(&a.start));
    
    Ok(sessions)
}
