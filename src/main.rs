#![allow(dead_code)]
mod command;
mod config;
mod date_time;
mod display;
mod session;
mod tui;

use crate::config::Config;
use crate::session::{serialize_session, Session, SessionRatings, SessionService, SessionState};

use chrono::{Duration as ChronoDuration, Utc};
use command::Command;
use dirs::home_dir;
use home;
use rand::Rng;
use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use tui::app::App;

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
                fs::write(&config_path, default_config.clone())?;
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

    let pomodoro_session_dir = std::env::var("POMODORO_SESSION_DIR")
        .unwrap_or_else(|_| config.pomodoro_config.pomodoro_session_dir);

    let session_service = SessionService {
        pomodoro_session_dir,
    };

    match opts.cmd {
        Command::Tui => {
            let sessions = session_service.load_sessions()?;
            let mut app = App::new(sessions, session_service.pomodoro_session_dir.clone());
            app.run()?;
        }
        Command::InitSessionDir => {
            println!("init session dir");
            session_service.init_session_dir()?;
        }
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
        Command::GenerateTestData { number } => {
            let test_data_dir = "test-data";
            if !std::path::Path::new(test_data_dir).exists() {
                fs::create_dir_all(test_data_dir)?;
            }
            println!(
                "Generating {} test sessions in {}...",
                number, test_data_dir
            );
            let mut rng = rand::rng();
            let now = Utc::now();

            let descriptions = vec![
                "Implement feature X",
                "Fix bug Y",
                "Code review",
                "Planning meeting",
                "Refactoring",
                "Documentation",
                "Learning Rust",
                "Setup environment",
            ];
            let tags_pool = vec![
                "work", "personal", "urgent", "learning", "rust", "tui", "fun",
            ];

            for _ in 0..number {
                let days_ago = rng.random_range(0..30);
                let hours_ago = rng.random_range(0..24);
                let minutes_ago = rng.random_range(0..60);
                let duration_minutes = 25;

                let start_time = now
                    - ChronoDuration::days(days_ago)
                    - ChronoDuration::hours(hours_ago)
                    - ChronoDuration::minutes(minutes_ago);

                let desc = descriptions[rng.random_range(0..descriptions.len())].to_string();

                let num_tags = rng.random_range(0..4);
                let mut session_tags = Vec::new();
                for _ in 0..num_tags {
                    let tag = tags_pool[rng.random_range(0..tags_pool.len())].to_string();
                    if !session_tags.contains(&tag) {
                        session_tags.push(tag);
                    }
                }

                let ratings = if rng.random_bool(0.7) {
                    Some(SessionRatings {
                        mental_energy: rng.random_range(1..=5),
                        physical_energy: rng.random_range(1..=5),
                        cognitive_load: rng.random_range(1..=5),
                        motivation: rng.random_range(1..=5),
                    })
                } else {
                    None
                };

                let session = Session {
                    description: desc,
                    duration: Duration::from_secs(duration_minutes * 60),
                    start: start_time,
                    tags: session_tags,
                    notes: if rng.random_bool(0.3) {
                        "Generated test note.".to_string()
                    } else {
                        String::new()
                    },
                    state: SessionState::Done,
                    ratings,
                };

                serialize_session(&session, test_data_dir, start_time)?;
            }
            println!("Done.");
        }
    }

    Ok(())
}

