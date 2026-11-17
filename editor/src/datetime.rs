//! Datum-/Uhrzeit-Auswahl: Kalender-Popup (deutsch, Wochenstart Montag),
//! Stunde/Minute und „Jetzt“. Gespeichert wird weiterhin ISO-8601 mit
//! Zeitzone; der Offset (+01:00 / +02:00) ergibt sich automatisch aus der
//! lokalen Zeitzone für das gewählte Datum, inklusive Sommerzeit.

use chrono::{DateTime, Datelike, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike};
use egui::{RichText, Ui};

use crate::model::now_iso;
use crate::theme;

const WEEKDAYS: [&str; 7] = ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];
const MONTHS: [&str; 12] = [
    "Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember",
];

/// ISO-8601 für eine lokale Wanduhrzeit, mit passendem Offset der lokalen Zeitzone.
pub fn to_iso(naive: NaiveDateTime) -> String {
    let dt = Local
        .from_local_datetime(&naive)
        .earliest()
        // Zeit fällt in die Sommerzeit-Lücke: eine Stunde weiter
        .or_else(|| Local.from_local_datetime(&(naive + Duration::hours(1))).earliest())
        .map(|d| d.fixed_offset())
        .unwrap_or_else(|| FixedOffset::east_opt(3600).unwrap().from_local_datetime(&naive).unwrap());
    dt.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

pub fn parse(s: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(s).ok()
}

/// Lesbare Kurzform, z. B. „Di, 17.11.2026“.
pub fn format_date(d: NaiveDate) -> String {
    format!("{}, {}", WEEKDAYS[d.weekday().num_days_from_monday() as usize], d.format("%d.%m.%Y"))
}

/// Datum/Uhrzeit-Feld für einen ISO-String. Gibt `true` zurück, wenn sich der Wert geändert hat.
pub fn edit(ui: &mut Ui, salt: impl std::hash::Hash + std::fmt::Debug, value: &mut String) -> bool {
    let id = ui.make_persistent_id(salt);
    let raw_id = id.with("raw");
    let mut show_raw = ui.data_mut(|d| d.get_temp::<bool>(raw_id)).unwrap_or(false);
    let parsed = parse(value);
    let mut changed = false;

    ui.horizontal(|ui| {
        match parsed {
            Some(dt) if !show_raw => {
                let naive = dt.naive_local();
                let mut date = naive.date();
                let mut hour = naive.hour();
                let mut minute = naive.minute();

                let resp = ui.button(format!("📅 {}", format_date(date))).on_hover_text("Datum im Kalender wählen");
                egui::Popup::from_toggle_button_response(&resp)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        if let Some(d) = calendar(ui, id.with("cal"), date) {
                            date = d;
                            ui.close();
                        }
                    });

                let time_changed = time_field(ui, &mut hour, 23) | {
                    ui.label(":");
                    time_field(ui, &mut minute, 59)
                };
                ui.label(RichText::new("Uhr").color(theme::INK_MUTED));

                if date != naive.date() || time_changed {
                    let time = NaiveTime::from_hms_opt(hour, minute, naive.second()).unwrap_or(naive.time());
                    *value = to_iso(date.and_time(time));
                    changed = true;
                }
                ui.label(RichText::new(dt.format("UTC%:z").to_string()).small().color(theme::INK_MUTED))
                    .on_hover_text("Zeitzone wird automatisch aus Datum und lokaler Zeitzone (inkl. Sommerzeit) bestimmt.");
            }
            _ => {
                let mut te = egui::TextEdit::singleline(value)
                    .desired_width(210.0)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("2026-11-17T17:00:00+01:00");
                if parsed.is_none() {
                    te = te.text_color(theme::ERROR);
                }
                if ui.add(te).changed() {
                    changed = true;
                }
                if parsed.is_none() {
                    ui.label(RichText::new("ungültig").color(theme::ERROR).small());
                }
            }
        }
        if ui.small_button("Jetzt").clicked() {
            *value = now_iso();
            changed = true;
        }
        let toggle = ui
            .selectable_label(show_raw, RichText::new("ISO").small().monospace())
            .on_hover_text("Rohwert (ISO-8601) direkt bearbeiten");
        if toggle.clicked() {
            show_raw = !show_raw;
        }
    });
    ui.data_mut(|d| d.insert_temp(raw_id, show_raw));
    changed
}

/// Stunde/Minute als zweistelliges Zahlenfeld (ziehen oder klicken und tippen).
fn time_field(ui: &mut Ui, v: &mut u32, max: u32) -> bool {
    let r = ui.add(
        egui::DragValue::new(v)
            .range(0..=max)
            .speed(0.15)
            .custom_formatter(|n, _| format!("{:02}", n as u32))
            .custom_parser(|s| s.trim().parse::<f64>().ok()),
    );
    r.on_hover_text("Ziehen oder klicken und tippen").changed()
}

/// Monatskalender. Liefert das angeklickte Datum.
fn calendar(ui: &mut Ui, id: egui::Id, selected: NaiveDate) -> Option<NaiveDate> {
    let shown_id = id.with("month");
    let (mut year, mut month) = ui
        .data_mut(|d| d.get_temp::<(i32, u32)>(shown_id))
        .unwrap_or((selected.year(), selected.month()));
    let today = Local::now().date_naive();
    let mut picked = None;

    ui.set_min_width(250.0);
    ui.horizontal(|ui| {
        if ui.button("‹").on_hover_text("Vormonat").clicked() {
            (year, month) = if month == 1 { (year - 1, 12) } else { (year, month - 1) };
        }
        ui.add_sized(
            [170.0, 20.0],
            egui::Label::new(RichText::new(format!("{} {year}", MONTHS[month as usize - 1])).strong()),
        );
        if ui.button("›").on_hover_text("Nächster Monat").clicked() {
            (year, month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
        }
    });
    ui.add_space(4.0);

    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(selected);
    let offset = first.weekday().num_days_from_monday() as i64;
    let start = first - Duration::days(offset);
    egui::Grid::new(id.with("grid")).spacing([2.0, 2.0]).show(ui, |ui| {
        for (i, wd) in WEEKDAYS.iter().enumerate() {
            let color = if i >= 5 { theme::AMBER_HOVER } else { theme::INK_MUTED };
            ui.add_sized([32.0, 18.0], egui::Label::new(RichText::new(*wd).small().color(color)));
        }
        ui.end_row();
        for week in 0..6 {
            for wd in 0..7 {
                let day = start + Duration::days(week * 7 + wd);
                let in_month = day.month() == month;
                let mut text = RichText::new(day.day().to_string());
                if !in_month {
                    text = text.color(theme::HAIRLINE);
                } else if day == today {
                    text = text.strong().color(theme::BLUE);
                }
                let btn = egui::Button::selectable(day == selected, text).min_size(egui::vec2(32.0, 24.0));
                let r = ui.add(btn);
                let r = if day == today { r.on_hover_text("Heute") } else { r };
                if r.clicked() {
                    picked = Some(day);
                }
            }
            ui.end_row();
        }
    });
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Heute").clicked() {
            picked = Some(today);
        }
        if ui.button("Gestern").clicked() {
            picked = Some(today - Duration::days(1));
        }
    });

    if picked.is_some() {
        ui.data_mut(|d| d.remove::<(i32, u32)>(shown_id));
    } else {
        ui.data_mut(|d| d.insert_temp(shown_id, (year, month)));
    }
    picked
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Offset folgt der lokalen Zeitzone, inkl. Sommerzeit (Test setzt TZ nicht, prüft nur Konsistenz).
    #[test]
    fn iso_roundtrip() {
        let naive = NaiveDate::from_ymd_opt(2026, 11, 17).unwrap().and_hms_opt(17, 30, 0).unwrap();
        let iso = to_iso(naive);
        let back = parse(&iso).unwrap();
        assert_eq!(back.naive_local(), naive);
        assert!(iso.starts_with("2026-11-17T17:30:00"));
        assert_eq!(format_date(naive.date()), "Di, 17.11.2026");
    }
}
