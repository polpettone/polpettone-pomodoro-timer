use crate::domain::repository::{SessionRepository, UserRepository};
use crate::domain::session::Session;
use crate::domain::user::User;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_db(&self) -> Result<(), Box<dyn Error>> {
        // Create users table with JSONB
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;

        // Create sessions table with JSONB
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id UUID PRIMARY KEY,
                start_time TIMESTAMPTZ NOT NULL,
                data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;

        // Seed default users if empty
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        if count == 0 {
            let admin_hash = hash("admin", DEFAULT_COST)?;
            let user_hash = hash("password", DEFAULT_COST)?;

            let admin = User {
                id: Uuid::new_v4(),
                username: "admin".to_string(),
                password_hash: admin_hash,
            };

            let standard_user = User {
                id: Uuid::new_v4(),
                username: "user".to_string(),
                password_hash: user_hash,
            };

            UserRepository::save(self, &admin).await?;
            UserRepository::save(self, &standard_user).await?;
        }

        Ok(())
    }
}

impl SessionRepository for PostgresRepository {
    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        let pool = self.pool.clone();
        let session = session.clone();

        Box::pin(async move {
            let data = serde_json::to_value(&session)?;

            sqlx::query(
                "INSERT INTO sessions (id, start_time, data, modified_at)
                 VALUES ($1, $2, $3, NOW())
                 ON CONFLICT (id) DO UPDATE SET
                    data = EXCLUDED.data,
                    modified_at = NOW()",
            )
            .bind(session.id)
            .bind(session.start)
            .bind(data)
            .execute(&pool)
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })
    }

    fn find_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let rows = sqlx::query("SELECT data FROM sessions ORDER BY start_time DESC")
                .fetch_all(&pool)
                .await?;

            let mut sessions = Vec::new();
            for row in rows {
                let data: serde_json::Value = row.get("data");
                let session: Session = serde_json::from_value(data)?;
                sessions.push(session);
            }
            Ok::<Vec<Session>, Box<dyn Error>>(sessions)
        })
    }

    fn find_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let rows = sqlx::query(
                "SELECT data FROM sessions
                 WHERE start_time >= $1 AND start_time <= $2
                 ORDER BY start_time DESC",
            )
            .bind(start)
            .bind(end)
            .fetch_all(&pool)
            .await?;

            let mut sessions = Vec::new();
            for row in rows {
                let data: serde_json::Value = row.get("data");
                let session: Session = serde_json::from_value(data)?;
                sessions.push(session);
            }
            Ok::<Vec<Session>, Box<dyn Error>>(sessions)
        })
    }

    fn init_storage(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        Box::pin(async move {
            self.init_db().await?;
            Ok(())
        })
    }
}

impl UserRepository for PostgresRepository {
    fn find_by_username(
        &self,
        username: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<User>, Box<dyn Error>>> + Send + '_>> {
        let pool = self.pool.clone();
        let username = username.to_string();

        Box::pin(async move {
            let row = sqlx::query("SELECT data FROM users WHERE username = $1")
                .bind(username)
                .fetch_optional(&pool)
                .await?;

            if let Some(row) = row {
                let data: serde_json::Value = row.get("data");
                let user: User = serde_json::from_value(data)?;
                Ok(Some(user))
            } else {
                Ok(None)
            }
        })
    }

    fn save(
        &self,
        user: &User,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        let pool = self.pool.clone();
        let user = user.clone();

        Box::pin(async move {
            let data = serde_json::to_value(&user)?;

            sqlx::query(
                "INSERT INTO users (id, username, data, modified_at)
                 VALUES ($1, $2, $3, NOW())
                 ON CONFLICT (id) DO UPDATE SET
                    username = EXCLUDED.username,
                    data = EXCLUDED.data,
                    modified_at = NOW()",
            )
            .bind(user.id)
            .bind(&user.username)
            .bind(data)
            .execute(&pool)
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })
    }
}
