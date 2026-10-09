// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
use eframe::egui::{self, Color32, FontFamily, FontId, RichText, Stroke};
pub const GOLD: Color32 = Color32::from_rgb(232, 189, 83);
pub const FOREST: Color32 = Color32::from_rgb(8, 43, 29);
const EMBLEM_TRIANGLES: [(f32, f32); 3] = [(23.0, 2.0), (12.0, 22.0), (34.0, 22.0)];
pub const PARCHMENT: Color32 = Color32::from_rgb(246, 237, 209);

pub fn apply(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("parchment".into(), egui::FontData::from_static(include_bytes!("../assets/DejaVuSerif.ttf")).into());
    fonts.families.insert(FontFamily::Name("parchment".into()), vec!["parchment".into()]);
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = Color32::from_rgba_unmultiplied(8, 43, 29, 215);
    style.visuals.window_fill = Color32::from_rgb(9, 47, 31);
    style.visuals.extreme_bg_color = Color32::from_rgb(6, 23, 15);
    style.visuals.faint_bg_color = Color32::from_rgb(20, 45, 29);
    style.visuals.override_text_color = Some(PARCHMENT);
    style.visuals.selection.bg_fill = Color32::from_rgb(76, 68, 35);
    style.visuals.selection.stroke = Stroke::new(1.0_f32, GOLD);
    style.visuals.window_stroke = Stroke::new(1.0_f32, GOLD);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.7_f32, Color32::from_rgb(115, 100, 55));
    for widget in [&mut style.visuals.widgets.inactive, &mut style.visuals.widgets.hovered, &mut style.visuals.widgets.active] {
        widget.corner_radius = egui::CornerRadius::same(4);
        widget.bg_fill = Color32::from_rgb(9, 48, 32);
        widget.weak_bg_fill = widget.bg_fill;
        widget.bg_stroke = Stroke::new(1.0_f32, GOLD);
        widget.fg_stroke = Stroke::new(1.0_f32, PARCHMENT);
    }
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(24, 75, 45);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, GOLD);
    style.visuals.widgets.active.bg_fill = GOLD;
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
        for (x,y) in EMBLEM_TRIANGLES {
            ui.painter().add(egui::Shape::convex_polygon(vec![point(x,y),point(x-11.0,y+20.0),point(x+11.0,y+20.0)], GOLD, Stroke::NONE));
        }
        ui.vertical(|ui| {
            ui.label(RichText::new("Z3-Launcher").heading().size(30.0).strong());
            ui.small(format!("{} · v{}", std::env::consts::OS, env!("CARGO_PKG_VERSION")));
        });
    });
}

/// Transparent native window icon using the same gold and geometry as the header.
/// Supersampling keeps triangle edges smooth at taskbar and title-bar sizes.
pub fn window_icon() -> egui::IconData {
    let size = 64_u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let color = GOLD.to_array();
    for y in 0..size {
        for x in 0..size {
            let mut coverage = 0_u32;
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = (x as f32 + (sx as f32 + 0.5) / 4.0) * 46.0 / size as f32;
                    let py = (y as f32 + (sy as f32 + 0.5) / 4.0) * 46.0 / size as f32;
                    if EMBLEM_TRIANGLES.iter().any(|&(cx, top)| {
                        let height = py - top;
                        (0.0..=20.0).contains(&height)
                            && (px - cx).abs() <= 11.0 * height / 20.0
                    }) {
                        coverage += 1;
                    }
                }
            }
            rgba.extend_from_slice(&[color[0], color[1], color[2], (coverage * 255 / 16) as u8]);
        }
    }
    egui::IconData { rgba, width: size, height: size }
}
