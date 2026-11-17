# Braupause — Heimbrauerei München-Sendling

Statische Brau- und Met-Dokumentation von **Lukas Holzner** auf Basis von **Schema.org / JSON-LD** (`schema.org/Recipe`).

Jeder Sud liegt als einzelne, validierte JSON-Datei im Ordner [`batches/`](./batches/). Ein leichtgewichtiger Generator baut daraus ohne externe Frontend-Frameworks eine statische Website (`dist/`) inklusive eingebettetem JSON-LD und veröffentlicht sie per GitHub Actions auf **GitHub Pages**.

---

## Schnellstart

```bash
# 1. Abhängigkeiten für den JSON-Schema-Validator installieren
npm install

# 2. Alle Sude gegen das JSON-Schema & die Volumenbilanz prüfen + Website bauen
npm test

# 3. Gebaute Seite lokal im Browser ansehen (http://localhost:4173)
npm run serve
```

---

## Desktop-Editor (Rust)

Unter [`editor/`](./editor/) liegt ein natives Desktop-Tool (Rust, egui), mit dem du Sude ohne JSON-Handarbeit anlegen, bearbeiten und anschließend committen & pushen kannst.

```bash
# Starten (findet das Repo automatisch über das aktuelle Verzeichnis)
cargo run --release --manifest-path editor/Cargo.toml
```

- **Sude anlegen:** „+ Neuer Sud“ schlägt die nächste freie Kennung vor (z. B. `MET-2026-02`) und erzeugt eine schema-konforme Vorlage.
- **Bearbeiten:** Tabs für Übersicht/Eckdaten, Protokoll (Schritte mit Messwerten, Einstellungen, Zugaben), Volumen, Zutaten, Equipment und einen Roh-JSON-Editor. Messwerte lassen sich per Klick in die Eckdaten übernehmen, Zugaben in die Gesamt-Zutatenliste.
- **Volumen buchen:** „Entnahme buchen“ legt den Ledger-Eintrag an und passt `currentVolume` an, die Bilanz wird live angezeigt.
- **Fotos:** Im Tab „Fotos“ (oder per Drag & Drop ins Fenster, oder „+ Foto“ direkt im Protokoll-Schritt) importieren. Der Editor dreht nach Kamera-Ausrichtung, skaliert auf höchstens 1600 px (wählbar: 1024/1600/2048), speichert als JPEG (Qualität 82) unter `assets/batches/<ID>/` und entfernt dabei Metadaten wie GPS. iPhone-HEIC wird über macOS-`sips` gewandelt. Die Fotos in den Schritten werden automatisch aus der Foto-Liste gespiegelt.
- **Live-Prüfung:** JSON-Schema (Draft 2020-12), dieselben Plausibilitätsregeln wie `scripts/validate.mjs` und zusätzlich: Bilddatei vorhanden, Schritt existiert, Bildunterschrift gesetzt.
- **Git:** Dateien auswählen, Commit-Nachricht (wird vorgeschlagen), dann „Commit & Push“. Vorher wird gespeichert, geprüft und `npm test` ausgeführt; schlägt etwas fehl, wird nicht committet. Genutzt wird dein installiertes `git` mit deinen vorhandenen Zugangsdaten.

Unbekannte Felder bleiben erhalten, und unveränderte Dateien werden byte-genau so zurückgeschrieben, wie sie gelesen wurden.

---

## Wie füttere ich neue Infos per AI ein?

Öffne dieses Projekt in deinem AI-Agenten (z. B. Antigravity, Cursor, Claude Code) und gib einfach deine Notizen durch:

> *„Trage bei MET-2026-01 für heute (20.11.2026, 19 Uhr) eine Messung ein: 10,0 °Bx stabil, Temperatur 18 °C, geschmacklich rund und klar. Habe 1,0 L in zwei 0,5-L-Bügelflaschen abgefüllt."*

Der Agent liest automatisch [`AGENTS.md`](./AGENTS.md) sowie [`schemas/brew-batch.schema.json`](./schemas/brew-batch.schema.json), trägt den neuen Prozessschritt, die Messwerte und die Volumenbuchung in [`batches/MET-2026-01.json`](./batches/MET-2026-01.json) ein und prüft per `npm test`, ob das Schema und die Volumenbilanz exakt eingehalten werden.

---

## GitHub Pages Einrichtung

1. Repository auf GitHub pushen (`main` Branch).
2. In den Repository-Einstellungen unter **Settings → Pages → Build and deployment** als **Source** die Option **GitHub Actions** auswählen.
3. Bei jedem Push oder Pull Request prüft der Workflow [`.github/workflows/ci-and-pages.yml`](./.github/workflows/ci-and-pages.yml) zuerst alle Dateien in `batches/*.json` gegen das Schema und veröffentlicht bei Erfolg automatisch die aktualisierte Seite.
