mod config;
mod session;

use crate::config::Config;
use crate::session::SessionService;

use structopt::StructOpt;

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

        /// Command to execute when session finished
        #[structopt(short = "f", long = "finishCommand", default_value = "i3lock")]
        finish_command: String,
    },
    /// Show all sessions
    Show,
    Active,
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
            finish_command,
        } => {
            println!("Starting session: {} for {} minutes", description, duration);

            println!("Duration: {} minutes", duration);
            println!("Description: {}", description);
            println!("Finish Command: {}", finish_command);

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
    }

    Ok(())
}
