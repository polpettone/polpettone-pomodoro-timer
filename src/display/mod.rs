use crate::session::Session;
use chrono::Duration as ChronoDuration;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use std::error::Error;

use crate::date_time::duration_in_minutes;

pub fn print_table(sessions: Vec<Session>) -> Result<(), Box<dyn Error>> {
    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("Description").add_attribute(Attribute::Bold),
            Cell::new("Duration").add_attribute(Attribute::Bold),
            Cell::new("Start Time").add_attribute(Attribute::Bold),
        ])
        .set_content_arrangement(ContentArrangement::Dynamic);

    for session in sessions {
        table.add_row(vec![
            Cell::new(session.description),
            Cell::new(format!("{:?}", duration_in_minutes(session.duration))), // Format duration as needed
            Cell::new(session.start.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]);
    }

    println!("{}", table);

    Ok(())
}

pub fn export_to_ascii_table(sessions: Vec<Session>) -> Result<(), Box<dyn Error>> {
    // Sortiere Sessions nach Startzeit
    let mut sorted_sessions = sessions;
    sorted_sessions.sort_by(|a, b| a.start.cmp(&b.start));

    // Berechne die Gesamtdauer
    let total_duration = sorted_sessions
        .iter()
        .fold(ChronoDuration::zero(), |acc, session| {
            acc + ChronoDuration::from_std(session.duration).unwrap_or(ChronoDuration::zero())
        });

    // Erstelle die Ausgabe
    let mut output = String::new();

    // Header
    output.push_str("|   No   |           Start       |   Dauer   |     Beschreibung   |\n");
    output.push_str("|--------|-----------------------|-----------|--------------------|\n");

    // Sessions
    for (i, session) in sorted_sessions.iter().enumerate() {
        let duration_formatted = format!(
            "{:02}:{:02}",
            session.duration.as_secs() / 60,
            session.duration.as_secs() % 60
        );

        output.push_str(&format!(
            "| {:6} | {:21} | {:9} | {:18} |\n",
            i + 1,
            session.start.format("%Y-%m-%d %H:%M:%S"),
            duration_formatted,
            session.description,
        ));
    }

    let total_minutes = total_duration.num_minutes();
    output.push_str(&format!(
        "| Total  |            --         | {:02}:{:02}     |      --------      |\n",
        total_minutes / 60,
        total_minutes % 60
    ));

    // Ausgabe auf der Konsole
    print!("{}", output);

    Ok(())
}
