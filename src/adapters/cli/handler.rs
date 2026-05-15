use crate::adapters::cli::command::Command;
use crate::adapters::cli::display;
use crate::adapters::gui;
use crate::adapters::http::server;
use crate::adapters::tui::app::App;
use crate::application::auth_service::AuthService;
use crate::application::service::SessionService;
use crate::domain::repository::SessionRepository;
use crate::domain::session::{Session, SessionState};
use chrono::{Duration as ChronoDuration, NaiveDateTime, Utc};
use rand::Rng;
use std::error::Error;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub async fn handle_command<R: SessionRepository + Send + Sync + 'static + Clone>(
    cmd: Command,
    session_service: &SessionService<R>,
    auth_service: AuthService,
) -> Result<(), Box<dyn Error>> {
    // For CLI/TUI, we try to find a default user named 'user' to scope sessions.
    // In a real CLI application, this would be handled via a login command or config.
    let user = match auth_service
        .user_repository()
        .find_by_username("user")
        .await?
    {
        Some(u) => u,
        None => {
            return Err("Default user 'user' not found. Please register via HTTP first.".into());
        }
    };
    let user_id = user.id;

    match cmd {
        Command::Tui => handle_tui(user_id, session_service).await?,
        Command::InitSessionDir => handle_init_session_dir(session_service).await?,
        Command::Start {
            duration,
            description,
        } => handle_start(user_id, session_service, duration, description).await?,
        Command::Active => handle_active(user_id, session_service).await?,
        Command::Watch => handle_watch(user_id, session_service).await?,
        Command::FindSessionsInRange {
            start_date,
            end_date,
            search_query,
            export,
        } => {
            handle_find_sessions_in_range(
                user_id,
                session_service,
                start_date,
                end_date,
                search_query,
                export,
            )
            .await?;
        }
        Command::FindSessionFromToday {
            search_query,
            export,
        } => {
            handle_find_today(user_id, session_service, search_query, export).await?;
        }
        Command::FindSessionFromYesterday {
            search_query,
            export,
        } => {
            handle_find_yesterday(user_id, session_service, search_query, export).await?;
        }
        Command::GenerateTestData { number } => {
            handle_generate_test_data(user_id, session_service, number).await?;
        }
        Command::Server { host, port } => {
            handle_server(session_service, auth_service, host, port).await?;
        }
        Command::Gui => {
            handle_gui(user_id, session_service)?;
        }
    }
    Ok(())
}

fn handle_gui<R: SessionRepository + Clone + Send + 'static>(
    user_id: Uuid,
    session_service: &SessionService<R>,
) -> Result<(), Box<dyn Error>> {
    gui::run(user_id, session_service.clone())?;
    Ok(())
}

async fn handle_server<R: SessionRepository + Send + Sync + 'static + Clone>(
    session_service: &SessionService<R>,
    auth_service: AuthService,
    host: String,
    port: u16,
) -> Result<(), Box<dyn Error>> {
    server::run_server(session_service.clone(), auth_service, host, port).await?;
    Ok(())
}

async fn handle_tui<R: SessionRepository + Clone + Send + Sync + 'static>(
    user_id: Uuid,
    session_service: &SessionService<R>,
) -> Result<(), Box<dyn Error>> {
    session_service.cleanup_expired_sessions(user_id).await?;
    let sessions = session_service.load_sessions(user_id).await?;
    let repository = session_service.repository().clone();
    let session_dir = session_service.pomodoro_session_dir_clone();

    let tokio_handle = tokio::runtime::Handle::current();
    let handle = thread::spawn(move || {
        let _guard = tokio_handle.enter();
        let mut app = App::new(user_id, sessions, session_dir, repository);
        app.run().map_err(|e| e.to_string())
    });

    handle
        .join()
        .unwrap()
        .map_err(|e| Box::from(e) as Box<dyn Error>)?;
    Ok(())
}

async fn handle_init_session_dir<R: SessionRepository>(
    session_service: &SessionService<R>,
) -> Result<(), Box<dyn Error>> {
    println!("Initializing storage...");
    session_service.init_session_dir().await?;
    Ok(())
}

async fn handle_start<R: SessionRepository>(
    user_id: Uuid,
    session_service: &SessionService<R>,
    duration: u64,
    description: String,
) -> Result<(), Box<dyn Error>> {
    println!("Starting session: {} for {} minutes", description, duration);
    session_service
        .start_session(user_id, &description, duration * 60, None, None, None)
        .await?;
    Ok(())
}

async fn handle_active<R: SessionRepository>(
    user_id: Uuid,
    session_service: &SessionService<R>,
) -> Result<(), Box<dyn Error>> {
    println!("Showing active sessions...");
    match session_service.find_all_active_sessions(user_id).await {
        Ok(sessions) => {
            if sessions.is_empty() {
                println!("No active sessions.");
            } else if let Err(e) = display::print_table(sessions) {
                println!("Error printing table: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error loading sessions: {}", e);
        }
    }
    Ok(())
}

async fn handle_watch<R: SessionRepository>(
    user_id: Uuid,
    session_service: &SessionService<R>,
) -> Result<(), Box<dyn Error>> {
    loop {
        match session_service.find_all_active_sessions(user_id).await {
            Ok(sessions) => {
                let _ = session_service.update_pomodoro_status(user_id).await;
                const ANSI_ESCAPE_CODE_FOR_SCREEN_ERASE: &str = "\x1B[2J\x1B[1;1H";
                print!("{}", ANSI_ESCAPE_CODE_FOR_SCREEN_ERASE);
                if sessions.is_empty() {
                    println!("No active sessions.");
                }
                for session in sessions {
                    let duration_secs = session.duration.as_secs();
                    let duration_mins = duration_secs / 60;
                    let duration_remaining_secs = duration_secs % 60;

                    let elapsed_secs = session.elapsed_duration().as_secs();
                    let elapsed_mins = elapsed_secs / 60;
                    let elapsed_remaining_secs = elapsed_secs % 60;

                    println!(
                        "{} | Started: {} | Target: {}:{:02} | Elapsed: {}:{:02}",
                        session.description,
                        session.start.format("%H:%M:%S"),
                        duration_mins,
                        duration_remaining_secs,
                        elapsed_mins,
                        elapsed_remaining_secs
                    )
                }
            }
            Err(e) => {
                eprintln!("Error loading sessions: {}", e)
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}

async fn handle_find_sessions_in_range<R: SessionRepository>(
    user_id: Uuid,
    session_service: &SessionService<R>,
    start_date: String,
    end_date: String,
    search_query: Option<String>,
    export: bool,
) -> Result<(), Box<dyn Error>> {
    let parsed_start =
        NaiveDateTime::parse_from_str(&start_date, "%Y-%m-%d %H:%M:%S").map(|dt| dt.and_utc());
    let parsed_end =
        NaiveDateTime::parse_from_str(&end_date, "%Y-%m-%d %H:%M:%S").map(|dt| dt.and_utc());

    match (parsed_start, parsed_end) {
        (Ok(start), Ok(end)) => {
            match session_service
                .find_sessions_in_range(user_id, start, end, search_query)
                .await
            {
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
        _ => println!("Failed to parse dates. Use YYYY-MM-DD HH:MM:SS"),
    }
    Ok(())
}

async fn handle_find_today<R: SessionRepository>(
    user_id: Uuid,
    session_service: &SessionService<R>,
    search_query: Option<String>,
    export: bool,
) -> Result<(), Box<dyn Error>> {
    let now = Utc::now();
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

    match session_service
        .find_sessions_in_range(user_id, start, end, search_query)
        .await
    {
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

async fn handle_find_yesterday<R: SessionRepository>(
    user_id: Uuid,
    session_service: &SessionService<R>,
    search_query: Option<String>,
    export: bool,
) -> Result<(), Box<dyn Error>> {
    let now = Utc::now();
    let yesterday = (now - ChronoDuration::days(1)).date_naive();
    let start = yesterday.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = yesterday.and_hms_opt(23, 59, 59).unwrap().and_utc();

    match session_service
        .find_sessions_in_range(user_id, start, end, search_query)
        .await
    {
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

async fn handle_generate_test_data<R: SessionRepository>(
    user_id: Uuid,
    session_service: &SessionService<R>,
    number: u32,
) -> Result<(), Box<dyn Error>> {
    println!("Generating {} test sessions for user...", number);
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
            id: Uuid::new_v4(),
            user_id,
            description: descriptions[rng.random_range(0..descriptions.len())].to_string(),
            duration: Duration::from_secs(25 * 60),
            start: start_time,
            tags: vec![],
            notes: String::new(),
            state: SessionState::Done,
            ratings: None,
        };

        session_service.save_session(&session).await?;
    }
    println!("Done.");
    Ok(())
}
