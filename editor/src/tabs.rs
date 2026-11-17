//! Die Bearbeitungs-Tabs eines Suds. Jede Funktion gibt `true` zurück, wenn
//! sich das JSON geändert hat.

use std::path::{Path, PathBuf};

use egui::{RichText, TextEdit, Ui};
use serde_json::{Map, Value, json};

use crate::images;

use crate::model::{
    self, BatchDoc, DocUi, is_valid_datetime, new_ingredient, new_property, new_step, now_iso, qv,
    qv_get, renumber_steps, sort_steps_by_date, to_liters,
};
use crate::theme;
use crate::vocab::{self, Choices};
use crate::widgets::{self as w, FIELD_W, Obj};

/// Kontext außerhalb des Dokuments, den einige Tabs brauchen (Repo-Pfad, Foto-Import).
pub struct Env<'a> {
    pub repo: &'a Path,
    pub import: &'a mut Vec<ImportRequest>,
    pub trash: &'a mut Vec<String>,
    pub importing: bool,
    pub max_edge: &'a mut u32,
}

pub struct ImportRequest {
    pub paths: Vec<PathBuf>,
    pub step: Option<u64>,
}

pub fn pick_images() -> Vec<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Fotos hinzufügen")
        .add_filter("Bilder", images::FILE_FILTER)
        .pick_files()
        .unwrap_or_default()
}

fn thumbnail(ui: &mut Ui, repo: &Path, url: &str, max: egui::Vec2) -> egui::Response {
    let abs = repo.join(url);
    if !abs.is_file() {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(max.x, max.y * 0.6), egui::Sense::hover());
        ui.painter().rect_filled(rect, 4.0, theme::ERROR_TINT);
        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "Datei fehlt", egui::FontId::proportional(13.0), theme::ERROR);
        return resp;
    }
    let resp = ui
        .add(
            egui::Image::new(format!("file://{}", abs.display()))
                .max_size(max)
                .corner_radius(4.0)
                .sense(egui::Sense::click()),
        )
        .on_hover_text("Klicken zum Öffnen in der Vorschau");
    if resp.clicked() {
        let _ = std::process::Command::new("open").arg(&abs).spawn();
    }
    resp
}

// ───────────────────────────── Fotos ─────────────────────────────

pub fn photos(ui: &mut Ui, doc: &mut BatchDoc, env: &mut Env) -> bool {
    let file_id = doc.file_id.clone();
    let Some(root) = doc.value.as_object_mut() else { return false };
    let mut changed = false;

    ui.horizontal(|ui| {
        w::section_heading(ui, "Fotos");
        if ui
            .add_enabled(!env.importing, theme::primary_button("+ Fotos hinzufügen …"))
            .on_hover_text("JPEG, PNG, WebP, HEIC, TIFF – oder Dateien einfach ins Fenster ziehen.")
            .clicked()
        {
            let paths = pick_images();
            if !paths.is_empty() {
                env.import.push(ImportRequest { paths, step: None });
            }
        }
        ui.label(RichText::new("Max. Kantenlänge").color(theme::INK_MUTED));
        egui::ComboBox::from_id_salt("img.edge")
            .width(90.0)
            .selected_text(format!("{} px", env.max_edge))
            .show_ui(ui, |ui| {
                for e in images::EDGE_CHOICES {
                    ui.selectable_value(env.max_edge, e, format!("{e} px"));
                }
            });
        if env.importing {
            ui.spinner();
            ui.label(RichText::new("Fotos werden verarbeitet …").color(theme::INK_MUTED));
        }
    });
    ui.label(
        RichText::new(format!(
            "Beim Import wird nach Kamera-Ausrichtung gedreht, auf höchstens {} px skaliert, als JPEG (Qualität {}) unter assets/batches/{file_id}/ gespeichert und von Metadaten wie GPS befreit.",
            env.max_edge,
            images::JPEG_QUALITY
        ))
        .small()
        .color(theme::INK_MUTED),
    );
    ui.add_space(6.0);

    let step_choices: Vec<(u64, String)> = root
        .get("recipeInstructions")
        .and_then(Value::as_array)
        .map(|steps| {
            steps
                .iter()
                .filter_map(|s| {
                    let p = s.get("position")?.as_u64()?;
                    Some((p, format!("#{p} {}", s.get("name").and_then(Value::as_str).unwrap_or(""))))
                })
                .collect()
        })
        .unwrap_or_default();

    let images_list = w::arr_mut(root, "image");
    if images_list.is_empty() {
        ui.label(RichText::new("Noch keine Fotos.").color(theme::INK_MUTED));
    }
    let len = images_list.len();
    let mut action = None;
    for (i, img) in images_list.iter_mut().enumerate() {
        let Some(m) = img.as_object_mut() else { continue };
        ui.push_id(("img", i), |ui| {
            w::card(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let url = w::str_of(m, "contentUrl").to_string();
                    thumbnail(ui, env.repo, &url, egui::vec2(220.0, 165.0));
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Bildunterschrift").small().color(theme::INK_MUTED));
                        let width = (ui.available_width() - 8.0).clamp(160.0, FIELD_W * 1.5);
                        if m.get("caption").is_none() {
                            m.insert("caption".into(), Value::String(String::new()));
                            changed = true;
                        }
                        changed |= w::text_multiline(ui, m, "caption", false, width);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Schritt").small().color(theme::INK_MUTED));
                            changed |= step_combo(ui, m, &step_choices);
                        });
                        let abs = env.repo.join(&url);
                        let mut info = url.clone();
                        if let Some((wd, ht)) = images::dimensions(&abs) {
                            info.push_str(&format!("  ·  {wd} × {ht} px"));
                        }
                        if let Ok(meta) = std::fs::metadata(&abs) {
                            info.push_str(&format!("  ·  {}", images::human_size(meta.len())));
                        }
                        ui.label(RichText::new(info).monospace().small().color(theme::INK_MUTED));
                        w::list_controls(ui, len, i, &mut action);
                    });
                });
            });
        });
    }
    changed |= w::apply_list_action(images_list, action);
    if images_list.is_empty() && root.get("image").is_some_and(|v| v.as_array().is_some_and(Vec::is_empty)) {
        root.shift_remove("image");
    }

    // Dateien im Asset-Ordner, die (noch) nicht eingetragen sind
    let referenced: Vec<String> = root
        .get("image")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|i| i.get("contentUrl").and_then(Value::as_str).map(str::to_string)).collect())
        .unwrap_or_default();
    let orphans = images::orphans(env.repo, &file_id, &referenced);
    if !orphans.is_empty() {
        w::section_heading(ui, "Nicht verwendete Dateien");
        ui.label(
            RichText::new(format!("Liegen in assets/batches/{file_id}/, stehen aber in keinem Foto-Eintrag."))
                .small()
                .color(theme::INK_MUTED),
        );
        for url in orphans {
            ui.push_id(("orphan", &url), |ui| {
                w::card(ui, |ui| {
                    ui.horizontal(|ui| {
                        thumbnail(ui, env.repo, &url, egui::vec2(110.0, 80.0));
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&url).monospace().small());
                            ui.horizontal(|ui| {
                                if ui.button("Wieder eintragen").clicked() {
                                    let caption = Path::new(&url)
                                        .file_stem()
                                        .map(|s| images::caption_from_stem(&s.to_string_lossy()))
                                        .unwrap_or_default();
                                    w::arr_mut(root, "image").push(model::new_image(&url, &caption, None));
                                    changed = true;
                                }
                                if ui.button("In den Papierkorb").clicked() {
                                    env.trash.push(url.clone());
                                }
                            });
                        });
                    });
                });
            });
        }
    }
    if changed && !root.contains_key("image") {
        return true;
    }
    if changed && let Some(v) = root.shift_remove("image") {
        // an die übliche Stelle hinter dateModified setzen
        model::set_after(root, "image", v, "dateModified");
    }
    changed
}

fn step_combo(ui: &mut Ui, m: &mut Obj, choices: &[(u64, String)]) -> bool {
    let current = m.get("stepPosition").and_then(Value::as_u64);
    let shown = current
        .and_then(|p| choices.iter().find(|(c, _)| *c == p).map(|(_, l)| l.clone()))
        .unwrap_or_else(|| match current {
            Some(p) => format!("#{p} (fehlt)"),
            None => "— ganzer Sud".into(),
        });
    let mut changed = false;
    egui::ComboBox::from_id_salt("img.step").width(260.0).selected_text(shown).show_ui(ui, |ui| {
        if ui.selectable_label(current.is_none(), "— ganzer Sud").clicked() && current.is_some() {
            m.shift_remove("stepPosition");
            changed = true;
        }
        for (p, label) in choices {
            if ui.selectable_label(current == Some(*p), label).clicked() && current != Some(*p) {
                m.insert("stepPosition".into(), Value::from(*p));
                changed = true;
            }
        }
    });
    changed
}

// ───────────────────────────── Übersicht ─────────────────────────────

pub fn overview(ui: &mut Ui, doc: &mut BatchDoc) -> bool {
    let Some(root) = doc.value.as_object_mut() else { return false };
    let mut changed = false;

    w::section_heading(ui, "Stammdaten");
    w::card(ui, |ui| {
        egui::Grid::new("ov.grid").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
            w::row(ui, "Kennung", |ui| {
                ui.label(RichText::new(w::str_of(root, "identifier")).monospace().strong())
                    .on_hover_text("Entspricht dem Dateinamen und ist nach dem Anlegen fest.");
            });
            w::row(ui, "Name", |ui| changed |= w::text(ui, root, "name", false, FIELD_W * 1.5));
            w::row(ui, "Beschreibung", |ui| {
                changed |= w::text_multiline(ui, root, "description", true, FIELD_W * 1.5)
            });
            w::row(ui, "Kategorie", |ui| {
                changed |= w::combo(ui, "ov.cat", root, "recipeCategory", vocab::CATEGORIES, false, 200.0)
            });
            w::row(ui, "Status", |ui| {
                ui.horizontal(|ui| {
                    let before = w::str_of(root, "brew:status").to_string();
                    if w::combo(ui, "ov.status", root, "brew:status", vocab::STATUS, false, 200.0) {
                        changed = true;
                        // Anzeigenamen mitziehen, solange er nicht individuell angepasst wurde.
                        let label = w::str_of(root, "brew:statusLabel").to_string();
                        if label.is_empty() || label == vocab::label(vocab::STATUS, &before) {
                            let new = vocab::label(vocab::STATUS, w::str_of(root, "brew:status"));
                            model::set_after(root, "brew:statusLabel", Value::String(new), "brew:status");
                        }
                    }
                    ui.label(RichText::new("Anzeige:").color(theme::INK_MUTED));
                    changed |= w::text(ui, root, "brew:statusLabel", true, 220.0);
                });
            });
            w::row(ui, "Angestellt am", |ui| changed |= w::datetime(ui, root, "dateCreated"));
            w::row(ui, "Zuletzt geändert", |ui| changed |= w::datetime(ui, root, "dateModified"));
            w::row(ui, "Schlagwörter", |ui| changed |= keywords(ui, root));
        });
    });

    w::section_heading(ui, "Stil");
    w::card(ui, |ui| {
        if root.get("brew:style").is_some_and(Value::is_object) {
            let mut remove = false;
            egui::Grid::new("ov.style").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
                let style = w::obj_mut(root, "brew:style");
                w::row(ui, "Stil", |ui| changed |= w::text(ui, style, "name", false, FIELD_W));
                w::row(ui, "BJCP-Code", |ui| changed |= w::text(ui, style, "bjcpCode", true, 100.0));
                w::row(ui, "Süße", |ui| {
                    changed |= w::combo(ui, "ov.sweet", style, "sweetness", vocab::SWEETNESS, true, 160.0)
                });
                w::row(ui, "Kohlensäure", |ui| {
                    changed |= w::combo(ui, "ov.carb", style, "carbonation", vocab::CARBONATION, true, 160.0)
                });
                w::row(ui, "", |ui| remove = ui.small_button("Stilangabe entfernen").clicked());
            });
            if remove {
                root.shift_remove("brew:style");
                changed = true;
            }
        } else if ui.button("+ Stil angeben").clicked() {
            model::set_after(root, "brew:style", json!({ "name": "" }), "recipeCategory");
            changed = true;
        }
    });

    w::section_heading(ui, "Autor & Brauerei");
    w::card(ui, |ui| {
        egui::Grid::new("ov.author").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
            let author = w::obj_mut(root, "author");
            if w::str_of(author, "@type") != "Person" {
                author.shift_insert(0, "@type".into(), Value::String("Person".into()));
                changed = true;
            }
            w::row(ui, "Brauer:in", |ui| changed |= w::text(ui, author, "name", false, FIELD_W));
            if author.get("worksFor").is_some_and(Value::is_object) {
                let mut remove = false;
                {
                    let brewery = w::obj_mut(author, "worksFor");
                    w::row(ui, "Brauerei", |ui| changed |= w::text(ui, brewery, "name", false, FIELD_W));
                    w::row(ui, "Ort", |ui| changed |= w::text(ui, brewery, "location", true, FIELD_W));
                }
                w::row(ui, "", |ui| remove = ui.small_button("Brauerei entfernen").clicked());
                if remove {
                    author.shift_remove("worksFor");
                    changed = true;
                }
            } else {
                w::row(ui, "Brauerei", |ui| {
                    if ui.small_button("+ Brauerei angeben").clicked() {
                        author.insert(
                            "worksFor".into(),
                            json!({ "@type": "Brewery", "name": "Heimbrauerei München-Sendling" }),
                        );
                        changed = true;
                    }
                });
            }
        });
    });

    w::section_heading(ui, "Eckdaten");
    w::card(ui, |ui| {
        let derived = derive_vitals(root);
        ui.horizontal(|ui| {
            let resp = ui
                .add_enabled(!derived.is_empty(), egui::Button::new("Aus Messwerten übernehmen"))
                .on_hover_text(
                    "Übernimmt aktuellen Extrakt, Stammwürze, Alkohol und pH aus den jüngsten Messungen im Protokoll.",
                );
            if resp.clicked() {
                let vitals = w::obj_mut(root, "brew:vitalStatistics");
                for (key, value, unit) in &derived {
                    let q = w::obj_mut(vitals, key);
                    q.entry("@type").or_insert_with(|| Value::String("QuantitativeValue".into()));
                    q.insert("value".into(), Value::from(*value));
                    q.insert("unitText".into(), Value::String(unit.to_string()));
                }
                changed = true;
            }
            if !derived.is_empty() {
                let hint: Vec<String> = derived
                    .iter()
                    .map(|(k, v, u)| format!("{}: {} {u}", vital_label(k), w::fmt_num(*v)))
                    .collect();
                ui.label(RichText::new(hint.join("  ·  ")).color(theme::INK_MUTED).small());
            }
        });
        ui.add_space(6.0);
        egui::Grid::new("ov.vitals").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
            let vitals = w::obj_mut(root, "brew:vitalStatistics");
            for (key, unit) in [
                ("originalGravity", "°P"),
                ("currentGravity", "°Bx"),
                ("finalGravity", "°P"),
                ("abv", "% vol"),
                ("ph", "pH"),
            ] {
                w::row(ui, vital_label(key), |ui| {
                    changed |= w::opt_quantity(ui, &format!("vit.{key}"), vitals, key, unit, vocab::UNITS)
                });
            }
        });
    });
    changed
}

fn vital_label(key: &str) -> &'static str {
    match key {
        "originalGravity" => "Stammwürze",
        "currentGravity" => "Aktueller Extrakt",
        "finalGravity" => "Restextrakt (final)",
        "abv" => "Alkohol",
        "ph" => "pH-Wert",
        _ => "",
    }
}

/// Jüngste passende Messwerte aus dem Protokoll: (vitals-Schlüssel, Wert, Einheit).
fn derive_vitals(root: &Obj) -> Vec<(&'static str, f64, String)> {
    let mut out: Vec<(&'static str, f64, String)> = Vec::new();
    let Some(steps) = root.get("recipeInstructions").and_then(Value::as_array) else { return out };
    let mut set = |key: &'static str, v: f64, u: &str| {
        out.retain(|(k, _, _)| *k != key);
        out.push((key, v, u.to_string()));
    };
    for step in steps {
        let Some(ms) = step.get("brew:measurements").and_then(Value::as_array) else { continue };
        let find = |id: &str| {
            ms.iter()
                .find(|m| m.get("propertyID").and_then(Value::as_str) == Some(id))
                .and_then(qv_get)
        };
        if let Some((v, u)) = find("target_brix").or_else(|| find("refractometer_brix")).or_else(|| find("specific_gravity")) {
            set("currentGravity", v, u);
        }
        if let Some((v, u)) = find("original_gravity").or_else(|| find("gravity_calculated_total")) {
            set("originalGravity", v, u);
        }
        if let Some((v, u)) = find("abv") {
            set("abv", v, u);
        }
        if let Some((v, u)) = find("ph") {
            set("ph", v, u);
        }
    }
    out
}

fn keywords(ui: &mut Ui, root: &mut Obj) -> bool {
    let current: Vec<String> = root
        .get("keywords")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let id = ui.make_persistent_id("ov.keywords");
    let mut buf = ui.data_mut(|d| d.get_temp::<String>(id)).unwrap_or_else(|| current.join(", "));
    let r = ui.add(
        TextEdit::singleline(&mut buf)
            .id(id.with("te"))
            .desired_width(FIELD_W * 1.5)
            .hint_text("kommagetrennt, z. B. Met, Honigwein"),
    );
    let mut changed = false;
    if r.changed() {
        let mut list: Vec<String> = Vec::new();
        for k in buf.split(',').map(str::trim).filter(|k| !k.is_empty()) {
            if !list.iter().any(|x| x == k) {
                list.push(k.to_string());
            }
        }
        if list != current {
            if list.is_empty() {
                root.shift_remove("keywords");
            } else {
                let arr = Value::Array(list.into_iter().map(Value::String).collect());
                model::set_after(root, "keywords", arr, "brew:style");
            }
            changed = true;
        }
    }
    if !r.has_focus() {
        buf = current.join(", ");
        if changed {
            buf = root
                .get("keywords")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
        }
    }
    ui.data_mut(|d| d.insert_temp(id, buf));
    changed
}

// ───────────────────────────── Volumen ─────────────────────────────

pub fn volumes(ui: &mut Ui, doc: &mut BatchDoc) -> bool {
    let BatchDoc { value, ui: dui, .. } = doc;
    let Some(root) = value.as_object_mut() else { return false };
    let mut changed = false;
    let vols = w::obj_mut(root, "brew:volumes");

    w::section_heading(ui, "Volumen");
    w::card(ui, |ui| {
        egui::Grid::new("vol.grid").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
            w::row(ui, "Anstellvolumen", |ui| {
                changed |= w::quantity(ui, "vol.init", w::obj_mut(vols, "initialVolume"), vocab::VOLUME_UNITS, false)
            });
            w::row(ui, "Aktuelles Volumen", |ui| {
                changed |= w::quantity(ui, "vol.curr", w::obj_mut(vols, "currentVolume"), vocab::VOLUME_UNITS, false)
            });
        });

        // Bilanz wie in scripts/validate.mjs
        let init = vols.get("initialVolume").and_then(to_liters);
        let curr = vols.get("currentVolume").and_then(to_liters);
        let sum: f64 = vols
            .get("ledger")
            .and_then(Value::as_array)
            .map(|l| l.iter().filter_map(|e| e.get("delta").and_then(to_liters)).sum())
            .unwrap_or(0.0);
        if let (Some(init), Some(curr)) = (init, curr) {
            let expected = ((init + sum) * 1000.0).round() / 1000.0;
            let diff = (expected - curr).abs();
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let text = format!(
                    "Bilanz: {} L {} {} L (Buchungen) = {} L",
                    w::fmt_num(init),
                    if sum < 0.0 { "−" } else { "+" },
                    w::fmt_num((sum.abs() * 1000.0).round() / 1000.0),
                    w::fmt_num(expected)
                );
                if diff > 0.05 {
                    ui.label(RichText::new(format!("{text} ≠ {} L aktuell", w::fmt_num(curr))).color(theme::ERROR));
                    if ui.button("Aktuelles Volumen = Bilanz").clicked() {
                        vols.insert("currentVolume".into(), qv(expected, "L"));
                        changed = true;
                    }
                } else {
                    ui.label(RichText::new(format!("{text}  ✓")).color(theme::OLIVE));
                }
            });
        }
    });

    w::section_heading(ui, "Entnahme / Zugabe buchen");
    w::card(ui, |ui| {
        changed |= booking_form(ui, dui, vols);
    });

    w::section_heading(ui, "Buchungen (Ledger)");
    let ledger = w::arr_mut(vols, "ledger");
    if ledger.is_empty() {
        ui.label(RichText::new("Noch keine Buchungen.").color(theme::INK_MUTED));
    }
    let len = ledger.len();
    let mut action = None;
    for (i, entry) in ledger.iter_mut().enumerate() {
        let Some(e) = entry.as_object_mut() else { continue };
        ui.push_id(("ledger", i), |ui| {
            w::card(ui, |ui| {
                egui::Grid::new("le").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
                    w::row(ui, "Zeitpunkt", |ui| changed |= w::datetime(ui, e, "timestamp"));
                    w::row(ui, "Art", |ui| {
                        changed |= w::combo(ui, "type", e, "type", vocab::LEDGER_TYPES, false, 230.0)
                    });
                    w::row(ui, "Bezeichnung", |ui| changed |= w::text(ui, e, "label", false, FIELD_W));
                    w::row(ui, "Änderung", |ui| {
                        changed |= w::quantity(ui, "delta", w::obj_mut(e, "delta"), vocab::VOLUME_UNITS, false)
                    });
                    w::row(ui, "Ziel", |ui| changed |= w::text(ui, e, "destination", true, FIELD_W));
                    w::row(ui, "", |ui| w::list_controls(ui, len, i, &mut action));
                });
            });
        });
    }
    changed |= w::apply_list_action(ledger, action);
    changed
}

fn booking_form(ui: &mut Ui, dui: &mut DocUi, vols: &mut Obj) -> bool {
    let mut changed = false;
    egui::Grid::new("book").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
        w::row(ui, "Menge (L)", |ui| {
            w::number(ui, "book.amount", &mut dui.draw_amount, 72.0);
        });
        w::row(ui, "Art", |ui| {
            choice_combo(ui, "book.type", &mut dui.draw_type, vocab::LEDGER_TYPES, 230.0);
        });
        w::row(ui, "Bezeichnung", |ui| {
            ui.add(TextEdit::singleline(&mut dui.draw_label).desired_width(FIELD_W));
        });
        w::row(ui, "Ziel", |ui| {
            ui.add(TextEdit::singleline(&mut dui.draw_destination).desired_width(FIELD_W).hint_text("optional, z. B. 2 × 0,5-L-Bügelflasche"));
        });
        w::row(ui, "Zeitpunkt", |ui| {
            crate::datetime::edit(ui, "book.ts", &mut dui.draw_timestamp);
        });
        w::row(ui, "", |ui| {
            let is_addition = dui.draw_type == "addition";
            let ok = dui.draw_amount > 0.0 && is_valid_datetime(&dui.draw_timestamp) && !dui.draw_label.trim().is_empty();
            let label = if is_addition { "Zugabe buchen" } else { "Entnahme buchen" };
            if ui
                .add_enabled(ok, theme::primary_button(label))
                .on_hover_text("Legt einen Ledger-Eintrag an und passt das aktuelle Volumen an.")
                .clicked()
            {
                let delta = if is_addition { dui.draw_amount } else { -dui.draw_amount };
                let mut entry = Map::new();
                entry.insert("timestamp".into(), Value::String(dui.draw_timestamp.clone()));
                entry.insert("type".into(), Value::String(dui.draw_type.clone()));
                entry.insert("label".into(), Value::String(dui.draw_label.trim().to_string()));
                entry.insert("delta".into(), qv(delta, "L"));
                if !dui.draw_destination.trim().is_empty() {
                    entry.insert("destination".into(), Value::String(dui.draw_destination.trim().to_string()));
                }
                w::arr_mut(vols, "ledger").push(Value::Object(entry));
                let curr = vols.get("currentVolume").and_then(to_liters).unwrap_or(0.0);
                let new = ((curr + delta) * 1000.0).round() / 1000.0;
                vols.insert("currentVolume".into(), qv(new, "L"));
                dui.draw_timestamp = now_iso();
                changed = true;
            }
        });
    });
    changed
}

fn choice_combo(ui: &mut Ui, salt: &str, value: &mut String, choices: Choices, width: f32) {
    egui::ComboBox::from_id_salt(salt)
        .width(width)
        .selected_text(vocab::label(choices, value))
        .show_ui(ui, |ui| {
            for (v, l) in choices {
                ui.selectable_value(value, (*v).to_string(), *l);
            }
        });
}

// ───────────────────────────── Zutaten ─────────────────────────────

pub fn ingredients(ui: &mut Ui, doc: &mut BatchDoc) -> bool {
    let Some(root) = doc.value.as_object_mut() else { return false };
    let mut changed = false;

    w::section_heading(ui, "Summen");
    ingredient_totals(ui, root);

    ui.horizontal(|ui| {
        w::section_heading(ui, "Zutaten (alle Phasen)");
        if ui.button("+ Zutat").clicked() {
            w::arr_mut(root, "brew:ingredients").push(new_ingredient("other", "primary", "", qv(0.0, "g")));
            changed = true;
        }
    });

    let items = w::arr_mut(root, "brew:ingredients");
    if items.is_empty() {
        ui.label(RichText::new("Mindestens eine Zutat ist laut Schema Pflicht.").color(theme::ERROR));
    }
    let len = items.len();
    let mut action = None;
    for (i, item) in items.iter_mut().enumerate() {
        let Some(m) = item.as_object_mut() else { continue };
        ui.push_id(("ing", i), |ui| {
            w::card(ui, |ui| {
                egui::Grid::new("ing").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
                    w::row(ui, "Name", |ui| changed |= w::text(ui, m, "name", false, FIELD_W));
                    w::row(ui, "Kategorie", |ui| {
                        changed |= w::combo(ui, "cat", m, "brew:category", vocab::INGREDIENT_CATEGORIES, false, 260.0)
                    });
                    w::row(ui, "Phase", |ui| {
                        changed |= w::combo(ui, "stage", m, "brew:stage", vocab::STAGES, false, 260.0)
                    });
                    w::row(ui, "Menge", |ui| {
                        changed |= w::quantity(ui, "qty", w::obj_mut(m, "requiredQuantity"), vocab::UNITS, false)
                    });
                    w::row(ui, "Kosten (€)", |ui| changed |= w::opt_number_field(ui, "cost", m, "estimatedCost", 72.0));
                    w::row(ui, "Notiz", |ui| changed |= w::text(ui, m, "description", true, FIELD_W * 1.5));
                    if let Some(props) = m.get("brew:properties") {
                        w::row(ui, "Eigenschaften", |ui| {
                            ui.label(RichText::new(props.to_string()).monospace().small().color(theme::INK_MUTED))
                                .on_hover_text("Freie Zusatzangaben – im Tab „JSON“ bearbeiten.");
                        });
                    }
                    w::row(ui, "", |ui| w::list_controls(ui, len, i, &mut action));
                });
            });
        });
    }
    changed |= w::apply_list_action(items, action);
    changed
}

fn ingredient_totals(ui: &mut Ui, root: &Obj) {
    let Some(items) = root.get("brew:ingredients").and_then(Value::as_array) else { return };
    // (Kategorie, Basiseinheit) -> Summe; g/mg -> kg, mL -> L
    let mut totals: Vec<(String, String, f64)> = Vec::new();
    for it in items {
        let cat = it.get("brew:category").and_then(Value::as_str).unwrap_or("other");
        let Some((v, u)) = it.get("requiredQuantity").and_then(qv_get) else { continue };
        let (v, u) = match u {
            "g" => (v / 1000.0, "kg"),
            "mg" => (v / 1_000_000.0, "kg"),
            "mL" => (v / 1000.0, "L"),
            other => (v, other),
        };
        match totals.iter_mut().find(|(c, un, _)| c == cat && un == u) {
            Some(t) => t.2 += v,
            None => totals.push((cat.to_string(), u.to_string(), v)),
        }
    }
    w::card(ui, |ui| {
        egui::Grid::new("ing.totals").num_columns(2).spacing([24.0, 4.0]).show(ui, |ui| {
            for (cat, unit, v) in &totals {
                ui.label(vocab::label(vocab::INGREDIENT_CATEGORIES, cat));
                let shown = if unit == "kg" && *v < 1.0 {
                    format!("{} g", w::fmt_num((v * 1000.0 * 100.0).round() / 100.0))
                } else {
                    format!("{} {unit}", w::fmt_num((v * 1000.0).round() / 1000.0))
                };
                ui.label(RichText::new(shown).monospace());
                ui.end_row();
            }
        });
    });
}

// ───────────────────────────── Protokoll ─────────────────────────────

pub fn steps(ui: &mut Ui, doc: &mut BatchDoc, env: &mut Env) -> bool {
    let BatchDoc { value, ui: dui, .. } = doc;
    let Some(root) = value.as_object_mut() else { return false };
    let mut changed = false;

    ui.horizontal(|ui| {
        w::section_heading(ui, "Protokoll");
        ui.menu_button("+ Neuer Schritt", |ui| {
            for (phase, label) in vocab::PHASES {
                if ui.button(*label).clicked() {
                    let steps = w::arr_mut(root, "recipeInstructions");
                    steps.push(new_step(steps.len() + 1, phase, label));
                    dui.open_step = Some(steps.len() - 1);
                    changed = true;
                    ui.close();
                }
            }
        });
        if ui
            .button("Nach Datum sortieren")
            .on_hover_text("Sortiert die Schritte chronologisch und nummeriert sie neu.")
            .clicked()
        {
            sort_steps_by_date(root);
            changed = true;
        }
    });

    let ingredient_names: Vec<String> = root
        .get("brew:ingredients")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|i| i.get("name").and_then(Value::as_str).map(str::to_string)).collect())
        .unwrap_or_default();
    let mut to_ingredients: Vec<Value> = Vec::new();

    let steps = w::arr_mut(root, "recipeInstructions");
    let len = steps.len();
    let mut action = None;
    let open_step = dui.open_step.take();
    for (i, step) in steps.iter_mut().enumerate() {
        let Some(s) = step.as_object_mut() else { continue };
        let phase = w::str_of(s, "brew:phase").to_string();
        let date = w::str_of(s, "datePublished");
        let date_short = model::parse_ts(date)
            .map(|d| d.format("%d.%m.%Y %H:%M").to_string())
            .unwrap_or_else(|| date.to_string());
        let header = format!(
            "#{}  {}  ·  {}  ·  {}",
            s.get("position").and_then(Value::as_u64).unwrap_or(0),
            vocab::label(vocab::PHASES, &phase),
            w::str_of(s, "name"),
            date_short
        );
        ui.push_id(("step", i), |ui| {
            let mut ch = egui::CollapsingHeader::new(RichText::new(header).strong())
                .id_salt("hdr")
                .default_open(i + 1 == len);
            if let Some(o) = open_step {
                ch = ch.open(Some(o == i));
            }
            ch.show_unindented(ui, |ui| {
                w::card(ui, |ui| {
                    changed |= step_editor(ui, s, &ingredient_names, &mut to_ingredients, env);
                    ui.add_space(4.0);
                    w::list_controls(ui, len, i, &mut action);
                });
            });
        });
    }
    if w::apply_list_action(steps, action) {
        changed = true;
    }
    if changed {
        renumber_steps(root);
    }
    if !to_ingredients.is_empty() {
        w::arr_mut(root, "brew:ingredients").extend(to_ingredients);
        changed = true;
    }
    changed
}

fn step_editor(
    ui: &mut Ui,
    s: &mut Obj,
    ingredient_names: &[String],
    to_ingredients: &mut Vec<Value>,
    env: &mut Env,
) -> bool {
    let mut changed = false;
    egui::Grid::new("step").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
        w::row(ui, "Phase", |ui| changed |= w::combo(ui, "phase", s, "brew:phase", vocab::PHASES, false, 260.0));
        w::row(ui, "Titel", |ui| changed |= w::text(ui, s, "name", false, FIELD_W * 1.3));
        w::row(ui, "Zeitpunkt", |ui| changed |= w::datetime(ui, s, "datePublished"));
        w::row(ui, "Beschreibung", |ui| changed |= w::text_multiline(ui, s, "description", true, FIELD_W * 1.5));
    });

    // Anweisungen
    sub_heading(ui, "Anweisungen / Notizen");
    let has_dirs = s.get("itemListElement").and_then(Value::as_array).is_some_and(|a| !a.is_empty());
    if has_dirs {
        let dirs = w::arr_mut(s, "itemListElement");
        let len = dirs.len();
        let mut action = None;
        for (j, d) in dirs.iter_mut().enumerate() {
            let Some(d) = d.as_object_mut() else { continue };
            ui.push_id(("dir", j), |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}.", j + 1)).color(theme::INK_MUTED));
                    changed |= w::text_multiline(ui, d, "text", false, (ui.available_width() - 110.0).max(200.0));
                    w::list_controls(ui, len, j, &mut action);
                });
            });
        }
        changed |= w::apply_list_action(dirs, action);
        if dirs.is_empty() {
            s.shift_remove("itemListElement");
        }
    }
    if ui.small_button("+ Anweisung").clicked() {
        w::arr_mut(s, "itemListElement").push(json!({ "@type": "HowToDirection", "text": "" }));
        changed = true;
    }

    sub_heading(ui, "Messwerte");
    changed |= property_list(ui, s, "brew:measurements", "+ Messwert");
    sub_heading(ui, "Einstellungen (z. B. Spunddruck)");
    changed |= property_list(ui, s, "brew:settings", "+ Einstellung");

    // Zugaben
    sub_heading(ui, "Zugaben");
    let phase = w::str_of(s, "brew:phase").to_string();
    if s.get("brew:additions").and_then(Value::as_array).is_some_and(|a| !a.is_empty()) {
        let adds = w::arr_mut(s, "brew:additions");
        let len = adds.len();
        let mut action = None;
        for (j, a) in adds.iter_mut().enumerate() {
            let Some(a) = a.as_object_mut() else { continue };
            ui.push_id(("add", j), |ui| {
                ui.horizontal(|ui| {
                    changed |= w::text(ui, a, "name", false, 220.0);
                    changed |= w::quantity(ui, "q", w::obj_mut(a, "quantity"), vocab::UNITS, false);
                    changed |= w::combo(ui, "cat", a, "brew:category", vocab::INGREDIENT_CATEGORIES, true, 150.0);
                    let name = w::str_of(a, "name").to_string();
                    let known = ingredient_names.contains(&name)
                        || to_ingredients.iter().any(|v| v.get("name").and_then(Value::as_str) == Some(&name));
                    if !name.is_empty() && !known {
                        let btn = egui::Button::new(RichText::new("→ Zutatenliste").color(theme::WARN)).small();
                        if ui
                            .add(btn)
                            .on_hover_text("Fehlt in brew:ingredients – mit passender Phase übernehmen (AGENTS.md 2C).")
                            .clicked()
                        {
                            let cat = w::str_of(a, "brew:category");
                            let cat = if vocab::INGREDIENT_CATEGORIES.iter().any(|(c, _)| *c == cat) { cat } else { "other" };
                            let quantity = a.get("quantity").cloned().unwrap_or_else(|| qv(0.0, "g"));
                            to_ingredients.push(new_ingredient(cat, &stage_for(&phase, cat), &name, quantity));
                        }
                    }
                    w::list_controls(ui, len, j, &mut action);
                });
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new("Notiz").small().color(theme::INK_MUTED));
                    changed |= w::text(ui, a, "description", true, FIELD_W);
                });
            });
        }
        changed |= w::apply_list_action(adds, action);
        if adds.is_empty() {
            s.shift_remove("brew:additions");
        }
    }
    if ui.small_button("+ Zugabe").clicked() {
        w::arr_mut(s, "brew:additions").push(json!({ "name": "", "quantity": qv(0.0, "g") }));
        changed = true;
    }

    // Fotos (gespiegelt aus der Foto-Liste; Bearbeitung der Unterschriften im Tab „Fotos“)
    sub_heading(ui, "Fotos");
    let pos = s.get("position").and_then(Value::as_u64);
    if let Some(imgs) = s.get("image").and_then(Value::as_array) {
        ui.horizontal_wrapped(|ui| {
            for img in imgs {
                let url = img.get("contentUrl").and_then(Value::as_str).unwrap_or("");
                let caption = img.get("caption").and_then(Value::as_str).unwrap_or("");
                ui.vertical(|ui| {
                    ui.set_max_width(170.0);
                    thumbnail(ui, env.repo, url, egui::vec2(160.0, 120.0)).on_hover_text(caption);
                    ui.label(RichText::new(caption).small().color(theme::INK_MUTED));
                });
            }
        });
    }
    ui.horizontal(|ui| {
        if ui.add_enabled(!env.importing && pos.is_some(), egui::Button::new("+ Foto").small()).clicked() {
            let paths = pick_images();
            if !paths.is_empty() {
                env.import.push(ImportRequest { paths, step: pos });
            }
        }
        ui.label(RichText::new("Bildunterschrift & Zuordnung im Tab „Fotos“").small().color(theme::INK_MUTED));
    });
    changed
}

/// Passende `brew:stage` für eine Zugabe in einer bestimmten Prozessphase.
fn stage_for(phase: &str, category: &str) -> String {
    match phase {
        "mashing" => "mash",
        "boiling" => "boil",
        "preparation" | "pitching" | "primary_fermentation" => "primary",
        "secondary_fermentation" | "degassing" | "measurement" | "racking" => "secondary",
        "stabilization_and_sweetening" if category == "fermentable" => "back_sweetening",
        "stabilization_and_sweetening" => "stabilization",
        "packaging" => "packaging",
        _ => "conditioning",
    }
    .to_string()
}

fn sub_heading(ui: &mut Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(text).strong().color(theme::INK_SECONDARY));
}

fn property_list(ui: &mut Ui, s: &mut Obj, key: &str, add_label: &str) -> bool {
    let mut changed = false;
    if s.get(key).and_then(Value::as_array).is_some_and(|a| !a.is_empty()) {
        let props = w::arr_mut(s, key);
        let len = props.len();
        let mut action = None;
        for (j, p) in props.iter_mut().enumerate() {
            let Some(p) = p.as_object_mut() else { continue };
            ui.push_id((key, j), |ui| {
                ui.horizontal_wrapped(|ui| {
                    // propertyID: frei editierbar + Auswahl der Standard-IDs
                    let before = w::str_of(p, "propertyID").to_string();
                    changed |= w::text(ui, p, "propertyID", false, 120.0);
                    ui.menu_button("▾", |ui| {
                        for (id, name, unit) in vocab::PROPERTY_IDS {
                            if ui.button(format!("{name}  ·  {id}")).clicked() {
                                let old_default = vocab::property_defaults(&before).map(|(n, _)| n);
                                let cur_name = w::str_of(p, "name").to_string();
                                p.insert("propertyID".into(), Value::String((*id).to_string()));
                                if cur_name.is_empty() || Some(cur_name.as_str()) == old_default {
                                    p.insert("name".into(), Value::String((*name).to_string()));
                                }
                                p.insert("unitText".into(), Value::String((*unit).to_string()));
                                changed = true;
                                ui.close();
                            }
                        }
                    });
                    changed |= w::text(ui, p, "name", true, 150.0);
                    changed |= w::number_field(ui, "v", p, "value", 56.0);
                    changed |= w::unit_combo(ui, "u", p, "unitText", vocab::UNITS);
                    ui.label(RichText::new("min").small().color(theme::INK_MUTED));
                    changed |= w::opt_number_field(ui, "min", p, "minValue", 44.0);
                    ui.label(RichText::new("max").small().color(theme::INK_MUTED));
                    changed |= w::opt_number_field(ui, "max", p, "maxValue", 44.0);
                    w::list_controls(ui, len, j, &mut action);
                });
            });
        }
        changed |= w::apply_list_action(props, action);
        if props.is_empty() {
            s.shift_remove(key);
        }
    }
    ui.menu_button(add_label, |ui| {
        for (id, name, unit) in vocab::PROPERTY_IDS {
            if ui.button(format!("{name} ({unit})")).clicked() {
                w::arr_mut(s, key).push(new_property(id));
                changed = true;
                ui.close();
            }
        }
        ui.separator();
        if ui.button("Eigene Kennung …").clicked() {
            w::arr_mut(s, key).push(json!({
                "@type": "PropertyValue", "propertyID": "custom", "name": "", "value": 0.0, "unitText": "°C"
            }));
            changed = true;
            ui.close();
        }
    });
    changed
}

// ───────────────────────────── Equipment ─────────────────────────────

pub fn tools(ui: &mut Ui, doc: &mut BatchDoc) -> bool {
    let Some(root) = doc.value.as_object_mut() else { return false };
    let mut changed = false;
    ui.horizontal(|ui| {
        w::section_heading(ui, "Equipment");
        if ui.button("+ Gerät").clicked() {
            w::arr_mut(root, "tool").push(json!({ "@type": "HowToTool", "name": "", "brew:role": "other" }));
            changed = true;
        }
    });
    let items = w::arr_mut(root, "tool");
    let len = items.len();
    let mut action = None;
    for (i, item) in items.iter_mut().enumerate() {
        let Some(m) = item.as_object_mut() else { continue };
        ui.push_id(("tool", i), |ui| {
            w::card(ui, |ui| {
                ui.horizontal(|ui| {
                    changed |= w::text(ui, m, "name", false, FIELD_W);
                    changed |= w::combo(ui, "role", m, "brew:role", vocab::TOOL_ROLES, true, 190.0);
                    w::list_controls(ui, len, i, &mut action);
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Notiz").small().color(theme::INK_MUTED));
                    changed |= w::text(ui, m, "description", true, FIELD_W * 1.5);
                });
            });
        });
    }
    changed |= w::apply_list_action(items, action);
    changed
}

// ───────────────────────────── JSON ─────────────────────────────

/// Roh-Editor. Änderungen werden erst mit „Übernehmen“ ins Dokument geschrieben.
pub fn json(ui: &mut Ui, doc: &mut BatchDoc) -> bool {
    let mut changed = false;
    if !doc.ui.json_edited && doc.ui.json_synced_gen != doc.generation {
        doc.ui.json_buf = serde_json::to_string_pretty(&doc.value).unwrap_or_default();
        doc.ui.json_synced_gen = doc.generation;
        doc.ui.json_error = None;
    }
    ui.horizontal(|ui| {
        let apply = ui.add_enabled(doc.ui.json_edited, theme::primary_button("Übernehmen"));
        if apply.clicked() {
            match serde_json::from_str::<Value>(&doc.ui.json_buf) {
                Ok(v) => {
                    doc.value = v;
                    doc.parse_error = None;
                    doc.ui.json_edited = false;
                    doc.ui.json_error = None;
                    changed = true;
                }
                Err(e) => doc.ui.json_error = Some(e.to_string()),
            }
        }
        if ui.add_enabled(doc.ui.json_edited && doc.parse_error.is_none(), egui::Button::new("Verwerfen")).clicked() {
            doc.ui.json_edited = false;
            doc.ui.json_synced_gen = u64::MAX;
        }
        if doc.ui.json_edited {
            ui.label(RichText::new("Nicht übernommene Änderungen").color(theme::WARN));
        }
    });
    if let Some(e) = doc.parse_error.as_ref().or(doc.ui.json_error.as_ref()) {
        ui.label(RichText::new(e).color(theme::ERROR));
    }
    egui::ScrollArea::both().id_salt("json.scroll").auto_shrink([false, false]).show(ui, |ui| {
        let r = ui.add(
            TextEdit::multiline(&mut doc.ui.json_buf)
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(40),
        );
        if r.changed() {
            doc.ui.json_edited = true;
        }
    });
    changed
}
