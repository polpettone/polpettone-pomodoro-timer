mod adapters;
mod application;
mod config;
mod date_time;
mod domain;

use crate::adapters::cli::command::Command;
use crate::adapters::cli::handler::handle_command;
use crate::adapters::persistence::postgres_repository::PostgresRepository;
use crate::application::auth_service::AuthService;
use crate::application::service::SessionService;
use crate::config::{Config, LogFormat};
use crate::domain::repository::SessionRepository;

use dirs::home_dir;
use sqlx::PgPool;
use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(StructOpt, Debug)]
#[structopt(name = "pomodoro")]
struct Opts {
    /// Optional configuration file
    #[structopt(short = "c", long = "config")]
    config: Option<String>,

    #[structopt(subcommand)]
    cmd: Command,
}

fn init_logging(config: &Config) {
    let log_level = std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| config.pomodoro_config.log_config.level.clone());

    let log_format = std::env::var("LOG_FORMAT")
        .ok()
        .and_then(|s| match s.to_lowercase().as_str() {
            "line" => Some(LogFormat::Line),
            "json" => Some(LogFormat::Json),
            _ => None,
        })
        .unwrap_or(config.pomodoro_config.log_config.format.clone());

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    let registry = tracing_subscriber::registry().with(filter);

    match log_format {
        LogFormat::Json => {
            registry.with(fmt::layer().json()).init();
        }
        LogFormat::Line => {
            registry.with(fmt::layer()).init();
        }
    }
}

fn get_config_path(custom_path: Option<String>) -> PathBuf {
    if let Some(path) = custom_path {
        PathBuf::from(path)
    } else {
        let local_config = PathBuf::from("config.toml");
        if local_config.exists() {
            local_config
        } else {
            home_dir()
                .unwrap_or_default()
                .join(".config")
                .join("polpettone-pomodoro-timer")
                .join("config.toml")
        }
    }
}

fn load_config(config_path: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let config_string = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                let home_dir = home::home_dir().expect("could not determine home dir");
                let home_str = home_dir.to_str().expect("broken");
                let default_config = format!(
                    r#"
    [pomodoro_config]
    pomodoro_session_dir = "{}/polpettone-pomodoro-timer-sessions/"
    persistence_mode = "file"
    "#,
                    home_str
                );
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(config_path, default_config.clone())?;
                // We can't use tracing here yet because it's not initialized
                eprintln!("A default config file created {:?}.", config_path);
                default_config.to_string()
            } else {
                eprintln!("Could not read config {:?} : {}", config_path, e);
                return Err(Box::new(e));
            }
        }
    };
    let config: Config = match toml::from_str(&config_string) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error deserialize configuration: {}", e);
            return Err(Box::new(e));
        }
    };
    Ok(config)
}

fn expand_tilde(path: String) -> String {
    if path.starts_with("~/") {
        if let Some(home) = home_dir() {
            return path.replacen("~", &home.to_string_lossy(), 1);
        }
    }
    path
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let opts = Opts::from_args();
    let config_path = get_config_path(opts.config);
    let config = load_config(&config_path)?;

    init_logging(&config);

    let raw_session_dir = std::env::var("POMODORO_SESSION_DIR")
        .unwrap_or_else(|_| config.pomodoro_config.pomodoro_session_dir.clone());
    let pomodoro_session_dir = expand_tilde(raw_session_dir);

    info!("Starting application with Postgres persistence");

    let database_url = std::env::var("DATABASE_URL")
        .ok()
        .or(config.pomodoro_config.database_url)
        .expect("DATABASE_URL must be set (via env or config)");

    let pool = PgPool::connect(&database_url).await?;
    let repo = PostgresRepository::new(pool);
    let repository = repo.clone();
    let user_repository =
        Arc::new(repo) as Arc<dyn crate::domain::repository::UserRepository + Send + Sync>;

    repository.init_storage().await?;

    let session_service = SessionService::new(repository, pomodoro_session_dir);
    let auth_service = AuthService::new(user_repository);

    handle_command(opts.cmd, &session_service, auth_service).await
}
