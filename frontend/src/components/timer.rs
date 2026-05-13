use leptos::*;
use chrono::Utc;
use crate::models::Session;

#[component]
pub fn Timer(session: Session) -> impl IntoView {
    let (now, set_now) = create_signal(Utc::now());

    // Update die aktuelle Zeit jede Sekunde
    set_interval(
        move || {
            set_now.set(Utc::now());
        },
        std::time::Duration::from_secs(1),
    );

    let remaining = move || {
        // Parse das Startdatum vom Backend
        let start = chrono::NaiveDateTime::parse_from_str(&session.start, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc())
            .unwrap_or_else(|_| Utc::now());
        
        let duration = chrono::Duration::seconds(session.duration.secs as i64);
        let end = start + duration;
        let diff = end - now.get();
        
        if diff.num_seconds() <= 0 {
            "00:00 (Beendet)".to_string()
        } else {
            let minutes = diff.num_minutes();
            let seconds = diff.num_seconds() % 60;
            format!("{:02}:{:02}", minutes, seconds)
        }
    };

    view! {
        <span class="timer-display">{remaining}</span>
    }
}
