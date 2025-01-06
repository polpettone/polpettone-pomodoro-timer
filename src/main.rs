use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "pomodoro")]
struct Cli {
    /// Duration in minutes
    #[structopt(short = "t", long = "duration", default_value = "25")]
    duration: i32,

    /// Description of this session
    #[structopt(short = "d", long = "description", default_value = "no description")]
    description: String,

    /// Command to execute when session finished
    #[structopt(short = "f", long = "finishCommand", default_value = "i3lock")]
    finish_command: String,
}

use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

// Defining the Session struct in Rust
#[derive(Serialize, Deserialize, Debug)]
struct Session {
    description: String,
    duration: Duration,
    #[serde(with = "ts_seconds")]
    start: DateTime<Utc>,
}

struct SessionService;

impl SessionService {
    fn start_session(&self, description: &str, duration_seconds: u64) -> Session {
        let start = SystemTime::now();
        let datetime: DateTime<Utc> = start.into();

        Session {
            description: description.to_string(),
            duration: Duration::new(duration_seconds, 0),
            start: datetime,
        }
    }
}

fn main() {
    let args = Cli::from_args();
    println!("Duration: {} minutes", args.duration);
    println!("Description: {}", args.description);
    println!("Finish Command: {}", args.finish_command);

    let session_service = SessionService;
    let session = session_service.start_session("New session", 3600);
    println!("{:?}", session);
}
