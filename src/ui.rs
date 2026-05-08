use crate::theme::ThemeColors;
use eframe::egui;

pub fn card<R>(
    ui: &mut egui::Ui,
    colors: &ThemeColors,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::default()
        .fill(colors.surface)
        .stroke(egui::Stroke::new(1.0, colors.border))
        .inner_margin(egui::Margin::same(20))
        .corner_radius(12)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui)
        })
        .inner
}

pub fn banner<R>(
    ui: &mut egui::Ui,
    colors: &ThemeColors,
    accent: egui::Color32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let bg = mix(accent, colors.surface, 0.9);
    egui::Frame::default()
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, mix(accent, colors.border, 0.5)))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .corner_radius(10)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui)
        })
        .inner
}

pub fn section_title(ui: &mut egui::Ui, colors: &ThemeColors, title: &str) {
    ui.label(
        egui::RichText::new(title.to_ascii_uppercase())
            .size(11.0)
            .strong()
            .color(colors.muted),
    );
}

pub fn field_label(ui: &mut egui::Ui, colors: &ThemeColors, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(12.0)
            .strong()
            .color(colors.text),
    );
}

pub fn caption(ui: &mut egui::Ui, colors: &ThemeColors, text: impl Into<String>) {
    ui.label(
        egui::RichText::new(text.into())
            .size(11.0)
            .color(colors.muted),
    );
}

pub fn primary_button(colors: &ThemeColors, text: &str, size: [f32; 2]) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text.to_owned())
            .size(14.0)
            .strong()
            .color(egui::Color32::WHITE),
    )
    .fill(colors.accent)
    .corner_radius(8)
    .min_size(egui::vec2(size[0], size[1]))
}

pub fn danger_button(colors: &ThemeColors, text: &str, size: [f32; 2]) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text.to_owned())
            .size(15.0)
            .strong()
            .color(egui::Color32::WHITE),
    )
    .fill(colors.danger)
    .corner_radius(8)
    .min_size(egui::vec2(size[0], size[1]))
}

pub fn quiet_button<'a>(
    ui: &mut egui::Ui,
    colors: &ThemeColors,
    label: impl Into<egui::WidgetText> + 'a,
) -> egui::Response {
    ui.add(
        egui::Button::new(label)
            .fill(colors.surface_alt)
            .stroke(egui::Stroke::new(1.0, colors.border))
            .corner_radius(7)
            .min_size(egui::vec2(108.0, 32.0)),
    )
}

pub fn pill<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    colors: &ThemeColors,
    current: &mut T,
    value: T,
    label: &str,
    enabled: bool,
) -> egui::Response {
    let selected = *current == value;
    let fill = if selected {
        colors.accent
    } else {
        colors.surface_alt
    };
    let text_color = if selected {
        egui::Color32::WHITE
    } else {
        colors.text
    };
    let stroke_color = if selected {
        colors.accent
    } else {
        colors.border
    };
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(egui::RichText::new(label).size(13.0).color(text_color))
            .fill(fill)
            .stroke(egui::Stroke::new(1.0, stroke_color))
            .corner_radius(7)
            .min_size(egui::vec2(78.0, 32.0)),
    );

    if response.clicked() {
        *current = value;
    }
    response
}

pub fn metric(ui: &mut egui::Ui, colors: &ThemeColors, value: &str, label: &str, ok: bool) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(value).size(28.0).strong().color(if ok {
            colors.success
        } else {
            colors.muted
        }));
        ui.add_space(8.0);
        ui.label(egui::RichText::new(label).size(12.0).color(colors.muted));
    });
}

fn mix(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| ((x as f32) * (1.0 - t) + (y as f32) * t).round() as u8;
    egui::Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
}
