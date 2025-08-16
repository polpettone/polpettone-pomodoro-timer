use eframe::egui;

// The public function that starts and runs the GUI.
pub fn show() -> eframe::Result {
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
            Ok(Box::<PomodoroSession>::default())
        }),
    )
}

// Represents the possible states of the timer.
#[derive(PartialEq, Debug)]
enum State {
    Running,
    Stopped,
    Paused,
}

// Holds the entire state of our application.
struct PomodoroSession {
    name: String,
    minutes: u32,
    difficulty: u8,
    state: State,
}

// Provides the default initial state for the application.
impl Default for PomodoroSession {
    fn default() -> Self {
        Self {
            name: "Progamming".to_owned(),
            minutes: 25,
            difficulty: 3, // Default to medium
            state: State::Stopped,
        }
    }
}

// Main application implementation block.
impl eframe::App for PomodoroSession {
    /// This is the main update loop, called on every frame.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(2.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            // This layout centers the frame that contains all our widgets.
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    let frame = egui::Frame::group(ui.style())
                        .rounding(10.0)
                        .stroke(egui::Stroke::NONE);

                    frame.show(ui, |ui| {
                        // Call the main drawing function to build the UI.
                        // This keeps the update loop clean and delegates the UI construction.
                        self.draw_main_ui(ui);
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
    fn draw_main_ui(&mut self, ui: &mut egui::Ui) {
        // This vertical layout stacks all our widgets and constrains their width.
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.set_width(280.0);

            self.draw_header(ui);
            self.draw_session_input(ui);
            self.draw_duration_slider(ui);
            self.draw_difficulty_selector(ui);
            self.draw_action_button(ui);
        });
    }

    /// Draws the main title heading.
    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.heading(
            egui::RichText::new("Polpettone Pomodor Timer").font(egui::FontId::proportional(40.0)),
        );
        ui.add_space(25.0);
    }

    /// Draws the "Session" label and text input field.
    fn draw_session_input(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let name_label = ui.label("Session: ");
            ui.text_edit_singleline(&mut self.name)
                .labelled_by(name_label.id);
        });
        ui.add_space(10.0);
    }

    /// Draws the "Minutes" slider.
    fn draw_duration_slider(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.minutes, 0..=60).text("Minutes"));
        ui.add_space(10.0);
    }

    /// Draws the "Difficulty" radio buttons.
    fn draw_difficulty_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Difficulty:");
            ui.radio_value(&mut self.difficulty, 1, "1");
            ui.radio_value(&mut self.difficulty, 2, "2");
            ui.radio_value(&mut self.difficulty, 3, "3");
            ui.radio_value(&mut self.difficulty, 4, "4");
            ui.radio_value(&mut self.difficulty, 5, "5");
        });
        ui.add_space(20.0);
    }

    /// Draws the main action button ("Run", "Stop", etc.) and handles its state-changing logic.
    fn draw_action_button(&mut self, ui: &mut egui::Ui) {
        let button_text_str = match self.state {
            State::Running => "Stop",
            State::Stopped => "Run",
            State::Paused => "Resume",
        };

        let button_text =
            egui::RichText::new(button_text_str).font(egui::FontId::proportional(30.0));

        if ui.add(egui::Button::new(button_text)).clicked() {
            // Toggle the state when the button is clicked.
            self.state = match self.state {
                State::Running => State::Stopped,
                State::Stopped => State::Running,
                State::Paused => State::Running,
            };
        }
    }
}
