use crate::application::service::SessionService;
use crate::domain::repository::SessionRepository;
use crate::domain::session::{Session, SessionState};
use nannou::prelude::*;
use once_cell::sync::Lazy;
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

enum GuiCommand {
    StartSession(String, u64),
}

struct GuiData {
    sessions: Vec<Session>,
}

static CMD_TX: Lazy<Mutex<Option<Sender<GuiCommand>>>> = Lazy::new(|| Mutex::new(None));
static DATA_RX: Lazy<Mutex<Option<Receiver<GuiData>>>> = Lazy::new(|| Mutex::new(None));

struct Model {
    sessions: Vec<Session>,
    description_input: String,
}

pub fn run<R: SessionRepository + Send + 'static>(
    session_service: SessionService<R>,
) -> Result<(), Box<dyn Error>> {
    let (cmd_tx, cmd_rx) = channel();
    let (data_tx, data_rx) = channel();

    {
        let mut tx_guard = CMD_TX.lock().unwrap();
        *tx_guard = Some(cmd_tx);
        let mut rx_guard = DATA_RX.lock().unwrap();
        *rx_guard = Some(data_rx);
    }

    // Service thread
    std::thread::spawn(move || {
        loop {
            // Check for commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    GuiCommand::StartSession(desc, dur) => {
                        let _ = session_service.start_session(&desc, dur);
                    }
                }
            }

            // Send periodic updates
            if let Ok(mut sessions) = session_service.load_sessions() {
                sessions.sort_by(|a, b| b.start.cmp(&a.start));
                let _ = data_tx.send(GuiData { sessions });
            }

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::RefreshSync)
        .run();
    Ok(())
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1000, 700)
        .title("Polpettone Pomodoro")
        .event(event)
        .view(view)
        .build()
        .unwrap();

    Model {
        sessions: Vec::new(),
        description_input: String::new(),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    let rx_guard = DATA_RX.lock().unwrap();
    if let Some(rx) = rx_guard.as_ref() {
        while let Ok(data) = rx.try_recv() {
            model.sessions = data.sessions;
        }
    }
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyPressed(key) => match key {
            Key::Return => {
                if !model.description_input.is_empty() {
                    let tx_guard = CMD_TX.lock().unwrap();
                    if let Some(tx) = tx_guard.as_ref() {
                        let _ = tx.send(GuiCommand::StartSession(
                            model.description_input.clone(),
                            25 * 60,
                        ));
                    }
                    model.description_input.clear();
                }
            }
            Key::Back => {
                model.description_input.pop();
            }
            _ => {}
        },
        ReceivedCharacter(c) => {
            if !c.is_control() && c != '\r' && c != '\n' {
                model.description_input.push(c);
            }
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(rgb(0.1, 0.1, 0.15)); // Dunkler Hintergrund

    let win = app.window_rect();

    // Titel mit Schatteneffekt
    draw.text("Polpettone Pomodoro")
        .xy(win.top_left() + vec2(150.0, -40.0))
        .font_size(48)
        .color(rgb(1.0, 0.5, 0.0)); // Orange

    // Aktive Sitzung oder Timer
    let active_session = model
        .sessions
        .iter()
        .find(|s| s.state == SessionState::Running && s.remaining_duration().as_secs() > 0);

    if let Some(s) = active_session {
        let remaining = s.remaining_duration();
        let mins = remaining.as_secs() / 60;
        let secs = remaining.as_secs() % 60;

        // Timer mit Hintergrund
        draw.rect()
            .w_h(300.0, 100.0)
            .color(rgb(0.2, 0.2, 0.3))
            .xy(vec2(0.0, 150.0));
        draw.text(&format!("{:02}:{:02}", mins, secs))
            .font_size(72)
            .xy(vec2(0.0, 150.0))
            .color(GREEN);

        // Beschreibung
        draw.text(&s.description)
            .font_size(32)
            .xy(vec2(0.0, 50.0))
            .color(WHITE);
    } else {
        draw.text("Ready to Start")
            .font_size(48)
            .xy(vec2(0.0, 150.0))
            .color(GRAY);

        // Eingabefeld mit Rahmen
        let input_y = 50.0;
        draw.rect()
            .xy(vec2(0.0, input_y))
            .w_h(600.0, 50.0)
            .color(rgb(0.15, 0.15, 0.2));
        draw.rect()
            .xy(vec2(0.0, input_y))
            .w_h(600.0, 50.0)
            .stroke_weight(2.0)
            .color(rgb(0.4, 0.4, 0.6))
            .stroke_color(rgb(0.4, 0.4, 0.6));

        let cursor = if (app.time * 2.0) as i32 % 2 == 0 {
            "|"
        } else {
            ""
        };
        draw.text(&format!(
            "Description: {}{}",
            model.description_input, cursor
        ))
        .font_size(24)
        .xy(vec2(0.0, input_y))
        .color(WHITE);

        draw.text("Type and press Enter to start 25min session")
            .font_size(18)
            .xy(vec2(0.0, 0.0))
            .color(GRAY);
    }

    // History Section
    let history_top = -70.0;
    draw.text("Recent Sessions")
        .xy(vec2(0.0, history_top))
        .font_size(28)
        .color(CYAN);

    // Spaltenüberschriften
    let col_time = -400.0;
    let col_dur = -180.0;
    let col_desc = -100.0;
    let col_status = 350.0;

    let header_y = history_top - 40.0;
    draw.text("Start Time")
        .x(col_time)
        .y(header_y)
        .font_size(16)
        .color(GRAY)
        .left_justify();
    draw.text("Dur")
        .x(col_dur)
        .y(header_y)
        .font_size(16)
        .color(GRAY)
        .left_justify();
    draw.text("Description")
        .x(col_desc)
        .y(header_y)
        .font_size(16)
        .color(GRAY)
        .left_justify();
    draw.text("Status")
        .x(col_status)
        .y(header_y)
        .font_size(16)
        .color(GRAY)
        .left_justify();

    // Sitzungsliste
    for (i, s) in model.sessions.iter().take(8).enumerate() {
        let y = header_y - 35.0 - (i as f32 * 30.0);
        let color = match s.state {
            SessionState::Running => GREEN,
            SessionState::Done => BLUE,
            SessionState::Canceled => RED,
            SessionState::Deleted => GRAY,
        };

        let status = match s.state {
            SessionState::Running => "Running",
            SessionState::Done => "Done",
            SessionState::Canceled => "Canceled",
            SessionState::Deleted => "Deleted",
        };

        draw.text(&s.start.format("%Y-%m-%d %H:%M").to_string())
            .x(col_time)
            .y(y)
            .font_size(15)
            .color(color)
            .left_justify();
        draw.text(&format!("{} min", s.duration.as_secs() / 60))
            .x(col_dur)
            .y(y)
            .font_size(15)
            .color(color)
            .left_justify();

        let desc = if s.description.len() > 45 {
            format!("{}...", &s.description[..42])
        } else {
            s.description.clone()
        };
        draw.text(&desc)
            .x(col_desc)
            .y(y)
            .font_size(15)
            .color(color)
            .left_justify();

        draw.text(status)
            .x(col_status)
            .y(y)
            .font_size(15)
            .color(color)
            .left_justify();

        // Trennlinie
        draw.line()
            .start(vec2(-450.0, y - 15.0))
            .end(vec2(450.0, y - 15.0))
            .weight(1.0)
            .color(rgb(0.2, 0.2, 0.25));
    }

    draw.to_frame(app, &frame).unwrap();
}
