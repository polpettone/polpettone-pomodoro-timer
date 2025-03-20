use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Command {
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
    FindSessionFromToday,
    FindSessionFromYesterday,
    FindSessionsInRange {
        start_date: String,
        end_date: String,
    },
}
