use crate::domain::repository::{SessionRepository, UserRepository};
use crate::domain::session::{Session, SessionState};
use crate::domain::user::User;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::error::Error;
use std::time::Duration;

#[derive(Clone)]
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_db(&self) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY,
                password_hash TEXT NOT NULL
            )"
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id SERIAL PRIMARY KEY,
                description TEXT NOT NULL,
                duration_secs BIGINT NOT NULL,
                start_time TIMESTAMPTZ NOT NULL,
                state TEXT NOT NULL,
                notes TEXT NOT NULL,
                tags TEXT[] NOT NULL,
                ratings JSONB
            )"
        ).execute(&self.pool).await?;

        // Seed default users if empty
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool).await?;
        
        if count == 0 {
            sqlx::query("INSERT INTO users (username, password_hash) VALUES ($1, $2), ($3, $4)")
                .bind("admin")
                .bind("admin")
                .bind("user")
                .bind("password")
                .execute(&self.pool).await?;
        }

        Ok(())
    }
}

use std::future::Future;
use std::pin::Pin;

impl SessionRepository for PostgresRepository {
    fn save(&self, session: &Session) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        let pool = self.pool.clone();
        let session = session.clone();
        
        Box::pin(async move {
            let state_str = match session.state {
                SessionState::Running => "Running",
                SessionState::Done => "Done",
                SessionState::Deleted => "Deleted",
                SessionState::Canceled => "Canceled",
            };

            let ratings_json = session.ratings.as_ref().map(|r| serde_json::to_value(r).unwrap());

            sqlx::query(
                "INSERT INTO sessions (description, duration_secs, start_time, state, notes, tags, ratings)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(session.description)
            .bind(session.duration.as_secs() as i64)
            .bind(session.start)
            .bind(state_str)
            .bind(session.notes)
            .bind(session.tags)
            .bind(ratings_json)
            .execute(&pool)
            .await?;
            Ok::<(), Box<dyn Error>>(())
        })
    }

    fn find_all(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, Box<dyn Error>>> + Send + '_>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let rows = sqlx::query("SELECT description, duration_secs, start_time, state, notes, tags, ratings FROM sessions")
                .fetch_all(&pool)
                .await?;

            let mut sessions = Vec::new();
            for row in rows {
                sessions.push(row_to_session(&row));
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
                "SELECT description, duration_secs, start_time, state, notes, tags, ratings 
                 FROM sessions 
                 WHERE start_time >= $1 AND start_time <= $2"
            )
            .bind(start)
            .bind(end)
            .fetch_all(&pool)
            .await?;

            let mut sessions = Vec::new();
            for row in rows {
                sessions.push(row_to_session(&row));
            }
            Ok::<Vec<Session>, Box<dyn Error>>(sessions)
        })
    }

    fn init_storage(&self) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        Box::pin(async move {
            self.init_db().await?;
            Ok(())
        })
    }
}

impl UserRepository for PostgresRepository {
    fn find_by_username(&self, username: &str) -> Pin<Box<dyn Future<Output = Result<Option<User>, Box<dyn Error>>> + Send + '_>> {
        let pool = self.pool.clone();
        let username = username.to_string();
        
        Box::pin(async move {
            let user = sqlx::query_as::<_, UserRecord>("SELECT username, password_hash FROM users WHERE username = $1")
                .bind(username)
                .fetch_optional(&pool)
                .await?;

            Ok::<Option<User>, Box<dyn Error>>(user.map(|u| User {
                username: u.username,
                password_hash: u.password_hash,
            }))
        })
    }
}

#[derive(sqlx::FromRow)]
struct UserRecord {
    username: String,
    password_hash: String,
}

fn row_to_session(row: &sqlx::postgres::PgRow) -> Session {
    let state_str: String = row.get("state");
    let state = match state_str.as_str() {
        "Running" => SessionState::Running,
        "Deleted" => SessionState::Deleted,
        "Canceled" => SessionState::Canceled,
        _ => SessionState::Done,
    };

    let ratings_json: Option<serde_json::Value> = row.get("ratings");
    let ratings = ratings_json.map(|v| serde_json::from_value(v).unwrap());

    Session {
        description: row.get("description"),
        duration: Duration::from_secs(row.get::<i64, _>("duration_secs") as u64),
        start: row.get("start_time"),
        tags: row.get("tags"),
        notes: row.get("notes"),
        state,
        ratings,
    }
}
