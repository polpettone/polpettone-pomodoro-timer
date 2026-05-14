use crate::domain::repository::UserRepository;
use crate::domain::user::User;
use bcrypt::{hash, DEFAULT_COST};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct StaticUserRepository {
    users: Arc<Mutex<Vec<User>>>,
}

impl StaticUserRepository {
    pub fn new() -> Self {
        let users = vec![
            User {
                username: "admin".to_string(),
                password_hash: hash("admin", DEFAULT_COST).unwrap(),
            },
            User {
                username: "user".to_string(),
                password_hash: hash("password", DEFAULT_COST).unwrap(),
            },
        ];
        Self {
            users: Arc::new(Mutex::new(users)),
        }
    }
}

impl UserRepository for StaticUserRepository {
    fn find_by_username(
        &self,
        username: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<User>, Box<dyn Error>>> + Send + '_>> {
        let username = username.to_string();
        let users = self.users.clone();
        Box::pin(async move {
            let users = users.lock().map_err(|e| e.to_string())?;
            let user = users.iter().find(|u| u.username == username).cloned();
            Ok(user)
        })
    }

    fn save(
        &self,
        user: &User,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        let users = self.users.clone();
        let user = user.clone();
        Box::pin(async move {
            let mut users = users.lock().map_err(|e| e.to_string())?;
            if let Some(existing) = users.iter_mut().find(|u| u.username == user.username) {
                *existing = user;
            } else {
                users.push(user);
            }
            Ok(())
        })
    }
}
