use crate::domain::repository::UserRepository;
use crate::domain::user::{LoginRequest, LoginResponse, RegisterRequest, User};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

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
        let user = self
            .user_repository
            .find_by_username(&request.username)
            .await?;

        if let Some(user) = user {
            if verify(&request.password, &user.password_hash)? {
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

                return Ok(LoginResponse { token });
            }
        }

        Err("Ungültiger Benutzername oder Passwort".into())
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<(), Box<dyn Error>> {
        // Prüfen, ob der Benutzer bereits existiert
        if self
            .user_repository
            .find_by_username(&request.username)
            .await?
            .is_some()
        {
            return Err("Benutzername bereits vergeben".into());
        }

        // Passwort hashen
        let password_hash = hash(request.password, DEFAULT_COST)?;

        let user = User {
            id: Uuid::new_v4(),
            username: request.username,
            password_hash,
        };

        // In DB speichern
        self.user_repository.save(&user).await?;

        Ok(())
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
