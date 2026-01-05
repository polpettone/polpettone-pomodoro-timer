use chrono::{NaiveDate, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::ListState,
    Frame, Terminal,
};
use std::{env, error::Error, fs, io, process::Command, time::Duration};

use crate::session::{serialize_session, Session, SessionRatings, SessionState};
use crate::tui::components::{filter_bar, info_pane, keybinds, overlay_bar, session_list, zen};
use crate::tui::events;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InputField {
    Date,
    Search,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CreationField {
    Duration,
    Description,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RatingField {
    MentalEnergy,
    PhysicalEnergy,
    CognitiveLoad,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Mode {
    Input(InputField),
    Navigation,
    Tagging,
    Creation(CreationField),
    Notes,
    DeleteConfirm,
    Rating(RatingField),
    FastFilter,
    Zen,
    PendingG,
}

pub struct App {
    pub sessions: Vec<Session>,
    pub filtered_sessions: Vec<Session>,
    pub date_input: String,
    pub search_input: String,
    pub tags_input: String,
    pub notes_input: String,
    
    pub creation_duration: String,
    pub creation_description: String,

    pub rating_mental: u8,
    pub rating_physical: u8,
    pub rating_cognitive: u8,

    pub mode: Mode,
    pub list_state: ListState,
    pub session_dir: String,
}

impl App {
    pub fn new(sessions: Vec<Session>, session_dir: String) -> App {
        let mut sessions = sessions;
        sessions.sort_by(|a, b| b.start.cmp(&a.start));
        
        for session in sessions.iter_mut() {
            if session.state == SessionState::Running {
                let remaining = session.remaining_duration();
                if remaining.as_secs() == 0 {
                    session.state = SessionState::Done;
                    let _ = serialize_session(session, &session_dir, session.start);
                }
            }
        }

        let mut app = App {
            filtered_sessions: Vec::new(),
            sessions,
            date_input: String::new(),
            search_input: String::new(),
            tags_input: String::new(),
            notes_input: String::new(),
            creation_duration: String::new(),
            creation_description: String::new(),
            rating_mental: 0,
            rating_physical: 0,
            rating_cognitive: 0,
            mode: Mode::Navigation,
            list_state: ListState::default(),
            session_dir,
        };

        app.filter_sessions();

        if !app.filtered_sessions.is_empty() {
            app.list_state.select(Some(0));
        }

        app
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_sessions.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_sessions.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn to_top(&mut self) {
        if !self.filtered_sessions.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn to_bottom(&mut self) {
        if !self.filtered_sessions.is_empty() {
            self.list_state.select(Some(self.filtered_sessions.len() - 1));
        }
    }

    pub fn filter_sessions(&mut self) {
        let date_query = self.date_input.trim();
        let search_query = self.search_input.trim();
        let matcher = SkimMatcherV2::default();

        let filtered: Vec<Session> = self
            .sessions
            .iter()
            .filter(|s| {
                if s.state == SessionState::Deleted {
                    return false;
                }

                let date_match = if date_query.is_empty() {
                    true
                } else if date_query.contains(" - ") {
                    let parts: Vec<&str> = date_query.split(" - ").collect();
                    if parts.len() == 2 {
                        if let (Ok(start), Ok(end)) = (
                            NaiveDate::parse_from_str(parts[0], "%Y-%m-%d"),
                            NaiveDate::parse_from_str(parts[1], "%Y-%m-%d"),
                        ) {
                            let d = s.start.date_naive();
                            d >= start && d <= end
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else if let Ok(date) = NaiveDate::parse_from_str(date_query, "%Y-%m-%d") {
                    s.start.date_naive() == date
                } else {
                    false
                };

                let search_match = if search_query.is_empty() {
                    true
                } else {
                    let text_to_search = format!("{} {}", s.description, s.tags.join(" "));
                    matcher.fuzzy_match(&text_to_search, search_query).is_some()
                };

                date_match && search_match
            })
            .cloned()
            .collect();

        self.filtered_sessions = filtered;
        if !self.filtered_sessions.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    pub fn save_tags(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(selected_session) = self.filtered_sessions.get_mut(selected_idx) {
                let new_tags: Vec<String> = self
                    .tags_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                selected_session.tags = new_tags.clone();

                if let Some(original_session) = self
                    .sessions
                    .iter_mut()
                    .find(|s| s.start == selected_session.start)
                {
                    original_session.tags = new_tags;
                }

                serialize_session(selected_session, &self.session_dir, selected_session.start)?;
            }
        }
        Ok(())
    }

    pub fn save_notes(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(selected_session) = self.filtered_sessions.get_mut(selected_idx) {
                let new_notes = self.notes_input.clone();
                selected_session.notes = new_notes.clone();

                if let Some(original_session) = self
                    .sessions
                    .iter_mut()
                    .find(|s| s.start == selected_session.start)
                {
                    original_session.notes = new_notes;
                }

                serialize_session(selected_session, &self.session_dir, selected_session.start)?;
            }
        }
        Ok(())
    }

    pub fn save_ratings(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(selected_session) = self.filtered_sessions.get_mut(selected_idx) {
                let ratings = SessionRatings {
                    mental_energy: self.rating_mental,
                    physical_energy: self.rating_physical,
                    cognitive_load: self.rating_cognitive,
                };
                selected_session.ratings = Some(ratings.clone());

                if let Some(original_session) = self
                    .sessions
                    .iter_mut()
                    .find(|s| s.start == selected_session.start)
                {
                    original_session.ratings = Some(ratings);
                }

                serialize_session(selected_session, &self.session_dir, selected_session.start)?;
            }
        }
        Ok(())
    }

    pub fn cancel_session(&mut self) -> Result<(), Box<dyn Error>> {
         if let Some(selected_idx) = self.list_state.selected() {
            if let Some(selected_session) = self.filtered_sessions.get_mut(selected_idx) {
                if selected_session.state == SessionState::Running {
                     selected_session.state = SessionState::Canceled;
                     
                     if let Some(original_session) = self.sessions.iter_mut().find(|s| s.start == selected_session.start) {
                         original_session.state = SessionState::Canceled;
                     }
                     
                     serialize_session(selected_session, &self.session_dir, selected_session.start)?;
                }
            }
         }
          Ok(())
    }

    pub fn delete_session(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(selected_session) = self.filtered_sessions.get(selected_idx) {
                let mut deleted_session = selected_session.clone();
                deleted_session.state = SessionState::Deleted;

                if let Some(original_session) = self.sessions.iter_mut().find(|s| s.start == selected_session.start) {
                    original_session.state = SessionState::Deleted;
                }

                serialize_session(&deleted_session, &self.session_dir, deleted_session.start)?;
                self.filter_sessions();
            }
        }
        Ok(())
    }

    pub fn duplicate_and_start_session(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(selected_session) = self.filtered_sessions.get(selected_idx) {
                let start = Utc::now();
                let new_session = Session {
                    description: selected_session.description.clone(),
                    duration: selected_session.duration,
                    start,
                    tags: selected_session.tags.clone(),
                    notes: selected_session.notes.clone(),
                    state: SessionState::Running,
                    ratings: selected_session.ratings.clone(),
                };

                serialize_session(&new_session, &self.session_dir, start)?;
                self.sessions.push(new_session);
                self.sessions.sort_by(|a, b| b.start.cmp(&a.start));
                self.filter_sessions();
            }
        }
        Ok(())
    }

    pub fn handle_edit_session(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(selected_session) = self.filtered_sessions.get(selected_idx).cloned() {
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
                let temp_path = env::temp_dir().join("polpettone_edit.yaml");
                let yaml_content = serde_yaml::to_string(&selected_session)?;
                fs::write(&temp_path, &yaml_content)?;

                let status = Command::new(editor).arg(&temp_path).status()?;

                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                terminal.clear()?;

                if status.success() {
                    let new_content = fs::read_to_string(&temp_path)?;
                    if let Ok(edited_session) = serde_yaml::from_str::<Session>(&new_content) {
                        if edited_session.start != selected_session.start {
                            let old_filename = format!(
                                "{}-session.yaml",
                                selected_session.start.format("%Y%m%d%H%M%S")
                            );
                            let old_path =
                                std::path::Path::new(&self.session_dir).join(old_filename);
                            if old_path.exists() {
                                let _ = fs::remove_file(old_path);
                            }
                        }

                        if let Some(idx) = self
                            .sessions
                            .iter()
                            .position(|s| s.start == selected_session.start)
                        {
                            self.sessions[idx] = edited_session.clone();
                        }
                        self.sessions.sort_by(|a, b| b.start.cmp(&a.start));

                        serialize_session(
                            &edited_session,
                            &self.session_dir,
                            edited_session.start,
                        )?;

                        self.filter_sessions();
                    }
                }
                let _ = fs::remove_file(&temp_path);
            }
        }
        Ok(())
    }

    pub fn create_session(&mut self) -> Result<(), Box<dyn Error>> {
        let duration_mins: u64 = self.creation_duration.trim().parse().unwrap_or(25);
        let description = self.creation_description.trim().to_string();
        
        let start = Utc::now();
        let session = Session {
            description,
            duration: Duration::from_secs(duration_mins * 60),
            start,
            tags: Vec::new(),
            notes: String::new(),
            state: SessionState::Running,
            ratings: None,
        };
        
        serialize_session(&session, &self.session_dir, start)?;
        
        self.sessions.push(session);
        self.sessions.sort_by(|a, b| b.start.cmp(&a.start));
        
        self.filter_sessions();
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            let mut changed = false;
            for session in self.sessions.iter_mut() {
                if session.state == SessionState::Running && session.remaining_duration().as_secs() == 0 {
                    session.state = SessionState::Done;
                    let _ = serialize_session(session, &self.session_dir, session.start);
                    changed = true;
                }
            }
            if changed {
                self.filter_sessions();
            }

            terminal.draw(|f| ui(f, self))?;

            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if !events::handle_key_event(key, self, &mut terminal)? {
                        break;
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    if app.mode == Mode::Zen {
        let running_session = app.sessions.iter().find(|s| s.state == SessionState::Running);
        zen::render(f, running_session);
        return;
    }

    let constraints = if let Mode::Creation(_) = app.mode {
        vec![
            Constraint::Length(3), 
            Constraint::Length(3), 
            Constraint::Min(0),    
            Constraint::Length(3), 
        ]
    } else if app.mode == Mode::DeleteConfirm {
        vec![
            Constraint::Length(3), 
            Constraint::Length(3), 
            Constraint::Min(0),    
            Constraint::Length(3), 
        ]
    } else if app.mode == Mode::FastFilter {
        vec![
            Constraint::Length(3), 
            Constraint::Min(0),    
            Constraint::Length(3), 
            Constraint::Length(3), 
        ]
    } else {
        vec![
            Constraint::Length(3), 
            Constraint::Min(0),    
            Constraint::Length(3), 
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(f.area());

    let top_chunk = chunks[0];
    let (middle_chunk, main_content_chunk, fast_filter_chunk, keybinds_chunk) = if let Mode::Creation(_) = app.mode {
        (Some(chunks[1]), chunks[2], None, chunks[3])
    } else if app.mode == Mode::DeleteConfirm {
        (Some(chunks[1]), chunks[2], None, chunks[3])
    } else if app.mode == Mode::FastFilter {
        (None, chunks[1], Some(chunks[2]), chunks[3])
    } else {
        (None, chunks[1], None, chunks[2])
    };

    // --- Filter Bar ---
    filter_bar::render(f, top_chunk, app);

    // --- Overlay Bar (Creation or Delete Confirm) ---
    if let Some(m_chunk) = middle_chunk {
        overlay_bar::render(f, m_chunk, app);
    }

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(main_content_chunk);
    
    let list_area = content_chunks[0];
    let right_pane_area = content_chunks[1];

    // --- Session List ---
    session_list::render(f, list_area, app);

    // --- Info Pane (Ratings, Tags, Notes) ---
    info_pane::render(f, right_pane_area, app);
    
    // --- Keybinds & Fast Filter ---
    if let Some(chunk) = fast_filter_chunk {
        f.render_widget(keybinds::render_fast_filter(), chunk);
    }
    f.render_widget(keybinds::render_keybinds(), keybinds_chunk);

    // --- Cursor Handling ---
    if let Some((x, y)) = filter_bar::get_cursor_position(top_chunk, app) {
        f.set_cursor_position((x, y));
    } else if let Some(m_chunk) = middle_chunk {
        if let Some((x, y)) = overlay_bar::get_cursor_position(m_chunk, app) {
            f.set_cursor_position((x, y));
        }
    } else if let Some((x, y)) = info_pane::get_cursor_position(right_pane_area, app) {
        f.set_cursor_position((x, y));
    }
}
