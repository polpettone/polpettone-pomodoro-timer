use eframe::egui;

pub fn show() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 340.0]),
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
}

impl Default for PomodoroSession {
    fn default() -> Self {
        Self {
            name: "Progamming".to_owned(),
            minutes: 25,
        }
    }
}

impl eframe::App for PomodoroSession {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            // This special layout centers a single item vertically and horizontally.
            ui.centered_and_justified(|ui| {
                // The "single item" is a vertical layout containing all our widgets.
                // By setting a max width on this inner ui, we prevent greedy widgets
                // like Sliders from expanding to fill the whole window. The centered_and_justified
                // layout then correctly centers our constrained-width vertical layout.
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.set_max_width(280.0); // Set a max width for the content block

                    ui.heading(
                        egui::RichText::new("Polpettone Pomodor Timer")
                            .font(egui::FontId::proportional(30.0)),
                    );
                    ui.add_space(25.0);

                    ui.horizontal(|ui| {
                        let name_label = ui.label("Session: ");
                        ui.text_edit_singleline(&mut self.name)
                            .labelled_by(name_label.id);
                    });
                    ui.add_space(10.0);

                    ui.add(egui::Slider::new(&mut self.minutes, 1..=60).text("Minutes"));
                    ui.add_space(120.0);

                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("Start").font(egui::FontId::proportional(30.0)),
                        ))
                        .clicked()
                    {
                        println!("Started PomodoroSession")
                    }

                    // The status label is commented out as it affects the centering layout.
                    // It would be better placed in a different part of the UI, like a status bar.
                    /*
                    ui.label(format!(
                        "Started Session: '{}', with {} minutes",
                        self.name, self.minutes
                    ));
                    */
                });
            });
        });
    }
}
