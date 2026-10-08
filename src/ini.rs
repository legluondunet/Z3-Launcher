// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
//! Line-preserving INI editing. Unknown keys and comments remain untouched.
#[derive(Clone, Default)]
pub struct Ini { lines: Vec<String>, newline: &'static str, trailing: bool }
impl Ini {
    pub fn parse(text: &str) -> Self {
        Self { lines: text.lines().map(str::to_owned).collect(), newline: if text.contains("\r\n") { "\r\n" } else { "\n" }, trailing: text.ends_with('\n') }
    }
    pub fn text(&self) -> String { let mut s = self.lines.join(self.newline); if self.trailing { s.push_str(self.newline); } s }
    pub fn get(&self, section: &str, key: &str) -> Option<String> {
        let mut current = ""; let mut result = None;
        for line in &self.lines {
            let t = line.trim();
            if t.starts_with('[') && t.ends_with(']') { current = &t[1..t.len()-1]; }
            else if current.eq_ignore_ascii_case(section) && !t.starts_with('#') && !t.starts_with(';') {
                if let Some((k,v)) = line.split_once('=') { if k.trim().eq_ignore_ascii_case(key) { result = Some(v.trim().to_owned()); } }
            }
        }
        result
    }
    pub fn value(&self, section: &str, key: &str, default: &str) -> String { self.get(section,key).unwrap_or_else(|| default.to_owned()) }
    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        let mut current = String::new(); let mut indices = Vec::new(); let mut insert = None;
        for (i,line) in self.lines.iter().enumerate() {
            let t = line.trim();
            if t.starts_with('[') && t.ends_with(']') {
                if current.eq_ignore_ascii_case(section) { insert = Some(i); }
                current = t[1..t.len()-1].to_owned();
                if current.eq_ignore_ascii_case(section) { insert = Some(i+1); }
            } else if current.eq_ignore_ascii_case(section) {
                insert = Some(i+1);
                if !t.starts_with('#') && !t.starts_with(';') {
                    if let Some((k,_)) = line.split_once('=') { if k.trim().eq_ignore_ascii_case(key) { indices.push(i); } }
                }
            }
        }
        if !indices.is_empty() {
            for i in indices { let prefix = self.lines[i].split_once('=').unwrap().0.to_owned(); self.lines[i] = format!("{prefix}= {value}"); }
        } else if let Some(i) = insert { self.lines.insert(i, format!("{key} = {value}")); }
        else { self.lines.push(String::new()); self.lines.push(format!("[{section}]")); self.lines.push(format!("{key} = {value}")); }
        self.trailing = true;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn preserves_comments_unknown_keys_and_line_endings() {
        let mut ini=Ini::parse("[General]\r\n# Keep me\r\nAutosave = 0\r\nFuture = xyz\r\n[KeyMap]\r\nPause=p\r\n");
        ini.set("General","Autosave","1"); ini.set("General","Language","fr");
        assert!(ini.text().contains("# Keep me\r\n")); assert!(ini.text().contains("Future = xyz\r\n"));
        assert_eq!(ini.get("general","autosave").as_deref(),Some("1"));
        assert_eq!(ini.get("KeyMap","Pause").as_deref(),Some("p"));
        assert!(ini.text().find("Language = fr").unwrap()<ini.text().find("[KeyMap]").unwrap());
    }
    #[test] fn leaves_commented_defaults_and_updates_duplicate_active_entries() {
        let mut ini=Ini::parse("[KeyMap]\n#Controls = q,w\nControls=a,b\nControls=c,d\n");
        ini.set("KeyMap","Controls","x,y");
        assert_eq!(ini.text().matches("= x,y").count(),2); assert!(ini.text().contains("#Controls = q,w"));
    }
}
