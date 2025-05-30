#![allow(dead_code)]
mod command;
mod config;
mod date_time;
mod display;
mod session;

use crate::config::Config;
use crate::session::SessionService;

use command::Command;
use dirs::home_dir;
use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
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

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();
    let config_path = get_config_path(opts.config);

    let config_string = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Could not read config {:?} : {}", config_path, e);

            if e.kind() == ErrorKind::NotFound {
                let default_config = r#"
    [pomodoro_config]
    pomodoro_session_dir = "/tmp/sessions"
    pomodoro_status_path = "/tmp/status"
    "#;
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&config_path, default_config)?;
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

    let session_service = SessionService {
        pomodoro_session_dir: config.pomodoro_config.pomodoro_session_dir,
        pomodoro_status_path: config.pomodoro_config.pomodoro_status_path,
    };

    match opts.cmd {
        Command::Start {
            duration,
            description,
        } => {
            println!("Starting session: {} for {} minutes", description, duration);

            println!("Duration: {} minutes", duration);
            println!("Description: {}", description);

            session_service.start_session(&description, duration * 60)?;
        }
        Command::Active => {
            println!("Showing all sessions");
            match session_service.find_all_active_sessions() {
                Ok(sessions) => {
                    if let Err(e) = display::print_table(sessions) {
                        println!("Error printing table: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error loading sessions: {}", e);
                }
            }
        }

        Command::Watch => loop {
            match session_service.find_all_active_sessions() {
                Ok(sessions) => {
                    session_service.update_pomodoro_status()?;
                    const ANSI_ESCAPE_CODE_FOR_SCREEN_ERASE: &str = "\x1B[2J\x1B[1;1H";
                    print!("{}", ANSI_ESCAPE_CODE_FOR_SCREEN_ERASE);
                    for session in sessions {
                        let duration_secs = session.duration.as_secs();
                        let duration_mins = duration_secs / 60;
                        let duration_remaining_secs = duration_secs % 60;

                        let elapsed_secs = session.elapsed_duration().as_secs();
                        let elapsed_mins = elapsed_secs / 60;
                        let elapsed_remaining_secs = elapsed_secs % 60;

                        println!(
                            "{}, {}, {}:{:02}, {}:{:02}",
                            session.description,
                            session.start,
                            duration_mins,
                            duration_remaining_secs,
                            elapsed_mins,
                            elapsed_remaining_secs
                        )
                    }
                }
                Err(e) => {
                    eprintln!("Error loadings sessions: {}", e)
                }
            }
            thread::sleep(Duration::from_secs(1));
        },

        Command::FindSessionsInRange {
            start_date,
            end_date,
            search_query,
            export,
        } => {
            use chrono::prelude::*;

            // Parse dates using NaiveDateTime first
            let parsed_start = NaiveDateTime::parse_from_str(&start_date, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc());
            let parsed_end = NaiveDateTime::parse_from_str(&end_date, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc());

            match (parsed_start, parsed_end) {
                (Ok(start), Ok(end)) => {
                    match session_service.find_sessions_in_range(start, end, search_query) {
                        Ok(sessions) => {
                            if export {
                                display::export_to_ascii_table(sessions)?;
                            } else {
                                display::print_table(sessions)?;
                            }
                        }
                        Err(err) => println!("Error finding sessions: {}", err),
                    }
                }
                (Err(e1), Err(e2)) => {
                    println!("Failed to parse dates: {} and {}", e1, e2);
                }
                (Err(e), _) => {
                    println!("Failed to parse start date: {}", e);
                }
                (_, Err(e)) => {
                    println!("Failed to parse end date: {}", e);
                }
            }
        }
        Command::FindSessionFromToday {
            search_query,
            export,
        } => {
            use chrono::prelude::*;
            let now = Utc::now();
            let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
            let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

            match session_service.find_sessions_in_range(start, end, search_query) {
                Ok(sessions) => {
                    if export {
                        display::export_to_ascii_table(sessions)?;
                    } else {
                        display::print_table(sessions)?;
                    }
                }
                Err(err) => println!("Error finding sessions: {}", err),
            }
        }
        Command::FindSessionFromYesterday {
            search_query,
            export,
        } => {
            use chrono::prelude::*;
            let now = Utc::now();
            let yesterday = (now - chrono::Duration::days(1)).date_naive();
            let start = yesterday
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap();
            let end = yesterday
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap();

            match session_service.find_sessions_in_range(start, end, search_query) {
                Ok(sessions) => {
                    if export {
                        display::export_to_ascii_table(sessions)?;
                    } else {
                        display::print_table(sessions)?;
                    }
                }
                Err(err) => println!("Error finding sessions: {}", err),
            }
        }
    }

    Ok(())
}
