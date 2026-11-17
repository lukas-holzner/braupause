---
name: Braupause
description: Sud-Register, Kellerbuch und Gärprotokoll im Maker's Blueprint & Honey Cellar Journal
colors:
  bg-page: "#EBF1FA"
  bg-surface: "#FBF7EE"
  bg-subtle: "#EDE3CE"
  bg-row-hover: "#FFF6E0"
  ink-primary: "#12284A"
  ink-secondary: "#334766"
  ink-muted: "#526685"
  blueprint-blue: "#1F4E8C"
  blueprint-tint: "#E6EFFA"
  amber-accent: "#B45309"
  amber-gold: "#F4A93A"
  amber-hover: "#92400E"
  amber-tint: "#FFF0C2"
  olive-status: "#1E5631"
  olive-tint: "#DCFCE7"
  grape-purple: "#7E44A8"
  grape-tint: "#F3E8FF"
  border-hairline: "#D8CCB4"
  border-strong: "#12284A"
typography:
  display:
    fontFamily: '"Iowan Old Style", "Palatino Linotype", Palatino, "Book Antiqua", Georgia, serif'
    fontSize: "clamp(1.85rem, 3.5vw, 2.65rem)"
    fontWeight: 700
    lineHeight: 1.18
    letterSpacing: "-0.015em"
  headline:
    fontFamily: '"Iowan Old Style", "Palatino Linotype", Palatino, "Book Antiqua", Georgia, serif'
    fontSize: "1.4rem"
    fontWeight: 700
    lineHeight: 1.3
    letterSpacing: "-0.015em"
  title:
    fontFamily: '"Iowan Old Style", "Palatino Linotype", Palatino, "Book Antiqua", Georgia, serif'
    fontSize: "1.12rem"
    fontWeight: 700
    lineHeight: 1.35
    letterSpacing: "-0.015em"
  body:
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif'
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.55
  label:
    fontFamily: 'ui-monospace, "SFMono-Regular", Menlo, Monaco, Consolas, monospace'
    fontSize: "0.85rem"
    fontWeight: 500
    lineHeight: 1.4
    fontFeature: '"tnum" 1'
rounded:
  xs: "4px"
  sm: "6px"
  md: "10px"
  pill: "999px"
spacing:
  xs: "0.5rem"
  sm: "0.75rem"
  md: "1rem"
  lg: "1.5rem"
  xl: "2rem"
  2xl: "2.75rem"
components:
  button-primary:
    backgroundColor: "{colors.amber-accent}"
    textColor: "{colors.bg-surface}"
    typography: "{typography.body}"
    rounded: "{rounded.sm}"
    padding: "0.55rem 1rem"
  button-primary-hover:
    backgroundColor: "{colors.amber-hover}"
    textColor: "{colors.bg-surface}"
  button-secondary:
    backgroundColor: "{colors.bg-subtle}"
    textColor: "{colors.ink-primary}"
    typography: "{typography.body}"
    rounded: "{rounded.sm}"
    padding: "0.5rem 0.95rem"
  input-search:
    backgroundColor: "{colors.bg-page}"
    textColor: "{colors.ink-primary}"
    typography: "{typography.body}"
    rounded: "{rounded.sm}"
    padding: "0.55rem 0.85rem"
  chip-code-id:
    backgroundColor: "{colors.bg-subtle}"
    textColor: "{colors.ink-primary}"
    typography: "{typography.label}"
    rounded: "{rounded.xs}"
    padding: "0.25rem 0.6rem"
  chip-step-badge:
    backgroundColor: "{colors.amber-tint}"
    textColor: "{colors.amber-hover}"
    typography: "{typography.label}"
    rounded: "{rounded.xs}"
    padding: "0.25rem 0.6rem"
  chip-status-pill:
    backgroundColor: "{colors.olive-tint}"
    textColor: "{colors.olive-status}"
    rounded: "{rounded.pill}"
    padding: "0.25rem 0.75rem"
  card-protocol:
    backgroundColor: "{colors.bg-surface}"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.md}"
    padding: "1.35rem 1.5rem"
  meter-volume:
    backgroundColor: "{colors.bg-surface}"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.md}"
    padding: "1.1rem 1.25rem"
---

# Design System: Braupause

## Overview

**Creative North Star: "Maker's Blueprint & Honey Cellar Journal"**

Braupause überträgt die Bildsprache des sechseckigen Waben-Blaupausen-Emblems direkt in ein gemütliches, handwerklich-verspieltes Brau- & Werkbuch. Auf einer hellen Blaupausen-Unterlage (`#EBF1FA`) liegen warme Creme-Karten (`#FBF7EE`), eingefasst von kräftigen Tief-Marine-Konturen (`#12284A`) wie in einem Werkstatt-Wappen. Gerasterte Blaupausen-Banner (`#1F4E8C`) setzen den technischen Rahmen, während honiggoldene Schaugläser, montierte Keller-Schnappschüsse und wappencodierte Zutaten-Badges Wärme und Braufreude transportieren.

Jedes Detail verbindet präzise Messwert-Treue mit einladender Zugänglichkeit: Echte Fotos aus dem Gärkeller stehen gleichberechtigt neben klaren Stammwürze-, Restextrakt- und Spunddruck-Werten; nichts wird erfunden, alles erzählt den realen Sud. Bewusste Anti-Referenzen bleiben steife Behördenformulare, kühle Grautabellen ohne Atmosphäre sowie überladene Neon-Dashboards.

**Key Characteristics:**
- **Blaupause trifft Honigkeller:** Das tiefe Blaupausen-Blau (`#1F4E8C`) mit feinem 24px-Raster bildet die Kopfzeilen der Sud-Dossiers, während Waben-Gold (`#F4A93A`) und Spund-Bernstein (`#B45309`) lebendige Füllstände und Prozess-Meilensteine markieren.
- **Kräftige Wappen-Konturen & sanfter Taktiler Lift:** Warme Werkbuch-Karten (`#FBF7EE`) tragen markante `1.5px`–`2px` Konturen in Tief-Marine (`#12284A`) mit weichem Offset-Schatten — genau wie die Umrisse des Braupause-Emblems.
- **Emblem-Farbcodierung für Zutaten & Phasen:** Honig und Vergärbares leuchten in Waben-Gold, Fruchtsäfte in Trauben-Violett (`#7E44A8`), Hefe und Hopfen in Hopfen-Grün (`#1E5631`) und Wasser sowie Stabilisatoren in Blaupausen-Blau.
- **Montierte Schnappschüsse statt leerer Flächen:** Echte Sud-Fotos werden wie Werkbuch-Abzüge auf Hafermalz-Matt (`#EDE3CE`) mit Tief-Marine-Rahmen und Bildunterschrift präsentiert und per Klick in einer nativen Lightbox vergrößert.

## Colors

Die Farbpalette stammt unmittelbar aus dem Braupause-Emblem (`assets/logo/braupause-emblem.svg`) und verbindet warmes Werkbuch-Papier mit den charakteristischen Schild-, Honig-, Hopfen- und Trauben-Tönen.

### Primary
- **Blaupausen-Blau / Blueprint Blue** (`#1F4E8C`): Strukturierende Leitfarbe für die gerasterten Dossier-Banner, Timeline-Verbindungslinien und das Wappen-Motiv.
- **Spund-Bernstein / Mead Amber** (`#B45309`): Primäre Interaktions- und Akzentfarbe für Links, Fokusringe (`2px solid`), aktive Schaltflächen und die Basis des Honig-Füllstandsbalkens.
- **Waben-Gold / Honeycomb Gold** (`#F4A93A`): Warme Highlight-Farbe im Verlauf des Fass-Schauglases und im Emblem.
- **Dunkler Waldhonig / Deep Mead Amber** (`#92400E`): Kontrastreicher Hover-Zustand sowie Textfarbe auf Jungmet-Glanz-Badges (`#FFF0C2`) für sichere Lesbarkeit (> 6:1).
- **Jungmet-Glanz / Warm Honey Tint** (`#FFF0C2`): Warmer Hintergrund für Prozessphasen-Stempel (`.step-badge`), Textmarkierungen (`::selection`) und Honig-/Malz-Zutatenbadges.

### Secondary
- **Hopfen-Oliv / Bottle Green** (`#1E5631`) & **Hopfen-Grünglas** (`#DCFCE7`): Statusfarbe in `.status-pill` sowie Kategorie-Badge für Hefe, Nährstoffe und Hopfen.
- **Trauben-Violett / Berry Must** (`#7E44A8`) & **Trauben-Duft** (`#F3E8FF`): Kategorie-Badge für Fruchtsaft- und Fruchtgaben im Sud.
- **Blaupausen-Nebel** (`#E6EFFA`): Kategorie-Badge-Hintergrund für Wasser, Säure und Stabilisatoren (mit `#1F4E8C` Text).

### Neutral
- **Blaupausen-Unterlage / Blueprint Mat** (`#EBF1FA`): Grundfläche der gesamten Seite (`--bg-page`), direkt aus dem Blaupausen-Blau abgeleitet.
- **Warmes Werkbuch-Creme / Journal Card Cream** (`#FBF7EE`): Primäre Karten- und Tabellenfläche (`--bg-surface`).
- **Hafermalz-Beige / Malt Tint** (`#EDE3CE`): Sekundäre Fläche (`--bg-subtle`) für Tabellenköpfe, Chargen-IDs, Foto-Passepartouts und leere Füllstandsspuren.
- **Honig-Schimmer / Warm Row Hover** (`#FFF6E0`): Sanfte Zeilen-Hervorhebung in Tabellen.
- **Tief-Marine / Emblem Ink** (`#12284A`): Primäre Textfarbe (`--ink-primary`) sowie markante Außenkontur (`--border-strong`) von Karten, Buttons und Fotomatten.
- **Werkstatt-Schieferblau** (`#334766`): Sekundäre Textfarbe (`--ink-secondary`) für Untertitel, Labels und Zeitstempel.
- **Skalen-Blaugrau** (`#526685`): Zurückgenommene Textfarbe (`--ink-muted`) für Fußnoten und Referenzwerte.
- **Werkbuch-Haarlinie** (`#D8CCB4`): Feine innere Trennlinie (`--border-hairline`) zwischen Tabellenzeilen und Messwert-Einträgen.

### Named Rules
**The Emblem Outline Rule.** Äußere Hauptkarten (`.batch-hero`, `.batch-showcase-card`, `.timeline-step`) und montierte Foto-Rahmen tragen eine klare `1.5px` bis `2px` Kontur in Tief-Marine (`#12284A`) — angelehnt an die prägnante Strichstärke des Braupause-Emblems.

**The Blueprint Mat & Cream Card Rule.** Die Grundfläche ruht auf heller Blaupausen-Unterlage (`#EBF1FA`), darauf liegen Karten und Register in warmem Werkbuch-Creme (`#FBF7EE`) — so wirken Blaupausen-Blau und Honig-Gold gemütlich zusammen, ohne kühle Grauflächen.

## Typography

**Display Font:** `"Iowan Old Style", "Palatino Linotype", Palatino, "Book Antiqua", Georgia, serif`
**Body Font:** `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`
**Label/Mono Font:** `ui-monospace, "SFMono-Regular", Menlo, Monaco, Consolas, monospace`

**Character:** Buchdruck-Serifen geben Überschriften und der Wortmarke Charakter, während klare System-Sans und Monospace mit tabellarischen Ziffern jede Messung im Keller blitzschnell ablesbar halten.

### Hierarchy
- **Display** (`700`, `clamp(1.85rem, 3.5vw, 2.65rem)`, `line-height: 1.18`, `letter-spacing: -0.015em`): Seitentitel (`h1`).
- **Headline** (`700`, `1.4rem`, `line-height: 1.3`, `letter-spacing: -0.015em`): Sektionsüberschriften (`h2`) und die Header-Wortmarke `.site-brand`.
- **Title** (`700`, `1.12rem`, `line-height: 1.35`, `letter-spacing: -0.015em`): Schritt-Titel (`h3`) und Steckbrief-Überschriften.
- **Body** (`400`, `1rem`, `line-height: 1.55`): Fließtext (`max-width: 70ch`), Anweisungen (`max-width: 68ch`), Tabellen und Ledger.
- **Label** (`500`–`600`, `0.85rem`, `line-height: 1.4`): Einheitliche Label-Stufe in `--font-mono` (`.code-id`, `.step-badge`, `.step-time`) sowie kompakten Metadaten, Chips, Tabellenköpfen (`th`) und Bildunterschriften.

### Named Rules
**The Tabular Measurement Rule.** Jede Zahl — Stammwürze (`°P`), Refraktometer (`°Bx`), Alkoholgehalt (`% vol`), Spunddruck (`PSI`), Volumen (`L`) oder Datum — wird mit `font-variant-numeric: tabular-nums` (`.num`) und geschütztem Leerzeichen (`\u00A0`) zur Einheit gesetzt.

**The Five-Step Type Discipline.** Sämtliche CSS-Regeln bleiben strikt auf den fünf dokumentierten Größenstufen (`clamp(1.85rem, 3.5vw, 2.65rem)`, `1.4rem`, `1.12rem`, `1rem`, `0.85rem`), ohne beliebige Zwischenstufen einzuführen.

## Layout

Zentrierter Werkbuch-Container (`.container`, `max-width: 68rem`, `margin: 0 auto`, `padding: 2.25rem 1.5rem 5rem`; unter `640px` auf `1.35rem 1rem 3.5rem`).

- **Seitenkopf (`.site-header`):** Flex-Zeile mit `.brand-lockup` links (`4rem`-Emblem, Wortmarke „Brau**pause**“ und Untertitel) und `.header-meta` rechts, abgeschlossen durch `2px solid #12284A`.
- **Sud-Steckbrief-Showcase (`index.html`):** Karten-Grid (`.batch-showcase-grid`) oberhalb der Such- und Tabellenansicht; jede Karte kombiniert gerastertes Blaupausen-Kopfband, montiertes Sud-Foto, Kennzahl-Chips und Honig-Schauglas.
- **Zweispaltiges Dossier & Spezifikations-Deck (`batches/*.html`):** Im `.batch-hero` liegen Titel, Beschreibung und Metadaten links neben der klickbaren Foto-Pinwand (`.hero-photo-deck`) rechts; darunter ordnet `.spec-grid` Kennzahlen-Ledger und Volumen-Schauglas nebeneinander an.
- **Durchgezogener Protokoll-Pfad (`.timeline`):** Chronologische Schritte hängen an einer `2px` starken Blaupausen-Linie (`#1F4E8C`) und bündeln Anweisungen, Messwert-Boxen und Schritt-Fotos in einer Karte.

## Elevation & Depth

Weiche, wappenhafte Zweischicht-Tiefe: Innere Register (`.dl-ledger`, `.volume-meter`, `table`, `.step-box`) bleiben flach mit `1.5px` Linien (`#D8CCB4` bzw. `#12284A`), während äußere Werkbuch-Karten (`.batch-hero`, `.batch-showcase-card`, `.timeline-step`) einen kompakten, weichen Tief-Marine-Schatten (`0 3px 8px -2px rgba(18, 40, 74, 0.14)`) tragen.

### Named Rules
**The Flat Ledger Rule.** Verschachtelte Kästen innerhalb einer Karte werfen keinen eigenen Schatten — Tiefe im Inneren entsteht ausschließlich über Linien, Raster und Farbtönungen.

## Shapes

- **Stempel & Badges (`4px`):** `.code-id`, `.step-badge`, `.cat-badge`, `.tag-item` und Foto-Ecken.
- **Kleine Instrumente & Rahmen (`6px`):** `.search-input`, Buttons, `.hero-blueprint-bar`, `.step-box`, `.hero-photo-card`, `.step-photo-figure`.
- **Werkbuch-Karten & Register (`10px`):** `.batch-hero`, `.batch-showcase-card`, `.timeline-step`, `.dl-ledger`, `.volume-meter`, `.filter-bar`, `<dialog>`.
- **Status- & Füllstandskapseln (`999px`):** Ausschließlich `.status-pill` sowie `.volume-bar-track` / `.mini-gauge-track`.

## Components

### Buttons
- **Shape:** `6px` Radius mit `1.5px solid #12284A`-Kontur.
- **Primary:** Spund-Bernstein (`#B45309`) auf `#FBF7EE` Schrift, hover `#92400E`.
- **Secondary:** Hafermalz-Beige (`#EDE3CE`) mit Tief-Marine-Schrift (`#12284A`), hover wechselt auf `#FFF0C2` mit leichtem Hub (`translateY(-1px)`).

### Chips
- **Batch ID Stamp (`.code-id`):** Monospace (`0.85rem`), `#EDE3CE` auf `#12284A` (auf Blaupausen-Bannern invertiert via `.code-id-light` in `#FBF7EE` mit `#12284A` Text).
- **Phase Step Badge (`.step-badge`):** Monospace (`0.85rem`), `#FFF0C2` Hintergrund mit `#92400E` Text (`4px` Radius).
- **Ingredient Category Badge (`.cat-badge`):** Semantische Emblem-Farbcodierung in Gold, Trauben-Violett, Hopfen-Grün oder Blaupausen-Blau (`0.85rem`, `4px` Radius).
- **Status Pill (`.status-pill`):** `#DCFCE7` Hintergrund, `#1E5631` Text, `999px` Radius.

### Cards & Photo Mounts
- **Werkbuch-Foto-Rahmen (`.hero-photo-card`, `.step-photo-card`):** Montiert auf Hafermalz-Beige (`#EDE3CE`) mit `1.5px solid #12284A`-Kontur, `0.6rem` Passepartout-Padding, knackiger Bildunterschrift (`0.85rem`) und sanfter Neigung (`rotate(-1.2deg)` / `rotate(1.2deg)` im Hero-Deck), die beim Hover gerade richtet und per nativem `<dialog>` vergrößert werden kann.

### Volume & Gauge Meters (Signature Component)
- **Fass-Schauglas (`.volume-meter`):** Weiß-cremiges Instrumenten-Panel (`10px` Radius, `1.5px solid #12284A`) mit `0.85rem` hohem Schauglas-Rohr (`#EDE3CE`, `1.5px solid #12284A`, `999px` Radius), gefüllt mit Honig-Verlauf (`#B45309` → `#F4A93A`).

## Do's and Don'ts

### Do:
- **Do** zeige echte Sud-Fotos aus `batches/*.json` als gerahmte Werkbuch-Abzüge mit ehrlichen Bildunterschriften und Lightbox-Ansicht.
- **Do** halte alle Schriftgrößen strikt auf den fünf dokumentierten Stufen (`clamp(1.85rem, 3.5vw, 2.65rem)`, `1.4rem`, `1.12rem`, `1rem`, `0.85rem`) und alle Zahlen in `tabular-nums`.
- **Do** nutze die Emblem-Farbcodierung konsequent für Zutatenkategorien, Blaupausen-Banner und Status-Badges.

### Don't:
- **Don't** erfinde künstliche Rechner, synthetische Barcodes oder nicht belegte Messwerte — jede Zahl und jedes Foto stammt direkt aus `batches/*.json`.
- **Don't** setze helle/bernsteinfarbene Schrift mit zu geringem Kontrast auf `#FFF0C2` (nutze dort mindestens `#92400E` oder `#12284A`).
- **Don't** zeige JSON-Schema-Links oder technische API-Hinweise im Frontend.
