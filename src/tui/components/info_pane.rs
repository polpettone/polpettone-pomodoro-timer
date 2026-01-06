use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::{App, Mode, RatingField};
use crate::tui::components::ratings;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(6), // Increased for extra rating
                Constraint::Percentage(40), 
                Constraint::Min(5),    
            ]
            .as_ref()
        )
        .split(area);
    
    let rating_chunk = right_chunks[0];
    let tags_chunk = right_chunks[1];
    let notes_chunk = right_chunks[2];

    // --- Ratings Pane ---
    let (ratings_mental, ratings_physical, ratings_cognitive, ratings_motivation) = if let Mode::Rating(_) = app.mode {
        (app.rating_mental, app.rating_physical, app.rating_cognitive, app.rating_motivation)
    } else {
        if let Some(idx) = app.list_state.selected() {
            if let Some(session) = app.filtered_sessions.get(idx) {
                if let Some(r) = &session.ratings {
                    (r.mental_energy, r.physical_energy, r.cognitive_load, r.motivation)
                } else {
                    (0,0,0,0)
                }
            } else { (0,0,0,0) }
        } else { (0,0,0,0) }
    };

    let ratings_title = if let Mode::Rating(_) = app.mode { "Ratings (Active)" } else { "Ratings" };
    let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    
    let format_rating_line = |label: &str, val: u8, is_active: bool| {
        let stars = ratings::render_stars(val);
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
        lines.push(ratatui::text::Line::from(format_rating_line("Motivation", ratings_motivation, *field == RatingField::Motivation)));
    } else {
        lines.push(ratatui::text::Line::from(format_rating_line("Mental Energy", ratings_mental, false)));
        lines.push(ratatui::text::Line::from(format_rating_line("Physical Energy", ratings_physical, false)));
        lines.push(ratatui::text::Line::from(format_rating_line("Cognitive Load", ratings_cognitive, false)));
        lines.push(ratatui::text::Line::from(format_rating_line("Motivation", ratings_motivation, false)));
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
}

pub fn get_cursor_position(area: Rect, app: &App) -> Option<(u16, u16)> {
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Percentage(40), Constraint::Min(5)].as_ref())
        .split(area);
    
    match app.mode {
        Mode::Tagging => Some((
            right_chunks[1].x + app.tags_input.len() as u16 + 1,
            right_chunks[1].y + 1,
        )),
        Mode::Notes => Some((
            right_chunks[2].x + app.notes_input.len() as u16 + 1,
            right_chunks[2].y + 1,
        )),
        _ => None,
    }
}