use nannou::prelude::*;
use crate::application::service::SessionService;
use crate::domain::repository::SessionRepository;
use crate::domain::session::{Session, SessionState};
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use once_cell::sync::Lazy;
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

pub fn run<R: SessionRepository + Send + 'static>(session_service: SessionService<R>) -> Result<(), Box<dyn Error>> {
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
        KeyPressed(key) => {
            match key {
                Key::Return => {
                    if !model.description_input.is_empty() {
                         let tx_guard = CMD_TX.lock().unwrap();
                         if let Some(tx) = tx_guard.as_ref() {
                             let _ = tx.send(GuiCommand::StartSession(model.description_input.clone(), 25 * 60));
                         }
                         model.description_input.clear();
                    }
                }
                Key::Back => {
                    model.description_input.pop();
                }
                _ => {}
            }
        }
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
    draw.background().color(rgb(0.1, 0.1, 0.12));

    let win = app.window_rect();

    // Title
    draw.text("Polpettone Pomodoro")
        .xy(win.top_left() + vec2(150.0, -40.0))
        .font_size(32)
        .color(TOMATO);

    // Active Session / Timer
    let active_session = model.sessions.iter().find(|s| s.state == SessionState::Running && s.remaining_duration().as_secs() > 0);

    if let Some(s) = active_session {
        let remaining = s.remaining_duration();
        let mins = remaining.as_secs() / 60;
        let secs = remaining.as_secs() % 60;
        
        draw.text(&format!("{:02}:{:02}", mins, secs))
            .font_size(150)
            .y(150.0)
            .color(RED);
            
        draw.text(&s.description)
            .font_size(40)
            .y(250.0)
            .color(WHITE);
    } else {
        draw.text("No active session")
            .font_size(50)
            .y(150.0)
            .color(GRAY);
            
        draw.rect()
            .y(50.0)
            .w_h(500.0, 50.0)
            .color(rgb(0.2, 0.2, 0.23));
            
        draw.text(&format!("Description: {}", model.description_input))
            .font_size(24)
            .y(50.0)
            .color(WHITE);
            
        draw.text("Type and press Enter to start 25min session")
            .font_size(18)
            .y(0.0)
            .color(GRAY);
    }

    // Recent Sessions
    let list_x = win.left() + 250.0;
    let list_y_start = -50.0;

    draw.text("Recent Sessions")
        .x(list_x)
        .y(list_y_start + 40.0)
        .font_size(28)
        .color(CYAN);

    for (i, s) in model.sessions.iter().take(15).enumerate() {
        let status = match s.state {
            SessionState::Running => "Running",
            SessionState::Done => "Done",
            SessionState::Canceled => "Canceled",
            SessionState::Deleted => "Deleted",
        };
        
        let text = format!(
            "{:<16} | {:<2} min | {:<25} | {}",
            s.start.format("%Y-%m-%d %H:%M"),
            s.duration.as_secs() / 60,
            if s.description.len() > 25 { format!("{}...", &s.description[..22]) } else { s.description.clone() },
            status
        );
        
        draw.text(&text)
            .font_size(18)
            .x(list_x + 150.0)
            .y(list_y_start - (i as f32 * 30.0))
            .left_justify()
            .color(if s.state == SessionState::Running { YELLOW } else { WHITE });
    }

    draw.to_frame(app, &frame).unwrap();
}
