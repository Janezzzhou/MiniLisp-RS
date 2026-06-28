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
    show_clear_input_dialog: bool,
    show_doc_window: bool,
    special_forms: Vec<(String, String)>,
    builtins: Vec<(String, String)>,
}

impl GuiApp {
    /// Build the app state and apply the shared theme once at startup.
    fn new(cc: &eframe::CreationContext<'_>, theme: GuiTheme) -> Self {
        apply_theme(&cc.egui_ctx, &theme);
        let (special_forms, builtins) = Self::generate_documentation();
        Self {
            input: "".to_string(),
            output: "MiniLisp-RS GUI\nType Lisp code and click Run.\n".to_string(),
            env: EvalEnv::new(),
            theme,
            input_cursor_index: None,
            show_clear_input_dialog: false,
            show_doc_window: false,
            special_forms,
            builtins,
        }
    }

    fn generate_documentation() -> (Vec<(String, String)>, Vec<(String, String)>) {
        let special_forms = vec![
            ("begin".into(), "Evaluate expressions in sequence, return last value."),
            ("cond".into(), "Conditional: each clause is (test expr...), else clause optional."),
            ("define".into(), "Define variable or function: (define name value) or (define (func params) body)."),
            ("lambda".into(), "Create anonymous function: (lambda (params) body)."),
            ("let".into(), "Local bindings: (let ((var expr) ...) body ...)."),
            ("quasiquote".into(), "Quasiquote with unquote splicing."),
            ("quote".into(), "Return expression unevaluated."),
            ("if".into(), "Conditional: (if test then else)."),
            ("and".into(), "Logical AND, short-circuit."),
            ("or".into(), "Logical OR, short-circuit."),
        ]
            .into_iter()
            .map(|(name, desc): (&str, &str)| (name.to_string(), desc.to_string()))
            .collect();
        let builtins = vec![
            ("append", "list ... - Concatenate lists."),
            ("abs", "number - Absolute value."),
            ("apply", "proc list - Apply procedure to list of arguments."),
            ("atom?", "obj - Check if object is atom (not pair)."),
            ("boolean?", "obj - Check if boolean."),
            ("car", "pair - Return first element of pair."),
            ("cdr", "pair - Return rest of pair."),
            ("cons", "obj1 obj2 - Construct pair."),
            ("display", "obj - Print object (string without quotes)."),
            ("displayln", "obj - Print object with newline."),
            ("eq?", "obj1 obj2 - Compare by identity (pointer)."),
            ("equal?", "obj1 obj2 - Structural equality."),
            ("error", "message - Signal error."),
            ("even?", "number - Check if even integer."),
            ("eval", "expr - Evaluate expression in current environment."),
            ("expt", "base exp - Exponentiation."),
            ("exit", "code - Exit interpreter with code."),
            ("filter", "pred list - Return list of elements satisfying pred."),
            ("integer?", "obj - Check if integer."),
            ("length", "list - Return length of list."),
            ("list", "obj ... - Create list."),
            ("list?", "obj - Check if proper list."),
            ("map", "proc list - Apply proc to each element and return list."),
            ("newline", " - Print newline."),
            ("not", "obj - Logical NOT."),
            ("null?", "obj - Check if empty list."),
            ("number?", "obj - Check if number."),
            ("odd?", "number - Check if odd integer."),
            ("pair?", "obj - Check if pair."),
            ("print", "obj ... - Print objects with newline."),
            ("procedure?", "obj - Check if procedure."),
            ("quotient", "dividend divisor - Integer division (truncate toward zero)."),
            ("reduce", "proc list - Reduce list using binary proc."),
            ("remainder", "dividend divisor - Remainder of division."),
            ("string?", "obj - Check if string."),
            ("symbol?", "obj - Check if symbol."),
            ("zero?", "number - Check if zero."),
            ("+", "num ... - Sum numbers."),
            ("-", "num ... - Subtract numbers (unary negate or binary)."),
            ("*", "num ... - Multiply numbers."),
            ("/", "num divisor or numerator denominator - Division."),
            ("=", "num1 num2 - Numeric equality."),
            ("<", "num1 num2 - Less than."),
            ("<=", "num1 num2 - Less or equal."),
            ("modulo", "dividend divisor - Modulo (result has same sign as divisor)."),
            (">", "num1 num2 - Greater than."),
            (">=", "num1 num2 - Greater or equal."),
        ]
            .into_iter()
            .map(|(name, desc): (&str, &str)| (name.to_string(), desc.to_string()))
            .collect();
        (special_forms, builtins)
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

        let editor_frame = egui::Frame::new()
            .stroke(self.theme.panel_stroke)
            .inner_margin(egui::Margin::same(4));

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

                    if ui
                        .add_sized(self.theme.button_size, egui::Button::new("Clear Input"))
                        .clicked()
                    {
                        self.show_clear_input_dialog = true;
                    }

                    if ui
                        .add_sized(self.theme.button_size, egui::Button::new("Reference"))
                        .clicked()
                    {
                        self.show_doc_window = true;
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
                    let input_rect = egui::Rect::from_min_size(
                        columns[0].cursor().min,
                        egui::vec2(columns[0].available_width(), columns[0].available_height()),
                    );
                    let mut input_output = None;
                    columns[0].scope_builder(egui::UiBuilder::new().max_rect(input_rect), |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("input_scroll")
                            .auto_shrink([false; 2])  // 内容少也不收缩，保持填满
                            .show(ui, |ui| {
                                let resp = egui::TextEdit::multiline(&mut self.input)
                                    .layouter(&mut layouter)
                                    .code_editor()
                                    .desired_rows(self.theme.editor_rows)
                                    .background_color(self.theme.editor_fill)
                                    .hint_text(egui::RichText::new("Type Lisp code here").size(18.0))
                                    .frame(editor_frame.clone())
                                    .show(ui);
                                input_output = Some(resp);
                            });
                    });
                    self.input_cursor_index = input_output.and_then(|out| out.cursor_range.map(|cr| cr.primary.index.0));
                    columns[1].heading(
                        egui::RichText::new("Output")
                            .size(self.theme.body_font_size)
                            .color(self.theme.text_color),
                    );
                    let output_rect = egui::Rect::from_min_size(
                        columns[1].cursor().min,
                        egui::vec2(columns[1].available_width(), columns[1].available_height()),
                    );
                    columns[1].scope_builder(egui::UiBuilder::new().max_rect(output_rect), |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("output_scroll")
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.output)
                                        .code_editor()
                                        .background_color(self.theme.editor_fill)
                                        .font(egui::FontId::monospace(self.theme.editor_font_size))
                                        .interactive(false)
                                        .frame(editor_frame.clone())
                                );
                            });
                    });
                });
            });
        });

        if run_shortcut {
            self.run_current_input();
        }

        if self.show_clear_input_dialog {
            egui::Window::new("Confirm Clear Input")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui: &mut egui::Ui| {
                    ui.label("Are you sure you want to clear all input content?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui: &mut egui::Ui| {
                        if ui.button("Yes").clicked() {
                            self.input.clear();
                            self.show_clear_input_dialog = false;
                        }
                        if ui.button("No").clicked() {
                            self.show_clear_input_dialog = false;
                        }
                    });
                });
        }

        egui::Window::new("Built-in Functions and Special Forms")
            .open(&mut self.show_doc_window)
            .default_size([600.0, 500.0])
            .resizable(true)
            .show(ui.ctx(), |ui| {
                //ui.heading("Documentation");
                //ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        // ------ Quick Examples ------
                        ui.label(egui::RichText::new("Quick Examples").size(16.0));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("(define (square x) (* x x))").monospace());
                        ui.label(egui::RichText::new("(square 5)  ; => 25").monospace());
                        ui.add_space(2.0);
                        ui.label(egui::RichText::new("(map square (list 1 2 3 4 5))  ; => (1 4 9 16 25)").monospace());
                        ui.add_space(2.0);
                        ui.label(egui::RichText::new("(let ((x 10) (y 20)) (+ x y))  ; => 30").monospace());
                        ui.add_space(2.0);
                        ui.label(egui::RichText::new("(cond ((> 3 2) \"yes\") (else \"no\"))  ; => \"yes\"").monospace());
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Special Forms").size(16.0));
                        ui.add_space(4.0);
                        for (name, desc) in &self.special_forms {
                            ui.label(format!("{} - {}", name, desc));
                        }
                        ui.add_space(8.0);

                        // 渲染 Builtins
                        ui.label(egui::RichText::new("Built-in Procedures").size(16.0));
                        ui.add_space(4.0);
                        for (name, desc) in &self.builtins {
                            ui.label(format!("{} {}", name, desc));
                        }
                    });
            });
    }
}