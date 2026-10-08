// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
//! JSON catalogs loaded once at startup. Strings are interned once, not per lookup.
use serde::Deserialize;
use std::{collections::BTreeMap, fs, io, path::PathBuf, sync::{OnceLock,RwLock}};
#[derive(Deserialize)]
struct Language { code:String, name:String }
#[derive(Deserialize)]
struct Translation { language:Language, messages:BTreeMap<String,String> }
struct Locale { name:String, messages:BTreeMap<String,&'static str> }
struct Catalog { locales:BTreeMap<String,Locale>, english:BTreeMap<String,&'static str>, warnings:Vec<(String,String)> }
static CATALOG:OnceLock<Catalog>=OnceLock::new();
static CURRENT:OnceLock<RwLock<String>>=OnceLock::new();
fn selection() -> &'static RwLock<String> { CURRENT.get_or_init(||RwLock::new("en".into())) }
fn preference() -> PathBuf { crate::platform::config_dir().join("language.txt") }
fn placeholders(text:&str) -> Vec<String> {
    let mut names=Vec::new();let mut remaining=text;
    while let Some(start)=remaining.find('{') {
        remaining=&remaining[start+1..];if let Some(end)=remaining.find('}') {names.push(remaining[..end].to_owned());remaining=&remaining[end+1..];}else{break;}
    }
    names.sort();names
}
fn install(locales:&mut BTreeMap<String,Locale>, file:Translation, baseline:&BTreeMap<String,String>) -> Result<(),String> {
    let code=&file.language.code;
    if code.is_empty() || !code.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_') {return Err("Invalid language code".into());}
    if file.language.name.trim().is_empty() {return Err("Empty language name".into());}
    let mut messages=BTreeMap::new();
    for (key,value) in file.messages {
        // Ignore unknown keys and incorrect placeholders; use English for these entries.
        if let Some(reference)=baseline.get(&key) {
            if placeholders(reference)==placeholders(&value) && !value.is_empty() {
                let value:&'static str=Box::leak(value.into_boxed_str());messages.insert(key,value);
            }
        }
    }
    locales.insert(file.language.code,Locale {name:file.language.name,messages});Ok(())
}
fn catalog() -> &'static Catalog {
    CATALOG.get_or_init(|| {
        let english:Translation=serde_json::from_str(include_str!("../locales/en.json")).expect("Embedded English catalog is invalid");
        let baseline=english.messages.clone();let mut locales=BTreeMap::new();let mut warnings=Vec::new();
        install(&mut locales,english,&baseline).expect("Invalid embedded English catalog");
        let english = locales["en"].messages.clone();
        let french:Translation=serde_json::from_str(include_str!("../locales/fr.json")).expect("Embedded French catalog is invalid");
        install(&mut locales,french,&baseline).expect("Invalid embedded French catalog");
        // First loaded directory has lowest priority. No need for a recompile.
        let mut directories=Vec::new();
        if let Ok(cwd)=std::env::current_dir() {directories.push(cwd.join("locales"));}
        if let Ok(exe)=std::env::current_exe() {if let Some(parent)=exe.parent(){directories.push(parent.join("locales"));}}
        directories.push(crate::platform::config_dir().join("locales"));
        let mut seen=Vec::new();
        for directory in directories {
            if let Ok(canonical)=fs::canonicalize(&directory) {if seen.contains(&canonical){continue;}seen.push(canonical);}
            let Ok(entries)=fs::read_dir(&directory) else {continue;};
            let mut paths:Vec<PathBuf>=entries.flatten().map(|e|e.path()).filter(|p|p.extension().and_then(|e|e.to_str())==Some("json")).collect();paths.sort();
            for path in paths {
                let result=fs::read_to_string(&path).map_err(|e|e.to_string()).and_then(|text|serde_json::from_str::<Translation>(&text).map_err(|e|e.to_string()));
                match result.and_then(|file|install(&mut locales,file,&baseline)) {
                    Ok(())=>{},Err(e)=>warnings.push((path.display().to_string(),e))
                }
            }
        }
        Catalog {locales,english,warnings}
    })
}
pub fn init() {
    let c=catalog();let saved=fs::read_to_string(preference()).unwrap_or_else(|_|"en".into());
    let code=if c.locales.contains_key(saved.trim()) {saved.trim()}else{"en"};
    *selection().write().unwrap_or_else(|e|e.into_inner())=code.to_owned();
}
pub fn current() -> String {selection().read().unwrap_or_else(|e|e.into_inner()).clone()}
pub fn languages() -> Vec<(String,String)> {catalog().locales.iter().map(|(code,l)|(code.clone(),l.name.clone())).collect()}
pub fn warnings() -> Vec<String> {catalog().warnings.iter().map(|(path,error)|tf("translations.ignored", &[("path",path.clone()),("error",error.clone())])).collect()}
pub fn set_language(code:&str) -> io::Result<()> {
    if !catalog().locales.contains_key(code) {return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown interface language"));}
    let path=preference();if let Some(parent)=path.parent(){fs::create_dir_all(parent)?;}
    fs::write(path,code)?;
    *selection().write().unwrap_or_else(|e|e.into_inner())=code.to_owned();Ok(())
}
pub fn tr(key:&str) -> &'static str {
    let c=catalog();let code=selection().read().unwrap_or_else(|e|e.into_inner());
    lookup(c, code.as_str(), key)
}
fn lookup(c:&Catalog,code:&str,key:&str) -> &'static str {
    c.locales.get(code).and_then(|l|l.messages.get(key)).copied()
        .or_else(||c.english.get(key).copied()).unwrap_or("[Missing translation]")
}
/// Named values are inserted once, without interpreting braces inside those values.
pub fn tf(key:&str,values:&[(&str,String)]) -> String {format_template(tr(key),values)}
fn format_template(template:&str,values:&[(&str,String)]) -> String {
    let mut result=String::new();let mut remaining=template;
    while let Some(start)=remaining.find('{') {
        result.push_str(&remaining[..start]);let after=&remaining[start+1..];
        let Some(end)=after.find('}') else {result.push_str(&remaining[start..]);return result;};
        let name=&after[..end];
        if let Some((_,value))=values.iter().find(|(key,_)|*key==name) {result.push_str(value);}else{result.push_str(&remaining[start..start+end+2]);}
        remaining=&after[end+1..];
    }
    result.push_str(remaining);result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn partial_catalog_and_bad_placeholders_are_safe() {
        let baseline=BTreeMap::from([("greeting".into(),"Hello {name}".into()),("save".into(),"Save".into())]);
        let mut locales=BTreeMap::new();
        let translated=Translation {language:Language {code:"es".into(),name:"Español".into()},messages:BTreeMap::from([("greeting".into(),"Hola {wrong}".into()),("save".into(),"Guardar".into())])};
        install(&mut locales,translated,&baseline).unwrap();
        assert!(!locales["es"].messages.contains_key("greeting"));assert_eq!(locales["es"].messages["save"],"Guardar");
    }
    #[test] fn missing_translation_and_partial_english_override_use_embedded_defaults() {
        let c=Catalog { locales:BTreeMap::from([("fr".into(),Locale {name:"Français".into(),messages:BTreeMap::new()}),("en".into(),Locale {name:"English".into(),messages:BTreeMap::new()})]), english:BTreeMap::from([("save".into(),"Save")]), warnings:Vec::new() };
        assert_eq!(lookup(&c,"fr","save"),"Save");
        assert_eq!(lookup(&c,"en","save"),"Save");
        assert_eq!(lookup(&c,"unknown","save"),"Save");
    }
    #[test] fn substitution_does_not_reinterpret_inserted_values() {
        assert_eq!(format_template("{name}: {error}",&[("name","{error}".into()),("error","failed".into())]),"{error}: failed");
    }
}
