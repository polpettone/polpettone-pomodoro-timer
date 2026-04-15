use crate::session::SessionState;
use crate::tui::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    let list_area_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(area);

    let list_items_chunk = list_area_chunks[0];
    let summary_chunk = list_area_chunks[1];

    let list_width = list_items_chunk.width.saturating_sub(5) as usize;
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
                }
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

    f.render_stateful_widget(list, list_items_chunk, &mut app.list_state);

    // --- Summary Bar ---
    let total_count = app.filtered_sessions.len();
    let total_duration: Duration = app.filtered_sessions.iter().map(|s| s.duration).sum();
    let total_mins = total_duration.as_secs() / 60;
    let total_hours = total_mins / 60;
    let remaining_mins = total_mins % 60;

    let summary_text = format!(
        "Count: {} | Total Duration: {:02}:{:02}",
        total_count, total_hours, remaining_mins
    );

    let summary_paragraph = Paragraph::new(summary_text).style(Style::default().fg(Color::Cyan));
    f.render_widget(summary_paragraph, summary_chunk);
}
