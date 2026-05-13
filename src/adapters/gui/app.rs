use crate::application::service::SessionService;
use crate::domain::repository::SessionRepository;
use crate::domain::session::{Session, SessionState};
use nannou::prelude::*;
use once_cell::sync::Lazy;
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use crate::adapters::gui::components::input_field;
use crate::adapters::gui::components::session_list;
use crate::adapters::gui::components::timer;
use crate::adapters::gui::styles::styles::Style;

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
    style: Style,
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
        let handle = tokio::runtime::Handle::current();
        loop {
            // Check for commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    GuiCommand::StartSession(desc, dur) => {
                        let _ = handle.block_on(session_service.start_session(&desc, dur));
                    }
                }
            }

            // Send periodic updates
            if let Ok(mut sessions) = handle.block_on(session_service.load_sessions()) {
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
        .size(
            Style::new().layout.window_size.0,
            Style::new().layout.window_size.1,
        )
        .title("Polpettone Pomodoro")
        .event(event)
        .view(view)
        .build()
        .unwrap();

    Model {
        sessions: Vec::new(),
        description_input: String::new(),
        style: Style::new(),
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
    draw.background().color(model.style.colors.background);

    // Titel zeichnen
    timer::draw_title(&draw, &model.style);

    // Aktive Sitzung oder Timer
    let active_session = model
        .sessions
        .iter()
        .find(|s| s.state == SessionState::Running && s.remaining_duration().as_secs() > 0);

    if let Some(s) = active_session {
        timer::draw_timer(&draw, &model.style, s);
    } else {
        input_field::draw_ready_state(&draw, &model.style, app, &model.description_input);
    }

    // Sitzungsliste
    session_list::draw_history_title(&draw, &model.style);
    session_list::draw_session_headers(&draw, &model.style);
    session_list::draw_session_list(&draw, &model.style, &model.sessions);

    draw.to_frame(app, &frame).unwrap();
}
