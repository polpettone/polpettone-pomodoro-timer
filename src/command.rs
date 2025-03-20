use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Command {
    Active,
    Watch,
    Start {
        /// Duration in minutes
        #[structopt(short = "t", long = "duration", default_value = "25")]
        duration: u64,

        /// Description of this session
        #[structopt(short = "d", long = "description", default_value = "no description")]
        description: String,
    },
    Show {
        #[structopt(short = "s", long = "search")]
        search_query: Option<String>,
    },
    FindSessionFromToday {
        #[structopt(short = "s", long = "search")]
        search_query: Option<String>,
    },
    FindSessionFromYesterday {
        #[structopt(short = "s", long = "search")]
        search_query: Option<String>,
    },
    FindSessionsInRange {
        start_date: String,
        end_date: String,
        #[structopt(short = "s", long = "search")]
        search_query: Option<String>,
    },
}
