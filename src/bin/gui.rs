#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

/*界面结构 */

#[path = "gui/editor.rs"]
mod editor;
#[path = "gui/theme.rs"]
mod theme;

use eframe::egui;
use mini_lisp_rs::{execute_source, output, EnvPtr, EvalEnv, LispError};
use crate::editor::{editor_decorations, highlight_lisp};
use crate::theme::{GuiTheme, apply_theme, default_theme};

fn main() -> eframe::Result<()> {
    let theme = default_theme();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(theme.window_size)
            .with_min_inner_size(theme.min_window_size),
        ..Default::default()
    };

    eframe::run_native(
        "MiniLisp RS GUI",
        options,
        Box::new(move |cc| Ok(Box::new(GuiApp::new(cc, theme)))),
    )
}

struct GuiApp {
    input: String,
    output: String,
    env: EnvPtr,
    theme: GuiTheme,
    input_cursor_index: Option<usize>,
}

impl GuiApp {
    /// Build the app state and apply the shared theme once at startup.
    fn new(cc: &eframe::CreationContext<'_>, theme: GuiTheme) -> Self {
        apply_theme(&cc.egui_ctx, &theme);

        Self {
            input: "(define (square x) (* x x))\n(square 12)".to_string(),
            output: "MiniLisp-RS GUI\nType Lisp code and click Run.\n\n".to_string(),
            env: EvalEnv::new(),
            theme,
            input_cursor_index: None,
        }
    }

    /// Run the current input buffer and append output to the right panel.
    fn run_current_input(&mut self) {
        let source = self.input.trim().to_string();
        if source.is_empty() {
            self.push_output("Please enter some Lisp code.");
            return;
        }

        self.push_output(">>> Running...");
        let env = self.env.clone();
        let (captured, result) = output::capture(|| execute_source(&source, &env, true));

        if !captured.is_empty() {
            self.output.push_str(&captured);
            if !captured.ends_with('\n') {
                self.output.push('\n');
            }
        }

        match result {
            Ok(()) => {}
            Err(LispError::Exit(code)) => {
                self.push_output(&format!("Program requested exit with code {}.", code));
            }
            Err(err) => {
                self.push_output(&format!("Error: {}", err));
            }
        }
    }

    /// Reset the interpreter environment, clearing prior definitions.
    fn reset_env(&mut self) {
        self.env = EvalEnv::new();
        self.push_output("Interpreter environment reset.");
    }

    /// Append one line to the output panel.
    fn push_output(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let run_shortcut = ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl);

        let frame = egui::Frame::new()
            .fill(self.theme.panel_fill)
            .stroke(self.theme.panel_stroke)
            .corner_radius(egui::CornerRadius::same(self.theme.corner_radius))
            .inner_margin(egui::Margin::symmetric(
                self.theme.panel_padding[0] as i8,
                self.theme.panel_padding[1] as i8,
            ));

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading(
                    egui::RichText::new("MiniLisp-RS")
                        .size(self.theme.title_font_size)
                        .color(self.theme.text_color),
                );
                ui.label(
                    egui::RichText::new("Edit the theme in default_theme() to tweak colors, sizes, and spacing.")
                        .size(self.theme.body_font_size)
                        .color(self.theme.muted_text_color),
                );
                ui.separator();

                ui.horizontal(|ui| {
                    if ui
                        .add_sized(self.theme.button_size, egui::Button::new("Run"))
                        .clicked()
                    {
                        self.run_current_input();
                    }

                    if ui
                        .add_sized(self.theme.button_size, egui::Button::new("Clear Output"))
                        .clicked()
                    {
                        self.output.clear();
                    }

                    if ui
                        .add_sized(self.theme.button_size, egui::Button::new("Reset Env"))
                        .clicked()
                    {
                        self.reset_env();
                    }

                    ui.label(
                        egui::RichText::new("Ctrl+Enter runs the current input")
                            .size(self.theme.body_font_size)
                            .color(self.theme.muted_text_color),
                    );
                });

                ui.separator();

                ui.columns(2, |columns| {
                    let theme = self.theme;
                    let decorations = editor_decorations(&self.input, self.input_cursor_index);
                    let mut layouter =
                        move |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                        let mut job = highlight_lisp(text.as_str(), &theme, decorations);
                        job.wrap.max_width = wrap_width;
                        ui.fonts_mut(|fonts| fonts.layout_job(job))
                    };

                    columns[0].heading(
                        egui::RichText::new("Input")
                            .size(self.theme.body_font_size)
                            .color(self.theme.text_color),
                    );
                    let editor_height = columns[0].available_height();
                    let input_rect = egui::Rect::from_min_size(
                        columns[0].cursor().min,
                        egui::vec2(columns[0].available_width(), editor_height),
                    );
                    let input_output = columns[0]
                        .scope_builder(egui::UiBuilder::new().max_rect(input_rect), |ui| {
                            egui::TextEdit::multiline(&mut self.input)
                                .layouter(&mut layouter)
                                .code_editor()
                                .desired_rows(self.theme.editor_rows)
                                .background_color(self.theme.editor_fill)
                                .hint_text("Type Lisp code here")
                                .show(ui)
                        })
                        .inner;
                    self.input_cursor_index =
                        input_output.cursor_range.map(|cursor_range| cursor_range.primary.index.0);

                    columns[1].heading(
                        egui::RichText::new("Output")
                            .size(self.theme.body_font_size)
                            .color(self.theme.text_color),
                    );
                    let output_rect = egui::Rect::from_min_size(
                        columns[1].cursor().min,
                        egui::vec2(columns[1].available_width(), editor_height),
                    );
                    columns[1].scope_builder(egui::UiBuilder::new().max_rect(output_rect), |ui| {
                        egui::TextEdit::multiline(&mut self.output)
                            .code_editor()
                            .desired_rows(self.theme.editor_rows)
                            .background_color(self.theme.editor_fill)
                            .font(egui::FontId::monospace(self.theme.editor_font_size))
                            .interactive(false)
                            .show(ui);
                    });
                });
            });
        });

        if run_shortcut {
            self.run_current_input();
        }
    }
}
