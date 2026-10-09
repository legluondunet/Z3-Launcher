// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
use crate::i18n::{tr, tf};
use crate::{core, ini::Ini};
use eframe::egui;
use std::{fs, path::PathBuf, time::{Duration, Instant}};
use sdl2::{controller::{Axis, Button, GameController}, event::Event};
fn control_labels() -> [&'static str;12] { [tr("text.up"), tr("text.down"), tr("text.left"), tr("text.right"), tr("mapping.select"), tr("mapping.start"), "A", "B", "X", "Y", "L", "R"] }
const KEY_DEFAULT: &str = "Up, Down, Left, Right, Right Shift, Return, x, z, s, a, c, v";
const PAD_DEFAULT: &str = "DpadUp, DpadDown, DpadLeft, DpadRight, Back, Start, B, A, Y, X, Lb, Rb";
const PAD_KEYS: [&str;17] = ["A","B","X","Y","Back","Guide","Start","L3","R3","Lb","Rb","DpadUp","DpadDown","DpadLeft","DpadRight","L2","R2"];
#[derive(Clone)]
struct Target { section: String, key: String, index: usize, count: usize, default: String }
struct Capture { target: Target, sequential: bool, since: Instant }
struct Pad {
    // Kept alive for the entire lifetime of subsystem, controllers and pump.
    _sdl: sdl2::Sdl, subsystem: sdl2::GameControllerSubsystem,
    controllers: Vec<GameController>, pump: sdl2::EventPump, triggers: [bool; 2],
}
impl Pad {
    fn new() -> Result<Self,String> {
        sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");
        let sdl=sdl2::init()?; let subsystem=sdl.game_controller()?; let pump=sdl.event_pump()?;
        let mut pad=Self { _sdl:sdl, subsystem, controllers:Vec::new(), pump, triggers: [false; 2] }; pad.refresh(); Ok(pad)
    }
    fn refresh(&mut self) {
        self.controllers.retain(|c| c.attached());
        for i in 0..self.subsystem.num_joysticks().unwrap_or(0) {
            if self.subsystem.is_game_controller(i) {
                if let Ok(c)=self.subsystem.open(i) {
                    if !self.controllers.iter().any(|old| old.instance_id()==c.instance_id()) { self.controllers.push(c); }
                }
            }
        }
    }
    fn poll(&mut self) -> Vec<String> {
        let mut result=Vec::new(); let mut refresh=false;
        for e in self.pump.poll_iter() { match e {
            Event::ControllerDeviceAdded {..}|Event::ControllerDeviceRemoved {..} => refresh=true,
            Event::ControllerButtonDown {button,..} => { if let Some(name)=button_name(button) { result.push(name.to_owned()); } },
            Event::ControllerAxisMotion {axis,value,..} => {
                let index = match axis { Axis::TriggerLeft => Some(0), Axis::TriggerRight => Some(1), _ => None };
                if let Some(index) = index {
                    let pressed = value > 16000;
                    if pressed && !self.triggers[index] { result.push(if index == 0 { "L2" } else { "R2" }.into()); }
                    self.triggers[index] = pressed;
                }
            },
            _=>{}
        } }
        if refresh { self.refresh(); } result
    }
}
fn button_name(button: Button) -> Option<&'static str> {
    match button {
        Button::A=>Some("A"), Button::B=>Some("B"), Button::X=>Some("X"), Button::Y=>Some("Y"),
        Button::Back=>Some("Back"), Button::Guide=>Some("Guide"), Button::Start=>Some("Start"),
        Button::LeftStick=>Some("L3"), Button::RightStick=>Some("R3"),
        Button::LeftShoulder=>Some("Lb"), Button::RightShoulder=>Some("Rb"),
        Button::DPadUp=>Some("DpadUp"), Button::DPadDown=>Some("DpadDown"),
        Button::DPadLeft=>Some("DpadLeft"), Button::DPadRight=>Some("DpadRight"), _=>None
    }
}
pub struct Options {
    doc: Option<Ini>, path: Option<PathBuf>, dirty: bool, last_attempt: Option<String>, page: usize,
    capture: Option<Capture>, pad: Option<Pad>, pad_error: String, confirm_close: bool, raw_text: String, raw_active: bool, raw_dirty: bool, disk_text: String, language_request: Option<String>,
}
impl Default for Options {
    fn default() -> Self { Self { doc:None,path:None,dirty:false,last_attempt:None,page:0,capture:None,pad:None,pad_error:String::new(),confirm_close:false,raw_text:String::new(),raw_active:false,raw_dirty:false,disk_text:String::new(),language_request:None } }
}
impl Options {
    pub fn take_language_request(&mut self) -> Option<(String, PathBuf)> {
        let code = self.language_request.take()?;
        let root = self.path.as_ref()?.parent()?.parent()?.to_path_buf();
        Some((code, root))
    }
    pub fn apply_game_language(&mut self, code: &str) {
        if let Some(doc) = self.doc.as_mut() { doc.set("General", "Language", code); self.dirty = true; }
    }
    pub fn has_changes(&self) -> bool { self.dirty || self.raw_dirty }
    pub fn enter_ini(&mut self, root: &str, status: &mut String) -> bool {
        self.autosave(status);
        if self.dirty { return false; }
        // Read the current file each time the editor is opened.
        if let Err(error) = self.load(root) {
            *status=tf("settings.unavailable", &[("error", error.to_string())]);
            return false;
        }
        self.raw_text=self.disk_text.clone(); self.raw_active=true;
        self.raw_dirty=false; self.capture=None;
        true
    }
    fn save_raw(&mut self, status: &mut String) -> bool {
        if !self.raw_dirty { return true; }
        let doc=Ini::parse(&self.raw_text);
        let Some(path)=self.path.as_ref() else { return false; };
        match validate(&doc).and_then(|_| core::save_ini(path, &self.raw_text)) {
            Ok(()) => {
                self.doc=Some(doc); self.disk_text=self.raw_text.clone();
                self.raw_dirty=false; self.dirty=false; self.last_attempt=None;
                *status=tr("settings.ini_saved").into(); true
            }
            Err(error) => {
                *status=tf("settings.auto_failed", &[("error", error.to_string())]); false
            }
        }
    }
    pub fn leave_ini(&mut self, status: &mut String) -> bool {
        if !self.save_raw(status) { return false; }
        self.raw_active=false; true
    }
    pub fn show_ini(&mut self, ui: &mut egui::Ui, busy: bool) {
        ui.label(tr("settings.ini_hint"));
        if let Some(path)=&self.path {
            ui.small(tf("settings.file", &[("path", path.display().to_string())]));
        }
        ui.add_enabled_ui(!busy, |ui| {
            egui::ScrollArea::both().id_salt("ini-scroll").show(ui, |ui| {
                if ui.add(egui::TextEdit::multiline(&mut self.raw_text)
                    .id_salt("ini-editor").font(egui::TextStyle::Monospace)
                    .code_editor().desired_width(f32::INFINITY).desired_rows(26)).changed() {
                    self.raw_dirty=self.raw_text != self.disk_text;
                }
            });
        });
    }
    pub fn cancel_capture(&mut self) { self.capture = None; }
    pub fn select_page(&mut self, page: usize) {
        self.page = page.min(4); self.capture = None;
    }
    pub fn autosave(&mut self, status: &mut String) {
        if self.raw_active || !self.dirty { return; }
        let (Some(doc), Some(path)) = (&self.doc, &self.path) else { return; };
        let text = doc.text();
        // Retry only after another edit or an explicit retry, not every repaint.
        if self.last_attempt.as_ref() == Some(&text) { return; }
        self.last_attempt = Some(text.clone());
        match validate(doc).and_then(|_| core::save_ini(path, &text)) {
            Ok(()) => { self.dirty = false; self.last_attempt = None; self.disk_text = text; *status = tr("settings.auto_saved").into(); }
            Err(error) => *status = tf("settings.auto_failed", &[("error", error.to_string())]),
        }
    }
    pub fn close_guard(&mut self, ctx: &egui::Context, status: &mut String) {
        if self.has_changes() && ctx.input(|i| i.viewport().close_requested()) {
            if self.raw_active { self.save_raw(status); } else { self.autosave(status); }
            if self.has_changes() { ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose); self.confirm_close = true; }
        }
        if self.confirm_close {
            egui::Window::new(tr("settings.auto_pending")).collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label(tr("settings.auto_close_hint"));
                if ui.button(tr("settings.retry")).clicked() {
                    self.last_attempt = None;
                    if self.raw_active { self.save_raw(status); } else { self.autosave(status); }
                    if !self.has_changes() { self.confirm_close = false; ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                }
                if ui.button(tr("text.close_without_saving")).clicked() {
                    self.dirty = false; self.raw_dirty = false; self.confirm_close = false; ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button(tr("text.continue")).clicked() { self.confirm_close = false; }
            });
        }
    }
    fn load(&mut self, root:&str) -> core::Result<()> {
        let path=core::Launcher::new(root)?.repo().join("zelda3.ini");
        let text=fs::read_to_string(&path)?;
        self.doc=Some(Ini::parse(&text)); self.path=Some(path); self.dirty=false; self.last_attempt=None; self.capture=None; self.disk_text=text; Ok(())
    }
    fn apply_capture(&mut self, value:String) {
        if let Some(mut capture)=self.capture.take() {
            if let Some(doc)=self.doc.as_mut() { set_binding(doc,&capture.target,&value); self.dirty=true; }
            if capture.sequential && capture.target.index+1<capture.target.count {
                capture.target.index+=1; capture.since=Instant::now(); self.capture=Some(capture);
            }
        }
    }
    pub fn show(&mut self, ui:&mut egui::Ui, ctx:&egui::Context, root:&str, busy:bool, status:&mut String) {
        if self.doc.is_none() && !busy { if let Err(e)=self.load(root) { *status=tf("settings.unavailable", &[("error", e.to_string())]); } }
        if let Some(path) = &self.path {
            let expected = crate::platform::canonicalize(root).ok().map(|p| p.join("zelda3/zelda3.ini"));
            if expected.as_ref() != Some(path) {
                if self.dirty {
                    ui.colored_label(egui::Color32::YELLOW, tr("settings.workspace_pending"));
                } else {
                    self.doc = None; self.path = None; self.capture = None;
                    if !busy { if let Err(e) = self.load(root) { *status = e.to_string(); } }
                }
            }
        }
        ui.label(tr("text.settings_will_take_effect_the_next_time_you_launch"));
        ui.horizontal(|ui| {
            if self.dirty {
                ui.colored_label(egui::Color32::YELLOW, tr("settings.auto_pending"));
                if ui.add_enabled(!busy, egui::Button::new(tr("settings.retry"))).clicked() { self.last_attempt=None; self.autosave(status); }
                if ui.add_enabled(!busy, egui::Button::new(tr("text.discard_changes"))).clicked() { if let Err(e)=self.load(root) { *status=e.to_string(); } }
            }
        });
        if self.doc.is_none() { return; }
        if busy { self.capture=None; }
        if let Some(pad)=self.pad.as_mut() {
            let events=pad.poll(); if !busy && self.capture.as_ref().is_some_and(|c| c.target.section=="GamepadMap") { if let Some(value)=events.into_iter().next() {self.apply_capture(value);} }
        }
        if let Some(c)=&self.capture {
            ctx.request_repaint_after(Duration::from_millis(16));
            if c.since.elapsed()>Duration::from_secs(15) {self.capture=None;*status=tr("text.capture_timed_out_the_previous_binding_was_kept").into();}
        }
        if self.capture.is_some() {
            let events=ctx.input(|i| i.events.clone());
            ctx.input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Key { .. } | egui::Event::Text(_))));
            for event in events { if let egui::Event::Key {key,pressed:true,repeat:false,modifiers,..}=event {
                if key==egui::Key::Escape { self.capture=None;break; }
                if self.capture.as_ref().is_some_and(|c| c.target.section=="KeyMap") {
                    if let Some(value)=keyboard_name(key,modifiers) {self.apply_capture(value);break;}
                }
            } }
        }
        if self.capture.is_some() {
            if let Some(c) = &self.capture {
                let label = if c.target.key == "Controls" { control_labels()[c.target.index].to_owned() } else { format!("{} {}", c.target.key, c.target.index + 1) };
                ui.colored_label(egui::Color32::YELLOW,tf("mapping.waiting", &[("label", label)]));
            }
            if ui.button(tr("text.cancel_capture")).clicked() {self.capture=None;}
        }
        let mut requested=None;
        ui.add_enabled_ui(!busy, |ui| {
            egui::ScrollArea::vertical().id_salt("options-scroll").show(ui, |ui| {
                let doc=self.doc.as_mut().unwrap();
                match self.page {
                    0=>game(ui,doc,&mut self.dirty,&mut self.language_request),
                    1=>graphics(ui,doc,&mut self.dirty),
                    2=>sound(ui,doc,&mut self.dirty),
                    3=>{
                        stacked_settings(ui, |ui| {
                            for pad in [false, true] {
                                let title=tr(if pad { "text.gamepad" } else { "text.keyboard" });
                                settings_section(ui, title, |ui| {
                                    ui.push_id(if pad { "GamepadMap" } else { "KeyMap" }, |ui| {
                                        controls_bindings(ui,doc,&mut self.dirty,pad,&mut self.pad,&mut self.pad_error,&mut requested);
                                    });
                                });
                            }
                        });
                    },
                    _=>shortcuts(ui,doc,&mut self.dirty,&mut requested),
                }
            });
        });
        if let Some((target,sequential))=requested {
            if target.section == "GamepadMap" && self.pad.is_none() {
                match Pad::new() { Ok(p) => self.pad = Some(p), Err(e) => { self.pad_error = e.clone(); *status = tf("mapping.gamepad_unavailable", &[("error", e.to_string())]); return; } }
            }
            // Drain stale SDL events before waiting for the next intentional input.
            if let Some(p)=self.pad.as_mut() {let _=p.poll();}
            if let Some(id) = ctx.memory(|m| m.focused()) { ctx.memory_mut(|m| m.surrender_focus(id)); }
            self.capture=Some(Capture {target,sequential,since:Instant::now()});ctx.request_repaint();
        }
    }
}
fn controls_bindings(ui: &mut egui::Ui, doc: &mut Ini, dirty: &mut bool, pad: bool,
    pad_state: &mut Option<Pad>, pad_error: &mut String, requested: &mut Option<(Target,bool)>) {
    let section=if pad {"GamepadMap"} else {"KeyMap"}; let default=if pad {PAD_DEFAULT} else {KEY_DEFAULT};
    if pad {
        ui.label(tr("text.sdl_names_a_b_x_y_refer_to_logical"));
        if pad_state.is_none() { match Pad::new() {Ok(p)=>*pad_state=Some(p),Err(e)=>*pad_error=e} }
        if let Some(p)=pad_state.as_ref() { for c in &p.controllers {ui.label(tf("mapping.gamepad", &[("name", c.name())]));} if p.controllers.is_empty() {ui.label(tr("text.no_sdl_gamepad_detected_manual_selection_is_available"));} }
        if !pad_error.is_empty() {ui.label(tf("mapping.unavailable", &[("error", pad_error.clone())]));}
    } else { ui.label(tr("text.ctrl_alt_and_shift_combinations_are_supported_enter_standalone"));
        ui.horizontal(|ui| { for (label,keys) in [("QWERTY",KEY_DEFAULT),("AZERTY","Up, Down, Left, Right, Right Shift, Return, x, w, s, q, c, v"),("QWERTZ","Up, Down, Left, Right, Right Shift, Return, x, y, s, a, c, v")] { if ui.button(label).on_hover_text(tr("help.action.preset")).clicked() {doc.set(section,"Controls",keys);*dirty=true;} } });
    }
    if pad && ui.button(tr("text.restore_default_gamepad_bindings")).on_hover_text(tr("help.action.restore_pad")).clicked() { doc.set(section,"Controls",PAD_DEFAULT); *dirty=true; }
    if ui.button(tr("text.assign_all_controls")).on_hover_text(tr("help.action.assign_all")).clicked() { *requested=Some((Target {section:section.into(),key:"Controls".into(),index:0,count:12,default:default.into()},true)); }
    split_columns(ui, |columns| {
    for (i,label) in control_labels().iter().enumerate() {
        let t=Target {section:section.into(),key:"Controls".into(),index:i,count:12,default:default.into()};
        binding_row(&mut columns[i / 6],doc,dirty,label,&t,pad,requested);
    }
    });
}
fn set_binding(doc:&mut Ini,target:&Target,value:&str) {
    let mut values:Vec<String>=doc.value(&target.section,&target.key,&target.default).split(',').map(|s|s.trim().to_owned()).collect();
    values.resize(target.count,String::new()); values[target.index]=value.to_owned();
    doc.set(&target.section,&target.key,&values.join(", "));
}
fn help(section: &str, key: &str) -> &'static str {
    let section = if section == "KeyMap" || section == "GamepadMap" { "binding" } else { section };
    tr(&format!("help.{section}.{key}"))
}
fn binding_row(ui:&mut egui::Ui,doc:&mut Ini,dirty:&mut bool,label:&str,t:&Target,pad:bool,request:&mut Option<(Target,bool)>) {
    let raw=doc.value(&t.section,&t.key,&t.default); let mut value=raw.split(',').nth(t.index).unwrap_or("").trim().to_owned();
    ui.push_id((&t.section,&t.key,t.index),|ui| {ui.horizontal_wrapped(|ui| {
        ui.add_sized([ui.available_width().min(130.0),22.0],egui::Label::new(label)).on_hover_text(help(&t.section, &t.key));
        if pad {
            let old=value.clone();egui::ComboBox::from_id_salt("button").selected_text(if value.is_empty(){tr("text.unassigned")}else{&value}).show_ui(ui,|ui|{ui.selectable_value(&mut value,String::new(),tr("text.unassigned"));for button in PAD_KEYS {ui.selectable_value(&mut value,button.into(),button);} }).response.on_hover_text(help(&t.section, &t.key));
            if old!=value {set_binding(doc,t,&value);*dirty=true;}
        } else if ui.add(egui::TextEdit::singleline(&mut value).desired_width(ui.available_width().min(120.0))).on_hover_text(help(&t.section, &t.key)).changed() && !value.contains(',') && !value.contains('\n') {set_binding(doc,t,&value);*dirty=true;}
        if ui.button(tr("text.capture")).on_hover_text(tr("help.action.capture")).clicked() {*request=Some((t.clone(),false));}
        if ui.button(tr("text.clear")).on_hover_text(tr("help.action.clear")).clicked() {set_binding(doc,t,"");*dirty=true;}
    });});
}
fn flag(ui:&mut egui::Ui,doc:&mut Ini,dirty:&mut bool,section:&str,key:&str,label:&str,default:bool) {
    let raw=doc.value(section,key,if default {"1"}else{"0"}); let mut value=raw=="1"||raw.eq_ignore_ascii_case("true");
    if ui.checkbox(&mut value,label).on_hover_text(help(section,key)).changed() {doc.set(section,key,if value {"1"}else{"0"});*dirty=true;}
}
fn choice(ui:&mut egui::Ui,doc:&mut Ini,dirty:&mut bool,section:&str,key:&str,label:&str,default:&str,values:&[(&str,&str)]) {
    let mut value=doc.value(section,key,default);let old=value.clone();
    ui.horizontal(|ui|{ui.label(label).on_hover_text(help(section,key));egui::ComboBox::from_id_salt((section,key)).selected_text(values.iter().find(|(v,_)|*v==value).map(|(_,l)|*l).unwrap_or(&value)).show_ui(ui,|ui|{for (v,l) in values {ui.selectable_value(&mut value,(*v).into(),*l).on_hover_text(help(section,key));} }).response.on_hover_text(help(section,key));});
    if value!=old {doc.set(section,key,&value);*dirty=true;}
}
fn text(ui:&mut egui::Ui,doc:&mut Ini,dirty:&mut bool,section:&str,key:&str,label:&str,default:&str) {
    let mut value=doc.value(section,key,default);ui.horizontal(|ui| {ui.label(label).on_hover_text(help(section,key));if ui.text_edit_singleline(&mut value).on_hover_text(help(section,key)).changed() {doc.set(section,key,&value);*dirty=true;} });
}
fn file(ui:&mut egui::Ui,doc:&mut Ini,dirty:&mut bool,key:&str,label:&str,extensions:&[&str]) {
    ui.push_id(key,|ui| {text(ui,doc,dirty,"Graphics",key,label,"");
        if ui.button(tr("text.choose_a_file")).on_hover_text(help("Graphics",key)).clicked() {
            let mut dialog=rfd::FileDialog::new().add_filter(label,extensions);
            if key == "Shader" {
                let directory=crate::platform::shaders_dir();
                if fs::create_dir_all(&directory).is_ok() { dialog=dialog.set_directory(directory); }
            }
            if let Some(path)=dialog.pick_file() {
                if let Some(value)=path.to_str() {
                    if key == "Shader" && !shader_path_allowed(value) { return; }
                    doc.set("Graphics",key,value);*dirty=true;
                }
            }
        }
    });
}
fn shader_path_allowed(value: &str) -> bool {
    value.is_empty() || std::path::Path::new(value).extension().and_then(|s|s.to_str())
        .is_some_and(|ext|ext.eq_ignore_ascii_case("glsl") || ext.eq_ignore_ascii_case("glslp"))
}

// Two-column layout with the vertical gold divider from the approved mockup.
// Center the separator in a reserved gutter, without covering either column.
fn split_columns<R>(ui: &mut egui::Ui, contents: impl FnOnce(&mut [egui::Ui]) -> R) -> R {
    ui.columns(2, |columns| {
        let gap = 32.0;
        let left_edge = columns[0].max_rect().right();
        let right_edge = columns[1].max_rect().left();
        let center = (left_edge + right_edge) * 0.5;
        // Constrain both content columns, leaving 16 px on each side of the line.
        let left_width = columns[0].available_width();
        columns[0].set_max_width((left_width - gap * 0.5).max(80.0));
        let right_start = columns[1].cursor().min;
        columns[1].set_cursor(egui::pos2(right_start.x + gap * 0.5, right_start.y));
        let result = contents(columns);
        let left = columns[0].min_rect();
        let right = columns[1].min_rect();
        let top = left.top().min(right.top());
        let bottom = left.bottom().max(right.bottom());
        if bottom > top {
            columns[0].painter().line_segment(
                [egui::pos2(center, top), egui::pos2(center, bottom)],
                egui::Stroke::new(1.0, crate::theme::GOLD),
            );
        }
        result
    })
}
fn settings_frame(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui)) {
    let width=(ui.available_width() - 24.0).max(0.0);
    egui::Frame::new().fill(egui::Color32::from_rgba_unmultiplied(4, 42, 28, 215))
        .stroke(egui::Stroke::new(1.0_f32, crate::theme::GOLD))
        .corner_radius(egui::CornerRadius::same(10)).inner_margin(egui::Margin::symmetric(16, 20))
        .show(ui, |ui| {
            ui.set_min_width(width);
            contents(ui);
        });
}
fn settings_section(ui: &mut egui::Ui, title: &str, contents: impl FnOnce(&mut egui::Ui)) {
    settings_frame(ui, |ui| {
        ui.label(egui::RichText::new(title).size(18.0).strong().color(crate::theme::GOLD));
        ui.separator();
        contents(ui);
    });
}
// Use nearly the full tab width, matching the approved forest mockup.
fn stacked_settings(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width() * 0.97;
    let margin = (ui.available_width() - width) * 0.5;
    ui.horizontal(|ui| {
        ui.add_space(margin);
        ui.allocate_ui_with_layout(egui::vec2(width, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(width);
            contents(ui);
        });
    });
}
fn game(ui:&mut egui::Ui,doc:&mut Ini,dirty:&mut bool,request:&mut Option<String>) {
    stacked_settings(ui, |ui| {
        settings_section(ui, tr("text.general"), |ui| {
            split_columns(ui, |columns| {
                columns[0].vertical(|ui| {
            for (key,label) in [("Autosave",tr("text.automatically_save_state_on_exit")),("DisplayPerfInTitle",tr("text.show_fps_in_the_window_title")),("DisableFrameDelay",tr("text.disable_frame_delay"))] {flag(ui,doc,dirty,"General",key,label,false);}
                });
                columns[1].vertical(|ui| {
            let original = doc.value("General", "Language", "");
            let mut selected = original.clone();
            let languages = [("", tr("text.us_english")), ("en", tr("game_language.european_english")), ("fr",tr("text.french")), ("fr-c",tr("text.canadian_french")), ("de",tr("text.german")), ("es",tr("text.spanish")), ("pl",tr("text.polish")), ("pt",tr("game_language.portuguese")), ("nl",tr("text.dutch")), ("sv",tr("text.swedish")), ("redux","Redux")];
            ui.vertical(|ui| {
                ui.label(tr("text.game_language_matching_assets_required")).on_hover_text(help("General","Language"));
                egui::ComboBox::from_id_salt("game-language").selected_text(languages.iter().find(|(code,_)|*code==selected).map(|(_,name)|*name).unwrap_or(&selected)).show_ui(ui, |ui| {
                    for (code,name) in languages { ui.selectable_value(&mut selected, code.to_owned(), name).on_hover_text(help("General","Language")); }
                }).response.on_hover_text(help("General","Language"));
            });
            if selected != original {
                if selected.is_empty() { doc.set("General", "Language", ""); *dirty = true; }
                else { *request = Some(selected); }
            }
            if !original.is_empty() && ui.button(tr("game_language.reimport")).on_hover_text(tr("help.action.reimport")).clicked() { *request = Some(original); }
            ui.small(tr("game_language.hint"));
                });
            });
        });
        settings_section(ui, tr("text.gameplay_enhancements"), |ui| {
            split_columns(ui, |columns| {
            for (index, (key,label)) in [
                ("ItemSwitchLR",tr("text.advanced_item_selection_with_l_r")),("ItemSwitchLRLimit",tr("text.limit_cycling_to_the_first_four_items")),
                ("TurnWhileDashing",tr("text.allow_turning_while_dashing")),("MirrorToDarkworld",tr("text.allow_mirror_travel_to_the_dark_world")),
                ("CollectItemsWithSword",tr("text.collect_items_with_the_sword")),("BreakPotsWithSword",tr("text.break_pots_with_the_sword")),
                ("DisableLowHealthBeep",tr("text.disable_the_low_health_alarm")),("SkipIntroOnKeypress",tr("text.skip_the_intro_on_keypress")),
                ("ShowMaxItemsInYellow",tr("text.show_maximum_resources_in_yellow")),("MoreActiveBombs",tr("text.allow_up_to_four_active_bombs")),
                ("CarryMoreRupees",tr("text.carry_up_to_9_999_rupees")),("MiscBugFixes",tr("text.minor_bug_fixes")),
                ("GameChangingBugFixes",tr("text.bug_fixes_that_change_gameplay")),("CancelBirdTravel",tr("text.cancel_bird_travel_with_x"))
            ].into_iter().enumerate() {flag(&mut columns[index / 7],doc,dirty,"Features",key,label,false);}
            });
        });
    });
}
fn graphics(ui:&mut egui::Ui,doc:&mut Ini,dirty:&mut bool) {
    stacked_settings(ui, |ui| {
        settings_section(ui, tr("text.general"), |ui| {
            split_columns(ui, |columns| {
            columns[0].vertical(|ui| {
            let raw=doc.value("General","ExtendedAspectRatio","4:3");let parts:Vec<&str>=raw.split(',').map(str::trim).collect();
            let mut ratio=parts.iter().find(|p|p.contains(':')).copied().unwrap_or("4:3").to_owned();
            let mut extended=parts.contains(&"extend_y");let mut unchanged=parts.contains(&"unchanged_sprites");let mut nofix=parts.contains(&"no_visual_fixes");
            let old=(ratio.clone(),extended,unchanged,nofix);
            egui::ComboBox::from_id_salt("ratio").selected_text(&ratio).show_ui(ui,|ui|{for r in ["4:3","16:9","16:10","18:9"] {ui.selectable_value(&mut ratio,r.into(),r);} }).response.on_hover_text(tr("help.aspect.ratio"));
            ui.checkbox(&mut extended,tr("text.extend_height_to_240_lines")).on_hover_text(tr("help.aspect.extend"));ui.checkbox(&mut unchanged,tr("text.preserve_original_sprite_behavior")).on_hover_text(tr("help.aspect.sprites"));ui.checkbox(&mut nofix,tr("text.disable_visual_fixes")).on_hover_text(tr("help.aspect.fixes"));
            if old!=(ratio.clone(),extended,unchanged,nofix) {let mut p=Vec::new();if extended {p.push("extend_y".to_owned());}p.push(ratio);if unchanged {p.push("unchanged_sprites".into());}if nofix {p.push("no_visual_fixes".into());}doc.set("General","ExtendedAspectRatio",&p.join(", "));*dirty=true;}
            });
            columns[1].vertical(|ui| {
            text(ui,doc,dirty,"Graphics","WindowSize",tr("text.window_size_auto_or_widthxheight"),"Auto");
            choice(ui,doc,dirty,"Graphics","Fullscreen",tr("text.fullscreen_mode"),"0",&[("0",tr("text.windowed")),("1",tr("text.borderless_fullscreen")),("2",tr("text.exclusive_fullscreen"))]);
            choice(ui,doc,dirty,"Graphics","WindowScale",tr("text.window_scale"),"3",&[("1","100 %"),("2","200 %"),("3","300 %"),("4","400 %"),("5","500 %"),("6","600 %")]);
            choice(ui,doc,dirty,"Graphics","OutputMethod",tr("text.renderer"),crate::core::DEFAULT_RENDERER,&[("SDL","SDL"),("SDL-Software",tr("text.sdl_software")),("OpenGL","OpenGL"),("OpenGL ES","OpenGL ES")]);
            });
            });
        });
        settings_section(ui, tr("text.renderer"), |ui| {
            split_columns(ui, |columns| {
            columns[0].vertical(|ui| {
            for (key,label,default) in [("NewRenderer",tr("text.optimized_snes_ppu"),true),("EnhancedMode7",tr("text.high_resolution_world_map_mode_7"),true),("IgnoreAspectRatio",tr("text.stretch_image"),false),("NoSpriteLimits",tr("text.remove_sprite_limits"),true),("LinearFiltering",tr("text.linear_filtering"),false),("DimFlashes",tr("text.reduce_flashing"),false)] {flag(ui,doc,dirty,"Graphics",key,label,default);}
            });
            columns[1].vertical(|ui| {
            file(ui,doc,dirty,"LinkGraphics",tr("text.link_sprite_zspr"),&["zspr"]);
            let renderer=doc.value("Graphics","OutputMethod",crate::core::DEFAULT_RENDERER);
            let shaders_enabled=renderer.eq_ignore_ascii_case("OpenGL") || renderer.eq_ignore_ascii_case("OpenGL ES");
            let shader_controls=ui.add_enabled_ui(shaders_enabled, |ui| {
                file(ui,doc,dirty,"Shader",tr("text.opengl_shader_glsl_glslp"),&["glsl","glslp","GLSL","GLSLP"]);
            });
            if !shaders_enabled { shader_controls.response.on_hover_text(tr("shaders.requires_opengl")); }
            });
            });
        });
    });
}
fn sound(ui:&mut egui::Ui,doc:&mut Ini,dirty:&mut bool) {
    stacked_settings(ui, |ui| {
        settings_section(ui, tr("text.sound"), |ui| {
            split_columns(ui, |columns| {
            columns[0].vertical(|ui| {
            flag(ui,doc,dirty,"Sound","EnableAudio",tr("text.enable_audio"),true);
            choice(ui,doc,dirty,"Sound","AudioChannels",tr("text.channels"),"2",&[("1","Mono"),("2",tr("text.stereo"))]);
            });
            columns[1].vertical(|ui| {
            choice(ui,doc,dirty,"Sound","AudioFreq",tr("text.sample_rate"),"44100",&[("11025","11025 Hz"),("22050","22050 Hz"),("32000","32000 Hz"),("44100","44100 Hz"),("48000","48000 Hz")]);
            choice(ui,doc,dirty,"Sound","AudioSamples",tr("text.buffer_size"),"512",&[("256","256"),("512","512"),("1024","1024"),("2048","2048"),("4096","4096")]);
            });
            });
        });
        settings_section(ui, tr("text.msu_music"), |ui| {
            split_columns(ui, |columns| {
            columns[0].vertical(|ui| {
            choice(ui,doc,dirty,"Sound","EnableMSU",tr("text.format"),"false",&[("false",tr("text.disabled")),("true","MSU PCM"),("deluxe","MSU Deluxe PCM"),("opuz","OPUZ"),("deluxe-opuz","OPUZ Deluxe")]);
            ui.small(tr("text.pcm_requires_44100_hz_opuz_requires_48000_hz_music"));
            });
            columns[1].vertical(|ui| {
            flag(ui,doc,dirty,"Sound","ResumeMSU",tr("text.resume_music_from_its_previous_position"),true);
            let mut volume=doc.value("Sound","MSUVolume","100%").trim_end_matches('%').parse::<u32>().unwrap_or(100);
            if ui.add(egui::Slider::new(&mut volume,0..=100).text(tr("text.msu_volume"))).on_hover_text(help("Sound","MSUVolume")).changed() {doc.set("Sound","MSUVolume",&format!("{volume}%"));*dirty=true;}
            });
            });
            text(ui,doc,dirty,"Sound","MSUPath",tr("text.music_track_path_prefix"),"msu/alttp_msu-");
        });
    });
}
fn shortcuts(ui:&mut egui::Ui,doc:&mut Ini,dirty:&mut bool,request:&mut Option<(Target,bool)>) {
    stacked_settings(ui, |ui| {
    ui.label(tr("text.keyboard_and_gamepad_bindings_are_independent_an_empty_field"));
    for section in ["KeyMap","GamepadMap"] {settings_section(ui, if section=="KeyMap" {tr("text.keyboard_shortcuts")} else {tr("text.gamepad_shortcuts")},|ui| {
        split_columns(ui, |columns| {
        for (index,(key,label,default)) in [("Reset",tr("text.reset_game"),"Ctrl+r"),("Pause",tr("mapping.pause"),"Shift+p"),("PauseDimmed",tr("text.pause_and_dim"),"p"),("Fullscreen",tr("text.fullscreen"),"Alt+Return"),("WindowBigger",tr("text.increase_window_size"),"Ctrl+Up"),("WindowSmaller",tr("text.decrease_window_size"),"Ctrl+Down"),("VolumeUp",tr("text.volume_up"),"Shift+="),("VolumeDown",tr("text.volume_down"),"Shift+-"),("CheatLife",tr("text.restore_health_and_magic"),"w"),("CheatKeys",tr("text.give_one_key"),"o"),("CheatWalkThroughWalls",tr("text.walk_through_walls"),"Ctrl+e"),("Turbo",tr("mapping.turbo"),"Tab"),("ReplayTurbo",tr("text.replay_speed"),"t"),("StopReplay",tr("text.stop_replay"),"l"),("ClearKeyLog",tr("text.clear_key_log"),"k"),("ToggleRenderer",tr("text.toggle_ppu"),""),("DisplayPerf",tr("text.show_performance"),"")].into_iter().enumerate() {
            let t=Target {section:section.into(),key:key.into(),index:0,count:1,default:if section=="KeyMap" {default.into()}else{String::new()}};
            binding_row(&mut columns[index / 9],doc,dirty,label,&t,section=="GamepadMap",request);
        }
        });
        for (key,label,prefix) in [("Load",tr("text.load"),""),("Save",tr("text.save_state"),"Shift+"),("Replay",tr("mapping.replay"),"Ctrl+")] {
            ui.separator();ui.label(label);
            let default=if section=="KeyMap" {(1..=10).map(|i|format!("{prefix}F{i}")).collect::<Vec<_>>().join(", ")}else{",".repeat(9)};
            split_columns(ui, |columns| {
            for index in 0..10 {let t=Target {section:section.into(),key:key.into(),index,count:10,default:default.clone()};binding_row(&mut columns[index / 5],doc,dirty,&tf("mapping.slot", &[("slot", (index+1).to_string())]),&t,section=="GamepadMap",request);}
            });
        }
    });}
    });
}
fn keyboard_name(key:egui::Key,m:egui::Modifiers) -> Option<String> {
    use egui::Key;
    if m.mac_cmd { return None; }
    let name: String=match key {
        Key::ArrowUp=>"Up".into(),Key::ArrowDown=>"Down".into(),Key::ArrowLeft=>"Left".into(),Key::ArrowRight=>"Right".into(),
        Key::Enter=>"Return".into(),Key::Space=>"Space".into(),Key::Backspace=>"Backspace".into(),Key::Delete=>"Delete".into(),Key::Insert=>"Insert".into(),Key::Home=>"Home".into(),Key::End=>"End".into(),Key::PageUp=>"PageUp".into(),Key::PageDown=>"PageDown".into(),Key::Tab=>"Tab".into(),
        Key::Comma=>return None, Key::Plus=>"=".into(),Key::Minus=>"-".into(),Key::Period=>".".into(),Key::Slash=>"/".into(),Key::Backslash=>"\\".into(),Key::Semicolon=>";".into(),Key::Quote=>"'".into(),Key::OpenBracket=>"[".into(),Key::CloseBracket=>"]".into(),Key::Backtick=>"`".into(),Key::Equals=>"=".into(),
        _=>{let s=format!("{key:?}");if s.len()==1 {s.to_lowercase()}else if s.starts_with("Num")&&s.len()==4 {s[3..].into()}else if s.starts_with('F')&&s[1..].parse::<u8>().is_ok() {s}else{return None;}}
    };
    let mut result=String::new();if m.ctrl {result.push_str("Ctrl+");}if m.alt {result.push_str("Alt+");}if m.shift {result.push_str("Shift+");}result.push_str(&name);Some(result)
}

fn validate(doc: &Ini) -> core::Result<()> {
    if !shader_path_allowed(&doc.value("Graphics","Shader","")) {
        return Err(tr("shaders.invalid_format").into());
    }
    let size = doc.value("Graphics", "WindowSize", "Auto");
    if !size.eq_ignore_ascii_case("Auto") {
        let valid = size.split_once('x').is_some_and(|(w,h)| {
            w.parse::<u32>().is_ok_and(|n| n > 0 && n <= 16384) && h.parse::<u32>().is_ok_and(|n| n > 0 && n <= 16384)
        });
        if !valid { return Err(tr("text.invalid_window_size_use_auto_or_widthxheight_for_example").into()); }
    }
    for section in ["KeyMap", "GamepadMap"] {
        if let Some(controls) = doc.get(section,"Controls") {
            if controls.split(',').count()!=12 { return Err(tf("mapping.invalid_count", &[("section", section.to_owned())]).into()); }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ini_editor_saves_on_exit_and_updates_forms_without_losing_invalid_drafts() {
        let dir=std::env::temp_dir().join(format!("ini-editor-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&dir).unwrap(); let path=dir.join("zelda3.ini");
        let original="; keep comment\n[Graphics]\nWindowSize=Auto\n";
        fs::write(&path,original).unwrap();
        let mut options=Options::default();
        options.path=Some(path.clone()); options.doc=Some(Ini::parse(original));
        options.disk_text=original.into(); options.raw_active=true;
        options.raw_text="; keep comment\n[Graphics]\nWindowSize=1280x\n".into();
        options.raw_dirty=true;
        let mut status=String::new();
        options.autosave(&mut status);
        assert_eq!(fs::read_to_string(&path).unwrap(),original);
        assert!(!options.leave_ini(&mut status));
        assert!(options.raw_active && options.has_changes());
        assert_eq!(fs::read_to_string(&path).unwrap(),original);
        options.raw_text="; keep comment\n[Graphics]\nWindowSize=1280x720\nCustom=preserved\n".into();
        assert!(options.leave_ini(&mut status));
        assert!(!options.raw_active && !options.has_changes());
        assert_eq!(fs::read_to_string(&path).unwrap(),options.raw_text);
        assert_eq!(options.doc.as_ref().unwrap().get("Graphics","WindowSize").as_deref(),Some("1280x720"));
        assert_eq!(options.doc.as_ref().unwrap().get("Graphics","Custom").as_deref(),Some("preserved"));
        assert_eq!(fs::read_to_string(path.with_extension("ini.bak")).unwrap(),original);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test] fn shader_filter_rejects_slang_presets() {
        assert!(shader_path_allowed("")); assert!(shader_path_allowed("CRT.GLSLP"));
        assert!(shader_path_allowed("/tmp/shader.glsl"));
        assert!(!shader_path_allowed("crt.slangp")); assert!(!shader_path_allowed("crt.SLANGP"));
    }
    #[test] fn autosave_preserves_valid_file_until_an_invalid_edit_is_corrected() {
        let dir=std::env::temp_dir().join(format!("autosave-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&dir).unwrap();
        let path=dir.join("zelda3.ini");
        let original="# keep this comment\n[Graphics]\nWindowSize=Auto\n";
        fs::write(&path,original).unwrap();
        let mut options=Options::default();
        options.doc=Some(Ini::parse(original)); options.path=Some(path.clone());
        options.doc.as_mut().unwrap().set("Graphics","WindowSize","1280x"); options.dirty=true;
        let mut status=String::new(); options.autosave(&mut status);
        assert!(options.has_changes()); assert_eq!(fs::read_to_string(&path).unwrap(),original);
        assert!(!path.with_extension("ini.bak").exists());
        options.doc.as_mut().unwrap().set("Graphics","WindowSize","1280x720");
        options.autosave(&mut status); assert!(!options.has_changes());
        assert!(fs::read_to_string(&path).unwrap().contains("1280x720"));
        assert_eq!(fs::read_to_string(path.with_extension("ini.bak")).unwrap(),original);
        let target=Target {section:"KeyMap".into(),key:"Controls".into(),index:6,count:12,default:KEY_DEFAULT.into()};
        options.capture=Some(Capture {target,sequential:false,since:Instant::now()});
        options.apply_capture("Ctrl+q".into()); options.autosave(&mut status);
        assert!(!options.has_changes()); assert!(fs::read_to_string(&path).unwrap().contains("Ctrl+q"));
        options.apply_game_language("fr"); options.autosave(&mut status);
        assert!(!options.has_changes());
        let saved=Ini::parse(&fs::read_to_string(&path).unwrap()); assert_eq!(saved.get("General","Language").as_deref(),Some("fr"));
        fs::remove_dir_all(dir).unwrap();
    }
    #[test] fn binding_edits_only_selected_slot() {
        let mut doc=Ini::parse("[KeyMap]\n# preserved\nControls = Up, Down, Left, Right, Right Shift, Return, x, z, s, a, c, v\n[GamepadMap]\nControls = DpadUp, DpadDown, DpadLeft, DpadRight, Back, Start, B, A, Y, X, Lb, Rb\n");
        let t=Target {section:"KeyMap".into(),key:"Controls".into(),index:6,count:12,default:KEY_DEFAULT.into()};
        set_binding(&mut doc,&t,"Ctrl+q");
        assert_eq!(doc.get("GamepadMap","Controls").unwrap(),PAD_DEFAULT);
        assert_eq!(doc.get("KeyMap","Controls").unwrap().split(',').nth(6).unwrap().trim(),"Ctrl+q");
        assert!(doc.text().contains("# preserved"));
    }
    #[test] fn keyboard_capture_uses_sdl_names_and_modifiers() {
        let m=egui::Modifiers {ctrl:true,shift:true,..Default::default()};
        assert_eq!(keyboard_name(egui::Key::ArrowUp,m).as_deref(),Some("Ctrl+Shift+Up"));
        assert_eq!(keyboard_name(egui::Key::Enter,egui::Modifiers::default()).as_deref(),Some("Return"));
    }
    #[test] fn rejects_invalid_window_dimensions() {
        let mut doc=Ini::parse("[Graphics]\nWindowSize = invalid\n");assert!(validate(&doc).is_err());
        doc.set("Graphics","WindowSize","1280x720");assert!(validate(&doc).is_ok());
    }
}
