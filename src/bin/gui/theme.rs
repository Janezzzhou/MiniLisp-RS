use eframe::egui;

/// Shared visual settings for the whole GUI.
/// Change colors, spacing, and font sizes here first.
#[derive(Clone, Copy)]
pub struct GuiTheme {
    pub window_size: [f32; 2],
    pub min_window_size: [f32; 2],
    pub button_size: [f32; 2],
    pub item_spacing: [f32; 2],
    pub panel_padding: [f32; 2],
    pub title_font_size: f32,
    pub body_font_size: f32,
    pub editor_font_size: f32,
    pub editor_rows: usize,
    pub background_fill: egui::Color32,
    pub panel_fill: egui::Color32,
    pub panel_stroke: egui::Stroke,
    pub accent_fill: egui::Color32,
    pub accent_hovered_fill: egui::Color32,
    pub accent_stroke: egui::Stroke,
    pub text_color: egui::Color32,
    pub muted_text_color: egui::Color32,
    pub editor_fill: egui::Color32,
    pub editor_stroke: egui::Stroke,
    pub syntax_comment_color: egui::Color32,
    pub syntax_string_color: egui::Color32,
    pub syntax_number_color: egui::Color32,
    pub syntax_boolean_color: egui::Color32,
    pub syntax_special_form_color: egui::Color32,
    pub syntax_builtin_color: egui::Color32,
    pub syntax_punctuation_color: egui::Color32,
    pub syntax_identifier_color: egui::Color32,
    pub current_line_highlight: egui::Color32,
    pub matched_bracket_fill: egui::Color32,
    pub matched_bracket_stroke: egui::Stroke,
    pub corner_radius: u8,
}

pub fn default_theme() -> GuiTheme {
    GuiTheme {
        window_size: [960.0, 625.0],
        min_window_size: [680.0, 520.0],
        button_size: [120.0, 36.0],
        item_spacing: [10.0, 10.0],
        panel_padding: [14.0, 14.0],
        title_font_size: 22.0,
        body_font_size: 16.0,
        editor_font_size: 16.0,
        editor_rows: 28,
        background_fill: egui::Color32::from_rgb(242, 239, 233),
        panel_fill: egui::Color32::from_rgb(252, 250, 246),
        panel_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(203, 195, 184)),
        accent_fill: egui::Color32::from_rgb(44, 108, 223),
        accent_hovered_fill: egui::Color32::from_rgb(65, 128, 237),
        accent_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(28, 73, 151)),
        text_color: egui::Color32::from_rgb(40, 36, 32),
        muted_text_color: egui::Color32::from_rgb(106, 98, 90),
        editor_fill: egui::Color32::from_rgb(255, 255, 255),
        editor_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(190, 182, 171)),
        syntax_comment_color: egui::Color32::from_rgb(120, 132, 114),
        syntax_string_color: egui::Color32::from_rgb(174, 84, 61),
        syntax_number_color: egui::Color32::from_rgb(79, 101, 204),
        syntax_boolean_color: egui::Color32::from_rgb(134, 64, 173),
        syntax_special_form_color: egui::Color32::from_rgb(183, 96, 24),
        syntax_builtin_color: egui::Color32::from_rgb(32, 136, 118),
        syntax_punctuation_color: egui::Color32::from_rgb(88, 83, 139),
        syntax_identifier_color: egui::Color32::from_rgb(40, 36, 32),
        current_line_highlight: egui::Color32::from_rgba_premultiplied(255, 220, 120, 14),
        matched_bracket_fill: egui::Color32::from_rgba_premultiplied(44, 108, 223, 36),
        matched_bracket_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(44, 108, 223)),
        corner_radius: 10,
    }
}

/// Apply theme values to egui's global style.
pub fn apply_theme(ctx: &egui::Context, theme: &GuiTheme) {
    let active_theme = egui::Theme::Light;
    let mut style = (*ctx.style_of(active_theme)).clone();

    style.spacing.item_spacing = egui::vec2(theme.item_spacing[0], theme.item_spacing[1]);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.visuals.panel_fill = theme.background_fill;
    style.visuals.extreme_bg_color = theme.editor_fill;
    style.visuals.override_text_color = Some(theme.text_color);
    style.visuals.widgets.inactive.bg_fill = theme.accent_fill;
    style.visuals.widgets.hovered.bg_fill = theme.accent_hovered_fill;
    style.visuals.widgets.active.bg_fill = theme.accent_hovered_fill;
    style.visuals.widgets.inactive.bg_stroke = theme.accent_stroke;
    style.visuals.widgets.hovered.bg_stroke = theme.accent_stroke;
    style.visuals.widgets.active.bg_stroke = theme.accent_stroke;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    style.visuals.widgets.noninteractive.bg_fill = theme.panel_fill;
    style.visuals.widgets.noninteractive.bg_stroke = theme.panel_stroke;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, theme.text_color);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(theme.corner_radius);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(theme.corner_radius);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(theme.corner_radius);
    style.visuals.selection.bg_fill = theme.accent_fill;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    style.visuals.text_edit_bg_color = Some(theme.editor_fill);
    style.visuals.window_fill = theme.background_fill;
    style.visuals.window_stroke = theme.panel_stroke;
    style.visuals.widgets.open.bg_stroke = theme.editor_stroke;
    style.visuals.widgets.open.bg_fill = theme.editor_fill;

    ctx.set_theme(active_theme);
    ctx.set_style_of(active_theme, style);
}
