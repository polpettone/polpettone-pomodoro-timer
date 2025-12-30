use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::{App, Mode, CreationField};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
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
            .split(area);

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
        f.render_widget(confirm_paragraph, area);
    }
}

pub fn get_cursor_position(area: Rect, app: &App) -> Option<(u16, u16)> {
    let creation_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(area);

    match app.mode {
        Mode::Creation(CreationField::Description) => Some((
            creation_chunks[0].x + app.creation_description.len() as u16 + 1,
            creation_chunks[0].y + 1,
        )),
        Mode::Creation(CreationField::Duration) => Some((
            creation_chunks[1].x + app.creation_duration.len() as u16 + 1,
            creation_chunks[1].y + 1,
        )),
        _ => None,
    }
}
