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

use crate::session::{serialize_session, Session, SessionState};

const KEYBINDS_TEXT: &str =
    "j/k: up/down | /: search | i: date filter | t: tags | a: create | e: edit | c: cancel | q: quit | Esc: back";

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
pub enum Mode {
    Input(InputField),
    Navigation,
    Tagging,
    Creation(CreationField),
}

pub struct App {
    sessions: Vec<Session>,
    pub filtered_sessions: Vec<Session>,
    pub date_input: String,
    pub search_input: String,
    pub tags_input: String,
    
    // Creation fields
    pub creation_duration: String,
    pub creation_description: String,

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
                    // We should save this change.
                    // Ignoring error here for simplicity in constructor, or log it.
                    let _ = serialize_session(session, &session_dir, session.start);
                }
            }
        }

        let mut list_state = ListState::default();
        if !sessions.is_empty() {
            list_state.select(Some(0));
        }

        App {
            filtered_sessions: sessions.clone(),
            sessions,
            date_input: String::new(),
            search_input: String::new(),
            tags_input: String::new(),
            creation_duration: String::new(),
            creation_description: String::new(),
            mode: Mode::Navigation,
            list_state,
            session_dir,
        }
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
                // Date Filter
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

                // Search Filter (Fuzzy)
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
                // Parse tags
                let new_tags: Vec<String> = self
                    .tags_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                selected_session.tags = new_tags.clone();

                // Update in main session list
                if let Some(original_session) = self
                    .sessions
                    .iter_mut()
                    .find(|s| s.start == selected_session.start)
                {
                    original_session.tags = new_tags;
                }

                // Persist to disk
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
                     
                     // Update in main list
                     if let Some(original_session) = self.sessions.iter_mut().find(|s| s.start == selected_session.start) {
                         original_session.state = SessionState::Canceled;
                     }
                     
                     serialize_session(selected_session, &self.session_dir, selected_session.start)?;
                }
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
            state: SessionState::Running,
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
            // Check for expired running sessions to auto-update UI to "Done"
            // We check this every loop to keep UI fresh
            // Only update if something changed to avoid flicker or performance hit?
            // Actually, calculating remaining time is cheap.
            // But we should persist the "Done" state if it just transitioned.
            let mut changed = false;
            for session in self.sessions.iter_mut() {
                if session.state == SessionState::Running && session.remaining_duration().as_secs() == 0 {
                    session.state = SessionState::Done;
                    let _ = serialize_session(session, &self.session_dir, session.start);
                    changed = true;
                }
            }
            if changed {
                self.filter_sessions(); // Refresh filtered list
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
                            }
                            KeyCode::Char('e') => self.handle_edit_session(&mut terminal)?,
                            KeyCode::Char('a') => {
                                self.creation_duration = "25".to_string();
                                self.creation_description = if let Some(first) = self.sessions.first() {
                                    first.description.clone()
                                } else {
                                    String::new()
                                };
                                self.mode = Mode::Creation(CreationField::Duration);
                            }
                            KeyCode::Char('c') => self.cancel_session()?,
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

fn ui(f: &mut Frame, app: &mut App) {
    // Define constraints based on mode
    let constraints = if let Mode::Creation(_) = app.mode {
        vec![
            Constraint::Length(3), // Top Inputs (Date + Search)
            Constraint::Length(3), // Creation Inputs (Duration + Description)
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Keybinds
        ]
    } else {
        vec![
            Constraint::Length(3), // Top Inputs (Date + Search)
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Keybinds
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(f.area());

    // Map chunks
    let top_chunk = chunks[0];
    let (creation_chunk, main_content_chunk, keybinds_chunk) = if let Mode::Creation(_) = app.mode {
        (Some(chunks[1]), chunks[2], chunks[3])
    } else {
        (None, chunks[1], chunks[2])
    };

    // --- Top Inputs Split (Date/Search) ---
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

    // --- Date Input ---
    let date_title = if let Mode::Input(InputField::Date) = app.mode {
        "Date (Active)"
    } else {
        "Date"
    };
    let date_input = Paragraph::new(app.date_input.as_str())
        .block(Block::default().borders(Borders::ALL).title(date_title));
    f.render_widget(date_input, date_chunk);

    // --- Search Input ---
    let search_title = if let Mode::Input(InputField::Search) = app.mode {
        "Search (Active)"
    } else {
        "Search (/)"
    };
    let search_input = Paragraph::new(app.search_input.as_str())
        .block(Block::default().borders(Borders::ALL).title(search_title));
    f.render_widget(search_input, search_chunk);


    // --- Creation Inputs (If Active) ---
    if let Some(c_chunk) = creation_chunk {
        if let Mode::Creation(ref field) = app.mode {
             let creation_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(50), 
                        Constraint::Percentage(50), 
                    ]
                    .as_ref(),
                )
                .split(c_chunk);

             let duration_title = if let CreationField::Duration = field { "Duration (min) (Active)" } else { "Duration (min)" };
             let desc_title = if let CreationField::Description = field { "Description (Active)" } else { "Description" };
             
             let duration_input = Paragraph::new(app.creation_duration.as_str())
                .block(Block::default().borders(Borders::ALL).title(duration_title));
             f.render_widget(duration_input, creation_chunks[0]);
             
             let desc_input = Paragraph::new(app.creation_description.as_str())
                .block(Block::default().borders(Borders::ALL).title(desc_title));
             f.render_widget(desc_input, creation_chunks[1]);
        }
    }


    // --- Split Main Content (List + Tags) ---
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
    let tags_chunk = content_chunks[1];

    // --- Session List ---
    let list_width = list_chunk.width.saturating_sub(5) as usize;
    let items: Vec<ListItem> = app
        .filtered_sessions
        .iter()
        .map(|s| {
            let base_text = s.to_string();
            // Determine Right-aligned status text
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
    
    // --- Keybinds ---
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
        Mode::Creation(CreationField::Duration) => {
            if let Some(c_chunk) = creation_chunk {
                 let creation_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(c_chunk);
                 f.set_cursor_position((
                    creation_chunks[0].x + app.creation_duration.len() as u16 + 1,
                    creation_chunks[0].y + 1,
                ));
            }
        }
        Mode::Creation(CreationField::Description) => {
            if let Some(c_chunk) = creation_chunk {
                 let creation_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(c_chunk);
                f.set_cursor_position((
                    creation_chunks[1].x + app.creation_description.len() as u16 + 1,
                    creation_chunks[1].y + 1,
                ));
            }
        }
        _ => {}
    }
}