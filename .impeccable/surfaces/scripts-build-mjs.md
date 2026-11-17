---
version: 1
slug: "scripts-build-mjs"
primary_target: "scripts/build.mjs"
related_targets: ["dist/index.html","dist/batches/MET-2026-01.html"]
---

# Surface Brief: Braupause — Sud-Register & Sud-Detailseite

- **Scope & Mode:** `operate` (beide Seiten gemeinsam: `dist/index.html` als Sud-Register & Kellerbuch und `dist/batches/MET-2026-01.html` als Sud-Detailprotokoll).
- **Users & Job:** Heimbrauer (inkl. Mitbrauer wie Lukas & Simon) prüfen am Handy im Keller oder am Desktop schnell Zustand, Fassbestand, Kennzahlen, Zutaten und chronologische Gärschritte anhand echter JSON-Daten und realer Fotos.
- **Constraints:** Keine erfundenen Rechner, Barcodes oder synthetischen Widgets; keine JSON-Schema-Links im Frontend; FabLab München e.V.-Hinweis im Footer der Startseite; Brauer (`author.name`) dynamisch pro Sud.

## Direction contract

- **THESIS:** Ein gemütliches, taktiles Brau- & Werkbuch (*Maker's Blueprint & Honey Cellar Journal*), das technische Braupräzision mit handwerklicher Kellerwärme verbindet und das Waben-Blaupausen-Emblem direkt in die gesamte Seitenarchitektur übersetzt.
- **OWN-WORLD:** Warmes Werkbuch-Creme (`#F6F0E2`) und frisches Etiketten-Blatt (`#FFFDF8`) mit kräftigen Tief-Marine-Konturen (`#12284A`), Blaupausen-Blau (`#1F4E8C`) mit dezentem 24px-Raster im Hero-Banner, Honig-Gold (`#F4A93A` / `#B45309`) für Waben-Akzente und Fass-Füllstände sowie Hopfen-Grün (`#1E5631` / `#DCFCE7`) und Trauben-Violett (`#7E44A8` / `#F3E8FF`) zur Farbcodierung von Zutaten und Status; Serif-Überschriften gepaart mit Sans-Body und Monospace-Ziffern (`tabular-nums`).
- **STORY:** Beim Öffnen erkennt der Besucher sofort die lebendige Braupraxis hinter den Zahlen — echte Fotos vom Jolly Keg und der Verkostungsprobe stehen gleichberechtigt neben klarem Fass-Füllstand, Stammwürze, Restextrakt und dem Schritt-für-Schritt-Gärprotokoll.
- **FIRST VIEWPORT:** Auf der Startseite zeigt das Header-Lockup das Waben-Emblem mit Wortmarke, gefolgt von einer einladenden Sud-Steckbrief-Karte mit Foto-Vorschau, vier kompakten Kennzahl-Chips und Honig-Füllstandsbalken über dem durchsuchbaren Register. Auf der Detailseite krönt ein gerastertes Blaupausen-Banner (`#1F4E8C`) den zweispaltigen Dossier-Hero aus Titel/Metadaten links und klickbaren Werkbuch-Schnappschüssen rechts.
- **FORM:** Candidate 1 ("Maker's Blueprint & Honey Cellar Journal", Impeccable's Pick), Seed Key `61b90611`.
- **FINISH:** unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
