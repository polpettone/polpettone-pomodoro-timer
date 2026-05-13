use crate::domain::user::{User, LoginRequest, LoginResponse};
use std::error::Error;
use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // Username
    exp: usize,
}

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
        
        // In Produktion sollte das Geheimnis über eine Umgebungsvariable gesetzt werden
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-key".to_string());
        
        Self {
            users,
            secret,
        }
    }

    pub fn login(&self, request: LoginRequest) -> Result<LoginResponse, Box<dyn Error>> {
        let user = self.users.iter().find(|u| u.username == request.username);
        
        if let Some(user) = user {
            if user.password_hash == request.password {
                // Produktionsreife Token-Generierung mit JWT
                let expiration = Utc::now()
                    .checked_add_signed(chrono::Duration::hours(24))
                    .expect("valid timestamp")
                    .timestamp() as usize;

                let claims = Claims {
                    sub: user.username.clone(),
                    exp: expiration,
                };

                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(self.secret.as_ref()),
                )?;

                return Ok(LoginResponse {
                    token,
                });
            }
        }
        
        Err("Ungültiger Benutzername oder Passwort".into())
    }

    pub fn validate_token(&self, token: &str) -> Option<String> {
        let validation = Validation::default();
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        ) {
            Ok(token_data) => {
                let username = token_data.claims.sub;
                // Optional: Prüfen, ob der Benutzer noch in unserer statischen Liste existiert
                if self.users.iter().any(|u| u.username == username) {
                    Some(username)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
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
