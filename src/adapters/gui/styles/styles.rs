use nannou::prelude::*;

// Farben
pub struct Colors {
    pub background: Rgb,
    pub title: Rgb,
    pub timer_background: Rgb,
    pub timer_text: Rgb,
    pub description_text: Rgb,
    pub input_background: Rgb,
    pub input_border: Rgb,
    pub input_text: Rgb,
    pub ready_text: Rgb,
    pub history_title: Rgb,
    pub header_text: Rgb,
    pub session_running: Rgb,
    pub session_done: Rgb,
    pub session_canceled: Rgb,
    pub session_deleted: Rgb,
    pub separator: Rgb,
}

impl Colors {
    pub fn new() -> Self {
        // Farbkonstanten für das Chocolate Theme
        let dark_brown = rgb(0.12, 0.10, 0.08);
        let gold_brown = rgb(0.82, 0.66, 0.48);
        let darker_brown = rgb(0.20, 0.16, 0.12);
        let light_brown = rgb(0.87, 0.72, 0.53);
        let beige = rgb(0.90, 0.85, 0.80);
        let medium_brown = rgb(0.18, 0.14, 0.10);
        let border_brown = rgb(0.50, 0.40, 0.30);
        let gray_brown = rgb(0.60, 0.50, 0.40);
        let accent_brown = rgb(0.70, 0.55, 0.40);
        let deep_brown = rgb(0.40, 0.30, 0.20);
        let separator_brown = rgb(0.30, 0.25, 0.20);

        // Session Status Farben
        let session_running = rgb(0.50, 0.80, 0.50); // Grün
        let session_done = rgb(0.40, 0.60, 0.80); // Blau
        let session_canceled = rgb(0.80, 0.40, 0.40); // Rot

        Colors {
            background: dark_brown,
            title: gold_brown,
            timer_background: darker_brown,
            timer_text: light_brown,
            description_text: beige,
            input_background: medium_brown,
            input_border: border_brown,
            input_text: beige,
            ready_text: gray_brown,
            history_title: accent_brown,
            header_text: gray_brown,
            session_running,
            session_done,
            session_canceled,
            session_deleted: deep_brown,
            separator: separator_brown,
        }
    }
}

// Schriftgrößen
pub struct FontSizes {
    pub title: u32,
    pub timer: u32,
    pub description: u32,
    pub ready_text: u32,
    pub history_title: u32,
    pub header: u32,
    pub session: u32,
    pub input: u32,
}

impl FontSizes {
    pub fn new() -> Self {
        FontSizes {
            title: 40,
            timer: 72,
            description: 32,
            ready_text: 48,
            history_title: 28,
            header: 16,
            session: 24,
            input: 24,
        }
    }
}

// Abstände und Positionen
pub struct Layout {
    pub window_size: (u32, u32),
    pub title_position: (f32, f32),
    pub timer_position: (f32, f32),
    pub description_position: (f32, f32),
    pub input_y: f32,
    pub history_top: f32,
    pub col_time: f32,
    pub col_dur: f32,
    pub col_desc: f32,
    pub col_status: f32,
}

impl Layout {
    pub fn new() -> Self {
        Layout {
            window_size: (1000, 700),
            title_position: (0.0, 360.0), // Header zentriert über dem Timer
            timer_position: (0.0, 150.0),
            description_position: (0.0, 50.0),
            input_y: 50.0,
            history_top: -70.0,
            col_time: -400.0,
            col_dur: -180.0,
            col_desc: -100.0,
            col_status: 350.0,
        }
    }
}

// Hauptstilstruktur
pub struct Style {
    pub colors: Colors,
    pub font_sizes: FontSizes,
    pub layout: Layout,
}

impl Style {
    pub fn new() -> Self {
        Style {
            colors: Colors::new(),
            font_sizes: FontSizes::new(),
            layout: Layout::new(),
        }
    }
}
