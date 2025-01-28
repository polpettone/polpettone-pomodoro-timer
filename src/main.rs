mod config;
mod session;

use crate::config::Config;
use crate::session::SessionService;

use structopt::StructOpt;

use std::error::Error;
use std::{thread, time::Duration};

#[derive(StructOpt, Debug)]
#[structopt(name = "pomodoro")]
struct Opts {
    /// Optional configuration file
    #[structopt(short = "c", long = "config", default_value = "config.toml")]
    config: String,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Start a new session
    Start {
        /// Duration in minutes
        #[structopt(short = "t", long = "duration", default_value = "25")]
        duration: u64,

        /// Description of this session
        #[structopt(short = "d", long = "description", default_value = "no description")]
        description: String,
    },
    /// Show all sessions
    Show,
    Active,
    Watch,
    FindSessionsInRange {
        start_date: String,
        end_date: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();

    let config_string = std::fs::read_to_string(&opts.config)?;
    let config: Config = match toml::from_str(&config_string) {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("Error deserialize configuration: {}", e);
            return Err(Box::new(e));
        }
    };

    let session_service = SessionService {
        pomodoro_session_dir: config.pomodoro_config.pomodoro_session_dir,
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
        Command::Show => {
            println!("Showing all sessions");

            match session_service.load_sessions() {
                Ok(sessions) => {
                    for session in sessions {
                        println!("{:?}", session);
                    }
                }
                Err(e) => {
                    eprintln!("Error loading sessions: {}", e);
                }
            }
        }
        Command::Active => {
            println!("Showing all sessions");
            match session_service.find_all_active_sessions() {
                Ok(sessions) => {
                    for session in sessions {
                        println!("{:?}", session);
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
                    const ANSI_ESCAPE_CODE_FOR_SCREEN_ERASE: &str = "\x1B[2J\x1B[1;1H";
                    print!("{}", ANSI_ESCAPE_CODE_FOR_SCREEN_ERASE);
                    for session in sessions {
                        println!(
                            "{}, {}, {}, {}",
                            session.description,
                            session.start,
                            session.duration.as_secs(),
                            session.elapsed_duration().as_secs()
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
        } => {
            use chrono::prelude::*;
            use std::str::FromStr;

            let parsed_start = DateTime::from_str(&start_date);
            let parsed_end = DateTime::from_str(&end_date);

            match (parsed_start, parsed_end) {
                (Ok(start), Ok(end)) => {
                    match session_service.find_sessions_in_range(start, end) {
                        Ok(sessions) => {
                            println!("Found {} sessions in range:", sessions.len());
                            for session in sessions {
                                println!("{:?}", session); // Annahme: Debug-Formatierung der Session-Ausgabe
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
    }

    Ok(())
}
