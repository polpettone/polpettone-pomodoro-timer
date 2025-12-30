use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};

use crate::session::Session;

pub fn render(f: &mut Frame, session: Option<&Session>) {
    let area = f.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(area);

    if let Some(s) = session {
        let remaining = s.remaining_duration();
        let mins = remaining.as_secs() / 60;
        let secs = remaining.as_secs() % 60;
        let time_str = format!("{:02}:{:02}", mins, secs);

        let big_text_lines = to_big_text(&time_str);

        let mut lines = Vec::new();

        // Description (Above)
        lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
            s.description.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        // Spacing
        lines.push(ratatui::text::Line::from(""));
        lines.push(ratatui::text::Line::from(""));

        // Time (Below)
        for l in big_text_lines {
            lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                l,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let p = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);

        // Center vertically in the middle chunk
        f.render_widget(p, vertical[1]);
    } else {
        let p = Paragraph::new("No active session")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::Red));
        f.render_widget(p, vertical[1]);
    };
}

// 5x7 block font for 0-9 and :
fn to_big_text(s: &str) -> Vec<String> {
    let mut lines = vec![String::new(); 7];

    for c in s.chars() {
        let art = match c {
            '0' => vec![
                " ##### ", "#     #", "#     #", "#     #", "#     #", "#     #", " ##### ",
            ],
            '1' => vec![
                "   #   ", "  ##   ", "   #   ", "   #   ", "   #   ", "   #   ", " ##### ",
            ],
            '2' => vec![
                " ##### ", "#     #", "      #", " ##### ", "#      ", "#      ", "#######",
            ],
            '3' => vec![
                " ##### ", "#     #", "      #", " ##### ", "      #", "#     #", " ##### ",
            ],
            '4' => vec![
                "#     #", "#     #", "#     #", "#######", "      #", "      #", "      #",
            ],
            '5' => vec![
                "#######", "#      ", "#      ", "#####  ", "     # ", "#    # ", " ####  ",
            ],
            '6' => vec![
                " ##### ", "#     #", "#      ", "###### ", "#     #", "#     #", " ##### ",
            ],
            '7' => vec![
                "#######", "#    # ", "    #  ", "   #   ", "  #    ", "  #    ", "  #    ",
            ],
            '8' => vec![
                " ##### ", "#     #", "#     #", " ##### ", "#     #", "#     #", " ##### ",
            ],
            '9' => vec![
                " ##### ", "#     #", "#     #", " ######", "      #", "#     #", " ##### ",
            ],
            ':' => vec![
                "       ", "   #   ", "   #   ", "       ", "   #   ", "   #   ", "       ",
            ],
            _ => vec![
                "       ", "       ", "       ", "       ", "       ", "       ", "       ",
            ],
        };

        for i in 0..7 {
            lines[i].push_str(art[i]);
            lines[i].push_str(" "); // Space between digits
        }
    }
    lines
}
