//! Kleine Formular-Bausteine, die direkt auf `serde_json::Map` arbeiten.
//! Jede Funktion gibt `true` zurück, wenn sich der Inhalt geändert hat.

use egui::{Color32, RichText, TextEdit, Ui};
use serde_json::{Map, Value};

use crate::model::qv;
use crate::theme;
use crate::vocab::Choices;

pub type Obj = Map<String, Value>;

pub const FIELD_W: f32 = 340.0;

/// Liefert das Objekt unter `key` und legt es bei Bedarf an.
pub fn obj_mut<'a>(m: &'a mut Obj, key: &str) -> &'a mut Obj {
    let v = m.entry(key.to_string()).or_insert_with(|| Value::Object(Map::new()));
    if !v.is_object() {
        *v = Value::Object(Map::new());
    }
    v.as_object_mut().expect("object")
}

/// Liefert das Array unter `key` und legt es bei Bedarf an.
pub fn arr_mut<'a>(m: &'a mut Obj, key: &str) -> &'a mut Vec<Value> {
    let v = m.entry(key.to_string()).or_insert_with(|| Value::Array(Vec::new()));
    if !v.is_array() {
        *v = Value::Array(Vec::new());
    }
    v.as_array_mut().expect("array")
}

pub fn str_of<'a>(m: &'a Obj, key: &str) -> &'a str {
    m.get(key).and_then(Value::as_str).unwrap_or("")
}

/// Grid-Zeile mit Beschriftung links.
pub fn row<R>(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.label(RichText::new(label).color(theme::INK_SECONDARY));
    let r = add(ui);
    ui.end_row();
    r
}

/// Textfeld. Optionale Felder werden beim Leeren aus dem JSON entfernt.
pub fn text(ui: &mut Ui, m: &mut Obj, key: &str, optional: bool, width: f32) -> bool {
    let mut s = str_of(m, key).to_string();
    let r = ui.add(TextEdit::singleline(&mut s).desired_width(width));
    if r.changed() {
        set_str(m, key, s, optional);
        return true;
    }
    false
}

pub fn text_multiline(ui: &mut Ui, m: &mut Obj, key: &str, optional: bool, width: f32) -> bool {
    let mut s = str_of(m, key).to_string();
    let r = ui.add(TextEdit::multiline(&mut s).desired_width(width).desired_rows(2));
    if r.changed() {
        set_str(m, key, s, optional);
        return true;
    }
    false
}

fn set_str(m: &mut Obj, key: &str, s: String, optional: bool) {
    if optional && s.is_empty() {
        m.shift_remove(key);
    } else {
        m.insert(key.to_string(), Value::String(s));
    }
}

/// Auswahlliste für Enum-Werte des Schemas.
pub fn combo(ui: &mut Ui, salt: &str, m: &mut Obj, key: &str, choices: Choices, optional: bool, width: f32) -> bool {
    let current = str_of(m, key).to_string();
    let shown = if current.is_empty() {
        "—".to_string()
    } else {
        crate::vocab::label(choices, &current)
    };
    let mut changed = false;
    egui::ComboBox::from_id_salt(salt).width(width).selected_text(shown).show_ui(ui, |ui| {
        if optional && ui.selectable_label(current.is_empty(), "—").clicked() && !current.is_empty() {
            m.shift_remove(key);
            changed = true;
        }
        for (value, label) in choices {
            let text = if label == value { label.to_string() } else { format!("{label}  ·  {value}") };
            if ui.selectable_label(current == *value, text).clicked() && current != *value {
                m.insert(key.to_string(), Value::String((*value).to_string()));
                changed = true;
            }
        }
    });
    changed
}

/// Einheitenauswahl.
pub fn unit_combo(ui: &mut Ui, salt: &str, m: &mut Obj, key: &str, units: &[&str]) -> bool {
    let current = str_of(m, key).to_string();
    let mut changed = false;
    egui::ComboBox::from_id_salt(salt).width(64.0).selected_text(current.clone()).show_ui(ui, |ui| {
        for u in units {
            if ui.selectable_label(current == *u, *u).clicked() && current != *u {
                m.insert(key.to_string(), Value::String((*u).to_string()));
                changed = true;
            }
        }
    });
    changed
}

pub fn fmt_num(x: f64) -> String {
    let s = format!("{x}");
    if s.contains('e') { format!("{x:.6}") } else { s }
}

/// Zahlenfeld, das auch deutsches Komma akzeptiert (`10,5`).
pub fn number(ui: &mut Ui, salt: impl std::hash::Hash + std::fmt::Debug, v: &mut f64, width: f32) -> bool {
    let id = ui.make_persistent_id(salt);
    let mut buf = ui.data_mut(|d| d.get_temp::<String>(id)).unwrap_or_else(|| fmt_num(*v));
    let parsed = buf.trim().replace(',', ".").parse::<f64>();
    let mut te = TextEdit::singleline(&mut buf).id(id.with("te")).desired_width(width);
    if parsed.is_err() {
        te = te.text_color(theme::ERROR);
    }
    let r = ui.add(te);
    let mut changed = false;
    if r.changed()
        && let Ok(x) = buf.trim().replace(',', ".").parse::<f64>()
        && x.is_finite()
        && x != *v
    {
        *v = x;
        changed = true;
    }
    if !r.has_focus() {
        buf = fmt_num(*v);
    }
    ui.data_mut(|d| d.insert_temp(id, buf));
    changed
}

/// Zahl in einem JSON-Feld bearbeiten (Feld wird bei Bedarf angelegt).
pub fn number_field(ui: &mut Ui, salt: &str, m: &mut Obj, key: &str, width: f32) -> bool {
    let mut x = m.get(key).and_then(Value::as_f64).unwrap_or(0.0);
    if number(ui, salt, &mut x, width) {
        m.insert(key.to_string(), Value::from(x));
        return true;
    }
    false
}

/// Optionales Zahlenfeld: leer = Feld entfernen.
pub fn opt_number_field(ui: &mut Ui, salt: &str, m: &mut Obj, key: &str, width: f32) -> bool {
    let id = ui.make_persistent_id(salt);
    let current = m.get(key).and_then(Value::as_f64);
    let mut buf = ui
        .data_mut(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| current.map(fmt_num).unwrap_or_default());
    let r = ui.add(TextEdit::singleline(&mut buf).id(id.with("te")).desired_width(width).hint_text("—"));
    let mut changed = false;
    if r.changed() {
        let t = buf.trim().replace(',', ".");
        if t.is_empty() {
            if m.shift_remove(key).is_some() {
                changed = true;
            }
        } else if let Ok(x) = t.parse::<f64>()
            && x.is_finite()
            && Some(x) != current
        {
            m.insert(key.to_string(), Value::from(x));
            changed = true;
        }
    }
    if !r.has_focus() {
        buf = m.get(key).and_then(Value::as_f64).map(fmt_num).unwrap_or_default();
    }
    ui.data_mut(|d| d.insert_temp(id, buf));
    changed
}

/// `QuantitativeValue`: Wert + Einheit (+ optional Beschreibung).
pub fn quantity(ui: &mut Ui, salt: &str, q: &mut Obj, units: &[&str], with_description: bool) -> bool {
    let mut changed = false;
    if str_of(q, "@type") != "QuantitativeValue" {
        q.shift_insert(0, "@type".into(), Value::String("QuantitativeValue".into()));
        changed = true;
    }
    ui.horizontal(|ui| {
        changed |= number_field(ui, &format!("{salt}.v"), q, "value", 72.0);
        if q.get("unitText").and_then(Value::as_str).is_none() {
            q.insert("unitText".into(), Value::String(units[0].to_string()));
            changed = true;
        }
        changed |= unit_combo(ui, &format!("{salt}.u"), q, "unitText", units);
        if with_description {
            changed |= text(ui, q, "description", true, (ui.available_width() - 8.0).clamp(80.0, 360.0));
        }
    });
    changed
}

/// Optionaler `QuantitativeValue` unter `key` (anlegen / entfernen).
pub fn opt_quantity(ui: &mut Ui, salt: &str, m: &mut Obj, key: &str, default_unit: &str, units: &[&str]) -> bool {
    let mut changed = false;
    if m.get(key).is_some_and(Value::is_object) {
        ui.horizontal(|ui| {
            changed |= quantity(ui, salt, obj_mut(m, key), units, true);
            if ui.small_button("✕").on_hover_text("Wert entfernen").clicked() {
                m.shift_remove(key);
                changed = true;
            }
        });
    } else if ui.small_button("+ Wert erfassen").clicked() {
        m.insert(key.to_string(), qv(0.0, default_unit));
        changed = true;
    }
    changed
}

/// ISO-8601-Zeitstempel mit Kalender, Uhrzeit und „Jetzt“ (siehe `datetime`).
pub fn datetime(ui: &mut Ui, m: &mut Obj, key: &str) -> bool {
    let mut s = str_of(m, key).to_string();
    if crate::datetime::edit(ui, ("dt", key), &mut s) {
        m.insert(key.to_string(), Value::String(s));
        return true;
    }
    false
}

/// Kleiner farbiger Chip (z. B. Status, Positionsnummer).
pub fn chip(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(3.0)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).monospace().color(fg).size(12.0));
        });
}

/// Verschiebe- und Löschknöpfe für Eintrag `i` einer Liste der Länge `len`.
pub fn list_controls(ui: &mut Ui, len: usize, i: usize, action: &mut Option<ListAction>) {
    ui.horizontal(|ui| {
        if ui.add_enabled(i > 0, egui::Button::new("↑").small()).on_hover_text("nach oben").clicked() {
            *action = Some(ListAction::Up(i));
        }
        if ui
            .add_enabled(i + 1 < len, egui::Button::new("↓").small())
            .on_hover_text("nach unten")
            .clicked()
        {
            *action = Some(ListAction::Down(i));
        }
        if ui.small_button("🗑").on_hover_text("entfernen").clicked() {
            *action = Some(ListAction::Remove(i));
        }
    });
}

#[derive(Clone, Copy)]
pub enum ListAction {
    Up(usize),
    Down(usize),
    Remove(usize),
}

pub fn apply_list_action(items: &mut Vec<Value>, action: Option<ListAction>) -> bool {
    match action {
        Some(ListAction::Up(i)) if i > 0 => items.swap(i, i - 1),
        Some(ListAction::Down(i)) if i + 1 < items.len() => items.swap(i, i + 1),
        Some(ListAction::Remove(i)) if i < items.len() => {
            items.remove(i);
        }
        _ => return false,
    }
    true
}

pub fn section_heading(ui: &mut Ui, text: &str) {
    ui.add_space(10.0);
    ui.label(RichText::new(text).size(17.0).strong().color(theme::INK_PRIMARY));
    ui.add_space(4.0);
}

pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::new()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0, theme::HAIRLINE))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}
