use chrono::NaiveDate;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};

use crate::session::{serialize_session, Session};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InputField {
    Date,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Mode {
    Input(InputField),
    Navigation,
    Tagging,
}

pub struct App {
    sessions: Vec<Session>,
    pub filtered_sessions: Vec<Session>,
    pub date_input: String,
    pub tags_input: String,
    pub mode: Mode,
    pub list_state: ListState,
    pub session_dir: String,
}

impl App {
    pub fn new(sessions: Vec<Session>, session_dir: String) -> App {
        let mut list_state = ListState::default();
        if !sessions.is_empty() {
            list_state.select(Some(0));
        }

        App {
            filtered_sessions: sessions.clone(),
            sessions,
            date_input: String::new(),
            tags_input: String::new(),
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
        let input = self.date_input.trim();
        let filtered: Vec<Session> = if input.is_empty() {
            self.sessions.clone()
        } else {
            // Try range: "YYYY-MM-DD - YYYY-MM-DD"
            if input.contains(" - ") {
                let parts: Vec<&str> = input.split(" - ").collect();
                if parts.len() == 2 {
                    if let (Ok(start), Ok(end)) = (
                        NaiveDate::parse_from_str(parts[0], "%Y-%m-%d"),
                        NaiveDate::parse_from_str(parts[1], "%Y-%m-%d"),
                    ) {
                        self.sessions
                            .iter()
                            .filter(|s| {
                                let d = s.start.date_naive();
                                d >= start && d <= end
                            })
                            .cloned()
                            .collect()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else if let Ok(date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
                // Try single date
                self.sessions
                    .iter()
                    .filter(|s| s.start.date_naive() == date)
                    .cloned()
                    .collect()
            } else {
                vec![]
            }
        };

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
                if let Some(original_session) = self.sessions.iter_mut().find(|s| s.start == selected_session.start) {
                    original_session.tags = new_tags;
                }

                // Persist to disk
                serialize_session(selected_session, &self.session_dir, selected_session.start)?;
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| ui(f, self))?;

            if let Event::Key(key) = event::read()? {
                match &self.mode {
                    Mode::Navigation => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('i') => self.mode = Mode::Input(InputField::Date),
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
                        _ => {}
                    },
                    Mode::Input(InputField::Date) => match key.code {
                        KeyCode::Char(c) => {
                            self.date_input.push(c);
                            self.filter_sessions();
                        }
                        KeyCode::Backspace => {
                            self.date_input.pop();
                            self.filter_sessions();
                        }
                        KeyCode::Esc => self.mode = Mode::Navigation,
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
                    }
                }
            }
        }

        // restore terminal
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Date input
                Constraint::Min(0),    // Main content
            ]
            .as_ref(),
        )
        .split(f.area());

    let date_chunk = chunks[0];
    let main_content_chunk = chunks[1];

    // --- Date Input ---
    let date_title = if let Mode::Input(InputField::Date) = app.mode {
        "Date (Active)"
    } else {
        "Date"
    };
    let date_input = Paragraph::new(app.date_input.as_str())
        .block(Block::default().borders(Borders::ALL).title(date_title));
    f.render_widget(date_input, date_chunk);

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
    let items: Vec<ListItem> = app
        .filtered_sessions
        .iter()
        .map(|s| ListItem::new(s.to_string()))
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

    // --- Cursor ---
    if let Mode::Input(InputField::Date) = app.mode {
        f.set_cursor_position((
            date_chunk.x + app.date_input.len() as u16 + 1,
            date_chunk.y + 1,
        ));
    } else if app.mode == Mode::Tagging {
         f.set_cursor_position((
            tags_chunk.x + app.tags_input.len() as u16 + 1,
            tags_chunk.y + 1,
        ));
    }
}