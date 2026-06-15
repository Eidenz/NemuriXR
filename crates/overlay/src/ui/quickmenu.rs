// The in-headset quick menu: TOGGLES + status only. Detailed value editing
// (brightness levels, whitelists, OSC, schedule times) lives in the desktop app.
//
//  - Home: the big Sleep Mode toggle + entry to Automations + a status footer.
//  - Automations: a list of on/off switches for each feature (screenshot 2).
use egui::{Align2, Color32, FontId, Sense};
use egui_phosphor::regular as icons;
use nemurixr_core::config::Config;
use nemurixr_core::SleepPhase;

use crate::theme;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Home,
    Automations,
}

pub enum MenuAction {
    None,
    SetPhase(SleepPhase),
    OpenAutomations,
    Back,
}

/// Build the menu for `screen`. Toggles mutate `cfg`; `changed` is set if any did.
/// `connected` reflects whether the desktop-hosted engine is reachable.
#[allow(clippy::too_many_arguments)]
pub fn build_menu(
    ctx: &egui::Context,
    screen: Screen,
    phase: SleepPhase,
    connected: bool,
    clock: &str,
    cfg: &mut Config,
    changed: &mut bool,
    alpha: u8,
) -> MenuAction {
    let mut action = MenuAction::None;
    egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
        theme::panel_card(ui, alpha, |ui| {
            if !connected {
                offline_banner(ui);
            }
            match screen {
                Screen::Home => home(ui, phase, clock, &mut action),
                Screen::Automations => automations(ui, cfg, changed, &mut action),
            }
        });
    });
    action
}

/// A warning strip shown when the desktop engine isn't running.
fn offline_banner(ui: &mut egui::Ui) {
    egui::Frame::default()
        .fill(Color32::from_rgba_unmultiplied(240, 180, 90, 28))
        .corner_radius(10)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("{}  Desktop app offline — open NemuriXR to sync", icons::WARNING))
                    .size(14.0)
                    .color(theme::WARN),
            );
        });
    ui.add_space(10.0);
}

fn home(ui: &mut egui::Ui, phase: SleepPhase, clock: &str, action: &mut MenuAction) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{}  ", icons::MOON)).size(26.0).color(theme::SLEEP));
        ui.label(egui::RichText::new("Nemuri").size(26.0).strong().color(Color32::WHITE));
        ui.label(egui::RichText::new("XR").size(26.0).strong().color(theme::SLEEP));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(format!("{}  {clock}", icons::CLOCK)).size(18.0).color(theme::ON_SURFACE_VAR));
        });
    });
    ui.add_space(14.0);

    // The big card toggles between Awake and Sleep.
    if sleep_card(ui, phase) {
        let target = if phase == SleepPhase::Awake { SleepPhase::Sleep } else { SleepPhase::Awake };
        *action = MenuAction::SetPhase(target);
    }
    ui.add_space(10.0);

    // Prepare-to-sleep (highlighted while preparing).
    if wide_button(ui, icons::BED, "Prepare to sleep", phase == SleepPhase::Prepare) {
        *action = MenuAction::SetPhase(SleepPhase::Prepare);
    }
    ui.add_space(8.0);

    // Entry to the automations toggle list.
    if wide_button(ui, icons::SLIDERS_HORIZONTAL, "Automations", false) {
        *action = MenuAction::OpenAutomations;
    }
}

fn automations(ui: &mut egui::Ui, cfg: &mut Config, changed: &mut bool, action: &mut MenuAction) {
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new(egui::RichText::new(icons::ARROW_LEFT).size(22.0)).frame(false)).clicked() {
            *action = MenuAction::Back;
        }
        ui.add_space(6.0);
        ui.label(egui::RichText::new("Automations").size(22.0).strong().color(Color32::WHITE));
    });
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(10.0);

    *changed |= toggle_row(ui, icons::MOON, "Sleep Schedule", "Enable/disable at set times", &mut cfg.sleep.schedule_enabled);
    *changed |= toggle_row(ui, icons::SUN, "Brightness on Sleep/Wake", "Dim the headset when you sleep", &mut cfg.brightness.enabled);
    *changed |= toggle_row(ui, icons::ENVELOPE, "Auto-Accept Invites", "Accept invite requests automatically", &mut cfg.vrchat.auto_accept.enabled);
    *changed |= toggle_row(ui, icons::BELL, "Join Notifications", "Sound when players come and go", &mut cfg.vrchat.join_notifications.enabled);
    *changed |= toggle_row(ui, icons::USERS_THREE, "Status Automations", "Set status by player count", &mut cfg.vrchat.status_automations.enabled);
    *changed |= toggle_row(ui, icons::GAME_CONTROLLER, "Block Game Input", "While pointing at panels", &mut cfg.block_game_input);
}

/// The big Sleep Mode card: grey (Awake), blue (Preparing), cyan (Sleeping).
/// Returns true on click.
fn sleep_card(ui: &mut egui::Ui, phase: SleepPhase) -> bool {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 128.0), Sense::click());
    let hot = resp.hovered();
    let fill = match phase {
        SleepPhase::Sleep => {
            if hot {
                theme::SLEEP
            } else {
                theme::SLEEP_DEEP
            }
        }
        SleepPhase::Prepare => Color32::from_rgb(70, 100, 140),
        SleepPhase::Awake => {
            if hot {
                Color32::from_rgb(56, 64, 82)
            } else {
                theme::SURFACE_CONTAINER_HIGH
            }
        }
    };
    let lit = phase != SleepPhase::Awake;
    let p = ui.painter();
    p.rect_filled(rect, egui::CornerRadius::same(18), fill);
    let fg = if lit { Color32::WHITE } else { theme::ON_SURFACE };
    let sub = if lit { Color32::from_white_alpha(210) } else { theme::ON_SURFACE_VAR };
    let status = match phase {
        SleepPhase::Awake => "Inactive",
        SleepPhase::Prepare => "Preparing",
        SleepPhase::Sleep => "Active",
    };
    p.text(egui::pos2(rect.left() + 78.0, rect.center().y), Align2::CENTER_CENTER, icons::MOON, FontId::proportional(60.0), fg);
    p.text(egui::pos2(rect.right() - 32.0, rect.center().y - 22.0), Align2::RIGHT_CENTER, "Sleep Mode", FontId::proportional(20.0), sub);
    p.text(egui::pos2(rect.right() - 32.0, rect.center().y + 18.0), Align2::RIGHT_CENTER, status, FontId::proportional(38.0), fg);
    resp.clicked()
}

/// A full-width button with an icon + label; `active` tints it with the accent.
fn wide_button(ui: &mut egui::Ui, icon: &str, label: &str, active: bool) -> bool {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 64.0), Sense::click());
    let fill = if active {
        Color32::from_rgb(70, 100, 140)
    } else if resp.hovered() {
        Color32::from_rgb(56, 64, 82)
    } else {
        theme::SURFACE_CONTAINER
    };
    let fg = if active { Color32::WHITE } else { theme::ON_SURFACE };
    let p = ui.painter();
    p.rect_filled(rect, egui::CornerRadius::same(14), fill);
    p.text(egui::pos2(rect.left() + 26.0, rect.center().y), Align2::LEFT_CENTER, icon, FontId::proportional(26.0), fg);
    p.text(egui::pos2(rect.left() + 64.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(19.0), fg);
    resp.clicked()
}

/// A settings row: icon + title + subtitle + a toggle switch. Whole row clickable.
/// Returns true if the value changed this frame.
fn toggle_row(ui: &mut egui::Ui, icon: &str, title: &str, subtitle: &str, value: &mut bool) -> bool {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 64.0), Sense::click());
    ui.add_space(8.0); // gap to the next row
    let fill = if resp.hovered() { Color32::from_rgb(48, 47, 56) } else { theme::SURFACE_CONTAINER };
    let p = ui.painter();
    p.rect_filled(rect, egui::CornerRadius::same(14), fill);
    p.text(egui::pos2(rect.left() + 28.0, rect.center().y), Align2::CENTER_CENTER, icon, FontId::proportional(26.0), theme::ON_SURFACE_VAR);
    p.text(egui::pos2(rect.left() + 56.0, rect.center().y - 11.0), Align2::LEFT_CENTER, title, FontId::proportional(18.0), theme::ON_SURFACE);
    p.text(egui::pos2(rect.left() + 56.0, rect.center().y + 12.0), Align2::LEFT_CENTER, subtitle, FontId::proportional(13.0), theme::ON_SURFACE_VAR);
    draw_switch(p, egui::pos2(rect.right() - 56.0, rect.center().y), *value);

    if resp.clicked() {
        *value = !*value;
        true
    } else {
        false
    }
}

/// Draw an iOS-style switch centered at `c`.
fn draw_switch(p: &egui::Painter, c: egui::Pos2, on: bool) {
    let w = 52.0;
    let h = 30.0;
    let track = egui::Rect::from_center_size(c, egui::vec2(w, h));
    let track_color = if on { theme::SLEEP } else { theme::SURFACE_CONTAINER_HIGH };
    p.rect_filled(track, egui::CornerRadius::same((h / 2.0) as u8), track_color);
    let knob_x = if on { track.right() - h / 2.0 } else { track.left() + h / 2.0 };
    p.circle_filled(egui::pos2(knob_x, c.y), h / 2.0 - 4.0, Color32::WHITE);
}
