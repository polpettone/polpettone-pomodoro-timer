use chrono::{NaiveDate, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{env, error::Error, fs, io, process::Command, time::Duration};

use crate::session::{serialize_session, Session, SessionRatings, SessionState};

const KEYBINDS_TEXT: &str =
    "j/k: up/down | /: search | i: date filter | t: tags | n: notes | r: rate | a: create | e: edit | c: cancel | x: delete | f: fast filter | q: quit | Esc: back";

const FAST_FILTER_TEXT: &str = "t: Today | w: Last Week | c: Clear Filter | Esc: Cancel";

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
}

pub struct App {
    sessions: Vec<Session>,
    pub filtered_sessions: Vec<Session>,
    pub date_input: String,
    pub search_input: String,
    pub tags_input: String,
    pub notes_input: String,
    
    // Creation fields
    pub creation_duration: String,
    pub creation_description: String,

    // Rating fields (temp storage for editing)
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
        
        // Auto-update expired running sessions to Done
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

    fn save_tags(&mut self) -> Result<(), Box<dyn Error>> {
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

    fn save_notes(&mut self) -> Result<(), Box<dyn Error>> {
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

    fn save_ratings(&mut self) -> Result<(), Box<dyn Error>> {
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

    fn cancel_session(&mut self) -> Result<(), Box<dyn Error>> {
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

    fn delete_session(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(selected_session) = self.filtered_sessions.get(selected_idx) {
                // Mark as Deleted
                let mut deleted_session = selected_session.clone();
                deleted_session.state = SessionState::Deleted;

                // Update main list
                if let Some(original_session) = self.sessions.iter_mut().find(|s| s.start == selected_session.start) {
                    original_session.state = SessionState::Deleted;
                }

                serialize_session(&deleted_session, &self.session_dir, deleted_session.start)?;
                self.filter_sessions();
            }
        }
        Ok(())
    }

    fn handle_edit_session(
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

    fn create_session(&mut self) -> Result<(), Box<dyn Error>> {
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
                    match &self.mode {
                        Mode::Navigation => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('i') => self.mode = Mode::Input(InputField::Date),
                            KeyCode::Char('/') => self.mode = Mode::Input(InputField::Search),
                            KeyCode::Char('j') => self.next(),
                            KeyCode::Char('k') => self.previous(),
                            KeyCode::Char('t') => {
                                if let Some(idx) = self.list_state.selected() {
                                    if let Some(session) = self.filtered_sessions.get(idx) {
                                        self.tags_input = session.tags.join(", ");
                                        self.mode = Mode::Tagging;
                                    }
                                }
                            },
                            KeyCode::Char('n') => {
                                if let Some(idx) = self.list_state.selected() {
                                    if let Some(session) = self.filtered_sessions.get(idx) {
                                        self.notes_input = session.notes.clone();
                                        self.mode = Mode::Notes;
                                    }
                                }
                            },
                            KeyCode::Char('r') => {
                                if let Some(idx) = self.list_state.selected() {
                                    if let Some(session) = self.filtered_sessions.get(idx) {
                                        if let Some(ratings) = &session.ratings {
                                            self.rating_mental = ratings.mental_energy;
                                            self.rating_physical = ratings.physical_energy;
                                            self.rating_cognitive = ratings.cognitive_load;
                                        } else {
                                            self.rating_mental = 0;
                                            self.rating_physical = 0;
                                            self.rating_cognitive = 0;
                                        }
                                        self.mode = Mode::Rating(RatingField::MentalEnergy);
                                    }
                                }
                            }
                            KeyCode::Char('e') => self.handle_edit_session(&mut terminal)?,
                            KeyCode::Char('a') => {
                                self.creation_duration = "25".to_string();
                                self.creation_description = if let Some(first) = self.sessions.first() {
                                    first.description.clone()
                                } else {
                                    String::new()
                                };
                                self.mode = Mode::Creation(CreationField::Description);
                            }
                            KeyCode::Char('c') => self.cancel_session()?,
                            KeyCode::Char('x') => {
                                if self.list_state.selected().is_some() {
                                    self.mode = Mode::DeleteConfirm;
                                }
                            },
                            KeyCode::Char('f') => {
                                self.mode = Mode::FastFilter;
                            },
                            KeyCode::Tab => {
                                self.mode = Mode::Input(InputField::Search);
                            }
                            _ => {}
                        },
                        Mode::Input(field) => match key.code {
                            KeyCode::Char(c) => {
                                match field {
                                    InputField::Date => self.date_input.push(c),
                                    InputField::Search => self.search_input.push(c),
                                }
                                self.filter_sessions();
                            }
                            KeyCode::Backspace => {
                                match field {
                                    InputField::Date => { self.date_input.pop(); }
                                    InputField::Search => { self.search_input.pop(); }
                                }
                                self.filter_sessions();
                            }
                            KeyCode::Esc => self.mode = Mode::Navigation,
                            KeyCode::Tab => {
                                self.mode = match field {
                                    InputField::Date => Mode::Input(InputField::Search),
                                    InputField::Search => Mode::Input(InputField::Date),
                                }
                            }
                            _ => {}
                        },
                        Mode::Tagging => match key.code {
                            KeyCode::Char(c) => self.tags_input.push(c),
                            KeyCode::Backspace => {
                                self.tags_input.pop();
                            }
                            KeyCode::Enter => {
                                self.save_tags()?;
                                self.mode = Mode::Navigation;
                            }
                            KeyCode::Esc => self.mode = Mode::Navigation,
                            _ => {}
                        },
                        Mode::Notes => match key.code {
                            KeyCode::Char(c) => self.notes_input.push(c),
                            KeyCode::Backspace => {
                                self.notes_input.pop();
                            },
                            KeyCode::Enter => {
                                self.save_notes()?;
                                self.mode = Mode::Navigation;
                            },
                            KeyCode::Esc => self.mode = Mode::Navigation,
                            _ => {}
                        },
                        Mode::Creation(field) => match key.code {
                             KeyCode::Char(c) => match field {
                                CreationField::Duration => self.creation_duration.push(c),
                                CreationField::Description => self.creation_description.push(c),
                            },
                            KeyCode::Backspace => match field {
                                CreationField::Duration => { self.creation_duration.pop(); }
                                CreationField::Description => { self.creation_description.pop(); }
                            },
                            KeyCode::Tab => {
                                self.mode = match field {
                                    CreationField::Duration => Mode::Creation(CreationField::Description),
                                    CreationField::Description => Mode::Creation(CreationField::Duration),
                                }
                            },
                            KeyCode::Enter => {
                                self.create_session()?;
                                self.mode = Mode::Navigation;
                            },
                            KeyCode::Esc => self.mode = Mode::Navigation,
                            _ => {}
                        },
                        Mode::DeleteConfirm => match key.code {
                            KeyCode::Char('y') | KeyCode::Enter => {
                                self.delete_session()?;
                                self.mode = Mode::Navigation;
                            },
                            KeyCode::Char('n') | KeyCode::Esc => {
                                self.mode = Mode::Navigation;
                            },
                            _ => {}
                        },
                        Mode::Rating(field) => match key.code {
                            KeyCode::Char('j') | KeyCode::Down => {
                                self.mode = match field {
                                    RatingField::MentalEnergy => Mode::Rating(RatingField::PhysicalEnergy),
                                    RatingField::PhysicalEnergy => Mode::Rating(RatingField::CognitiveLoad),
                                    RatingField::CognitiveLoad => Mode::Rating(RatingField::MentalEnergy),
                                }
                            },
                            KeyCode::Char('k') | KeyCode::Up => {
                                self.mode = match field {
                                    RatingField::MentalEnergy => Mode::Rating(RatingField::CognitiveLoad),
                                    RatingField::PhysicalEnergy => Mode::Rating(RatingField::MentalEnergy),
                                    RatingField::CognitiveLoad => Mode::Rating(RatingField::PhysicalEnergy),
                                }
                            },
                            KeyCode::Char('l') | KeyCode::Right => {
                                match field {
                                    RatingField::MentalEnergy => self.rating_mental = (self.rating_mental + 1).min(5),
                                    RatingField::PhysicalEnergy => self.rating_physical = (self.rating_physical + 1).min(5),
                                    RatingField::CognitiveLoad => self.rating_cognitive = (self.rating_cognitive + 1).min(5),
                                }
                            },
                            KeyCode::Char('h') | KeyCode::Left => {
                                match field {
                                    RatingField::MentalEnergy => self.rating_mental = self.rating_mental.saturating_sub(1),
                                    RatingField::PhysicalEnergy => self.rating_physical = self.rating_physical.saturating_sub(1),
                                    RatingField::CognitiveLoad => self.rating_cognitive = self.rating_cognitive.saturating_sub(1),
                                }
                            },
                            KeyCode::Enter => {
                                self.save_ratings()?;
                                self.mode = Mode::Navigation;
                            },
                            KeyCode::Esc => self.mode = Mode::Navigation,
                            _ => {}
                        },
                        Mode::FastFilter => match key.code {
                            KeyCode::Char('t') => {
                                let today = Utc::now().date_naive();
                                self.date_input = today.format("%Y-%m-%d").to_string();
                                self.filter_sessions();
                                self.mode = Mode::Navigation;
                            },
                            KeyCode::Char('w') => {
                                let today = Utc::now().date_naive();
                                let week_ago = today - chrono::Duration::days(7);
                                self.date_input = format!("{} - {}", week_ago.format("%Y-%m-%d"), today.format("%Y-%m-%d"));
                                self.filter_sessions();
                                self.mode = Mode::Navigation;
                            },
                            KeyCode::Char('c') => {
                                self.date_input = String::new();
                                self.filter_sessions();
                                self.mode = Mode::Navigation;
                            },
                            KeyCode::Esc => self.mode = Mode::Navigation,
                            _ => {}
                        }
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

fn keybinds_bar() -> Paragraph<'static> {
    Paragraph::new(KEYBINDS_TEXT)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Keybinds"))
}

fn fast_filter_bar() -> Paragraph<'static> {
    Paragraph::new(FAST_FILTER_TEXT)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Fast Filter"))
}

fn render_stars(val: u8) -> String {
    let mut s = String::new();
    for _ in 0..val {
        s.push('*');
    }
    for _ in val..5 {
        s.push('-');
    }
    s
}

fn ui(f: &mut Frame, app: &mut App) {
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

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50), 
                Constraint::Percentage(50), 
            ]
            .as_ref(),
        )
        .split(top_chunk);
    
    let date_chunk = top_chunks[0];
    let search_chunk = top_chunks[1];

    let date_title = if let Mode::Input(InputField::Date) = app.mode {
        "Date (Active)"
    } else {
        "Date"
    };
    let date_input = Paragraph::new(app.date_input.as_str())
        .block(Block::default().borders(Borders::ALL).title(date_title));
    f.render_widget(date_input, date_chunk);

    let search_title = if let Mode::Input(InputField::Search) = app.mode {
        "Search (Active)"
    } else {
        "Search (/)"
    };
    let search_input = Paragraph::new(app.search_input.as_str())
        .block(Block::default().borders(Borders::ALL).title(search_title));
    f.render_widget(search_input, search_chunk);

    // --- Middle Area ---
    if let Some(m_chunk) = middle_chunk {
        if let Mode::Creation(ref field) = app.mode {
             let creation_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(70),
                        Constraint::Percentage(30),
                    ]
                    .as_ref(),
                )
                .split(m_chunk);

             let desc_title = if let CreationField::Description = field { "Description (Active)" } else { "Description" };
             let duration_title = if let CreationField::Duration = field { "Duration (min) (Active)" } else { "Duration (min)" };
             
             let desc_input = Paragraph::new(app.creation_description.as_str())
                .block(Block::default().borders(Borders::ALL).title(desc_title));
             f.render_widget(desc_input, creation_chunks[0]);

             let duration_input = Paragraph::new(app.creation_duration.as_str())
                .block(Block::default().borders(Borders::ALL).title(duration_title));
             f.render_widget(duration_input, creation_chunks[1]);
        } else if app.mode == Mode::DeleteConfirm {
            let confirm_text = "Are you sure you want to delete this session? (y/n)";
            let confirm_paragraph = Paragraph::new(confirm_text)
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("Delete Confirmation"))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(confirm_paragraph, m_chunk);
        }
    }

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(main_content_chunk);
    
    let list_chunk = content_chunks[0];
    let right_chunk = content_chunks[1];

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5), 
                Constraint::Percentage(40), 
                Constraint::Min(5),    
            ]
            .as_ref()
        )
        .split(right_chunk);
    
    let rating_chunk = right_chunks[0];
    let tags_chunk = right_chunks[1];
    let notes_chunk = right_chunks[2];

    let list_width = list_chunk.width.saturating_sub(5) as usize;
    let items: Vec<ListItem> = app
        .filtered_sessions
        .iter()
        .map(|s| {
            let base_text = s.to_string();
            let status_text = match s.state {
                SessionState::Running => {
                     let remaining = s.remaining_duration();
                     if remaining.as_secs() == 0 {
                         "[Done]".to_string()
                     } else {
                         let mins = remaining.as_secs() / 60;
                         let secs = remaining.as_secs() % 60;
                         format!("[Running: {:02}:{:02}]", mins, secs)
                     }
                },
                SessionState::Done => "[Done]".to_string(),
                SessionState::Canceled => "[Canceled]".to_string(),
                SessionState::Deleted => "[Deleted]".to_string(),
            };

            let content_len = base_text.chars().count() + status_text.chars().count();
            let padding_len = list_width.saturating_sub(content_len);
            let padding = " ".repeat(padding_len);
            
            ListItem::new(format!("{}{}{}", base_text, padding, status_text))
        })
        .collect();
    let list = List::new(items)
        .block(Block::default().title("Sessions").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    
    f.render_stateful_widget(list, list_chunk, &mut app.list_state);
    
    // --- Ratings Pane ---
    let (ratings_mental, ratings_physical, ratings_cognitive) = if let Mode::Rating(_) = app.mode {
        (app.rating_mental, app.rating_physical, app.rating_cognitive)
    } else {
        if let Some(idx) = app.list_state.selected() {
            if let Some(session) = app.filtered_sessions.get(idx) {
                if let Some(r) = &session.ratings {
                    (r.mental_energy, r.physical_energy, r.cognitive_load)
                } else {
                    (0,0,0)
                }
            } else { (0,0,0) }
        } else { (0,0,0) }
    };

    let ratings_title = if let Mode::Rating(_) = app.mode { "Ratings (Active)" } else { "Ratings" };
    let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    
    let format_rating_line = |label: &str, val: u8, is_active: bool| {
        let stars = render_stars(val);
        let content = format!("{:<16} [{}]", label, stars);
        if is_active {
            ratatui::text::Span::styled(content, active_style)
        } else {
            ratatui::text::Span::raw(content)
        }
    };

    let mut lines = Vec::new();
    
    if let Mode::Rating(ref field) = app.mode {
        lines.push(ratatui::text::Line::from(format_rating_line("Mental Energy", ratings_mental, *field == RatingField::MentalEnergy)));
        lines.push(ratatui::text::Line::from(format_rating_line("Physical Energy", ratings_physical, *field == RatingField::PhysicalEnergy)));
        lines.push(ratatui::text::Line::from(format_rating_line("Cognitive Load", ratings_cognitive, *field == RatingField::CognitiveLoad)));
    } else {
        lines.push(ratatui::text::Line::from(format_rating_line("Mental Energy", ratings_mental, false)));
        lines.push(ratatui::text::Line::from(format_rating_line("Physical Energy", ratings_physical, false)));
        lines.push(ratatui::text::Line::from(format_rating_line("Cognitive Load", ratings_cognitive, false)));
    }

    let ratings_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(ratings_title));
    f.render_widget(ratings_widget, rating_chunk);


    // --- Tags Pane ---
    let tags_title = if app.mode == Mode::Tagging {
        "Tags (Active)"
    } else {
        "Tags"
    };

    let tags_text = if app.mode == Mode::Tagging {
        app.tags_input.clone()
    } else {
            if let Some(idx) = app.list_state.selected() {
            if let Some(session) = app.filtered_sessions.get(idx) {
                session.tags.join(", ")
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };

    let tags_widget = Paragraph::new(tags_text)
        .block(Block::default().borders(Borders::ALL).title(tags_title))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(tags_widget, tags_chunk);

    // --- Notes Pane ---
    let notes_title = if app.mode == Mode::Notes {
        "Notes (Active)"
    } else {
        "Notes"
    };

    let notes_text = if app.mode == Mode::Notes {
        app.notes_input.clone()
    } else {
        if let Some(idx) = app.list_state.selected() {
            if let Some(session) = app.filtered_sessions.get(idx) {
                session.notes.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };

    let notes_widget = Paragraph::new(notes_text)
        .block(Block::default().borders(Borders::ALL).title(notes_title))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(notes_widget, notes_chunk);
    
    if let Some(chunk) = fast_filter_chunk {
        f.render_widget(fast_filter_bar(), chunk);
    }
    
    f.render_widget(keybinds_bar(), keybinds_chunk);

    // --- Cursor ---
    match app.mode {
        Mode::Input(InputField::Date) => {
            f.set_cursor_position((
                date_chunk.x + app.date_input.len() as u16 + 1,
                date_chunk.y + 1,
            ));
        }
        Mode::Input(InputField::Search) => {
             f.set_cursor_position((
                search_chunk.x + app.search_input.len() as u16 + 1,
                search_chunk.y + 1,
            ));
        }
        Mode::Tagging => {
             f.set_cursor_position((
                tags_chunk.x + app.tags_input.len() as u16 + 1,
                tags_chunk.y + 1,
            ));
        }
        Mode::Notes => {
             f.set_cursor_position((
                notes_chunk.x + app.notes_input.len() as u16 + 1,
                notes_chunk.y + 1,
            ));
        }
        Mode::Creation(CreationField::Description) => {
            if let Some(m_chunk) = middle_chunk {
                 let creation_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .split(m_chunk);
                 f.set_cursor_position((
                    creation_chunks[0].x + app.creation_description.len() as u16 + 1,
                    creation_chunks[0].y + 1,
                ));
            }
        }
        Mode::Creation(CreationField::Duration) => {
            if let Some(m_chunk) = middle_chunk {
                 let creation_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .split(m_chunk);
                f.set_cursor_position((
                    creation_chunks[1].x + app.creation_duration.len() as u16 + 1,
                    creation_chunks[1].y + 1,
                ));
            }
        }
        _ => {}
    }
}
