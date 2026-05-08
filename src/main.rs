mod adapters;
mod application;
mod config;
mod date_time;
mod display;
mod domain;

use crate::config::Config;
use crate::adapters::persistence::file_repository::FileSessionRepository;
use crate::application::service::SessionService;
use crate::adapters::cli::handler::handle_command;
use crate::adapters::cli::command::Command;

use dirs::home_dir;
use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "pomodoro")]
struct Opts {
    /// Optional configuration file
    #[structopt(short = "c", long = "config")]
    config: Option<String>,

    #[structopt(subcommand)]
    cmd: Command,
}

fn get_config_path(custom_path: Option<String>) -> PathBuf {
    if let Some(path) = custom_path {
        PathBuf::from(path)
    } else {
        home_dir()
            .unwrap_or_default()
            .join(".config")
            .join("polpettone-pomodoro-timer")
            .join("config.toml")
    }
}

fn load_config(config_path: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let config_string = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Could not read config {:?} : {}", config_path, e);
            let home_dir = home::home_dir().expect("could not determine home dir");
            let home_str = home_dir.to_str().expect("broken");

            if e.kind() == ErrorKind::NotFound {
                let default_config = format!(
                    r#"
    [pomodoro_config]
    pomodoro_session_dir = "{}/polpettone-pomodoro-timer-sessions/"
    "#,
                    home_str
                );
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(config_path, default_config.clone())?;
                eprintln!("A default config file created {:?}.", config_path);
                default_config.to_string()
            } else {
                return Err(Box::new(e));
            }
        }
    };
    let config: Config = match toml::from_str(&config_string) {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("Error deserialize configuration: {}", e);
            return Err(Box::new(e));
        }
    };
    Ok(config)
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();
    let config_path = get_config_path(opts.config);
    let config = load_config(&config_path)?;

    let pomodoro_session_dir = std::env::var("POMODORO_SESSION_DIR")
        .unwrap_or_else(|_| config.pomodoro_config.pomodoro_session_dir);

    let repository = FileSessionRepository::new(pomodoro_session_dir.clone());
    let session_service = SessionService::new(repository, pomodoro_session_dir);

    handle_command(opts.cmd, &session_service)
}
