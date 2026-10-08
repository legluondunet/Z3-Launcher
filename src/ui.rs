// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
use crate::i18n::{tr, tf};
use crate::core::Launcher;
use eframe::egui;
use crate::theme;
use std::{fs, path::PathBuf, sync::{mpsc::{self, Receiver}, Arc}, thread};
enum Event { Line(String), Done(std::result::Result<(), String>), LanguageDone(String, std::result::Result<(), String>) }
struct App {
    root: String, rom: String, lines: Vec<String>, rx: Option<Receiver<Event>>,
    importing_language: bool, background: Option<egui::TextureHandle>, status: String, options: crate::options::Options, tab: usize,
}
impl Default for App {
    fn default() -> Self {
        let root = if crate::platform::is_portable() {
            crate::default_root().to_string_lossy().into_owned()
        } else { fs::read_to_string(preferences()).unwrap_or_else(|_| crate::default_root().to_string_lossy().into_owned()) };
        Self { root, rom: String::new(), lines: crate::i18n::warnings(), rx: None, importing_language: false, background: None, status: tr("text.ready").into(), options: crate::options::Options::default(), tab: 0 }
    }
}
fn preferences() -> PathBuf { crate::platform::config_dir().join("workspace.txt") }
impl App {
    fn import_game_language(&mut self, code: String, root: PathBuf, ctx: &egui::Context) {
        if self.rx.is_some() { return; }
        let launcher = match Launcher::new(root) { Ok(l) => l, Err(e) => { self.status = e.to_string(); return; } };
        let title = tf("game_language.select_rom", &[("language", code.clone())]);
        let Some(rom) = rfd::FileDialog::new().set_title(title).add_filter(tr("text.snes_rom_sfc_smc"), &["sfc", "smc", "SFC", "SMC"]).pick_file() else {
            self.status = tr("game_language.cancelled").into(); return;
        };
        let (tx, rx) = mpsc::channel(); self.rx = Some(rx); self.importing_language = true;
        self.status = tf("game_language.importing", &[("language", code.clone())]);
        let ctx = ctx.clone();
        thread::spawn(move || {
            let t = tx.clone(); let c = ctx.clone();
            let result = launcher.log(Arc::new(move |line| { let _ = t.send(Event::Line(line)); c.request_repaint(); }))
                .and_then(|log| launcher.import_language(&code, Some(&rom), &log)).map_err(|e| e.to_string());
            let _ = tx.send(Event::LanguageDone(code, result)); ctx.request_repaint();
        });
    }
    fn dispatch(&mut self, action: &'static str, ctx: &egui::Context) {
        self.options.autosave(&mut self.status);
        if self.options.has_changes() {
            self.status = tr("settings.resolve_before_action").into();
            if self.tab == 0 { self.tab = 1; self.options.select_page(0); }
            return;
        }
        let launcher = match Launcher::new(&self.root) { Ok(l) => l, Err(e) => { self.status = e.to_string(); return; } };
        self.root = launcher.root.to_string_lossy().into_owned();
        let prefs = preferences();
        if let Some(parent) = prefs.parent() { if let Err(e) = fs::create_dir_all(parent).and_then(|_| fs::write(&prefs, &self.root)) { self.status = e.to_string(); return; } }
        let rom = if self.rom.is_empty() { None } else { Some(PathBuf::from(&self.rom)) };
        let (tx, rx) = mpsc::channel(); self.rx = Some(rx); self.status = tf("status.running", &[("action", action.to_owned())]);
        let ctx = ctx.clone();
        thread::spawn(move || {
            let t = tx.clone(); let c = ctx.clone();
            let result = launcher.log(Arc::new(move |line| { let _ = t.send(Event::Line(line)); c.request_repaint(); }))
                .and_then(|log| launcher.action(action, rom.as_deref(), &log)).map_err(|e| e.to_string());
            let _ = tx.send(Event::Done(result)); ctx.request_repaint();
        });
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.importing_language {
            if ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.status = tr("game_language.wait_before_close").into();
            }
        } else { self.options.close_guard(ctx, &mut self.status); }
        let mut done = None;
        let mut imported = None;
        if let Some(rx) = &self.rx {
            for event in rx.try_iter() { match event {
                Event::Line(line) => { self.lines.push(line); if self.lines.len() > 2000 { self.lines.remove(0); } },
                Event::Done(result) => done = Some(result),
                Event::LanguageDone(code, result) => { imported = Some(code); done = Some(result); },
            } }
        }
        if let Some(result) = done {
            self.rx = None; self.importing_language = false;
            self.status = match result {
                Ok(()) => if let Some(code) = imported {
                    self.options.apply_game_language(&code);
                    tr("settings.language_imported").into()
                } else { tr("text.done").into() },
                Err(e) => tf("status.error", &[("error", e.to_string())]),
            };
        }
        let busy = self.rx.is_some();
        let dropped = ctx.input(|input| input.raw.dropped_files.clone());
        if !dropped.is_empty() {
            if busy {
                self.status = tr("text.an_operation_is_running_wait_for_it_to_finish").into();
            } else if dropped.len() != 1 {
                self.status = tr("text.drop_one_rom_file_at_a_time").into();
            } else if let Some(path) = &dropped[0].path {
                let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if path.is_file() && (extension.eq_ignore_ascii_case("sfc") || extension.eq_ignore_ascii_case("smc")) {
                    if let Some(path) = path.to_str() {
                        self.rom = path.to_owned();
                        if self.tab != 7 || self.options.leave_ini(&mut self.status) {
                            self.tab = 0; self.options.cancel_capture();
                            self.status = tr("text.rom_selected_click_install_and_build_to_use_it").into();
                        }
                    } else {
                        self.status = tr("text.the_rom_path_must_be_utf_8").into();
                    }
                } else {
                    self.status = tr("text.drop_a_sfc_or_smc_rom_file").into();
                }
            } else {
                self.status = tr("text.this_drop_has_no_local_file_path_enter_the").into();
            }
        }
        // Reserve the footer before laying out the central panel.
        let displayed_status=self.status.clone();
        egui::TopBottomPanel::bottom("status-bar").resizable(false)
            .frame(egui::Frame::new().fill(theme::FOREST).inner_margin(egui::Margin::symmetric(16, 8)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if busy { ui.spinner(); }
                    ui.add(egui::Label::new(egui::RichText::new(&displayed_status).size(13.0)).wrap());
                });
            });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::FOREST).inner_margin(24)).show(ctx, |ui| {
            if let Some(background) = &self.background {
                ui.painter().image(background.id(), ui.max_rect().expand(24.0),
                    egui::Rect::from_min_max(egui::pos2(0.0,0.0),egui::pos2(1.0,1.0)),egui::Color32::WHITE);
            }
            theme::header(ui);
            ui.add_space(8.0); ui.separator();
            ui.horizontal_wrapped(|ui| {
                let previous = self.tab;
                for (index, label) in [tr("text.general"), tr("text.gameplay"), tr("text.display"), tr("text.sound_msu"), tr("text.keyboard"), tr("text.gamepad"), tr("text.shortcuts"), tr("text.ini_tab")].iter().enumerate() {
                    let active=self.tab == index;
                    let button=egui::Button::new(egui::RichText::new(*label).size(11.0)
                        .color(if active { theme::FOREST } else { theme::PARCHMENT }))
                        .fill(if active { theme::GOLD } else { theme::FOREST })
                        .stroke(egui::Stroke::new(1.0_f32, theme::GOLD))
                        .corner_radius(egui::CornerRadius::same(8));
                    if ui.add(button).clicked() { self.tab=index; }
                }
                if self.tab != previous {
                    if previous == 7 && !self.options.leave_ini(&mut self.status) {
                        self.tab=previous;
                    } else if self.tab == 7 {
                        if busy || !self.options.enter_ini(&self.root, &mut self.status) { self.tab=previous; }
                    } else if self.tab > 0 { self.options.select_page(self.tab - 1); }
                    else { self.options.cancel_capture(); }
                }
            });
            ui.separator();
            if self.tab == 7 {
                self.options.show_ini(ui, busy);
            } else if self.tab > 0 {
                self.options.show(ui, ctx, &self.root, busy, &mut self.status);
            } else {
            ui.add_enabled_ui(!busy, |ui| {
                let previous = crate::i18n::current();
                let mut selected = previous.clone();
                let languages = crate::i18n::languages();
                let name = languages.iter().find(|(code,_)| code == &selected).map(|(_,name)| name.as_str()).unwrap_or("English");
                ui.horizontal(|ui| {
                    ui.label(tr("general.interface_language")).on_hover_text(tr("help.launcher.language"));
                    egui::ComboBox::from_id_salt("interface-language").selected_text(name).show_ui(ui, |ui| {
                        for (code,name) in &languages { ui.selectable_value(&mut selected, code.clone(), name); }
                    }).response.on_hover_text(tr("help.launcher.language"));
                });
                if selected != previous {
                    match crate::i18n::set_language(&selected) {
                        Ok(()) => { self.status = tr("text.ready").into(); ctx.request_repaint(); },
                        Err(e) => self.status = tf("language.save_failed", &[("error", e.to_string())]),
                    }
                }
                ui.separator();
                ui.label(tr("text.working_directory_full_path"));
                ui.add_enabled(!crate::platform::is_portable(), egui::TextEdit::singleline(&mut self.root).desired_width(f32::INFINITY)).on_hover_text(tr("help.launcher.workspace"));
                ui.label(tr("text.drop_your_sfc_or_smc_rom_here_or_select"));
                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 140.0).max(80.0);
                    ui.add_sized([width, 32.0], egui::TextEdit::singleline(&mut self.rom)).on_hover_text(tr("help.launcher.rom"));
                    if ui.button(tr("text.browse")).on_hover_text(tr("help.launcher.rom")).clicked() {
                        let mut dialog = rfd::FileDialog::new()
                            .set_title(tr("text.select_the_zelda_3_rom"))
                            .add_filter(tr("text.snes_rom_sfc_smc"), &["sfc", "smc", "SFC", "SMC"]);
                        if let Some(parent) = PathBuf::from(&self.rom).parent().filter(|p| p.is_dir()) {
                            dialog = dialog.set_directory(parent);
                        }
                        if let Some(path) = dialog.pick_file() {
                            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if path.is_file() && (extension.eq_ignore_ascii_case("sfc") || extension.eq_ignore_ascii_case("smc")) {
                                if let Some(path) = path.to_str() {
                                    self.rom = path.to_owned();
                                    self.status = tr("text.rom_selected_click_install_and_build_to_use_it").into();
                                } else {
                                    self.status = tr("text.the_rom_path_must_be_utf_8").into();
                                }
                            } else {
                                self.status = tr("text.select_a_sfc_or_smc_rom_file").into();
                            }
                        }
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    for (label, action) in [(tr("text.check_dependencies"), "check"), (tr("text.install_and_build"), "setup"), (tr("text.launch_game"), "run")] {
                        let button = if action == "run" {
                            egui::Button::new(egui::RichText::new(label).color(theme::FOREST).strong()).fill(theme::GOLD)
                        } else { egui::Button::new(label) };
                        if ui.add(button).on_hover_text(tr(&format!("help.launcher.{action}"))).clicked() { self.dispatch(action, ctx); }
                    }
                });
            });
                ui.add_space(12.0);
                ui.separator();
                ui.small(tr("text.full_log_launcher_log_in_the_working_directory"));
                // Keep room for the two buttons; the journal takes the remaining height.
                let footer_height=ui.spacing().interact_size.y.max(
                    ui.text_style_height(&egui::TextStyle::Button) + 2.0 * ui.spacing().button_padding.y);
                let journal_height=(ui.available_height() - footer_height - ui.spacing().item_spacing.y).max(32.0);
                egui::Frame::new().fill(ui.visuals().extreme_bg_color)
                    .stroke(egui::Stroke::new(1.0_f32, theme::GOLD))
                    .corner_radius(egui::CornerRadius::same(4)).inner_margin(4)
                    .show(ui, |ui| {
                    egui::ScrollArea::vertical().id_salt("journal-scroll").stick_to_bottom(true)
                        .max_height((journal_height - 8.0).max(24.0)).min_scrolled_height(0.0)
                        .auto_shrink([false, false]).show(ui, |ui| {
                        let text = self.lines.join("\n");
                        // An immutable TextBuffer allows selection and Ctrl+C without editing the log.
                        let mut buffer = text.as_str();
                        let id = ui.make_persistent_id("journal-text");
                        let previous = egui::text_edit::TextEditState::load(ctx, id)
                            .and_then(|state| state.cursor.char_range());
                        let mut output = egui::TextEdit::multiline(&mut buffer).id(id)
                            .font(egui::TextStyle::Monospace).desired_width(f32::INFINITY)
                            .frame(false).desired_rows(1).show(ui);
                        // egui 0.31 starts text selection on any mouse button press.
                        // Restore the existing range when a secondary click opens the copy menu.
                        if output.response.hovered() && ui.input(|i| i.pointer.secondary_pressed()) {
                            output.state.cursor.set_char_range(previous);
                            output.state.clone().store(ctx, id);
                            ctx.request_repaint();
                        }
                        let selected = output.state.cursor.char_range().map(|range| {
                            let [start, end] = range.sorted();
                            text.chars().skip(start.index).take(end.index.saturating_sub(start.index)).collect::<String>()
                        }).unwrap_or_default();
                        output.response.context_menu(|ui| {
                            if ui.add_enabled(!selected.is_empty(), egui::Button::new(tr("journal.copy_selection"))).clicked() {
                                ctx.copy_text(selected.clone()); ui.close_menu();
                            }
                            if ui.button(tr("text.copy_log")).clicked() {
                                ctx.copy_text(text.clone()); ui.close_menu();
                            }
                        });
                    });
                });
                ui.horizontal(|ui| {
                    if ui.button(tr("text.copy_log")).clicked() { ctx.copy_text(self.lines.join("\n")); }
                    if ui.button(tr("text.clear_display")).clicked() { self.lines.clear(); }
                });
            }
        });
        if !busy { self.options.autosave(&mut self.status); }
        if let Some((code, root)) = self.options.take_language_request() { self.import_game_language(code, root, ctx); }
        // Status changes made by widgets are reflected in the footer on the next frame.
        if self.status != displayed_status { ctx.request_repaint(); }
    }
}
pub fn start() -> eframe::Result<()> {
    let options = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size([1060.0, 840.0]).with_min_inner_size([760.0, 660.0]).with_icon(theme::window_icon()), ..Default::default() };
    eframe::run_native("Z3-Launcher", options, Box::new(|cc| {
        theme::apply(&cc.egui_ctx);
        let mut app=App::default(); app.background=theme::background(&cc.egui_ctx);
        Ok(Box::new(app))
    }))
}
