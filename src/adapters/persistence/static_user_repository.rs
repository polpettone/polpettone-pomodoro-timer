use crate::domain::repository::UserRepository;
use crate::domain::user::User;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct StaticUserRepository {
    users: Vec<User>,
}

impl StaticUserRepository {
    pub fn new() -> Self {
        let users = vec![
            User {
                username: "admin".to_string(),
                password_hash: "admin".to_string(),
            },
            User {
                username: "user".to_string(),
                password_hash: "password".to_string(),
            },
        ];
        Self { users }
    }
}

impl UserRepository for StaticUserRepository {
    fn find_by_username(&self, username: &str) -> Pin<Box<dyn Future<Output = Result<Option<User>, Box<dyn Error>>> + Send + '_>> {
        let username = username.to_string();
        let users = self.users.clone();
        Box::pin(async move {
            let user = users.iter().find(|u| u.username == username).cloned();
            Ok(user)
        })
    }
}
