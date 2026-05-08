use eframe::egui;

#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn toggled(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }

    pub fn other_label(self) -> &'static str {
        match self {
            Theme::Light => "Dark",
            Theme::Dark => "Light",
        }
    }
}

pub struct ThemeColors {
    pub bg: egui::Color32,
    pub surface: egui::Color32,
    pub surface_alt: egui::Color32,
    pub border: egui::Color32,
    pub text: egui::Color32,
    pub muted: egui::Color32,
    pub accent: egui::Color32,
    pub accent_soft: egui::Color32,
    pub danger: egui::Color32,
    pub success: egui::Color32,
}

impl ThemeColors {
    pub fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self {
                bg: egui::Color32::from_rgb(246, 247, 244),
                surface: egui::Color32::from_rgb(255, 255, 252),
                surface_alt: egui::Color32::from_rgb(238, 241, 237),
                border: egui::Color32::from_rgb(218, 224, 216),
                text: egui::Color32::from_rgb(27, 32, 30),
                muted: egui::Color32::from_rgb(99, 110, 105),
                accent: egui::Color32::from_rgb(22, 123, 100),
                accent_soft: egui::Color32::from_rgb(219, 241, 234),
                danger: egui::Color32::from_rgb(194, 65, 69),
                success: egui::Color32::from_rgb(30, 142, 92),
            },
            Theme::Dark => Self {
                bg: egui::Color32::from_rgb(18, 20, 22),
                surface: egui::Color32::from_rgb(29, 32, 34),
                surface_alt: egui::Color32::from_rgb(39, 44, 45),
                border: egui::Color32::from_rgb(62, 70, 70),
                text: egui::Color32::from_rgb(236, 240, 237),
                muted: egui::Color32::from_rgb(157, 167, 162),
                accent: egui::Color32::from_rgb(66, 184, 147),
                accent_soft: egui::Color32::from_rgb(37, 64, 56),
                danger: egui::Color32::from_rgb(239, 107, 103),
                success: egui::Color32::from_rgb(100, 210, 145),
            },
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = ctx.global_style().visuals.clone();
        visuals.panel_fill = self.bg;
        visuals.window_fill = self.bg;
        visuals.extreme_bg_color = self.surface;
        visuals.faint_bg_color = self.surface_alt;
        visuals.code_bg_color = self.surface_alt;
        visuals.window_stroke = egui::Stroke::new(1.0, self.border);
        visuals.selection.bg_fill = self.accent;
        visuals.hyperlink_color = self.accent;
        visuals.widgets.noninteractive.bg_fill = self.surface;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, self.border);
        visuals.widgets.inactive.bg_fill = self.surface_alt;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.border);
        visuals.widgets.inactive.fg_stroke.color = self.text;
        visuals.widgets.hovered.bg_fill = self.accent_soft;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, self.accent);
        visuals.widgets.hovered.fg_stroke.color = self.text;
        visuals.widgets.active.bg_fill = self.accent_soft;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, self.accent);
        visuals.widgets.active.fg_stroke.color = self.text;
        ctx.set_visuals(visuals);
    }
}
