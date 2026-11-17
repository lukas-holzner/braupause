//! Farben aus DESIGN.md („Kellerbuch"-Look: helle Flächen, Bernstein-Akzent).

use egui::{Color32, CornerRadius, Stroke, Visuals};

pub const AMBER: Color32 = Color32::from_rgb(0xB4, 0x53, 0x09);
pub const AMBER_HOVER: Color32 = Color32::from_rgb(0x92, 0x40, 0x0E);
pub const AMBER_TINT: Color32 = Color32::from_rgb(0xFE, 0xF3, 0xC7);
pub const AMBER_PRESSED: Color32 = Color32::from_rgb(0xFD, 0xE6, 0x8A);
pub const OLIVE: Color32 = Color32::from_rgb(0x1E, 0x56, 0x31);
pub const OLIVE_TINT: Color32 = Color32::from_rgb(0xDC, 0xFC, 0xE7);
pub const BLUE: Color32 = Color32::from_rgb(0x1F, 0x4E, 0x8C);
pub const BG_PAGE: Color32 = Color32::from_rgb(0xF2, 0xF5, 0xF7);
pub const SURFACE: Color32 = Color32::WHITE;
pub const BG_SUBTLE: Color32 = Color32::from_rgb(0xE6, 0xEC, 0xF0);
pub const ROW_HOVER: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xFC);
pub const INK_PRIMARY: Color32 = Color32::from_rgb(0x12, 0x18, 0x1F);
pub const INK_SECONDARY: Color32 = Color32::from_rgb(0x3E, 0x4C, 0x59);
pub const INK_MUTED: Color32 = Color32::from_rgb(0x61, 0x71, 0x80);
pub const HAIRLINE: Color32 = Color32::from_rgb(0xD5, 0xDF, 0xE6);
pub const ERROR: Color32 = Color32::from_rgb(0xB9, 0x1C, 0x1C);
pub const ERROR_TINT: Color32 = Color32::from_rgb(0xFE, 0xE2, 0xE2);
pub const WARN: Color32 = Color32::from_rgb(0x92, 0x40, 0x0E);

pub fn apply(ctx: &egui::Context) {
    let mut v = Visuals::light();
    v.override_text_color = None;
    v.panel_fill = BG_PAGE;
    v.window_fill = SURFACE;
    v.window_stroke = Stroke::new(1.0, HAIRLINE);
    v.extreme_bg_color = SURFACE;
    v.faint_bg_color = ROW_HOVER;
    v.code_bg_color = BG_SUBTLE;
    v.hyperlink_color = BLUE;
    v.warn_fg_color = WARN;
    v.error_fg_color = ERROR;
    v.selection.bg_fill = AMBER_TINT;
    v.selection.stroke = Stroke::new(1.0, AMBER);
    v.window_corner_radius = CornerRadius::same(6);
    v.menu_corner_radius = CornerRadius::same(4);

    let r = CornerRadius::same(4);
    let w = &mut v.widgets;
    w.noninteractive.bg_stroke = Stroke::new(1.0, HAIRLINE);
    w.noninteractive.fg_stroke = Stroke::new(1.0, INK_PRIMARY);
    w.noninteractive.corner_radius = r;
    w.inactive.bg_fill = BG_SUBTLE;
    w.inactive.weak_bg_fill = BG_SUBTLE;
    w.inactive.bg_stroke = Stroke::new(1.0, HAIRLINE);
    w.inactive.fg_stroke = Stroke::new(1.0, INK_PRIMARY);
    w.inactive.corner_radius = r;
    w.hovered.bg_fill = AMBER_TINT;
    w.hovered.weak_bg_fill = AMBER_TINT;
    w.hovered.bg_stroke = Stroke::new(1.0, AMBER);
    w.hovered.fg_stroke = Stroke::new(1.5, AMBER_HOVER);
    w.hovered.corner_radius = r;
    // Achtung: active.fg_stroke ist in egui auch die Farbe für „strong“-Text.
    w.active.bg_fill = AMBER_PRESSED;
    w.active.weak_bg_fill = AMBER_PRESSED;
    w.active.bg_stroke = Stroke::new(1.0, AMBER_HOVER);
    w.active.fg_stroke = Stroke::new(1.5, INK_PRIMARY);
    w.active.corner_radius = r;
    w.open.bg_fill = AMBER_TINT;
    w.open.weak_bg_fill = AMBER_TINT;
    w.open.bg_stroke = Stroke::new(1.0, AMBER);
    w.open.fg_stroke = Stroke::new(1.0, INK_PRIMARY);
    w.open.corner_radius = r;

    ctx.set_visuals_of(egui::Theme::Light, v);
    ctx.set_theme(egui::ThemePreference::Light);
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(8.0, 3.0);
        s.spacing.interact_size.y = 22.0;
    });
}

/// Primärer (Bernstein-)Knopf.
pub fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text.to_string()).color(SURFACE).strong()).fill(AMBER)
}
