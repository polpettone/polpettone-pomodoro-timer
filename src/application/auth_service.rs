use crate::domain::user::{LoginRequest, LoginResponse};
use crate::domain::repository::UserRepository;
use std::error::Error;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // Username
    exp: usize,
}

pub struct AuthService {
    user_repository: Arc<dyn UserRepository + Send + Sync>,
    secret: String,
}

impl AuthService {
    pub fn new(user_repository: Arc<dyn UserRepository + Send + Sync>) -> Self {
        // In Produktion sollte das Geheimnis über eine Umgebungsvariable gesetzt werden
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-key".to_string());
        
        Self {
            user_repository,
            secret,
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, Box<dyn Error>> {
        let user = self.user_repository.find_by_username(&request.username).await?;
        
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

    pub async fn validate_token(&self, token: &str) -> Option<String> {
        let validation = Validation::default();
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        ) {
            Ok(token_data) => {
                let username = token_data.claims.sub;
                // Prüfen, ob der Benutzer noch existiert
                match self.user_repository.find_by_username(&username).await {
                    Ok(Some(_)) => Some(username),
                    _ => None,
                }
            }
            Err(_) => None,
        }
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        Self {
            user_repository: self.user_repository.clone(),
            secret: self.secret.clone(),
        }
    }
}
