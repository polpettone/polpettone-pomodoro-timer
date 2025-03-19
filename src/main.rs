mod command;
mod config;
mod date_time;
mod session;

use crate::config::Config;
use command::Command;

use crate::session::Session;
use crate::session::SessionService;
use dialoguer::Select;

use std::time::Duration;

use structopt::StructOpt;

use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use std::thread;

use date_time::duration_in_minutes;
use std::error::Error;

#[derive(StructOpt, Debug)]
#[structopt(name = "pomodoro")]
struct Opts {
    /// Optional configuration file
    #[structopt(short = "c", long = "config", default_value = "config.toml")]
    config: String,

    #[structopt(subcommand)]
    cmd: Command,
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
                    print_table(sessions)?;
                    select_options();
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
                    session_service.update_pomodoro_status(
                        "/home/kenny/pomodoro/pomodoro-status".to_string(),
                    )?;
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

fn print_table(sessions: Vec<Session>) -> Result<(), Box<dyn Error>> {
    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("Description").add_attribute(Attribute::Bold),
            Cell::new("Duration").add_attribute(Attribute::Bold),
            Cell::new("Start Time").add_attribute(Attribute::Bold),
        ])
        .set_content_arrangement(ContentArrangement::Dynamic);

    for session in sessions {
        table.add_row(vec![
            Cell::new(session.description),
            Cell::new(format!("{:?}", duration_in_minutes(session.duration))), // Format duration as needed
            Cell::new(session.start.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]);
    }

    println!("{}", table);

    Ok(())
}

fn select_options() {
    let options = &["Session 1", "Session 2", "Session 3"];

    let selection = Select::new()
        .with_prompt("Select your session")
        .default(0)
        .items(&options[..])
        .interact()
        .unwrap();

    println!("You selected: {}", options[selection]);
}
