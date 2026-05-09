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
        Colors {
            background: rgb(0.1, 0.1, 0.15),
            title: rgb(1.0, 0.5, 0.0),
            timer_background: rgb(0.2, 0.2, 0.3),
            timer_text: rgb(0.0, 1.0, 0.0),
            description_text: rgb(1.0, 1.0, 1.0),
            input_background: rgb(0.15, 0.15, 0.2),
            input_border: rgb(0.4, 0.4, 0.6),
            input_text: rgb(1.0, 1.0, 1.0),
            ready_text: rgb(0.5, 0.5, 0.5),
            history_title: rgb(0.0, 1.0, 1.0),
            header_text: rgb(0.5, 0.5, 0.5),
            session_running: rgb(0.0, 1.0, 0.0),
            session_done: rgb(0.0, 0.0, 1.0),
            session_canceled: rgb(1.0, 0.0, 0.0),
            session_deleted: rgb(0.5, 0.5, 0.5),
            separator: rgb(0.2, 0.2, 0.25),
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
    pub hint: u32,
}

impl FontSizes {
    pub fn new() -> Self {
        FontSizes {
            title: 48,
            timer: 72,
            description: 32,
            ready_text: 48,
            history_title: 28,
            header: 16,
            session: 24,
            input: 24,
            hint: 18,
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
            title_position: (150.0, -40.0),
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
