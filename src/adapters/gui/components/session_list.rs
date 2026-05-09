use crate::adapters::gui::styles::styles::Style;
use crate::domain::session::{Session, SessionState};
use nannou::prelude::*;

pub fn draw_history_title(draw: &Draw, style: &Style) {
    draw.text("Recent Sessions")
        .xy(vec2(0.0, style.layout.history_top))
        .font_size(style.font_sizes.history_title)
        .color(style.colors.history_title);
}

pub fn draw_session_headers(draw: &Draw, style: &Style) {
    let header_y = style.layout.history_top - 40.0;
    draw.text("Start Time")
        .x(style.layout.col_time)
        .y(header_y)
        .font_size(style.font_sizes.header)
        .color(style.colors.header_text)
        .left_justify();
    draw.text("Dur")
        .x(style.layout.col_dur)
        .y(header_y)
        .font_size(style.font_sizes.header)
        .color(style.colors.header_text)
        .left_justify();
    draw.text("Description")
        .x(style.layout.col_desc)
        .y(header_y)
        .font_size(style.font_sizes.header)
        .color(style.colors.header_text)
        .left_justify();
    draw.text("Status")
        .x(style.layout.col_status)
        .y(header_y)
        .font_size(style.font_sizes.header)
        .color(style.colors.header_text)
        .left_justify();
}

pub fn draw_session_list(draw: &Draw, style: &Style, sessions: &[Session]) {
    let header_y = style.layout.history_top - 40.0;
    for (i, s) in sessions.iter().take(8).enumerate() {
        let y = header_y - 35.0 - (i as f32 * 30.0);
        let color = match s.state {
            SessionState::Running => style.colors.session_running,
            SessionState::Done => style.colors.session_done,
            SessionState::Canceled => style.colors.session_canceled,
            SessionState::Deleted => style.colors.session_deleted,
        };

        let status = match s.state {
            SessionState::Running => "Running",
            SessionState::Done => "Done",
            SessionState::Canceled => "Canceled",
            SessionState::Deleted => "Deleted",
        };

        draw.text(&s.start.format("%Y-%m-%d %H:%M").to_string())
            .x(style.layout.col_time)
            .y(y)
            .font_size(style.font_sizes.session)
            .color(color)
            .left_justify();
        draw.text(&format!("{} min", s.duration.as_secs() / 60))
            .x(style.layout.col_dur)
            .y(y)
            .font_size(style.font_sizes.session)
            .color(color)
            .left_justify();

        let desc = if s.description.len() > 45 {
            format!("{}...", &s.description[..42])
        } else {
            s.description.clone()
        };
        draw.text(&desc)
            .x(style.layout.col_desc)
            .y(y)
            .font_size(style.font_sizes.session)
            .color(color)
            .left_justify();

        draw.text(status)
            .x(style.layout.col_status)
            .y(y)
            .font_size(style.font_sizes.session)
            .color(color)
            .left_justify();

        // Trennlinie
        draw.line()
            .start(vec2(-450.0, y - 15.0))
            .end(vec2(450.0, y - 15.0))
            .weight(1.0)
            .color(style.colors.separator);
    }
}
