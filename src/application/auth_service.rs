use crate::domain::user::{User, LoginRequest, LoginResponse};
use std::error::Error;

pub struct AuthService {
    // Statische Liste von Benutzern
    users: Vec<User>,
    // In einer echten App wäre das ein Geheimnis für JWT oder Sessions
    secret: String,
}

impl AuthService {
    pub fn new() -> Self {
        let users = vec![
            User {
                username: "admin".to_string(),
                password_hash: "admin".to_string(), // In Iteration 1 einfach Klartext
            },
            User {
                username: "user".to_string(),
                password_hash: "password".to_string(),
            },
        ];
        Self {
            users,
            secret: "super-secret-key".to_string(),
        }
    }

    pub fn login(&self, request: LoginRequest) -> Result<LoginResponse, Box<dyn Error>> {
        let user = self.users.iter().find(|u| u.username == request.username);
        
        if let Some(user) = user {
            if user.password_hash == request.password {
                // Generiere ein einfaches Token (für Iteration 1 reicht der Username)
                // In einer echten App wäre das ein signiertes JWT
                return Ok(LoginResponse {
                    token: format!("token-for-{}", user.username),
                });
            }
        }
        
        Err("Ungültiger Benutzername oder Passwort".into())
    }

    pub fn validate_token(&self, token: &str) -> Option<String> {
        if token.starts_with("token-for-") {
            let username = &token[10..];
            if self.users.iter().any(|u| u.username == username) {
                return Some(username.to_string());
            }
        }
        None
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        Self {
            users: self.users.clone(),
            secret: self.secret.clone(),
        }
    }
}
