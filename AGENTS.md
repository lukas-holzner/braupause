# AGENTS.md — AI-Leitfaden für **Braupause**

Willkommen im **Braupause**-Repository der *Heimbrauerei München-Sendling* (Lukas Holzner).
Dieses Projekt dokumentiert Brau-, Met- und Gärprozesse als strukturierte **JSON-LD / Schema.org**-Dateien im Ordner [`batches/`](./batches/) und rendert daraus automatisch eine statische Website für **GitHub Pages**.

Wenn Lukas dir unstrukturierte Notizen, Messwerte, Fotos oder Sprachmemos zu einem Sud gibt (z. B. *„Heute 0,5 L abgezapft, Brix liegt bei 10,0, hab 150 g Honig nachgesüßt"*), befolge strikt diesen Leitfaden.

---

## 1. Projektstruktur

```text
braupause/
├── AGENTS.md                        # Diese Anleitung für AI-Agenten
├── README.md                        # Projekt-Übersicht & lokale Befehle
├── package.json                     # Skripte: validate, build, check/test
├── assets/logo/                     # Logo: Emblem & Wortmarke (SVG/PNG, werden nach dist/ kopiert) + VectorCraft-Quelle
├── schemas/
│   └── brew-batch.schema.json       # Verbindliches JSON-Schema (Draft 2020-12)
├── batches/
│   └── <BATCH-ID>.json              # Einzelne Sude (z. B. MET-2026-01.json)
├── scripts/
│   ├── validate.mjs                 # Schema- & Plausibilitäts-Validator (CI + lokal)
│   └── build.mjs                    # Statischer Site-Generator -> dist/
├── editor/                          # Rust-Desktop-Editor (egui) für batches/*.json inkl. Commit & Push
│   └── src/validate.rs              # Spiegelt die Regeln aus validate.mjs – bei Regeländerungen mitziehen!
└── .github/workflows/
    └── ci-and-pages.yml             # GitHub Actions: Schema-Test & GitHub Pages Deploy
```

---

## 2. Goldene Regeln beim Pflegen von Sud-Dateien (`batches/*.json`)

### A. Dateiname & Identifikatoren
1. Jede Datei unter `batches/` heißt exakt `<identifier>.json` (Großbuchstaben, Zahlen, Bindestriche, z. B. `MET-2026-01.json`, `HELL-2026-02.json`).
2. Jede Sud-Datei **muss** oben `"$schema": "../schemas/brew-batch.schema.json"` referenzieren.
3. Das Feld `"@id"` lautet immer `"urn:batch:<identifier>"` (z. B. `"urn:batch:MET-2026-01"`).

### B. Mengen & Einheiten (`QuantitativeValue`)
- **Niemals** Einheiten in Feldnamen codieren! Nutze immer `QuantitativeValue`:
  ```json
  { "@type": "QuantitativeValue", "value": 12.14, "unitText": "L" }
  ```
- Erlaubte Einheiten (`unitText` laut Schema `#/$defs/UnitCode`):
  - **Volumen:** `"L"`, `"mL"`
  - **Masse:** `"kg"`, `"g"`, `"mg"`
  - **Dichte / Extrakt:** `"°P"` (Grad Plato), `"°Bx"` (Grad Brix), `"BLG"` (Balling), `"SG"` (Specific Gravity)
  - **Alkohol / Druck / Temperatur / Säure:** `"% vol"`, `"PSI"`, `"bar"`, `"°C"`, `"pH"`
  - **Stückzahl:** `"pcs"`, `"pkg"`

### C. Zutaten synchron halten (`brew:ingredients` ↔ `recipeInstructions`)
- **`brew:ingredients` (Gesamt-Zutatenliste):** Enthält **alle** Zutaten über den gesamten Prozess hinweg, versehen mit `"brew:stage"`:
  - `"mash"`, `"boil"`, `"primary"`, `"secondary"`, `"stabilization"`, `"back_sweetening"`, `"conditioning"`, `"packaging"`
- Wenn der Brauer später im Prozess etwas zugibt (z. B. Kaliumsorbat, Campden-Tabletten, Nachsüß-Honig, Stopfhopfen):
  1. Dokumentiere die Gabe chronologisch im jeweiligen `HowToStep` unter `"brew:additions"`.
  2. Ergänze dieselbe Zutat **auch** oben in `"brew:ingredients"` mit der passenden `"brew:stage"`, damit die Gesamtsummen (z. B. Gesamtmenge Honig) jederzeit ohne Durchsuchen der Schritte stimmen.

### D. Prozessschritte (`recipeInstructions`) & Messwerte (`brew:measurements`)
- Jeder Eintrag in `recipeInstructions` ist ein `"@type": "HowToStep"` mit:
  - `"position"`: Fortlaufende Ganzzahl (`1, 2, 3, ...`).
  - `"brew:phase"`: Einer der erlaubten Phasenwerte (`"preparation"`, `"mashing"`, `"boiling"`, `"pitching"`, `"primary_fermentation"`, `"secondary_fermentation"`, `"degassing"`, `"measurement"`, `"stabilization_and_sweetening"`, `"conditioning"`, `"racking"`, `"packaging"`, `"tasting"`).
  - `"datePublished"`: Vollständiger **ISO-8601 Zeitstempel mit Uhrzeit und Zeitzone** (z. B. `"2026-11-17T17:00:00+01:00"`). Wenn der Nutzer nur ein Datum nennt, ergänze eine plausible Uhrzeit (z. B. `T12:00:00+01:00` bzw. `+02:00` in der Sommerzeit), damit das Schema (`format: "date-time"`) erfüllt ist.
- **Messwerte gehören niemals nur in den Fließtext!** Lege für jede genannte Zahl einen Eintrag in `"brew:measurements"` (oder bei Geräte-Einstellungen wie Spunddruck in `"brew:settings"`) an:
  - Standard-`propertyID`s:
    - `"gravity_measured_top"` (Gemessene Stammwürze bei Schichtung)
    - `"gravity_calculated_total"` / `"original_gravity"` (Stammwürze)
    - `"refractometer_brix"` (Abgelesener Refraktometerwert in `°Bx`)
    - `"real_extract"` (Tatsächlicher Restextrakt nach Alkoholfehler-Korrektur in `°P`)
    - `"specific_gravity"` (Dichte in `SG`)
    - `"target_brix"` (Zielwert nach Nachsüßung in `°Bx`)
    - `"abv"` (Alkoholgehalt in `% vol`)
    - `"spunding_pressure"` (Spunddruck in `PSI` oder `bar`)
    - `"temperature"` (Gär-/Lagertemperatur in `°C`)
    - `"ph"` (pH-Wert in `pH`)
    - `"volume_remaining"` (Restvolumen im Gebinde in `L`)
- Aktualisiere bei neuen Messwerten auch immer den Block `"brew:vitalStatistics"` (`currentGravity`, `finalGravity`, `abv`) sowie `"dateModified"` auf der obersten Ebene.

### E. Volumen-Buchführung (`brew:volumes`)
Unser Validator (`scripts/validate.mjs`) prüft mathematisch, ob die Volumenbilanz aufgeht:
$$\text{initialVolume} + \sum \text{ledger}[i].\text{delta} = \text{currentVolume}$$
- Wenn der Nutzer eine Probe zapft (`-0.5 L`), umschlaucht oder das aktuelle Restvolumen misst:
  1. Aktualisiere `"brew:volumes.currentVolume"`.
  2. Füge einen passenden Eintrag in `"brew:volumes.ledger"` hinzu (Entnahmen und Verluste haben ein **negatives** `delta.value`, z. B. `-0.5`).

### F. Fotos (`image`)
- Bilddateien liegen unter `assets/batches/<identifier>/` (Dateiname in Kleinbuchstaben mit Bindestrichen, z. B. `jolly-keg-blowoff.jpg`), webfreundlich skaliert (lange Kante ≤ 1600 px, JPEG).
- `contentUrl` ist relativ zur Repo-Wurzel, z. B. `"assets/batches/MET-2026-01/jolly-keg-blowoff.jpg"`; `caption` ist Pflicht.
- Die Liste `image` auf oberster Ebene enthält **alle** Fotos (Galerie im Kopf der Sud-Seite). Gehört ein Foto zu einem Schritt, bekommt es `"stepPosition"` und wird **identisch** zusätzlich im `image`-Array dieses `HowToStep` eingetragen.

---

## 3. Pflicht-Workflow nach jeder Änderung

Führe nach **jedem** Anlegen oder Bearbeiten einer JSON-Datei unter `batches/` zwingend folgenden Befehl aus:

```bash
npm test
```

*(Dies führt nacheinander `npm run validate` und `npm run build` aus.)*

1. **`npm run validate`** prüft alle Dateien in `batches/*.json` gegen [`schemas/brew-batch.schema.json`](./schemas/brew-batch.schema.json) sowie auf chronologische Reihenfolge, eindeutige IDs und korrekte Volumenbilanz.
2. **`npm run build`** generiert die statische Website in `dist/` neu, damit sichergestellt ist, dass der Build für GitHub Pages fehlerfrei durchläuft.

Beende deine Aufgabe erst, wenn `npm test` mit Exit-Code `0` durchläuft!
