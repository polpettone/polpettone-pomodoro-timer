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

use chrono::{serde::ts_seconds, DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Defining the Session struct in Rust
#[derive(Serialize, Deserialize, Debug)]
struct Session {
    description: String,
    duration: Duration,
    #[serde(with = "ts_seconds")]
    start: DateTime<Utc>,
}

fn main() {
    let args = Cli::from_args();
    println!("Duration: {} minutes", args.duration);
    println!("Description: {}", args.description);
    println!("Finish Command: {}", args.finish_command);

    let naive_datetime =
        NaiveDateTime::parse_from_str("2023-10-10 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let start: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_datetime, Utc);

    let test_session = Session {
        description: String::from("Example session"),
        duration: Duration::new(3600, 0), // 1 hour in seconds
        start,
    };

    println!("{:?}", test_session);
}
