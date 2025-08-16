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
    Canceled,
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
            state: State::Canceled,
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
            self.draw_session_input(ui);
            self.draw_duration_slider(ui);
            self.draw_difficulty_selector(ui);
            self.draw_action_button(ui, ctx);
        });
    }

    /// Draws the main title heading.
    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("").font(egui::FontId::proportional(40.0)));
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

    /// Draws the "Minutes" slider and handles mouse wheel input.
    fn draw_duration_slider(&mut self, ui: &mut egui::Ui) {
        let slider = egui::Slider::new(&mut self.minutes, 0..=60).text("Minutes");
        let response = ui.add(slider);

        // Allow mouse wheel to control the slider when hovered.
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta);
            if scroll.y > 0.0 && self.minutes < 60 {
                self.minutes += 1;
            } else if scroll.y < 0.0 && self.minutes > 0 {
                self.minutes -= 1;
            }
        }
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
                State::Running => State::Canceled,
                State::Canceled => State::Running,
            };
        }
    }
}
