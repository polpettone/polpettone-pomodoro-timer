use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

const KEYBINDS_TEXT: &str =
    "j/k: up/down | /: search | i: date filter | t: tags | n: notes | s: duplicated & start selected | r: rate | a: create | e: edit | c: cancel | x: delete | f: fast filter | z: zen | q: quit | Esc: back";

const FAST_FILTER_TEXT: &str = "t: Today | w: Last Week | c: Clear Filter | Esc: Cancel";

pub fn render_keybinds() -> Paragraph<'static> {
    Paragraph::new(KEYBINDS_TEXT)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Keybinds"))
}

pub fn render_fast_filter() -> Paragraph<'static> {
    Paragraph::new(FAST_FILTER_TEXT)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Fast Filter"))
}
