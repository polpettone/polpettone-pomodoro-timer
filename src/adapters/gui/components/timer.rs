use crate::adapters::gui::styles::styles::Style;
use crate::domain::session::Session;
use nannou::prelude::*;

pub fn draw_title(draw: &Draw, style: &Style, win: Rect) {
    draw.text("Polpettone Pomodoro")
        .xy(win.top_left() + vec2(style.layout.title_position.0, style.layout.title_position.1))
        .font_size(style.font_sizes.title)
        .color(style.colors.title);
}

pub fn draw_timer(draw: &Draw, style: &Style, session: &Session) {
    let remaining = session.remaining_duration();
    let mins = remaining.as_secs() / 60;
    let secs = remaining.as_secs() % 60;

    // Timer mit Hintergrund
    draw.rect()
        .w_h(300.0, 100.0)
        .color(style.colors.timer_background)
        .xy(vec2(
            style.layout.timer_position.0,
            style.layout.timer_position.1,
        ));
    draw.text(&format!("{:02}:{:02}", mins, secs))
        .font_size(style.font_sizes.timer)
        .xy(vec2(
            style.layout.timer_position.0,
            style.layout.timer_position.1,
        ))
        .color(style.colors.timer_text);

    // Beschreibung
    draw.text(&session.description)
        .font_size(style.font_sizes.description)
        .xy(vec2(
            style.layout.description_position.0,
            style.layout.description_position.1,
        ))
        .color(style.colors.description_text);
}
