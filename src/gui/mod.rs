use crate::date_time::duration_in_minutes;
use crate::session::SessionService;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::time::Duration;

// The public function that starts and runs the GUI.
pub fn show(session_service: SessionService) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 540.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Polpettone Pomodoro Timer",
        options,
        Box::new(|cc| {
            // Install egui image loaders (good practice).
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // Create the initial state of the app.
            Ok(Box::new(PomodoroSession::new(session_service)))
        }),
    )
}

// Represents the possible states of the timer.
#[derive(PartialEq, Debug)]
enum State {
    Running,
    Canceled,
}

use crate::session::Session;

// Holds the entire state of our application.
struct PomodoroSession {
    name: String,
    minutes: u32,
    difficulty: u8,
    state: State,
    session_service: SessionService,
    show_past_sessions: bool,
    current_session: Option<Session>,
}

// Provides the default initial state for the application.
impl PomodoroSession {
    fn new(session_service: SessionService) -> Self {
        Self {
            name: "Progamming".to_owned(),
            minutes: 25,
            difficulty: 3, // Default to medium
            state: State::Canceled,
            session_service,
            show_past_sessions: true,
            current_session: None,
        }
    }
}

// Main application implementation block.
impl eframe::App for PomodoroSession {
    /// This is the main update loop, called on every frame.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(1000));
        ctx.set_pixels_per_point(2.5);

        if let Ok(mut sessions) = self.session_service.find_all_active_sessions() {
            if let Some(session) = sessions.pop() {
                self.current_session = Some(session);
            } else {
                self.current_session = None;
            }
        }

        if let Some(session) = &self.current_session {
            self.name = session.description.clone();
            self.minutes = (session.duration.as_secs() / 60) as u32;
            self.difficulty = session.difficulty;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::T)) {
            self.show_past_sessions = !self.show_past_sessions;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // This layout centers the frame that contains all our widgets.
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    let frame = egui::Frame::group(ui.style())
                        .corner_radius(10.0)
                        .stroke(egui::Stroke::NONE);

                    frame.show(ui, |ui| {
                        // Pass the context down to the main drawing function
                        // as it's needed for keyboard input handling.
                        self.draw_main_ui(ui, ctx);
                    });
                },
            );
        });
    }
}

// Implementation block for our UI drawing helper functions.
// This separates the UI logic from the main application loop.
impl PomodoroSession {
    /// The primary UI construction function. It calls all the smaller,
    /// dedicated drawing functions in order.
    fn draw_main_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // This vertical layout stacks all our widgets and constrains their width.
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.set_width(280.0);

            self.draw_header(ui);
            self.draw_session_input(ui, ctx);
            self.draw_duration_slider(ui);
            self.draw_difficulty_selector(ui);
            self.draw_action_button(ui, ctx);
            self.draw_session_table(ui);
        });
    }

    /// Draws the main title heading.
    fn draw_header(&self, ui: &mut egui::Ui) {
        match self.session_service.find_all_active_sessions() {
            Ok(sessions) => {
                if let Some(session) = sessions.get(0) {
                    let timer_text = format!(
                        "{}: \n {} - {}",
                        session.description,
                        duration_in_minutes(session.duration),
                        duration_in_minutes(session.elapsed_duration())
                    );
                    ui.heading(
                        egui::RichText::new(timer_text).font(egui::FontId::proportional(30.0)),
                    );
                    ui.add_space(25.0);
                }
            }

            Err(e) => {
                eprint!("Error loading sessions: {}", e)
            }
        }
    }

    /// Draws the "Session" label and text input field.
    fn draw_session_input(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            let name_label = ui.label("Session: ");
            let response = ui
                .text_edit_singleline(&mut self.name)
                .labelled_by(name_label.id);

            if response.changed() {
                if let Some(session) = &mut self.current_session {
                    session.description = self.name.clone();
                    let _ = self.session_service.save_or_update_session(session);
                }
            }

            if ctx.input(|i| i.key_pressed(egui::Key::F)) && !response.has_focus() {
                response.request_focus();
            }
        });
        ui.add_space(10.0);
    }

    /// Draws the "Minutes" slider and handles mouse wheel input.
    fn draw_duration_slider(&mut self, ui: &mut egui::Ui) {
        let slider = egui::Slider::new(&mut self.minutes, 0..=60).text("Minutes");
        let _ = ui.add(slider);

        // Allow mouse wheel to control the slider when hovered.
        let scroll = ui.input(|i| i.raw_scroll_delta);
        if scroll.y > 0.0 && self.minutes < 60 {
            self.minutes += 1;
        } else if scroll.y < 0.0 && self.minutes > 0 {
            self.minutes -= 1;
        }
        ui.add_space(10.0);
    }

    /// Draws the "Difficulty" radio buttons.
    fn draw_difficulty_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Difficulty:");
            if ui.radio_value(&mut self.difficulty, 1, "1").changed() {
                if let Some(session) = &mut self.current_session {
                    session.difficulty = self.difficulty;
                    let _ = self.session_service.save_or_update_session(session);
                }
            }
            if ui.radio_value(&mut self.difficulty, 2, "2").changed() {
                if let Some(session) = &mut self.current_session {
                    session.difficulty = self.difficulty;
                    let _ = self.session_service.save_or_update_session(session);
                }
            }
            if ui.radio_value(&mut self.difficulty, 3, "3").changed() {
                if let Some(session) = &mut self.current_session {
                    session.difficulty = self.difficulty;
                    let _ = self.session_service.save_or_update_session(session);
                }
            }
            if ui.radio_value(&mut self.difficulty, 4, "4").changed() {
                if let Some(session) = &mut self.current_session {
                    session.difficulty = self.difficulty;
                    let _ = self.session_service.save_or_update_session(session);
                }
            }
            if ui.radio_value(&mut self.difficulty, 5, "5").changed() {
                if let Some(session) = &mut self.current_session {
                    session.difficulty = self.difficulty;
                    let _ = self.session_service.save_or_update_session(session);
                }
            }
        });
        ui.add_space(20.0);
    }

    /// Draws the main action button and handles its state-changing logic for clicks and spacebar presses.
    fn draw_action_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let button_text_str = match self.state {
            State::Running => "Cancel",
            State::Canceled => "Run",
        };

        let button_text =
            egui::RichText::new(button_text_str).font(egui::FontId::proportional(30.0));

        let button_response = ui.add(egui::Button::new(button_text));

        // The button can be activated by a click OR by pressing the spacebar,
        // as long as no other widget (like a text field) wants keyboard input.
        if button_response.clicked()
            || (ctx.input(|i| i.key_pressed(egui::Key::Space)) && !ctx.wants_keyboard_input())
        {
            // Toggle the state when the button is activated.
            self.state = match self.state {
                State::Canceled => {
                    if let Ok(session) = self.session_service.create_session(
                        &self.name,
                        (self.minutes * 60) as u64,
                        self.difficulty,
                    ) {
                        self.current_session = Some(session);
                    }
                    State::Running
                }
                State::Running => {
                    if let Some(session) = &mut self.current_session {
                        session.canceled = true;
                        let _ = self.session_service.save_or_update_session(session);
                    }
                    self.current_session = None;
                    State::Canceled
                }
            };
        }
    }

    fn draw_session_table(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        egui::CollapsingHeader::new("Past Sessions")
            .open(Some(self.show_past_sessions))
            .show(ui, |ui| {
                ui.add_space(10.0);

                use chrono::prelude::*;
                let now = Utc::now();
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

                if let Ok(mut sessions) = self
                    .session_service
                    .find_sessions_in_range(start, end, None)
                {
                    sessions.sort_by(|a, b| b.start.cmp(&a.start));

                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .column(Column::auto())
                        .column(Column::auto())
                        .column(Column::auto())
                        .column(Column::remainder());

                    table
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Start Time");
                            });
                            header.col(|ui| {
                                ui.strong("Description");
                            });
                            header.col(|ui| {
                                ui.strong("Duration");
                            });
                            header.col(|ui| {
                                ui.strong("D");
                            });
                        })
                        .body(|mut body| {
                            for session in sessions.iter() {
                                body.row(30.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(
                                            session.start.format("%Y-%m-%d %H:%M").to_string(),
                                        );
                                    });
                                    row.col(|ui| {
                                        ui.label(session.description.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{} min",
                                            session.duration.as_secs() / 60
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", session.difficulty));
                                    });
                                });
                            }
                        });
                }
            });
    }
}
