use crate::application::service::SessionService;
use crate::domain::repository::SessionRepository;
use crate::adapters::cli::command::Command;
use crate::display;
use crate::adapters::tui::app::App;
use chrono::{Duration as ChronoDuration, Utc, NaiveDateTime};
use std::error::Error;
use std::thread;
use std::time::Duration;
use crate::domain::session::{Session, SessionState};
use rand::Rng;

pub fn handle_command<R: SessionRepository>(
    cmd: Command,
    session_service: &SessionService<R>,
) -> Result<(), Box<dyn Error>> {
    match cmd {
        Command::Tui => handle_tui(session_service)?,
        Command::InitSessionDir => handle_init_session_dir(session_service)?,
        Command::Start {
            duration,
            description,
        } => handle_start(session_service, description, duration)?,
        Command::Active => handle_active(session_service)?,
        Command::Watch => handle_watch(session_service)?,
        Command::FindSessionsInRange {
            start_date,
            end_date,
            search_query,
            export,
        } => {
            handle_find_sessions_in_range(session_service, start_date, end_date, search_query, export)?;
        }
        Command::FindSessionFromToday {
            search_query,
            export,
        } => {
            handle_find_today(session_service, search_query, export)?;
        }
        Command::FindSessionFromYesterday {
            search_query,
            export,
        } => {
            handle_find_yesterday(session_service, search_query, export)?;
        }
        Command::GenerateTestData { number } => {
            handle_generate_test_data(session_service, number)?;
        }
    }
    Ok(())
}

fn handle_tui<R: SessionRepository>(session_service: &SessionService<R>) -> Result<(), Box<dyn Error>> {
    let sessions = session_service.load_sessions()?;
    // This is a bit tricky since App currently expects a String dir. 
    // We'll fix App later when moving TUI to adapters.
    let mut app = App::new(sessions, session_service.pomodoro_session_dir_clone());
    app.run()?;
    Ok(())
}

fn handle_init_session_dir<R: SessionRepository>(session_service: &SessionService<R>) -> Result<(), Box<dyn Error>> {
    println!("init session dir");
    session_service.init_session_dir()?;
    Ok(())
}

fn handle_start<R: SessionRepository>(
    session_service: &SessionService<R>,
    description: String,
    duration: u64,
) -> Result<(), Box<dyn Error>> {
    println!("Starting session: {} for {} minutes", description, duration);

    println!("Duration: {} minutes", duration);
    println!("Description: {}", description);

    session_service.start_session(&description, duration * 60)?;
    Ok(())
}

fn handle_active<R: SessionRepository>(session_service: &SessionService<R>) -> Result<(), Box<dyn Error>> {
    println!("Showing all sessions");
    match session_service.find_all_active_sessions() {
        Ok(sessions) => {
            if let Err(e) = display::print_table(sessions) {
                println!("Error printing table: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error loading sessions: {}", e);
        }
    }
    Ok(())
}

fn handle_watch<R: SessionRepository>(session_service: &SessionService<R>) -> Result<(), Box<dyn Error>> {
    loop {
        match session_service.find_all_active_sessions() {
            Ok(sessions) => {
                session_service.update_pomodoro_status()?;
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
    }
}

fn handle_find_sessions_in_range<R: SessionRepository>(
    session_service: &SessionService<R>,
    start_date: String,
    end_date: String,
    search_query: Option<String>,
    export: bool,
) -> Result<(), Box<dyn Error>> {
    let parsed_start = NaiveDateTime::parse_from_str(&start_date, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc());
    let parsed_end = NaiveDateTime::parse_from_str(&end_date, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc());

    match (parsed_start, parsed_end) {
        (Ok(start), Ok(end)) => {
            match session_service.find_sessions_in_range(start, end, search_query) {
                Ok(sessions) => {
                    if export {
                        display::export_to_ascii_table(sessions)?;
                    } else {
                        display::print_table(sessions)?;
                    }
                }
                Err(err) => println!("Error finding sessions: {}", err),
            }
        }
        _ => println!("Failed to parse dates."),
    }
    Ok(())
}

fn handle_find_today<R: SessionRepository>(
    session_service: &SessionService<R>,
    search_query: Option<String>,
    export: bool,
) -> Result<(), Box<dyn Error>> {
    let now = Utc::now();
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

    match session_service.find_sessions_in_range(start, end, search_query) {
        Ok(sessions) => {
            if export {
                display::export_to_ascii_table(sessions)?;
            } else {
                display::print_table(sessions)?;
            }
        }
        Err(err) => println!("Error finding sessions: {}", err),
    }
    Ok(())
}

fn handle_find_yesterday<R: SessionRepository>(
    session_service: &SessionService<R>,
    search_query: Option<String>,
    export: bool,
) -> Result<(), Box<dyn Error>> {
    let now = Utc::now();
    let yesterday = (now - chrono::Duration::days(1)).date_naive();
    let start = yesterday.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = yesterday.and_hms_opt(23, 59, 59).unwrap().and_utc();

    match session_service.find_sessions_in_range(start, end, search_query) {
        Ok(sessions) => {
            if export {
                display::export_to_ascii_table(sessions)?;
            } else {
                display::print_table(sessions)?;
            }
        }
        Err(err) => println!("Error finding sessions: {}", err),
    }
    Ok(())
}

fn handle_generate_test_data<R: SessionRepository>(
    session_service: &SessionService<R>,
    number: u32,
) -> Result<(), Box<dyn Error>> {
    println!("Generating {} test sessions...", number);
    let mut rng = rand::rng();
    let now = Utc::now();

    let descriptions = vec![
        "Implement feature X",
        "Fix bug Y",
        "Code review",
        "Planning meeting",
        "Refactoring",
        "Documentation",
        "Learning Rust",
        "Setup environment",
    ];

    for _ in 0..number {
        let days_ago = rng.random_range(0..30);
        let hours_ago = rng.random_range(0..24);
        let minutes_ago = rng.random_range(0..60);

        let start_time = now
            - ChronoDuration::days(days_ago)
            - ChronoDuration::hours(hours_ago)
            - ChronoDuration::minutes(minutes_ago);

        let session = Session {
            description: descriptions[rng.random_range(0..descriptions.len())].to_string(),
            duration: Duration::from_secs(25 * 60),
            start: start_time,
            tags: vec![],
            notes: String::new(),
            state: SessionState::Done,
            ratings: None,
        };

        // We use the repository directly or add a save method to service
        // For simplicity, let's say service has a load/save if needed or we use the repo
        // Actually, start_session creates a new one, but here we want to save a specific one.
        // I'll just use the repository from the service if I make it public or add a method.
        // Let's add a `save_session` to SessionService.
        session_service.save_session(&session)?;
    }
    println!("Done.");
    Ok(())
}
