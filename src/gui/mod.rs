use eframe::egui;

pub fn show() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Polpettone Pomodor Timer");
            ui.horizontal(|ui| {
                let name_label = ui.label("Session: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.minutes, 0..=60).text("Minutes"));

            if ui.button("Start").clicked() {
                println!("Started PomodoroSession")
            }

            ui.label(format!("Hello '{}', age {}", self.name, self.minutes));
        });
    }
}
