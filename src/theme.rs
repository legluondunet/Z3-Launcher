// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
use eframe::egui::{self, Color32, FontFamily, FontId, RichText, Stroke};
pub const GOLD: Color32 = Color32::from_rgb(213, 179, 94);
pub const FOREST: Color32 = Color32::from_rgb(8, 33, 21);
pub const PARCHMENT: Color32 = Color32::from_rgb(238, 230, 203);

pub fn apply(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("parchment".into(), egui::FontData::from_static(include_bytes!("../assets/DejaVuSerif.ttf")).into());
    fonts.families.insert(FontFamily::Name("parchment".into()), vec!["parchment".into()]);
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = FOREST;
    style.visuals.window_fill = Color32::from_rgb(15, 39, 28);
    style.visuals.extreme_bg_color = Color32::from_rgb(6, 23, 15);
    style.visuals.faint_bg_color = Color32::from_rgb(20, 45, 29);
    style.visuals.override_text_color = Some(PARCHMENT);
    style.visuals.selection.bg_fill = Color32::from_rgb(76, 68, 35);
    style.visuals.selection.stroke = Stroke::new(1.0_f32, GOLD);
    style.visuals.window_stroke = Stroke::new(1.0_f32, GOLD);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.7_f32, Color32::from_rgb(115, 100, 55));
    for widget in [&mut style.visuals.widgets.inactive, &mut style.visuals.widgets.hovered, &mut style.visuals.widgets.active] {
        widget.corner_radius = egui::CornerRadius::same(4);
        widget.bg_fill = Color32::from_rgb(17, 43, 29);
        widget.weak_bg_fill = widget.bg_fill;
        widget.bg_stroke = Stroke::new(1.0_f32, GOLD);
        widget.fg_stroke = Stroke::new(1.0_f32, PARCHMENT);
    }
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(35, 61, 37);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, GOLD);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(69, 71, 39);
    style.spacing.item_spacing = egui::vec2(12.0, 10.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.spacing.interact_size.y = 32.0;
    let family = FontFamily::Name("parchment".into());
    style.text_styles.insert(egui::TextStyle::Heading, FontId::new(28.0, family.clone()));
    style.text_styles.insert(egui::TextStyle::Body, FontId::new(14.0, family.clone()));
    style.text_styles.insert(egui::TextStyle::Button, FontId::new(14.0, family.clone()));
    style.text_styles.insert(egui::TextStyle::Small, FontId::new(11.0, family));
    ctx.set_style(style);
}
pub fn background(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let decoded = image::load_from_memory(include_bytes!("../assets/forest-frame.png")).ok()?.to_rgba8();
    let size = [decoded.width() as usize, decoded.height() as usize];
    let image = egui::ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
    Some(ctx.load_texture("forest-frame", image, egui::TextureOptions::LINEAR))
}
pub fn header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(46.0, 46.0), egui::Sense::hover());
        let point = |x: f32, y: f32| egui::pos2(rect.left()+x, rect.top()+y);
        for (x,y) in [(23.0,2.0),(12.0,22.0),(34.0,22.0)] {
            ui.painter().add(egui::Shape::convex_polygon(vec![point(x,y),point(x-11.0,y+20.0),point(x+11.0,y+20.0)], GOLD, Stroke::NONE));
        }
        ui.vertical(|ui| {
            ui.label(RichText::new("Z3-Launcher").heading().size(30.0).strong());
            ui.small(format!("{} · v{}", std::env::consts::OS, env!("CARGO_PKG_VERSION")));
        });
    });
}
