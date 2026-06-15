// Visual theme + a few shared egui building blocks. Adapted from monado-frame,
// tuned toward the OyasumiVR look (soft dark glass, cyan "sleep" accent).
use egui::Color32;

pub const PRIMARY: Color32 = Color32::from_rgb(160, 200, 255);
pub const SLEEP: Color32 = Color32::from_rgb(74, 178, 209); // active sleep-mode cyan
pub const SLEEP_DEEP: Color32 = Color32::from_rgb(40, 120, 150);
pub const SURFACE: Color32 = Color32::from_rgb(19, 19, 24);
pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(32, 31, 39);
pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(43, 42, 51);
pub const ON_SURFACE: Color32 = Color32::from_rgb(230, 225, 233);
pub const ON_SURFACE_VAR: Color32 = Color32::from_rgb(196, 199, 209);
pub const OK: Color32 = Color32::from_rgb(120, 210, 140);
pub const WARN: Color32 = Color32::from_rgb(240, 180, 90);

pub fn apply_style(ctx: &egui::Context) {
    use egui::{CornerRadius, FontFamily, FontId, Stroke, TextStyle};
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();
    v.panel_fill = SURFACE;
    v.window_fill = SURFACE_CONTAINER;
    v.faint_bg_color = SURFACE_CONTAINER;
    v.extreme_bg_color = Color32::from_rgb(14, 14, 18);
    v.override_text_color = Some(ON_SURFACE);
    v.selection.bg_fill = Color32::from_rgb(48, 78, 130);
    v.selection.stroke = Stroke::new(1.0, PRIMARY);
    v.hyperlink_color = PRIMARY;
    v.widgets.noninteractive.bg_fill = SURFACE;
    v.widgets.inactive.bg_fill = SURFACE_CONTAINER_HIGH;
    v.widgets.inactive.weak_bg_fill = SURFACE_CONTAINER_HIGH;
    v.widgets.hovered.bg_fill = Color32::from_rgb(56, 64, 82);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(56, 64, 82);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.active.bg_fill = PRIMARY;
    v.widgets.active.weak_bg_fill = PRIMARY;
    v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::BLACK);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(16);
        w.bg_stroke = Stroke::NONE;
    }
    style.visuals = v;
    style.spacing.item_spacing = egui::vec2(10.0, 12.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.slider_width = 220.0;
    style.spacing.interact_size.y = 30.0;
    style.text_styles.insert(TextStyle::Heading, FontId::new(26.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Body, FontId::new(17.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Button, FontId::new(17.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Small, FontId::new(12.0, FontFamily::Proportional));
    ctx.set_style(style);
}

/// The rounded, (optionally) translucent floating panel surface. With a
/// transparent framebuffer + an alpha-blended quad layer this gives rounded
/// outer corners and glass.
pub fn panel_card<R>(ui: &mut egui::Ui, alpha: u8, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let s = SURFACE;
    egui::Frame::default()
        .fill(Color32::from_rgba_unmultiplied(s.r(), s.g(), s.b(), alpha))
        .corner_radius(20)
        .outer_margin(10)
        .inner_margin(20)
        .shadow(egui::Shadow { offset: [0, 6], blur: 22, spread: 0, color: Color32::from_black_alpha(120) })
        .show(ui, add)
        .inner
}

/// A title row with an optional back button on the left and trailing text.
pub fn header(ui: &mut egui::Ui, title: &str, right: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).size(22.0).strong().color(Color32::WHITE));
        if let Some(r) = right {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(r).color(ON_SURFACE_VAR));
            });
        }
    });
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(10.0);
}
