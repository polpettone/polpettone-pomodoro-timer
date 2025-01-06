mod session;

use crate::session::SessionService;
use serde::Deserialize;
use structopt::StructOpt;

use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct PomodoroConfig {
    pub pomodoro_session_dir: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    pomodoro_config: PomodoroConfig,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "pomodoro")]
struct Cli {
    /// Duration in minutes
    #[structopt(short = "t", long = "duration", default_value = "25")]
    duration: u64,

    /// Description of this session
    #[structopt(short = "d", long = "description", default_value = "no description")]
    description: String,

    /// Command to execute when session finished
    #[structopt(short = "f", long = "finishCommand", default_value = "i3lock")]
    finish_command: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_string = match fs::read_to_string("config.toml") {
        Ok(content) => content,
        Err(e) => {
            println!("Error reading configuration file: {}", e);
            return Err(Box::new(e));
        }
    };

    let config: Config = match toml::from_str(&config_string) {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("Error deserialize configuration: {}", e);
            return Err(Box::new(e));
        }
    };

    let args = Cli::from_args();
    println!("Duration: {} minutes", args.duration);
    println!("Description: {}", args.description);
    println!("Finish Command: {}", args.finish_command);

    let session_service = SessionService {
        pomodoro_session_dir: config.pomodoro_config.pomodoro_session_dir,
    };
    let session = session_service.start_session(&args.description, args.duration * 60);
    println!("{:?}", session);

    Ok(())
}
