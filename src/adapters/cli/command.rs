use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Command {
    InitSessionDir,
    Active,
    Watch,
    Tui,
    Start {
        #[structopt(short = "t", long = "duration", default_value = "25")]
        duration: u64,

        #[structopt(short = "d", long = "description", default_value = "no description")]
        description: String,
    },
    FindSessionFromToday {
        #[structopt(short = "s", long = "search")]
        search_query: Option<String>,
        #[structopt(short = "e", long = "export")]
        export: bool,
    },
    FindSessionFromYesterday {
        #[structopt(short = "s", long = "search")]
        search_query: Option<String>,
        #[structopt(short = "e", long = "export")]
        export: bool,
    },
    FindSessionsInRange {
        start_date: String,
        end_date: String,
        #[structopt(short = "s", long = "search")]
        search_query: Option<String>,
        #[structopt(short = "e", long = "export")]
        export: bool,
    },
    GenerateTestData {
        #[structopt(short = "n", long = "number", default_value = "50")]
        number: u32,
    },
    Server {
        #[structopt(short = "h", long = "host", default_value = "127.0.0.1")]
        host: String,

        #[structopt(short = "p", long = "port", default_value = "3000")]
        port: u16,
    },
    Gui,
}
