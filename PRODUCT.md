# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

- **Primäre Nutzer (im Keller- & Braualltag):** Der Brauer und seine jeweiligen Mitbrauer nutzen Braupause als persönliches, mobiles und stationäres Keller- und Nachschlagewerk, um im Gärkeller oder am Schreibtisch schnell Fassbestand, Spunddruck, Gärverlauf, Restextrakt, Stabilisierung und anstehende Prozessschritte zu prüfen.
- **Sekundäre Nutzer (Heimbrauer & Met-Macher):** Andere Brauer und Met-Enthusiasten nutzen die Seite als offene, nachvollziehbare Rezept- und Prozess-Dokumentation inklusive maschinenlesbarer Rohdaten (Schema.org / JSON-LD), um Sude, Zutatenverhältnisse, Gärführung und Volumenbilanzen einzusehen oder nachzubrauen.

## Product Purpose

Braupause dokumentiert Brau-, Met- und Gärprozesse als strukturierte, validierte Schema.org / JSON-LD-Datensätze (`batches/*.json`) und bereitet sie als statische Website (`dist/`) für den praktischen Einsatz im Braukeller sowie als offene Dokumentation auf.

Über das bestehende Sud-Register und die Detailprotokolle hinaus soll das Produkt:
1. **Prozess- & Gärdaten visuell auswerten:** Gärverlaufs-, Extrakt-, Druck- und Temperaturkurven sowie reichhaltigere Prozess-Auswertungen direkt aus den chronologischen JSON-LD-Schritten (`recipeInstructions` / `brew:measurements` / `brew:settings`) rendern.
2. **Praxis-Features für Keller & Ausschank bieten:** Werkzeuge für den realen Kellerbetrieb bereitstellen, darunter Druck-/QR-Code-Etiketten für Kegs (z. B. Jolly Keg) und Flaschen, Zapf- und Abfüll-Übersichten sowie strukturierte Verkostungsnotizen (`tasting`).
3. **Interaktive Brau- & Met-Rechner im Browser bereitstellen:** Praxisrechner für den Brautag und die Kellerarbeit integrieren (z. B. Refraktometer-Alkoholfehler-Korrektur, Nachsüßungs-Berechnung auf Ziel-Brix/Plato, Spunddruck- und Karbonisierungsrechner).

## Positioning

Während klassische Brau-Apps (wie Brewfather oder Beersmith) geschlossene Cloud-Silos mit komplexen Eingabemasken sind, trennt Braupause Datenhaltung und Darstellung radikal:
- **AI-First Datenerfassung:** Unstrukturierte Notizen, Messwerte oder Sprachmemos aus dem Braukeller werden per AI-Agent direkt in streng schema-validierte, mathematisch geprüfte JSON-LD-Dateien (`batches/<BATCH-ID>.json`) übersetzt.
- **Offener Standard statt proprietärem Format:** Jeder Sud ist ein valides `schema.org/Recipe`-Dokument mit der Domänen-Erweiterung `https://heimbrauerei-sendling.de/schema/brew/v1#` und wird sowohl als statische HTML-Ansicht mit eingebettetem JSON-LD als auch über statische JSON-Endpunkte (`/api/batches.json`, `/api/batches/<ID>.json`) ausgeliefert.

## Operating Context

- **Keller- & Brau-Workflow:** Gebraut und vergoren wird u. a. in NC-Kegs (*Jolly Keg / Corny Keg*) mit Drucksteuerung über *Spunding Valve (BlowTie)* und Messung per optischem Refraktometer (`°Bx`, umgerechnet in realen Extrakt `°P` und `% vol` ABV).
- **Pflege-Ritual (`AGENTS.md`):** Während oder nach einem Arbeitsschritt im Keller (Anstellen, Druckanpassung, Messung, Stabilisierung mit Kaliumsorbat/Campden, Nachsüßung, Probenentnahme oder Abfüllung) gibt Lukas die Eckdaten an einen AI-Agenten weiter, der `batches/<ID>.json` aktualisiert und per `npm test` validiert.
- **Lese- & Einsatzkontext:** Abruf unterwegs am Smartphone direkt am Fass/Keg (z. B. über QR-Code am Gebinde oder kurzen Blick auf Restvolumen und aktuellen Status) sowie am Desktop beim Planen, Vergleichen oder Teilen von Rezepten.

## Capabilities and Constraints

- **Verbindliche Datenquelle (`batches/*.json`):** Alle Sud-Daten liegen im Ordner `batches/` und müssen zwingend gegen `schemas/brew-batch.schema.json` (JSON Schema Draft 2020-12) validieren (`npm run validate` / `scripts/validate.mjs`).
- **Mathematische Volumen-Buchführung (`brew:volumes`):** Der Validator prüft für jeden Sud exakt $\text{initialVolume} + \sum \text{ledger}[i].\text{delta} = \text{currentVolume}$. Jede Probeentnahme, Abschwenk-Differenz oder Abfüllung muss im Ledger verbucht sein.
- **Zutaten-Synchronität:** Spätere Prozesszugaben in `recipeInstructions[*].brew:additions` müssen synchron in der Gesamt-Zutatenliste `brew:ingredients` mit der jeweiligen `brew:stage` geführt werden.
- **Statischer Build & GitHub Pages (`scripts/build.mjs`):** Die Website wird ohne Laufzeit-Backend nach `dist/` generiert und via GitHub Actions (`.github/workflows/ci-and-pages.yml`) auf GitHub Pages veröffentlicht. Interaktive Rechner, Diagramme und Etiketten-Ansichten müssen rein clientseitig bzw. zur Build-Zeit im statischen Output funktionieren.
- **Pflicht-Check:** Jede Änderung an Sud-Daten oder am Build-Prozess muss `npm test` (`npm run validate && npm run build`) mit Exit-Code `0` bestehen.
- **Sprache & Lokalisierung:** Oberfläche, Fachterminologie und Datums-/Zahlenformatierung sind durchgehend Deutsch (`de-DE`, Zeitzone `Europe/Berlin`, dezimales Komma in der Anzeige).

## Brand Commitments

- **Name:** Braupause
- **Brauer-Angabe:** Pro Sud individuell über `author.name` im jeweiligen JSON-Datensatz (z. B. `Lukas & Simon` bei `MET-2026-01`), ohne feste Personen-Nennung im globalen Seiten-Footer.
- **Tonalität:** Fachkundig, präzise, handwerklich und schnörkellos — wie ein sorgfältig geführtes Keller- und Laborbuch ohne Marketing-Floskeln.

## Evidence on Hand

- **Reale Sud-Dokumentation:** `batches/MET-2026-01.json` (*Klassischer Met (Halbtrocken)*, Traditional Mead im Jolly Keg, 17,0 L Startvolumen, 12,14 L aktueller Bestand, OG 22,6 °P, 10,0 °Bx nach Stabilisierung und Nachsüßung, 11,8 % vol ABV, 5 dokumentierte Prozessschritte).
- **Schema & Validierung:** `schemas/brew-batch.schema.json` (v1.1.0) und `scripts/validate.mjs`.
- **Bestehender Generator:** `scripts/build.mjs` (erzeugt `dist/index.html`, `dist/batches/MET-2026-01.html`, `dist/styles.css`, `dist/api/batches.json`, `dist/api/batches/MET-2026-01.json` und `dist/schemas/brew-batch.schema.json`).
- **Bewusste Abwesenheiten (nicht erfinden):** Keine kommerziellen Verkaufsfunktionen, Preise, Kundenbewertungen, fiktiven Auszeichnungen oder erfundenen Sude/Messwerte; alle dargestellten Messpunkte und Mengen müssen aus echten Sud-Dateien stammen oder in Rechnern klar als Benutzereingabe erkennbar sein.

## Product Principles

1. **Messwert-Treue vor Dekoration:** Jede Zahl, Kurve und Bilanz auf der Website leitet sich nachvollziehbar aus den validierten JSON-LD-Daten ab; nichts wird geschätzt oder im Fließtext versteckt, was als strukturierter Messwert (`brew:measurements`, `brew:settings`, `brew:volumes`) vorliegt.
2. **Keller-Tauglichkeit auf einen Blick:** Im Braukeller zählen schnelle Ablesbarkeit von Status, Restvolumen im Gebinde, Druck, Extrakt und letztem Schritt — auf dem Smartphone am Keg ebenso wie am Desktop.
3. **Statische Robustheit mit praktischem Mehrwert:** Neue Funktionen wie Gärkurven, Braurechner und Keg-/Flaschen-Etiketten erweitern den praktischen Nutzen im Braualltag, ohne die Abhängigkeitfreiheit des statischen GitHub-Pages-Deployments aufzugeben.
4. **Offene Reproduzierbarkeit für Heimbrauer:** Rezepte, Gaben über alle Phasen und Prozessschritte werden so transparent dargestellt, dass andere Brauer und Met-Macher den Sud sowohl visuell als auch über das eingebettete Schema.org / JSON-LD vollständig nachvollziehen können.
