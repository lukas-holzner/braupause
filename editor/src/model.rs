//! Laden, Speichern und Vorlagen für Sud-Dateien. Die Datei wird als
//! `serde_json::Value` (mit erhaltener Feldreihenfolge) bearbeitet, damit
//! unbekannte oder freie Felder (z. B. `brew:properties`) nie verloren gehen.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::validate::Issue;

pub const SCHEMA_REF: &str = "../schemas/brew-batch.schema.json";
pub const SCHEMA_VERSION: &str = "1.1.0";
pub const BREW_NS: &str = "https://heimbrauerei-sendling.de/schema/brew/v1#";

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Overview,
    Volumes,
    Ingredients,
    Steps,
    Photos,
    Tools,
    Json,
}

/// Formular-Zustand, der nicht ins JSON gehört.
pub struct DocUi {
    pub tab: Tab,
    pub json_buf: String,
    pub json_error: Option<String>,
    pub json_synced_gen: u64,
    pub json_edited: bool,
    pub draw_amount: f64,
    pub draw_type: String,
    pub draw_label: String,
    pub draw_destination: String,
    pub draw_timestamp: String,
    pub open_step: Option<usize>,
}

impl Default for DocUi {
    fn default() -> Self {
        Self {
            tab: Tab::Overview,
            json_buf: String::new(),
            json_error: None,
            json_synced_gen: u64::MAX,
            json_edited: false,
            draw_amount: 0.5,
            draw_type: "sample_consumption".into(),
            draw_label: "Probe".into(),
            draw_destination: String::new(),
            draw_timestamp: now_iso(),
            open_step: None,
        }
    }
}

pub struct BatchDoc {
    pub path: PathBuf,
    /// Dateiname ohne `.json` – muss dem `identifier` entsprechen.
    pub file_id: String,
    pub value: Value,
    /// Gesetzt, wenn die Datei kein gültiges JSON enthält.
    pub parse_error: Option<String>,
    pub dirty: bool,
    pub is_new: bool,
    pub generation: u64,
    pub trailing_newline: bool,
    pub issues: Vec<Issue>,
    pub issues_gen: u64,
    pub ui: DocUi,
}

impl BatchDoc {
    pub fn load(path: &Path) -> Self {
        let file_id = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut ui = DocUi::default();
        let (value, parse_error, trailing_newline) = match fs::read_to_string(path) {
            Ok(text) => {
                let nl = text.ends_with('\n');
                match serde_json::from_str::<Value>(&text) {
                    Ok(v) => (v, None, nl),
                    Err(e) => {
                        ui.json_buf = text;
                        ui.tab = Tab::Json;
                        (Value::Null, Some(format!("Ungültiges JSON: {e}")), nl)
                    }
                }
            }
            Err(e) => (Value::Null, Some(format!("Lesefehler: {e}")), true),
        };
        if parse_error.is_some() {
            ui.json_synced_gen = 0;
            ui.json_edited = true;
        }
        Self {
            path: path.to_path_buf(),
            file_id,
            value,
            parse_error,
            dirty: false,
            is_new: false,
            generation: 0,
            trailing_newline,
            issues: Vec::new(),
            issues_gen: u64::MAX,
            ui,
        }
    }

    pub fn new_from_template(path: PathBuf, value: Value) -> Self {
        let file_id = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            path,
            file_id,
            value,
            parse_error: None,
            dirty: true,
            is_new: true,
            generation: 1,
            trailing_newline: true,
            issues: Vec::new(),
            issues_gen: u64::MAX,
            ui: DocUi::default(),
        }
    }

    pub fn touch(&mut self) {
        self.dirty = true;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn get_str(&self, key: &str) -> &str {
        self.value.get(key).and_then(Value::as_str).unwrap_or("")
    }

    pub fn serialize(&self) -> Result<String, String> {
        let mut text = serde_json::to_string_pretty(&self.value).map_err(|e| e.to_string())?;
        if self.trailing_newline {
            text.push('\n');
        }
        Ok(text)
    }

    pub fn save(&mut self, bump_modified: bool) -> Result<(), String> {
        if let Some(err) = &self.parse_error {
            return Err(format!(
                "{}: {err} – bitte im JSON-Tab korrigieren und übernehmen.",
                self.file_id
            ));
        }
        if bump_modified && let Some(root) = self.value.as_object_mut() {
            set_after(root, "dateModified", Value::String(now_iso()), "dateCreated");
        }
        let text = self.serialize()?;
        fs::write(&self.path, text).map_err(|e| format!("{}: {e}", self.path.display()))?;
        self.dirty = false;
        self.is_new = false;
        self.generation = self.generation.wrapping_add(1);
        Ok(())
    }
}

/// Aktueller Zeitpunkt als ISO-8601 mit lokaler Zeitzone, z. B. `2026-11-17T17:00:00+01:00`.
pub fn now_iso() -> String {
    chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:00%:z")
        .to_string()
}

pub fn is_valid_datetime(s: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(s).is_ok()
}

pub fn parse_ts(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    chrono::DateTime::parse_from_rfc3339(s).ok()
}

pub fn qv(value: f64, unit: &str) -> Value {
    json!({ "@type": "QuantitativeValue", "value": value, "unitText": unit })
}

pub fn qv_get(v: &Value) -> Option<(f64, &str)> {
    Some((v.get("value")?.as_f64()?, v.get("unitText")?.as_str()?))
}

pub fn to_liters(v: &Value) -> Option<f64> {
    match qv_get(v)? {
        (x, "L") => Some(x),
        (x, "mL") => Some(x / 1000.0),
        _ => None,
    }
}

/// Fügt `key` ein; ist er neu, landet er direkt hinter `after` (sonst am Ende).
pub fn set_after(map: &mut Map<String, Value>, key: &str, value: Value, after: &str) {
    if map.contains_key(key) {
        map.insert(key.to_string(), value);
        return;
    }
    match map.keys().position(|k| k == after) {
        Some(idx) => {
            map.shift_insert(idx + 1, key.to_string(), value);
        }
        None => {
            map.insert(key.to_string(), value);
        }
    }
}

/// Positionen der Schritte fortlaufend ab 1 durchnummerieren.
/// Positionen der Schritte fortlaufend ab 1 durchnummerieren. `stepPosition` der
/// Fotos wandert mit; Fotos eines gelöschten Schritts gelten danach für den ganzen Sud.
pub fn renumber_steps(root: &mut Map<String, Value>) {
    let mut moved: Vec<(u64, u64)> = Vec::new();
    if let Some(Value::Array(steps)) = root.get_mut("recipeInstructions") {
        for (i, step) in steps.iter_mut().enumerate() {
            if let Some(obj) = step.as_object_mut() {
                let new = i as u64 + 1;
                if let Some(old) = obj.get("position").and_then(Value::as_u64) {
                    moved.push((old, new));
                }
                obj.insert("position".into(), Value::from(new));
            }
        }
    }
    let identity = moved.iter().all(|(o, n)| o == n);
    if let Some(Value::Array(images)) = root.get_mut("image")
        && !identity
    {
        for img in images.iter_mut().filter_map(Value::as_object_mut) {
            let Some(old) = img.get("stepPosition").and_then(Value::as_u64) else { continue };
            match moved.iter().find(|(o, _)| *o == old) {
                Some((_, new)) => {
                    img.insert("stepPosition".into(), Value::from(*new));
                }
                None => {
                    img.shift_remove("stepPosition");
                }
            }
        }
    }
    sync_step_images(root);
}

/// Die Liste `image` auf oberster Ebene ist maßgeblich (Galerie im Kopf der Sud-Seite);
/// jeder Schritt bekommt in `image` die Fotos mit seiner `stepPosition` gespiegelt.
/// Fotos, die bisher nur in einem Schritt standen, werden vorher übernommen.
pub fn sync_step_images(root: &mut Map<String, Value>) {
    let mut master: Vec<Value> = root.get("image").and_then(Value::as_array).cloned().unwrap_or_default();
    let had_master = root.contains_key("image");
    let known = |m: &[Value], url: &str| m.iter().any(|i| i.get("contentUrl").and_then(Value::as_str) == Some(url));
    let mut absorbed = false;
    if let Some(Value::Array(steps)) = root.get("recipeInstructions") {
        for step in steps {
            let pos = step.get("position").and_then(Value::as_u64);
            for img in step.get("image").and_then(Value::as_array).into_iter().flatten() {
                let Some(url) = img.get("contentUrl").and_then(Value::as_str) else { continue };
                if !known(&master, url) {
                    let mut img = img.clone();
                    if let (Some(obj), Some(p)) = (img.as_object_mut(), pos) {
                        obj.insert("stepPosition".into(), Value::from(p));
                    }
                    master.push(img);
                    absorbed = true;
                }
            }
        }
    }
    if absorbed || had_master {
        if master.is_empty() {
            root.shift_remove("image");
        } else {
            set_after(root, "image", Value::Array(master.clone()), "dateModified");
        }
    }
    if let Some(Value::Array(steps)) = root.get_mut("recipeInstructions") {
        for step in steps.iter_mut().filter_map(Value::as_object_mut) {
            let pos = step.get("position").and_then(Value::as_u64);
            let mine: Vec<Value> = master
                .iter()
                .filter(|i| pos.is_some() && i.get("stepPosition").and_then(Value::as_u64) == pos)
                .cloned()
                .collect();
            if mine.is_empty() {
                step.shift_remove("image");
            } else {
                step.insert("image".into(), Value::Array(mine));
            }
        }
    }
}

pub fn new_image(content_url: &str, caption: &str, step_position: Option<u64>) -> Value {
    let mut v = json!({ "@type": "ImageObject", "contentUrl": content_url, "caption": caption });
    if let Some(p) = step_position {
        v["stepPosition"] = Value::from(p);
    }
    v
}

pub fn sort_steps_by_date(root: &mut Map<String, Value>) {
    if let Some(Value::Array(steps)) = root.get_mut("recipeInstructions") {
        steps.sort_by_key(|s| {
            s.get("datePublished")
                .and_then(Value::as_str)
                .and_then(parse_ts)
                .map(|t| t.timestamp())
                .unwrap_or(i64::MAX)
        });
    }
    renumber_steps(root);
}

pub fn new_step(position: usize, phase: &str, name: &str) -> Value {
    json!({
        "@type": "HowToStep",
        "position": position,
        "brew:phase": phase,
        "name": name,
        "datePublished": now_iso(),
    })
}

pub fn new_property(property_id: &str) -> Value {
    let (name, unit) = crate::vocab::property_defaults(property_id).unwrap_or(("", "°C"));
    json!({
        "@type": "PropertyValue",
        "propertyID": property_id,
        "name": name,
        "value": 0.0,
        "unitText": unit,
    })
}

pub fn new_ingredient(category: &str, stage: &str, name: &str, quantity: Value) -> Value {
    json!({
        "@type": "HowToSupply",
        "brew:category": category,
        "brew:stage": stage,
        "name": name,
        "requiredQuantity": quantity,
    })
}

pub struct NewBatchSpec {
    pub identifier: String,
    pub name: String,
    pub category: String,
    pub author: String,
    pub brewery: String,
    pub initial_volume: f64,
    pub date_created: String,
}

pub fn template(spec: &NewBatchSpec) -> Value {
    let mut author = json!({ "@type": "Person", "name": spec.author });
    if !spec.brewery.trim().is_empty() {
        author["worksFor"] = json!({ "@type": "Brewery", "name": spec.brewery.trim() });
    }
    json!({
        "$schema": SCHEMA_REF,
        "@context": { "@vocab": "https://schema.org/", "brew": BREW_NS },
        "@type": "Recipe",
        "@id": format!("urn:batch:{}", spec.identifier),
        "schemaVersion": SCHEMA_VERSION,
        "identifier": spec.identifier,
        "name": spec.name,
        "recipeCategory": spec.category,
        "author": author,
        "brew:status": "planned",
        "brew:statusLabel": "Geplant",
        "dateCreated": spec.date_created,
        "dateModified": spec.date_created,
        "brew:vitalStatistics": {},
        "brew:volumes": {
            "initialVolume": qv(spec.initial_volume, "L"),
            "currentVolume": qv(spec.initial_volume, "L"),
            "ledger": []
        },
        "tool": [],
        "brew:ingredients": [],
        "recipeInstructions": [{
            "@type": "HowToStep",
            "position": 1,
            "brew:phase": "preparation",
            "name": "Planung & Vorbereitung",
            "datePublished": spec.date_created,
        }]
    })
}

/// Nächste freie ID nach dem Muster `PRÄFIX-JAHR-NN`.
pub fn suggest_identifier(category: &str, existing: &[String]) -> String {
    let prefix = crate::vocab::id_prefix(category);
    let year = chrono::Local::now().format("%Y").to_string();
    let stem = format!("{prefix}-{year}-");
    let next = existing
        .iter()
        .filter_map(|id| id.strip_prefix(&stem)?.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    format!("{stem}{next:02}")
}

pub fn is_valid_identifier(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

/// Sucht das Repo-Wurzelverzeichnis (enthält `batches/` und `schemas/`).
pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(d) = dir {
        if d.join("batches").is_dir() && d.join("schemas/brew-batch.schema.json").is_file() {
            return Some(d.to_path_buf());
        }
        dir = d.parent();
    }
    None
}

pub fn load_all(repo: &Path) -> Result<Vec<BatchDoc>, String> {
    let dir = repo.join("batches");
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    Ok(paths.iter().map(|p| BatchDoc::load(p)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::{Level, SchemaValidator};

    fn repo() -> PathBuf {
        find_repo_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("repo root")
    }

    /// Laden + Speichern ohne Änderung darf keinen Diff erzeugen.
    #[test]
    fn roundtrip_is_lossless() {
        for doc in load_all(&repo()).unwrap() {
            let original = fs::read_to_string(&doc.path).unwrap();
            assert!(doc.parse_error.is_none(), "{}", doc.file_id);
            assert_eq!(doc.serialize().unwrap(), original, "{} ändert sich beim Speichern", doc.file_id);
        }
    }

    #[test]
    fn existing_batches_validate() {
        let schema = SchemaValidator::load(&repo());
        assert!(schema.load_error.is_none(), "{:?}", schema.load_error);
        for doc in load_all(&repo()).unwrap() {
            let issues = schema.check(&doc.file_id, &doc.value);
            let msgs: Vec<_> = issues.iter().map(|i| format!("{} {}", i.path, i.message)).collect();
            assert!(issues.is_empty(), "{}: {msgs:#?}", doc.file_id);
        }
    }

    #[test]
    fn schema_and_domain_errors_are_reported() {
        let schema = SchemaValidator::load(&repo());
        let doc = &load_all(&repo()).unwrap()[0];
        let mut v = doc.value.clone();
        v["brew:status"] = Value::String("brewing".into());
        v["recipeInstructions"][0]["datePublished"] = Value::String("2026-11-17".into());
        v["brew:volumes"]["currentVolume"]["value"] = Value::from(3.0);
        let issues = schema.check(&doc.file_id, &v);
        let has = |needle: &str| issues.iter().any(|i| i.level == Level::Error && i.path.contains(needle));
        assert!(has("/brew:status"));
        assert!(has("/recipeInstructions/0/datePublished"));
        assert!(has("/brew:volumes/currentVolume"));
    }

    #[test]
    fn template_needs_only_an_ingredient() {
        let schema = SchemaValidator::load(&repo());
        let spec = NewBatchSpec {
            identifier: "BIER-2026-01".into(),
            name: "Testsud".into(),
            category: "Beer".into(),
            author: "Lukas Holzner".into(),
            brewery: "Heimbrauerei München-Sendling".into(),
            initial_volume: 20.0,
            date_created: now_iso(),
        };
        let mut v = template(&spec);
        let issues = schema.check("BIER-2026-01", &v);
        assert_eq!(issues.len(), 1, "nur die fehlende Zutat erwartet");
        assert!(issues[0].path.contains("brew:ingredients"));
        v["brew:ingredients"] = json!([new_ingredient("fermentable", "mash", "Pilsner Malz", qv(4.2, "kg"))]);
        assert!(schema.check("BIER-2026-01", &v).is_empty());
    }

    /// Bilder folgen ihrem Schritt beim Verschieben/Löschen, Spiegelung bleibt konsistent.
    #[test]
    fn step_images_follow_their_step() {
        let doc = &load_all(&repo()).unwrap()[0];
        let mut v = doc.value.clone();
        let original = v.clone();
        let root = v.as_object_mut().unwrap();
        // Ohne Änderung: Sync erzeugt exakt dieselbe Struktur.
        sync_step_images(root);
        assert_eq!(Value::Object(root.clone()), original);
        // Schritt 2 (mit Foto) nach vorne tauschen.
        root.get_mut("recipeInstructions").unwrap().as_array_mut().unwrap().swap(0, 1);
        renumber_steps(root);
        let steps = root["recipeInstructions"].as_array().unwrap();
        assert_eq!(steps[0]["image"][0]["stepPosition"], 1);
        assert!(steps[1].get("image").is_none());
        let keg = root["image"].as_array().unwrap().iter().find(|i| i["contentUrl"].as_str().unwrap().contains("jolly")).unwrap();
        assert_eq!(keg["stepPosition"], 1);
        // Schritt mit Foto löschen -> Foto bleibt in der Galerie, ohne Schrittbezug.
        root.get_mut("recipeInstructions").unwrap().as_array_mut().unwrap().remove(0);
        renumber_steps(root);
        let keg = root["image"].as_array().unwrap().iter().find(|i| i["contentUrl"].as_str().unwrap().contains("jolly")).unwrap();
        assert!(keg.get("stepPosition").is_none());
        // Foto nur im Schritt -> wird in die Galerie übernommen.
        root.get_mut("recipeInstructions").unwrap()[0]["image"] = json!([new_image("assets/batches/X/neu.jpg", "Neu", None)]);
        sync_step_images(root);
        let master = root["image"].as_array().unwrap();
        assert!(master.iter().any(|i| i["contentUrl"] == "assets/batches/X/neu.jpg" && i["stepPosition"] == 1));
    }

    #[test]
    fn identifiers() {
        assert_eq!(suggest_identifier("Mead", &["MET-2026-01".into()]).len(), "MET-2026-02".len());
        assert!(suggest_identifier("Mead", &[format!("MET-{}-01", chrono::Local::now().format("%Y"))]).ends_with("-02"));
        assert!(is_valid_identifier("HELL-2026-02"));
        assert!(!is_valid_identifier("hell-2026"));
        assert!(is_valid_datetime(&now_iso()));
    }
}
