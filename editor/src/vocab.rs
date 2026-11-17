//! Erlaubte Werte aus `schemas/brew-batch.schema.json` samt deutschen Anzeigenamen
//! (gleiche Labels wie in `scripts/build.mjs`).

pub type Choices = &'static [(&'static str, &'static str)];

pub const UNITS: &[&str] = &[
    "L", "mL", "kg", "g", "mg", "°P", "°Bx", "BLG", "SG", "% vol", "PSI", "bar", "°C", "pH", "pcs",
    "pkg",
];
pub const VOLUME_UNITS: &[&str] = &["L", "mL"];

pub const CATEGORIES: Choices = &[
    ("Mead", "Met"),
    ("Beer", "Bier"),
    ("Cider", "Cider"),
    ("Wine", "Wein"),
    ("Hard Seltzer", "Hard Seltzer"),
    ("Other", "Sonstiges"),
];

/// Vorschlag für das ID-Präfix eines neuen Suds je Kategorie.
pub fn id_prefix(category: &str) -> &'static str {
    match category {
        "Mead" => "MET",
        "Beer" => "BIER",
        "Cider" => "CIDER",
        "Wine" => "WEIN",
        "Hard Seltzer" => "SELTZER",
        _ => "SUD",
    }
}

pub const STATUS: Choices = &[
    ("planned", "Geplant"),
    ("primary_fermentation", "Hauptgärung"),
    ("secondary_fermentation", "Nachgärung"),
    ("stabilization", "Stabilisierung"),
    ("conditioning", "Reifung"),
    ("packaged", "Abgefüllt"),
    ("archived", "Archiviert"),
];

pub const SWEETNESS: Choices = &[
    ("dry", "trocken"),
    ("semi-dry", "halbtrocken"),
    ("medium", "mittel"),
    ("semi-sweet", "halbsüß"),
    ("sweet", "süß"),
];

pub const CARBONATION: Choices = &[
    ("still", "still"),
    ("petillant", "perlend"),
    ("sparkling", "schäumend"),
];

pub const STAGES: Choices = &[
    ("mash", "Maischen"),
    ("boil", "Kochen"),
    ("primary", "Hauptansatz / Primärgärung"),
    ("secondary", "Sekundärgärung"),
    ("stabilization", "Stabilisierung"),
    ("back_sweetening", "Nachsüßung"),
    ("conditioning", "Reifung"),
    ("packaging", "Abfüllung"),
];

pub const INGREDIENT_CATEGORIES: Choices = &[
    ("water", "Wasser"),
    ("fermentable", "Vergärbarer Zucker / Malz / Honig"),
    ("juice", "Fruchtsaft"),
    ("fruit", "Frucht"),
    ("hops", "Hopfen"),
    ("spice_and_herb", "Gewürze & Kräuter"),
    ("yeast", "Hefe"),
    ("nutrient", "Hefenährsalz"),
    ("yeast_and_nutrient", "Hefe & Nährstoffe"),
    ("acid", "Säure"),
    ("stabilizer", "Stabilisator"),
    ("fining", "Schönungsmittel"),
    ("other", "Sonstiges"),
];

pub const PHASES: Choices = &[
    ("preparation", "Vorbereitung"),
    ("mashing", "Maischen"),
    ("boiling", "Würzekochen"),
    ("pitching", "Anstellen"),
    ("primary_fermentation", "Primärgärung"),
    ("secondary_fermentation", "Sekundärgärung"),
    ("degassing", "Entgasung & Druckreduktion"),
    ("measurement", "Messung"),
    ("stabilization_and_sweetening", "Stabilisierung & Nachsüßung"),
    ("conditioning", "Reifung & Klärung"),
    ("racking", "Umschlauchen"),
    ("packaging", "Abfüllung"),
    ("tasting", "Verkostung"),
];

pub const TOOL_ROLES: Choices = &[
    ("fermenter", "Gärbehälter"),
    ("pressure_control", "Drucksteuerung"),
    ("measurement", "Messgerät"),
    ("temperature_control", "Temperatursteuerung"),
    ("filtration", "Filtration"),
    ("bottling", "Abfüllung"),
    ("other", "Equipment"),
];

pub const LEDGER_TYPES: Choices = &[
    ("sample_consumption", "Probe / Verzehr"),
    ("trub_loss", "Trubverlust"),
    ("racking_loss", "Umschlauchverlust"),
    ("bottling", "Abfüllung Flaschen"),
    ("kegging", "Abfüllung Fass"),
    ("addition", "Zugabe"),
    ("unaccounted_or_previous_draws", "Schwund / frühere Entnahmen"),
    ("other", "Sonstiges"),
];

/// Standard-`propertyID`s aus AGENTS.md: (id, Anzeigename, Standard-Einheit).
pub const PROPERTY_IDS: &[(&str, &str, &str)] = &[
    ("refractometer_brix", "Refraktometer", "°Bx"),
    ("real_extract", "Tatsächlicher Restextrakt", "°P"),
    ("specific_gravity", "Dichte", "SG"),
    ("gravity_measured_top", "Gemessene Stammwürze (oben)", "°P"),
    ("gravity_calculated_total", "Berechnete Gesamtstammwürze", "°P"),
    ("original_gravity", "Stammwürze", "°P"),
    ("target_brix", "Zielwert nach Nachsüßung", "°Bx"),
    ("abv", "Alkoholgehalt", "% vol"),
    ("spunding_pressure", "Spunddruck", "PSI"),
    ("temperature", "Temperatur", "°C"),
    ("ph", "pH-Wert", "pH"),
    ("volume_remaining", "Restvolumen im Gebinde", "L"),
];

pub fn property_defaults(id: &str) -> Option<(&'static str, &'static str)> {
    PROPERTY_IDS
        .iter()
        .find(|(p, _, _)| *p == id)
        .map(|(_, name, unit)| (*name, *unit))
}

pub fn label(choices: Choices, value: &str) -> String {
    choices
        .iter()
        .find(|(v, _)| *v == value)
        .map(|(_, l)| l.to_string())
        .unwrap_or_else(|| value.to_string())
}
