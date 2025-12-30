use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::{App, InputField, Mode};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50), 
                Constraint::Percentage(50), 
            ]
            .as_ref(),
        )
        .split(area);
    
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
}

pub fn get_cursor_position(area: Rect, app: &App) -> Option<(u16, u16)> {
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    match app.mode {
        Mode::Input(InputField::Date) => Some((
            top_chunks[0].x + app.date_input.len() as u16 + 1,
            top_chunks[0].y + 1,
        )),
        Mode::Input(InputField::Search) => Some((
            top_chunks[1].x + app.search_input.len() as u16 + 1,
            top_chunks[1].y + 1,
        )),
        _ => None,
    }
}
