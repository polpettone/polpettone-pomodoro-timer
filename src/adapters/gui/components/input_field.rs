use crate::adapters::gui::styles::styles::Style;
use nannou::prelude::*;

pub fn draw_ready_state(draw: &Draw, style: &Style, app: &App, description_input: &str) {
    draw.text("Session")
        .font_size(style.font_sizes.ready_text)
        .xy(vec2(
            style.layout.timer_position.0,
            style.layout.timer_position.1,
        ))
        .color(style.colors.ready_text);

    // Eingabefeld mit Rahmen
    draw.rect()
        .xy(vec2(0.0, style.layout.input_y))
        .w_h(600.0, 50.0)
        .color(style.colors.input_background);
    draw.rect()
        .xy(vec2(0.0, style.layout.input_y))
        .w_h(600.0, 50.0)
        .stroke_weight(2.0)
        .color(style.colors.input_border)
        .stroke_color(style.colors.input_border);

    // Text ohne Cursor
    draw.text(&format!("{}", description_input))
        .font_size(style.font_sizes.input)
        .xy(vec2(0.0, style.layout.input_y))
        .color(style.colors.input_text);

    // Cursor separat zeichnen
    if (app.time * 2.0) as i32 % 2 == 0 {
        let cursor_x = 0.0 + (description_input.len() as f32 * 12.0); // Annäherung für die Cursor-Position
        draw.text("|")
            .font_size(style.font_sizes.input)
            .xy(vec2(cursor_x, style.layout.input_y))
            .color(style.colors.input_text);
    }
}
