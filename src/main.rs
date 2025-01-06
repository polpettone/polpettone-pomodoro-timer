mod session;

use crate::session::SessionService;

use structopt::StructOpt;

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

fn main() {
    let args = Cli::from_args();
    println!("Duration: {} minutes", args.duration);
    println!("Description: {}", args.description);
    println!("Finish Command: {}", args.finish_command);

    let session_service = SessionService;
    let session = session_service.start_session(&args.description, args.duration * 60);
    println!("{:?}", session);
}
