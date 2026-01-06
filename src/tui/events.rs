use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::error::Error;
use std::io::Stdout;
use chrono::{Utc};

use super::app::{App, CreationField, InputField, Mode, RatingField};

pub fn handle_key_event(
    key: KeyEvent,
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<bool, Box<dyn Error>> {
    match &app.mode {
        Mode::Navigation => match key.code {
            KeyCode::Char('q') => return Ok(false),
            KeyCode::Char('i') => app.mode = Mode::Input(InputField::Date),
            KeyCode::Char('/') => app.mode = Mode::Input(InputField::Search),
            KeyCode::Char('j') => app.next(),
            KeyCode::Char('k') => app.previous(),
            KeyCode::Char('t') => {
                if let Some(idx) = app.list_state.selected() {
                    if let Some(session) = app.filtered_sessions.get(idx) {
                        app.tags_input = session.tags.join(", ");
                        app.mode = Mode::Tagging;
                    }
                }
            },
            KeyCode::Char('n') => {
                if let Some(idx) = app.list_state.selected() {
                    if let Some(session) = app.filtered_sessions.get(idx) {
                        app.notes_input = session.notes.clone();
                        app.mode = Mode::Notes;
                    }
                }
            },
            KeyCode::Char('r') => {
                if let Some(idx) = app.list_state.selected() {
                    if let Some(session) = app.filtered_sessions.get(idx) {
                        if let Some(ratings) = &session.ratings {
                            app.rating_mental = ratings.mental_energy;
                            app.rating_physical = ratings.physical_energy;
                            app.rating_cognitive = ratings.cognitive_load;
                        } else {
                            app.rating_mental = 0;
                            app.rating_physical = 0;
                            app.rating_cognitive = 0;
                        }
                        app.mode = Mode::Rating(RatingField::MentalEnergy);
                    }
                }
            }
            KeyCode::Char('e') => app.handle_edit_session(terminal)?,
            KeyCode::Char('a') => {
                app.creation_duration = "25".to_string();
                app.creation_description = if let Some(first) = app.sessions.first() {
                    first.description.clone()
                } else {
                    String::new()
                };
                app.mode = Mode::Creation(CreationField::Description);
            }
            KeyCode::Char('c') => app.cancel_session()?,
            KeyCode::Char('s') => app.duplicate_and_start_session()?,
            KeyCode::Char('x') => {
                if app.list_state.selected().is_some() {
                    app.mode = Mode::DeleteConfirm;
                }
            },
            KeyCode::Char('f') => {
                app.mode = Mode::FastFilter;
            },
            KeyCode::Char('z') => {
                app.mode = Mode::Zen;
            },
            KeyCode::Tab => {
                app.mode = Mode::Input(InputField::Search);
            }
            KeyCode::Char('G') => app.to_bottom(),
            KeyCode::Char('g') => app.mode = Mode::PendingG,
            _ => {}
        },
        Mode::PendingG => match key.code {
            KeyCode::Char('g') => {
                app.to_top();
                app.mode = Mode::Navigation;
            },
            KeyCode::Esc => app.mode = Mode::Navigation,
            _ => app.mode = Mode::Navigation,
        },
        Mode::Input(field) => match key.code {
            KeyCode::Char(c) => {
                match field {
                    InputField::Date => app.date_input.push(c),
                    InputField::Search => app.search_input.push(c),
                }
                app.filter_sessions();
            }
            KeyCode::Backspace => {
                match field {
                    InputField::Date => { app.date_input.pop(); }
                    InputField::Search => { app.search_input.pop(); }
                }
                app.filter_sessions();
            }
            KeyCode::Esc => app.mode = Mode::Navigation,
            KeyCode::Tab => {
                app.mode = match field {
                    InputField::Date => Mode::Input(InputField::Search),
                    InputField::Search => Mode::Input(InputField::Date),
                }
            }
            _ => {}
        },
        Mode::Tagging => match key.code {
            KeyCode::Char(c) => app.tags_input.push(c),
            KeyCode::Backspace => {
                app.tags_input.pop();
            }
            KeyCode::Enter => {
                app.save_tags()?;
                app.mode = Mode::Navigation;
            }
            KeyCode::Esc => app.mode = Mode::Navigation,
            _ => {}
        },
        Mode::Notes => match key.code {
            KeyCode::Char(c) => app.notes_input.push(c),
            KeyCode::Backspace => {
                app.notes_input.pop();
            },
            KeyCode::Enter => {
                app.save_notes()?;
                app.mode = Mode::Navigation;
            },
            KeyCode::Esc => app.mode = Mode::Navigation,
            _ => {}
        },
        Mode::Creation(field) => match key.code {
             KeyCode::Char(c) => match field {
                CreationField::Duration => app.creation_duration.push(c),
                CreationField::Description => app.creation_description.push(c),
            },
            KeyCode::Backspace => match field {
                CreationField::Duration => { app.creation_duration.pop(); }
                CreationField::Description => { app.creation_description.pop(); }
            },
            KeyCode::Tab => {
                app.mode = match field {
                    CreationField::Duration => Mode::Creation(CreationField::Description),
                    CreationField::Description => Mode::Creation(CreationField::Duration),
                }
            },
            KeyCode::Enter => {
                app.create_session()?;
                app.mode = Mode::Navigation;
            },
            KeyCode::Esc => app.mode = Mode::Navigation,
            _ => {}
        },
        Mode::DeleteConfirm => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                app.delete_session()?;
                app.mode = Mode::Navigation;
            },
            KeyCode::Char('n') | KeyCode::Esc => {
                app.mode = Mode::Navigation;
            },
            _ => {}
        },
        Mode::Rating(field) => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.mode = match field {
                    RatingField::MentalEnergy => Mode::Rating(RatingField::PhysicalEnergy),
                    RatingField::PhysicalEnergy => Mode::Rating(RatingField::CognitiveLoad),
                    RatingField::CognitiveLoad => Mode::Rating(RatingField::Motivation),
                    RatingField::Motivation => Mode::Rating(RatingField::MentalEnergy),
                }
            },
            KeyCode::Char('k') | KeyCode::Up => {
                app.mode = match field {
                    RatingField::MentalEnergy => Mode::Rating(RatingField::Motivation),
                    RatingField::PhysicalEnergy => Mode::Rating(RatingField::MentalEnergy),
                    RatingField::CognitiveLoad => Mode::Rating(RatingField::PhysicalEnergy),
                    RatingField::Motivation => Mode::Rating(RatingField::CognitiveLoad),
                }
            },
            KeyCode::Char('l') | KeyCode::Right => {
                match field {
                    RatingField::MentalEnergy => app.rating_mental = (app.rating_mental + 1).min(5),
                    RatingField::PhysicalEnergy => app.rating_physical = (app.rating_physical + 1).min(5),
                    RatingField::CognitiveLoad => app.rating_cognitive = (app.rating_cognitive + 1).min(5),
                    RatingField::Motivation => app.rating_motivation = (app.rating_motivation + 1).min(5),
                }
            },
            KeyCode::Char('h') | KeyCode::Left => {
                match field {
                    RatingField::MentalEnergy => app.rating_mental = app.rating_mental.saturating_sub(1),
                    RatingField::PhysicalEnergy => app.rating_physical = app.rating_physical.saturating_sub(1),
                    RatingField::CognitiveLoad => app.rating_cognitive = app.rating_cognitive.saturating_sub(1),
                    RatingField::Motivation => app.rating_motivation = app.rating_motivation.saturating_sub(1),
                }
            },
            KeyCode::Enter => {
                app.save_ratings()?;
                app.mode = Mode::Navigation;
            },
            KeyCode::Esc => app.mode = Mode::Navigation,
            _ => {}
        },
        Mode::FastFilter => match key.code {
            KeyCode::Char('t') => {
                let today = Utc::now().date_naive();
                app.date_input = today.format("%Y-%m-%d").to_string();
                app.filter_sessions();
                app.mode = Mode::Navigation;
            },
            KeyCode::Char('w') => {
                let today = Utc::now().date_naive();
                let week_ago = today - chrono::Duration::days(7);
                app.date_input = format!("{} - {}", week_ago.format("%Y-%m-%d"), today.format("%Y-%m-%d"));
                app.filter_sessions();
                app.mode = Mode::Navigation;
            },
            KeyCode::Char('c') => {
                app.date_input = String::new();
                app.filter_sessions();
                app.mode = Mode::Navigation;
            },
            KeyCode::Esc => app.mode = Mode::Navigation,
            _ => {}
        },
        Mode::Zen => match key.code {
            KeyCode::Char('z') | KeyCode::Esc => app.mode = Mode::Navigation,
            KeyCode::Char('q') => return Ok(false),
            _ => {}
        }
    }
    Ok(true)
}
