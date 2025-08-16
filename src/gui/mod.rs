use eframe::egui;

pub fn show() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 540.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Polpettone Pomodoro Timer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<PomodoroSession>::default())
        }),
    )
}

#[derive(PartialEq, Debug)]
enum State {
    Running,
    Canceled,
}

struct PomodoroSession {
    name: String,
    minutes: u32,
    difficulty: u8,
    state: State,
}

impl Default for PomodoroSession {
    fn default() -> Self {
        Self {
            name: "Progamming".to_owned(),
            minutes: 25,
            difficulty: 3, // Default to medium
            state: State::Canceled,
        }
    }
}

impl eframe::App for PomodoroSession {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(2.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            // Use a layout that centers its content both horizontally and vertically.
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    // This frame is the single child that will be centered.
                    egui::Frame::group(ui.style())
                        .stroke(egui::Stroke::NONE)
                        .show(ui, |ui| {
                            // Inside the frame, use a vertical layout to stack widgets.
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.set_width(280.0); // Constrain width for predictable centering

                                ui.heading(
                                    egui::RichText::new("Polpettone Pomodor Timer")
                                        .font(egui::FontId::proportional(40.0)),
                                );
                                ui.add_space(25.0);

                                ui.horizontal(|ui| {
                                    let name_label = ui.label("Session: ");
                                    ui.text_edit_singleline(&mut self.name)
                                        .labelled_by(name_label.id);
                                });
                                ui.add_space(10.0);

                                ui.add(
                                    egui::Slider::new(&mut self.minutes, 0..=60).text("Minutes"),
                                );
                                ui.add_space(10.0);

                                ui.horizontal(|ui| {
                                    ui.label("Difficulty:");
                                    ui.radio_value(&mut self.difficulty, 1, "1");
                                    ui.radio_value(&mut self.difficulty, 2, "2");
                                    ui.radio_value(&mut self.difficulty, 3, "3");
                                    ui.radio_value(&mut self.difficulty, 4, "4");
                                    ui.radio_value(&mut self.difficulty, 5, "5");
                                });
                                ui.add_space(20.0);

                                // Determine button text based on the current state
                                let button_text_str = match self.state {
                                    State::Running => "Cancel",
                                    State::Canceled => "Run",
                                };

                                let button_text = egui::RichText::new(button_text_str)
                                    .font(egui::FontId::proportional(30.0));

                                // Add the button and handle the click event
                                if ui.add(egui::Button::new(button_text)).clicked() {
                                    // Toggle state on click
                                    self.state = match self.state {
                                        State::Running => State::Canceled,
                                        State::Canceled => State::Running,
                                    };
                                }
                            }); // End of inner vertical layout
                        }); // End of frame
                },
            ); // End of centering layout
        }); // End of CentralPanel
    } // End of update method
} // End of impl eframe::App
