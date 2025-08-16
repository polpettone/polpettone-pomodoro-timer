use eframe::egui;

pub fn show() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 440.0]),
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

struct PomodoroSession {
    name: String,
    minutes: u32,
    difficulty: u8,
}

impl Default for PomodoroSession {
    fn default() -> Self {
        Self {
            name: "Progamming".to_owned(),
            minutes: 25,
            difficulty: 3, // Default to medium
        }
    }
}

impl eframe::App for PomodoroSession {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        // The CentralPanel is the main container that fills the window.
        egui::CentralPanel::default().show(ctx, |ui| {
            // This helper function centers its single child.
            ui.centered_and_justified(|ui| {
                // We create a Frame, which will act as our bordered container.
                // This frame is the single child that will be centered.
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    // Inside the frame, we use a vertical layout to stack our widgets.
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        // By setting a fixed width, we ensure the frame's content
                        // doesn't expand, which makes centering predictable.
                        ui.set_width(280.0);

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

                        ui.add(egui::Slider::new(&mut self.minutes, 0..=60).text("Minutes"));
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

                        if ui
                            .add(egui::Button::new(
                                egui::RichText::new("Start").font(egui::FontId::proportional(30.0)),
                            ))
                            .clicked()
                        {
                            println!(
                                "Started PomodoroSession with difficulty {}",
                                self.difficulty
                            )
                        }
                    });
                });
            });
        });
    }
}
