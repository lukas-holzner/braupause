//! Live-Validierung im Editor: JSON-Schema (Draft 2020-12, inkl. Formate) und
//! dieselben Plausibilitätsregeln wie `scripts/validate.mjs`. `npm test` bleibt
//! die verbindliche Prüfung vor dem Commit.

use std::collections::HashSet;

use serde_json::Value;

use crate::model::{parse_ts, to_liters};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
}

#[derive(Clone)]
pub struct Issue {
    pub level: Level,
    pub path: String,
    pub message: String,
}

impl Issue {
    fn err(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { level: Level::Error, path: path.into(), message: message.into() }
    }
    fn warn(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { level: Level::Warning, path: path.into(), message: message.into() }
    }
}

pub struct SchemaValidator {
    validator: Option<jsonschema::Validator>,
    repo: std::path::PathBuf,
    pub load_error: Option<String>,
}

impl SchemaValidator {
    pub fn load(repo: &std::path::Path) -> Self {
        let path = repo.join("schemas/brew-batch.schema.json");
        let result = std::fs::read_to_string(&path)
            .map_err(|e| format!("{}: {e}", path.display()))
            .and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| e.to_string()))
            .and_then(|schema| {
                jsonschema::options()
                    .with_draft(jsonschema::Draft::Draft202012)
                    .should_validate_formats(true)
                    .build(&schema)
                    .map_err(|e| e.to_string())
            });
        match result {
            Ok(v) => Self { validator: Some(v), repo: repo.to_path_buf(), load_error: None },
            Err(e) => Self {
                validator: None,
                repo: repo.to_path_buf(),
                load_error: Some(format!("Schema nicht ladbar: {e}")),
            },
        }
    }

    pub fn check(&self, file_id: &str, data: &Value) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Some(v) = &self.validator {
            for e in v.iter_errors(data) {
                let path = e.instance_path().to_string();
                issues.push(Issue::err(if path.is_empty() { "/".into() } else { path }, e.to_string()));
            }
        }
        domain_checks(file_id, data, &mut issues);
        image_checks(&self.repo, file_id, data, &mut issues);
        issues
    }
}

fn domain_checks(file_id: &str, data: &Value, issues: &mut Vec<Issue>) {
    let identifier = data.get("identifier").and_then(Value::as_str).unwrap_or("");
    if identifier != file_id {
        issues.push(Issue::err(
            "/identifier",
            format!("identifier \"{identifier}\" passt nicht zum Dateinamen \"{file_id}.json\"."),
        ));
    }
    if let Some(id) = data.get("@id").and_then(Value::as_str) {
        let expected = format!("urn:batch:{identifier}");
        if id != expected {
            issues.push(Issue::err("/@id", format!("\"@id\" sollte \"{expected}\" lauten (aktuell \"{id}\").")));
        }
    }

    let steps = data.get("recipeInstructions").and_then(Value::as_array);
    if let Some(steps) = steps {
        let mut prev: Option<i64> = None;
        for (i, step) in steps.iter().enumerate() {
            let pos = step.get("position").and_then(Value::as_u64);
            if pos != Some(i as u64 + 1) {
                issues.push(Issue::err(
                    format!("/recipeInstructions/{i}/position"),
                    format!("position ist {}, erwartet {}.", pos.map_or("–".into(), |p| p.to_string()), i + 1),
                ));
            }
            if let Some(ts) = step.get("datePublished").and_then(Value::as_str).and_then(parse_ts) {
                let t = ts.timestamp();
                if prev.is_some_and(|p| t < p) {
                    let name = step.get("name").and_then(Value::as_str).unwrap_or("");
                    issues.push(Issue::err(
                        format!("/recipeInstructions/{i}/datePublished"),
                        format!("Schritt \"{name}\" liegt zeitlich vor dem vorherigen Schritt."),
                    ));
                }
                prev = Some(t);
            }
        }
    }

    if let Some(vol) = data.get("brew:volumes") {
        let init = vol.get("initialVolume").and_then(to_liters);
        let curr = vol.get("currentVolume").and_then(to_liters);
        let ledger = vol.get("ledger").and_then(Value::as_array);
        if let (Some(init), Some(curr), Some(ledger)) = (init, curr, ledger)
            && !ledger.is_empty()
        {
            let sum: f64 = ledger.iter().filter_map(|e| e.get("delta").and_then(to_liters)).sum();
            let expected = ((init + sum) * 1000.0).round() / 1000.0;
            let diff = (expected - curr).abs();
            if diff > 0.05 {
                issues.push(Issue::err(
                    "/brew:volumes/currentVolume",
                    format!(
                        "Volumenbilanz geht nicht auf: {init} L + Ledger ({sum:.2} L) = {expected} L, aber currentVolume ist {curr} L."
                    ),
                ));
            }
        }
    }

    // AGENTS.md 2C: Zugaben in Schritten müssen auch in brew:ingredients stehen.
    let ingredient_names: HashSet<&str> = data
        .get("brew:ingredients")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|i| i.get("name").and_then(Value::as_str)).collect())
        .unwrap_or_default();
    if let Some(steps) = steps {
        for (i, step) in steps.iter().enumerate() {
            let Some(adds) = step.get("brew:additions").and_then(Value::as_array) else { continue };
            for (j, add) in adds.iter().enumerate() {
                let name = add.get("name").and_then(Value::as_str).unwrap_or("");
                if !name.is_empty() && !ingredient_names.contains(name) {
                    issues.push(Issue::warn(
                        format!("/recipeInstructions/{i}/brew:additions/{j}"),
                        format!("Zugabe \"{name}\" fehlt in der Gesamt-Zutatenliste (brew:ingredients)."),
                    ));
                }
            }
        }
    }

    if data.get("dateModified").is_none() {
        issues.push(Issue::warn("/dateModified", "dateModified fehlt."));
    }
}

fn image_checks(repo: &std::path::Path, file_id: &str, data: &Value, issues: &mut Vec<Issue>) {
    let step_count = data.get("recipeInstructions").and_then(Value::as_array).map_or(0, Vec::len) as u64;
    let master = data.get("image").and_then(Value::as_array).cloned().unwrap_or_default();
    let prefix = format!("assets/batches/{file_id}/");
    for (i, img) in master.iter().enumerate() {
        let path = format!("/image/{i}");
        let url = img.get("contentUrl").and_then(Value::as_str).unwrap_or("");
        if !url.is_empty() && !url.contains("://") && !repo.join(url).is_file() {
            issues.push(Issue::err(format!("{path}/contentUrl"), format!("Bilddatei \"{url}\" existiert nicht.")));
        }
        if !url.is_empty() && !url.contains("://") && !url.starts_with(&prefix) {
            issues.push(Issue::warn(format!("{path}/contentUrl"), format!("Fotos gehören nach \"{prefix}\".")));
        }
        if img.get("caption").and_then(Value::as_str).is_some_and(|c| c.trim().is_empty()) {
            issues.push(Issue::warn(format!("{path}/caption"), format!("Bildunterschrift für \"{url}\" fehlt.")));
        }
        if let Some(p) = img.get("stepPosition").and_then(Value::as_u64)
            && p > step_count
        {
            issues.push(Issue::err(format!("{path}/stepPosition"), format!("Schritt #{p} existiert nicht.")));
        }
    }
    let Some(steps) = data.get("recipeInstructions").and_then(Value::as_array) else { return };
    for (i, step) in steps.iter().enumerate() {
        let pos = step.get("position").and_then(Value::as_u64);
        let expected: Vec<&Value> = master
            .iter()
            .filter(|m| pos.is_some() && m.get("stepPosition").and_then(Value::as_u64) == pos)
            .collect();
        let actual: Vec<&Value> = step.get("image").and_then(Value::as_array).map(|a| a.iter().collect()).unwrap_or_default();
        if expected != actual {
            issues.push(Issue::warn(
                format!("/recipeInstructions/{i}/image"),
                "Fotos des Schritts weichen von der Foto-Liste (image) ab – wird beim nächsten Bearbeiten abgeglichen.",
            ));
        }
    }
}
