use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};

use crate::session::Session;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InputField {
    Date,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Mode {
    Input(InputField),
    Navigation,
}

pub struct App {
    sessions: Vec<Session>,
    pub date_input: String,
    pub mode: Mode,
}

impl App {
    pub fn new(sessions: Vec<Session>) -> App {
        App {
            sessions,
            date_input: String::new(),
            mode: Mode::Navigation,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match &self.mode {
                    Mode::Navigation => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('i') => self.mode = Mode::Input(InputField::Date),
                        _ => {}
                    },
                    Mode::Input(InputField::Date) => match key.code {
                        KeyCode::Char(c) => self.date_input.push(c),
                        KeyCode::Backspace => {
                            self.date_input.pop();
                        }
                        KeyCode::Esc => self.mode = Mode::Navigation,
                        _ => {}
                    },
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

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // Date input
                    Constraint::Min(0),    // Session list
                ]
                .as_ref(),
            )
            .split(f.area());

        let date_chunk = chunks[0];
        let list_chunk = chunks[1];

        // --- Date Input ---
        let date_title = if let Mode::Input(InputField::Date) = self.mode {
            "Date (Active)"
        } else {
            "Date"
        };
        let date_input = Paragraph::new(self.date_input.as_str())
            .block(Block::default().borders(Borders::ALL).title(date_title));
        f.render_widget(date_input, date_chunk);

        // --- Session List ---
        let items: Vec<ListItem> = self
            .sessions
            .iter()
            .map(|s| ListItem::new(s.to_string()))
            .collect();
        let list = List::new(items).block(Block::default().title("Sessions").borders(Borders::ALL));
        f.render_widget(list, list_chunk);

        // --- Cursor ---
        if let Mode::Input(InputField::Date) = self.mode {
            f.set_cursor_position((
                date_chunk.x + self.date_input.len() as u16 + 1,
                date_chunk.y + 1,
            ));
        }
    }
}
