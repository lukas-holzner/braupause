import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, '..');
const BATCHES_DIR = path.join(ROOT_DIR, 'batches');
const SCHEMA_PATH = path.join(ROOT_DIR, 'schemas', 'brew-batch.schema.json');
const DIST_DIR = path.join(ROOT_DIR, 'dist');
const LOGO_DIR = path.join(ROOT_DIR, 'assets', 'logo');
const PAGES_DIR = path.join(ROOT_DIR, 'pages');
const BATCH_ASSETS_DIR = path.join(ROOT_DIR, 'assets', 'batches');

const STAGE_LABELS = {
  mash: 'Maischen',
  boil: 'Kochen',
  primary: 'Hauptansatz / Primärgärung',
  secondary: 'Sekundärgärung',
  stabilization: 'Stabilisierung',
  back_sweetening: 'Nachsüßung',
  conditioning: 'Reifung',
  packaging: 'Abfüllung'
};

const CATEGORY_LABELS = {
  water: 'Wasser',
  fermentable: 'Vergärbarer Zucker / Malz / Honig',
  juice: 'Fruchtsaft',
  fruit: 'Frucht',
  hops: 'Hopfen',
  spice_and_herb: 'Gewürze & Kräuter',
  yeast: 'Hefe',
  nutrient: 'Hefenährsalz',
  yeast_and_nutrient: 'Hefe & Nährstoffe',
  acid: 'Säure',
  stabilizer: 'Stabilisator',
  fining: 'Schönungsmittel',
  other: 'Sonstiges'
};

const CATEGORY_TONES = {
  fermentable: 'tone-amber',
  juice: 'tone-grape',
  fruit: 'tone-grape',
  yeast: 'tone-olive',
  nutrient: 'tone-olive',
  yeast_and_nutrient: 'tone-olive',
  hops: 'tone-olive',
  spice_and_herb: 'tone-olive',
  water: 'tone-blueprint',
  acid: 'tone-blueprint',
  stabilizer: 'tone-blueprint',
  fining: 'tone-blueprint',
  other: 'tone-blueprint'
};

const PHASE_LABELS = {
  preparation: 'Vorbereitung',
  mashing: 'Maischen',
  boiling: 'Würzekochen',
  pitching: 'Anstellen',
  primary_fermentation: 'Primärgärung',
  secondary_fermentation: 'Sekundärgärung',
  degassing: 'Entgasung & Druckreduktion',
  measurement: 'Messung',
  stabilization_and_sweetening: 'Stabilisierung & Nachsüßung',
  conditioning: 'Reifung & Klärung',
  racking: 'Umschlauchen',
  packaging: 'Abfüllung',
  tasting: 'Verkostung'
};

const ROLE_LABELS = {
  fermenter: 'Gärbehälter',
  pressure_control: 'Drucksteuerung',
  measurement: 'Messgerät',
  temperature_control: 'Temperatursteuerung',
  filtration: 'Filtration',
  bottling: 'Abfüllung',
  other: 'Equipment'
};

function escapeHtml(str) {
  if (str === null || str === undefined) return '';
  return String(str)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

/** Favicon & touch icon links; `root` is the relative path to dist/ ('./' or '../'). */
function renderIconLinks(root) {
  return `<link rel="icon" href="${root}assets/logo/braupause-favicon-32.png" type="image/png" sizes="32x32">
  <link rel="icon" href="${root}assets/logo/braupause-favicon.svg" type="image/svg+xml">
  <link rel="apple-touch-icon" href="${root}assets/logo/braupause-touch-icon.png">`;
}

/** Emblem + serif wordmark + subtitle for the site header. `subtitleHtml` must already be escaped. */
function renderBrandLockup(root, subtitleHtml) {
  return `<div class="brand-lockup">
        <a class="brand-emblem" href="${root}index.html" tabindex="-1" aria-hidden="true">
          <img src="${root}assets/logo/braupause-emblem.svg" alt="" width="80" height="80">
        </a>
        <div>
          <p class="site-brand"><a href="${root}index.html">Brau<span class="brand-accent">pause</span></a></p>
          <p class="site-subtitle">${subtitleHtml}</p>
        </div>
      </div>`;
}

function formatNumber(val, decimals = 2) {
  if (typeof val !== 'number') return '';
  return val.toLocaleString('de-DE', {
    minimumFractionDigits: Number.isInteger(val) ? 0 : 1,
    maximumFractionDigits: decimals
  });
}

function formatQuantity(qv) {
  if (!qv || typeof qv.value !== 'number') return '—';
  const prefix = qv.maxValue && qv.value < qv.maxValue ? `< ${formatNumber(qv.maxValue)}` : formatNumber(qv.value);
  return `${prefix}\u00A0${escapeHtml(qv.unitText || '')}`;
}

function formatDate(isoStr, includeTime = true) {
  if (!isoStr) return '—';
  const d = new Date(isoStr);
  if (Number.isNaN(d.getTime())) return escapeHtml(isoStr);
  const datePart = d.toLocaleDateString('de-DE', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    timeZone: 'Europe/Berlin'
  });
  if (!includeTime) return datePart;
  const timePart = d.toLocaleTimeString('de-DE', {
    hour: '2-digit',
    minute: '2-digit',
    timeZone: 'Europe/Berlin'
  });
  return `${datePart}, ${timePart}\u00A0Uhr`;
}

function findLatestProperty(batch, propertyIds) {
  const ids = Array.isArray(propertyIds) ? propertyIds : [propertyIds];
  const steps = batch.recipeInstructions || [];
  for (let i = steps.length - 1; i >= 0; i--) {
    const step = steps[i];
    const items = [...(step['brew:measurements'] || []), ...(step['brew:settings'] || [])];
    for (let j = items.length - 1; j >= 0; j--) {
      if (ids.includes(items[j].propertyID)) {
        return items[j];
      }
    }
  }
  return null;
}

function cleanJsonLd(batch) {
  const clone = structuredClone(batch);
  delete clone.$schema;
  return JSON.stringify(clone, null, 2);
}

function buildStyles() {
  return `:root {
  --bg-page: #EBF1FA;
  --bg-surface: #FBF7EE;
  --bg-subtle: #EDE3CE;
  --bg-row-hover: #FFF6E0;
  --ink-primary: #12284A;
  --ink-secondary: #334766;
  --ink-muted: #526685;
  --blueprint-blue: #1F4E8C;
  --blueprint-tint: #E6EFFA;
  --amber-accent: #B45309;
  --amber-gold: #F4A93A;
  --amber-hover: #92400E;
  --amber-tint: #FFF0C2;
  --olive-status: #1E5631;
  --olive-tint: #DCFCE7;
  --grape-purple: #7E44A8;
  --grape-tint: #F3E8FF;
  --border-hairline: #D8CCB4;
  --border-strong: #12284A;
  --shadow-card: 0 3px 8px -2px rgba(18, 40, 74, 0.14);
  --font-serif: "Iowan Old Style", "Palatino Linotype", Palatino, "Book Antiqua", Georgia, serif;
  --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  --font-mono: ui-monospace, "SFMono-Regular", Menlo, Monaco, Consolas, monospace;
}

::selection {
  background-color: var(--amber-tint);
  color: var(--ink-primary);
}

*, *::before, *::after {
  box-sizing: border-box;
}

html {
  background-color: var(--bg-page);
  color: var(--ink-primary);
  font-family: var(--font-sans);
  line-height: 1.55;
  -webkit-font-smoothing: antialiased;
  scrollbar-color: var(--border-strong) var(--bg-page);
  accent-color: var(--amber-accent);
  caret-color: var(--amber-accent);
}

body {
  margin: 0;
  padding: 0;
  font-size: 1rem;
}

a {
  color: var(--amber-hover);
  text-decoration: underline;
  text-underline-offset: 0.2em;
  text-decoration-thickness: 1.5px;
  transition: color 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

a:hover {
  color: var(--blueprint-blue);
}

a:focus-visible,
input:focus-visible,
select:focus-visible,
button:focus-visible {
  outline: 2px solid var(--amber-accent);
  outline-offset: 2px;
}

.container {
  max-width: 68rem;
  margin: 0 auto;
  padding: 2.25rem 1.5rem 5rem;
}

/* Header */
.site-header {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 1rem 1.5rem;
  padding-bottom: 1.35rem;
  margin-bottom: 2.25rem;
  border-bottom: 2px solid var(--border-strong);
}

.brand-lockup {
  display: flex;
  align-items: center;
  gap: 0.9rem;
}

.brand-emblem {
  display: block;
  flex: none;
  line-height: 0;
}

.brand-emblem img {
  display: block;
  width: 5rem;
  height: 5rem;
  transition: transform 200ms cubic-bezier(0.22, 1, 0.36, 1);
}

.brand-lockup:hover .brand-emblem img {
  transform: rotate(-6deg) scale(1.05);
}

.site-brand {
  font-family: var(--font-serif);
  font-size: 1.75rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin: 0;
}

.site-brand a {
  color: var(--ink-primary);
  text-decoration: none;
}

.brand-accent {
  color: var(--blueprint-blue);
}

.site-subtitle {
  margin: 0.2rem 0 0;
  color: var(--ink-secondary);
  font-size: 0.95rem;
  font-weight: 500;
}

.header-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.85rem;
  font-size: 0.85rem;
  color: var(--ink-secondary);
}

/* Typography */
h1, h2, h3 {
  font-family: var(--font-serif);
  color: var(--ink-primary);
  letter-spacing: -0.015em;
  text-wrap: balance;
}

h1 {
  font-size: clamp(1.85rem, 3.5vw, 2.65rem);
  line-height: 1.18;
  margin: 0 0 0.75rem;
}

h2 {
  font-size: 1.4rem;
  line-height: 1.3;
  margin: 2.75rem 0 1rem;
  padding-bottom: 0.45rem;
  border-bottom: 1.5px solid var(--border-hairline);
}

h3 {
  font-size: 1.12rem;
  line-height: 1.35;
  margin: 0 0 0.35rem;
}

p {
  max-width: 70ch;
  margin: 0 0 1rem;
}

.num,
td.num,
th.num {
  font-variant-numeric: tabular-nums;
}

/* Buttons */
.btn-primary,
.btn-secondary {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  font-family: var(--font-sans);
  font-weight: 600;
  line-height: 1.4;
  text-decoration: none;
  border: 1.5px solid var(--border-strong);
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
    transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.btn-primary {
  font-size: 1rem;
  padding: 0.55rem 1rem;
  background-color: var(--amber-accent);
  color: var(--bg-surface);
}

.btn-primary:hover {
  background-color: var(--amber-hover);
  color: var(--bg-surface);
  transform: translateY(-1px);
}

.btn-secondary {
  font-size: 0.85rem;
  padding: 0.5rem 0.95rem;
  background-color: var(--bg-subtle);
  color: var(--ink-primary);
}

.btn-secondary:hover {
  background-color: var(--amber-tint);
  color: var(--ink-primary);
  transform: translateY(-1px);
}

/* Badges & Chips */
.code-id {
  display: inline-block;
  font-family: var(--font-mono);
  font-size: 0.85rem;
  font-weight: 600;
  line-height: 1.4;
  color: var(--ink-primary);
  background: var(--bg-subtle);
  padding: 0.25rem 0.6rem;
  border-radius: 4px;
}

.code-id-light {
  background: var(--bg-surface);
  color: var(--ink-primary);
}

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.85rem;
  font-weight: 600;
  padding: 0.25rem 0.75rem;
  border-radius: 999px;
  background-color: var(--olive-tint);
  color: var(--olive-status);
}

.cat-badge {
  display: inline-block;
  font-size: 0.85rem;
  font-weight: 600;
  line-height: 1.4;
  padding: 0.25rem 0.6rem;
  border-radius: 4px;
}

.tone-amber {
  background: var(--amber-tint);
  color: var(--amber-hover);
}

.tone-olive {
  background: var(--olive-tint);
  color: var(--olive-status);
}

.tone-grape {
  background: var(--grape-tint);
  color: var(--grape-purple);
}

.tone-blueprint {
  background: var(--blueprint-tint);
  color: var(--blueprint-blue);
}

/* Blueprint Banner inside Cards */
.blueprint-banner {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 0.65rem 1rem;
  padding: 0.8rem 1.1rem;
  margin-bottom: 1.35rem;
  background-color: var(--blueprint-blue);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24'%3E%3Cpath d='M24 0H0v24' fill='none' stroke='%23FFFDF8' stroke-opacity='0.15' stroke-width='1'/%3E%3C/svg%3E");
  border: 1.5px solid var(--border-strong);
  border-radius: 6px;
  color: var(--bg-surface);
}

.banner-group {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.6rem;
}

.banner-brewer {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--bg-surface);
}

/* Index Showcase Cards */
.batch-showcase-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 32rem), 1fr));
  gap: 1.5rem;
  margin: 1.5rem 0 2.5rem;
}

.batch-showcase-card {
  background: var(--bg-surface);
  border: 2px solid var(--border-strong);
  border-radius: 10px;
  padding: 1.35rem;
  box-shadow: var(--shadow-card);
}

.showcase-body {
  display: grid;
  grid-template-columns: minmax(0, 14rem) minmax(0, 1fr);
  gap: 1.25rem;
  align-items: start;
}

.showcase-body.no-photo {
  grid-template-columns: 1fr;
}

.showcase-photo-link {
  display: block;
  text-decoration: none;
}

.showcase-photo-frame {
  background: var(--bg-subtle);
  border: 1.5px solid var(--border-strong);
  border-radius: 6px;
  padding: 0.55rem;
  transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.showcase-photo-link:hover .showcase-photo-frame {
  transform: rotate(-1.5deg) translateY(-2px);
}

.showcase-photo-frame img {
  display: block;
  width: 100%;
  aspect-ratio: 4 / 3;
  object-fit: cover;
  border-radius: 4px;
}

.showcase-title {
  margin: 0 0 0.45rem;
  font-size: 1.4rem;
}

.showcase-title a {
  color: var(--ink-primary);
  text-decoration: none;
}

.showcase-title a:hover {
  color: var(--amber-accent);
  text-decoration: underline;
}

.showcase-sub {
  font-size: 0.85rem;
  color: var(--ink-secondary);
  margin: 0 0 0.75rem;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.6rem;
  margin: 0.85rem 0;
}

.metric-pill {
  background: var(--bg-page);
  border: 1px solid var(--border-hairline);
  border-radius: 6px;
  padding: 0.55rem 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.metric-pill-label {
  font-size: 0.85rem;
  color: var(--ink-secondary);
}

.metric-pill-val {
  font-family: var(--font-mono);
  font-size: 1rem;
  font-weight: 700;
  color: var(--ink-primary);
  font-variant-numeric: tabular-nums;
}

.mini-gauge-wrap {
  margin-top: 0.85rem;
}

.mini-gauge-track {
  height: 0.65rem;
  background: var(--bg-subtle);
  border: 1.5px solid var(--border-strong);
  border-radius: 999px;
  overflow: hidden;
}

.mini-gauge-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--amber-accent), var(--amber-gold));
  border-radius: 999px;
}

.showcase-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 0.75rem;
  margin-top: 1rem;
  padding-top: 0.85rem;
  border-top: 1px dashed var(--border-hairline);
}

/* Controls & Search */
.filter-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.25rem;
  padding: 0.9rem 1.15rem;
  background: var(--bg-surface);
  border: 1.5px solid var(--border-strong);
  border-radius: 10px;
}

.filter-bar label {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--ink-secondary);
}

.search-input {
  flex: 1 1 16rem;
  padding: 0.55rem 0.85rem;
  font-family: var(--font-sans);
  font-size: 1rem;
  color: var(--ink-primary);
  background: var(--bg-page);
  border: 1.5px solid var(--border-strong);
  border-radius: 6px;
}

.search-input::placeholder {
  color: var(--ink-muted);
}

/* Tables & Ledgers */
.table-wrap {
  overflow-x: auto;
  margin-top: 0.5rem;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 1rem;
  background: var(--bg-surface);
  border: 1.5px solid var(--border-strong);
}

th, td {
  padding: 0.8rem 1rem;
  text-align: left;
  border-bottom: 1px solid var(--border-hairline);
  vertical-align: top;
}

tbody tr:last-child td {
  border-bottom: none;
}

th {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--ink-primary);
  background: var(--bg-subtle);
  border-bottom: 1.5px solid var(--border-strong);
  white-space: nowrap;
}

tbody tr.batch-row {
  transition: background-color 140ms ease;
}

tbody tr.batch-row:hover {
  background-color: var(--bg-row-hover);
}

th.num, td.num {
  text-align: right;
  white-space: nowrap;
}

.batch-substyle,
.item-desc {
  font-size: 0.85rem;
  color: var(--ink-secondary);
  margin-top: 0.2rem;
}

.vol-ref {
  font-size: 0.85rem;
  color: var(--ink-muted);
}

.empty-msg {
  margin-top: 1rem;
  color: var(--ink-secondary);
}

/* Batch Detail Hero & Photo Pinboard */
.page-title {
  font-size: 1.15rem;
  line-height: 1.3;
  margin: 0 0 0.2rem;
}

.page-lede {
  font-size: 0.9rem;
  color: var(--ink-secondary);
  margin: 0 0 1.5rem;
}

.batch-hero {
  background: var(--bg-surface);
  border: 2px solid var(--border-strong);
  border-radius: 10px;
  padding: 1.5rem;
  box-shadow: var(--shadow-card);
}

.hero-body-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.35fr) minmax(0, 1fr);
  gap: 1.5rem;
  align-items: start;
}

.hero-body-grid.no-photos {
  grid-template-columns: 1fr;
}

.hero-meta-chips {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 10.5rem), 1fr));
  gap: 0.6rem;
  margin-top: 1rem;
}

.hero-meta-chip {
  background: var(--bg-page);
  border: 1px solid var(--border-hairline);
  border-radius: 6px;
  padding: 0.5rem 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}

.hero-meta-label {
  font-size: 0.85rem;
  color: var(--ink-secondary);
}

.hero-meta-value {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--ink-primary);
}

.hero-photo-deck {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 11rem), 1fr));
  gap: 1rem;
  align-items: start;
}

.hero-photo-card,
.step-photo-card {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  width: 100%;
  text-align: left;
  background: var(--bg-subtle);
  border: 1.5px solid var(--border-strong);
  border-radius: 6px;
  padding: 0.6rem;
  cursor: pointer;
  transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
    background-color 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.hero-photo-card:nth-child(odd) {
  transform: rotate(-1.2deg);
}

.hero-photo-card:nth-child(even) {
  transform: rotate(1.2deg);
}

.hero-photo-card:hover,
.step-photo-card:hover {
  transform: rotate(0deg) translateY(-2px);
  background: var(--amber-tint);
}

.hero-photo-card img,
.step-photo-card img {
  display: block;
  width: 100%;
  aspect-ratio: 4 / 3;
  object-fit: cover;
  border-radius: 4px;
  border: 1px solid var(--border-hairline);
}

.photo-caption {
  font-family: var(--font-sans);
  font-size: 0.85rem;
  line-height: 1.4;
  color: var(--ink-primary);
}

/* Spec Grid & Ledgers */
.spec-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 20rem), 1fr));
  gap: 2rem;
  margin-top: 2.25rem;
}

.spec-panel {
  padding-top: 0.25rem;
}

.spec-panel h2 {
  margin-top: 0;
  font-size: 1.4rem;
}

.dl-ledger {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 0.65rem 1rem;
  margin: 0;
  padding: 1.1rem 1.25rem;
  background: var(--bg-surface);
  border: 1.5px solid var(--border-strong);
  border-radius: 10px;
  font-size: 1rem;
}

.dl-ledger dt {
  color: var(--ink-secondary);
}

.dl-ledger dd {
  margin: 0;
  font-weight: 700;
  color: var(--ink-primary);
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.dl-note {
  grid-column: 1 / -1;
  font-size: 0.85rem;
  color: var(--ink-muted);
  margin-top: -0.35rem;
  padding-bottom: 0.4rem;
  border-bottom: 1px dashed var(--border-hairline);
}

/* Volume Sight Glass */
.volume-meter {
  margin: 0 0 1.25rem;
  padding: 1.1rem 1.25rem;
  background: var(--bg-surface);
  border: 1.5px solid var(--border-strong);
  border-radius: 10px;
}

.volume-bar-track {
  height: 0.85rem;
  background: var(--bg-subtle);
  border: 1.5px solid var(--border-strong);
  border-radius: 999px;
  overflow: hidden;
}

.volume-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--amber-accent), var(--amber-gold));
  border-radius: 999px;
}

.volume-caption {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 0.35rem 1rem;
  font-size: 0.85rem;
  color: var(--ink-secondary);
  margin-top: 0.55rem;
  font-variant-numeric: tabular-nums;
}

.volume-caption strong {
  color: var(--ink-primary);
}

.equipment-wrap {
  margin-top: 1.25rem;
}

.equipment-heading {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--ink-secondary);
  margin: 0 0 0.45rem;
}

/* Timeline */
.timeline {
  position: relative;
  list-style: none;
  padding: 0 0 0 1.35rem;
  margin: 0;
}

.timeline::before {
  content: "";
  position: absolute;
  left: 0;
  top: 0.75rem;
  bottom: 0.75rem;
  width: 2px;
  background-color: var(--blueprint-blue);
}

.timeline-step {
  padding: 1.35rem 1.5rem;
  margin-bottom: 1.25rem;
  background: var(--bg-surface);
  border: 2px solid var(--border-strong);
  border-radius: 10px;
  box-shadow: var(--shadow-card);
}

.timeline-step:last-child {
  margin-bottom: 0;
}

.step-header {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: baseline;
  gap: 0.5rem 1rem;
  margin-bottom: 0.65rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border-hairline);
}

.step-title-group {
  display: flex;
  align-items: baseline;
  gap: 0.65rem;
  flex-wrap: wrap;
}

.step-badge {
  display: inline-block;
  font-family: var(--font-mono);
  font-size: 0.85rem;
  font-weight: 600;
  line-height: 1.4;
  color: var(--amber-hover);
  background: var(--amber-tint);
  padding: 0.25rem 0.6rem;
  border-radius: 4px;
}

.step-time {
  font-size: 0.85rem;
  color: var(--ink-secondary);
  font-variant-numeric: tabular-nums;
}

.step-directions {
  margin: 0.5rem 0;
  padding-left: 1.2rem;
  color: var(--ink-primary);
}

.step-directions li {
  margin-bottom: 0.3rem;
  max-width: 68ch;
}

.step-subgrid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 15rem), 1fr));
  gap: 1rem;
  margin-top: 1rem;
  padding-top: 0.9rem;
  border-top: 1px solid var(--border-hairline);
}

.step-box {
  background: var(--bg-page);
  border: 1px solid var(--border-hairline);
  border-radius: 6px;
  padding: 0.75rem 0.95rem;
  font-size: 1rem;
}

.step-box-title {
  font-weight: 700;
  font-size: 0.85rem;
  color: var(--ink-secondary);
  margin: 0 0 0.45rem;
}

.step-photo-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 14rem), 18rem));
  gap: 1rem;
  margin-top: 1rem;
}

.kv-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.kv-list li {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.25rem 0;
  border-bottom: 1px dashed var(--border-hairline);
}

.kv-list li:last-child {
  border-bottom: none;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  list-style: none;
  padding: 0;
  margin: 1rem 0 0;
}

.tag-item {
  font-size: 0.85rem;
  color: var(--ink-primary);
  background: var(--bg-subtle);
  padding: 0.25rem 0.65rem;
  border-radius: 4px;
}

/* Lightbox Dialog */
.lightbox-dialog {
  max-width: min(92vw, 56rem);
  padding: 1.15rem;
  background: var(--bg-surface);
  color: var(--ink-primary);
  border: 2px solid var(--border-strong);
  border-radius: 10px;
  box-shadow: var(--shadow-card);
}

.lightbox-dialog::backdrop {
  background: rgba(18, 40, 74, 0.72);
}

.lightbox-img {
  display: block;
  max-width: 100%;
  max-height: 72vh;
  width: auto;
  height: auto;
  margin: 0 auto;
  border-radius: 6px;
  border: 1.5px solid var(--border-strong);
}

.lightbox-bar {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 0.75rem 1rem;
  margin-top: 0.9rem;
}

.lightbox-caption {
  margin: 0;
  font-size: 1rem;
  color: var(--ink-primary);
}

/* Kontaktformular */
.contact-form {
  display: grid;
  gap: 1rem;
  max-width: 36rem;
}

.contact-form label {
  display: grid;
  gap: 0.3rem;
  font-weight: 600;
  font-size: 0.9rem;
}

.contact-form input,
.contact-form textarea {
  font: inherit;
  font-weight: 400;
  color: var(--ink-primary);
  background: var(--bg-surface);
  border: 1.5px solid var(--border-strong);
  border-radius: 6px;
  padding: 0.55rem 0.85rem;
}

.contact-form textarea {
  min-height: 9rem;
  resize: vertical;
}

.contact-form button {
  justify-self: start;
  font: inherit;
  font-weight: 600;
  color: var(--ink-primary);
  background: var(--bg-subtle);
  border: 1.5px solid var(--border-strong);
  border-radius: 6px;
  padding: 0.55rem 1.1rem;
  cursor: pointer;
}

/* Footer */
.site-footer {
  margin-top: 4rem;
  padding-top: 1.5rem;
  border-top: 2px solid var(--border-strong);
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 1rem;
  font-size: 0.85rem;
  color: var(--ink-secondary);
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    transition-duration: 0.01ms !important;
  }
  .hero-photo-card:nth-child(odd),
  .hero-photo-card:nth-child(even) {
    transform: none;
  }
}

@media (max-width: 780px) {
  .hero-body-grid {
    grid-template-columns: 1fr;
  }
  .showcase-body {
    grid-template-columns: minmax(0, 11rem) minmax(0, 1fr);
    gap: 1rem;
  }
}

@media (max-width: 520px) {
  .showcase-body {
    grid-template-columns: 1fr;
  }
  .showcase-photo-link {
    max-width: 14rem;
  }
}

@media (max-width: 640px) {
  .container {
    padding: 1.35rem 1rem 3.5rem;
  }
  .batch-hero,
  .batch-showcase-card,
  .timeline-step {
    padding: 1.1rem;
  }
  .timeline {
    padding-left: 0.9rem;
  }
  th, td {
    padding: 0.65rem 0.75rem;
  }
}
`;
}

// Anzahl der großen Karten auf der Startseite (neueste zuerst); alle Sude stehen in der Tabelle.
const SHOWCASE_LIMIT = 3;

function renderIndexPage(batches) {
  const showcaseCardsHtml = batches
    .slice(0, SHOWCASE_LIMIT)
    .map((b) => {
      const og = formatQuantity(b['brew:vitalStatistics']?.originalGravity);
      const cg = formatQuantity(
        b['brew:vitalStatistics']?.finalGravity || b['brew:vitalStatistics']?.currentGravity
      );
      const abv = formatQuantity(b['brew:vitalStatistics']?.abv);
      const currVol = formatQuantity(b['brew:volumes']?.currentVolume);
      const initVol = formatQuantity(b['brew:volumes']?.initialVolume);
      const initVal = b['brew:volumes']?.initialVolume?.value || 0;
      const currVal = b['brew:volumes']?.currentVolume?.value || 0;
      const fillPercent = initVal > 0 ? Math.min(100, Math.max(0, Math.round((currVal / initVal) * 100))) : 0;
      const leadImg = (b.image || [])[0] || null;
      const brewerName = b.author?.name?.trim() || '';
      const batchUrl = `./batches/${encodeURIComponent(b.identifier)}.html`;

      return `<article class="batch-showcase-card">
        <div class="blueprint-banner">
          <div class="banner-group">
            <span class="status-pill">${escapeHtml(b['brew:statusLabel'] || b['brew:status'])}</span>
          </div>
          ${brewerName ? `<span class="banner-brewer">Gebraut von ${escapeHtml(brewerName)}</span>` : ''}
        </div>
        <div class="showcase-body${leadImg ? '' : ' no-photo'}">
          ${leadImg
          ? `<a class="showcase-photo-link" href="${batchUrl}">
                  <div class="showcase-photo-frame">
                    <img src="./${escapeHtml(leadImg.contentUrl)}" alt="${escapeHtml(leadImg.caption)}" loading="lazy">
                  </div>
                </a>`
          : ''
        }
          <div>
            <h2 class="showcase-title"><a href="${batchUrl}">${escapeHtml(b.name)}</a></h2>
            <p class="showcase-sub">${escapeHtml(b['brew:style']?.name || b.recipeCategory)} · Angestellt am ${formatDate(b.dateCreated, false)}</p>
            ${b.description ? `<p>${escapeHtml(b.description)}</p>` : ''}
            <div class="metric-grid">
              <div class="metric-pill">
                <span class="metric-pill-label">Stammwürze</span>
                <span class="metric-pill-val">${og}</span>
              </div>
              <div class="metric-pill">
                <span class="metric-pill-label">Aktueller Wert</span>
                <span class="metric-pill-val">${cg}</span>
              </div>
              <div class="metric-pill">
                <span class="metric-pill-label">Alkoholgehalt</span>
                <span class="metric-pill-val">${abv}</span>
              </div>
              <div class="metric-pill">
                <span class="metric-pill-label">Fassbestand</span>
                <span class="metric-pill-val">${currVol} / ${initVol}</span>
              </div>
            </div>
            <div class="mini-gauge-wrap">
              <div class="mini-gauge-track" role="progressbar" aria-label="Füllstand ${escapeHtml(b.identifier)}" aria-valuenow="${currVal}" aria-valuemin="0" aria-valuemax="${initVal}">
                <div class="mini-gauge-fill" style="width: ${fillPercent}%;"></div>
              </div>
            </div>
            <div class="showcase-actions">
              <span class="vol-ref num">Noch ${fillPercent}\u00A0% im Fass · Stand ${formatDate(b.dateModified || b.dateCreated, false)}</span>
              <a class="btn-secondary" href="${batchUrl}">Protokoll öffnen →</a>
            </div>
          </div>
        </div>
      </article>`;
    })
    .join('\n');

  const rowsHtml = batches
    .map((b) => {
      const og = formatQuantity(b['brew:vitalStatistics']?.originalGravity);
      const cg = formatQuantity(
        b['brew:vitalStatistics']?.finalGravity || b['brew:vitalStatistics']?.currentGravity
      );
      const abv = formatQuantity(b['brew:vitalStatistics']?.abv);
      const currVol = formatQuantity(b['brew:volumes']?.currentVolume);
      const initVol = formatQuantity(b['brew:volumes']?.initialVolume);
      const searchBlob = [
        b.identifier,
        b.name,
        b.recipeCategory,
        b['brew:style']?.name,
        b['brew:statusLabel'],
        b.author?.name,
        ...(b.keywords || [])
      ]
        .filter(Boolean)
        .join(' ')
        .toLowerCase();

      return `<tr class="batch-row" data-search="${escapeHtml(searchBlob)}">
        <td>
          <strong><a href="./batches/${encodeURIComponent(b.identifier)}.html">${escapeHtml(b.name)}</a></strong>
          <div class="batch-substyle">
            ${escapeHtml(b['brew:style']?.name || b.recipeCategory)}
          </div>
        </td>
        <td><span class="status-pill">${escapeHtml(b['brew:statusLabel'] || b['brew:status'])}</span></td>
        <td class="num">${formatDate(b.dateCreated, false)}</td>
        <td class="num">${og}</td>
        <td class="num">${cg}</td>
        <td class="num">${abv}</td>
        <td class="num">${currVol} <span class="vol-ref">/ ${initVol}</span></td>
      </tr>`;
    })
    .join('\n');

  return `<!DOCTYPE html>
<html lang="de">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Braupause — Kellerbuch</title>
  <meta name="description" content="Braupause — Kellerbuch für Brau-, Met- und Gärprotokolle.">
  ${renderIconLinks('./')}
  <link rel="stylesheet" href="./styles.css">
</head>
<body>
  <div class="container">
    <header class="site-header">
      ${renderBrandLockup('./', 'Nach Blaupause gebraut · Kellerbuch')}
    </header>

    <main>
      <h1 class="page-title">Kellerbuch</h1>
      <p class="page-lede">Alle Brau-, Met- und Gärsude im Überblick — vom ersten Ansatz im Fass bis zur letzten Verkostungsprobe.</p>

      <section class="batch-showcase-grid" aria-label="Aktuelle Sude im Keller">
        ${showcaseCardsHtml}
      </section>

      <h2>Alle Sude durchsuchen</h2>
      <div class="filter-bar">
        <label for="batch-filter">Sud suchen:</label>
        <input
          id="batch-filter"
          class="search-input"
          type="search"
          placeholder="Nach Schlagwort filtern…"
          autocomplete="off"
        >
      </div>

      <div class="table-wrap">
        <table aria-label="Dokumentierte Sude">
          <thead>
            <tr>
              <th>Rezept &amp; Stil</th>
              <th>Status</th>
              <th class="num">Brautag</th>
              <th class="num">Stammwürze</th>
              <th class="num">Aktuell / Rest</th>
              <th class="num">Alkohol</th>
              <th class="num">Bestand</th>
            </tr>
          </thead>
          <tbody id="batch-table-body">
            ${rowsHtml}
          </tbody>
        </table>
      </div>
      <p id="empty-filter-msg" class="empty-msg" hidden>
        Kein Sud entspricht deinem Suchbegriff.
      </p>
    </main>

    <footer class="site-footer">
      <span>Braupause · Nach Blaupause gebraut · <a href="./datenschutz.html">Datenschutz</a> · <a href="./kontakt.html">Kontakt</a></span>
      <span>Wenn ihr auch so coole Projekte umsetzen wollt, schaut gerne mal im <a href="https://www.fablab-muenchen.de/" target="_blank" rel="noopener noreferrer">FabLab München e.V.</a> vorbei.</span>
    </footer>
  </div>

  <script>
    const input = document.getElementById('batch-filter');
    const rows = Array.from(document.querySelectorAll('#batch-table-body tr.batch-row'));
    const emptyMsg = document.getElementById('empty-filter-msg');

    input?.addEventListener('input', () => {
      const q = input.value.trim().toLowerCase();
      let visible = 0;
      for (const row of rows) {
        const hay = row.getAttribute('data-search') || '';
        const match = !q || hay.includes(q);
        row.hidden = !match;
        if (match) visible++;
      }
      if (emptyMsg) emptyMsg.hidden = visible !== 0;
    });
  </script>
</body>
</html>`;
}

function renderBatchDetailPage(batch) {
  const vs = batch['brew:vitalStatistics'] || {};
  const vols = batch['brew:volumes'] || {};
  const initVolVal = vols.initialVolume?.value || 0;
  const currVolVal = vols.currentVolume?.value || 0;
  const fillPercent = initVolVal > 0 ? Math.min(100, Math.max(0, Math.round((currVolVal / initVolVal) * 100))) : 0;
  const brewerName = batch.author?.name?.trim() || '';
  const heroImages = batch.image || [];

  const latestRealExtract = findLatestProperty(batch, 'real_extract');
  const latestPressure = findLatestProperty(batch, 'spunding_pressure');

  const heroPhotosHtml = heroImages
    .map(
      (img) => `<button type="button" class="hero-photo-card" data-lightbox-src="../${escapeHtml(img.contentUrl)}" data-lightbox-caption="${escapeHtml(img.caption)}">
        <img src="../${escapeHtml(img.contentUrl)}" alt="${escapeHtml(img.caption)}" loading="lazy">
        <span class="photo-caption">${escapeHtml(img.caption)}</span>
      </button>`
    )
    .join('\n');

  const ingredientsHtml = (batch['brew:ingredients'] || [])
    .map((ing) => {
      const stage = STAGE_LABELS[ing['brew:stage']] || ing['brew:stage'];
      const cat = CATEGORY_LABELS[ing['brew:category']] || ing['brew:category'];
      const toneClass = CATEGORY_TONES[ing['brew:category']] || 'tone-blueprint';
      return `<tr>
        <td><strong>${escapeHtml(ing.name)}</strong>${ing.description ? `<div class="item-desc">${escapeHtml(ing.description)}</div>` : ''
        }</td>
        <td><span class="cat-badge ${toneClass}">${escapeHtml(cat)}</span></td>
        <td><span class="step-badge">${escapeHtml(stage)}</span></td>
        <td class="num"><strong>${formatQuantity(ing.requiredQuantity)}</strong></td>
      </tr>`;
    })
    .join('\n');

  const ledgerRowsHtml = (vols.ledger || [])
    .map(
      (entry) => `<tr>
        <td class="num">${formatDate(entry.timestamp, false)}</td>
        <td><strong>${escapeHtml(entry.label)}</strong></td>
        <td>${escapeHtml(entry.destination || '—')}</td>
        <td class="num">${formatQuantity(entry.delta)}</td>
      </tr>`
    )
    .join('\n');

  const toolsHtml = (batch.tool || [])
    .map(
      (t) => `<li class="tag-item"><strong>${escapeHtml(t.name)}</strong> (${escapeHtml(ROLE_LABELS[t['brew:role']] || t['brew:role'] || 'Equipment')})</li>`
    )
    .join('\n');

  const stepsHtml = (batch.recipeInstructions || [])
    .map((step) => {
      const phaseLabel = PHASE_LABELS[step['brew:phase']] || step['brew:phase'];
      const directions = (step.itemListElement || [])
        .map((d) => `<li>${escapeHtml(d.text)}</li>`)
        .join('\n');

      const measurements = step['brew:measurements'] || [];
      const settings = step['brew:settings'] || [];
      const additions = step['brew:additions'] || [];
      const stepImages = step.image || [];

      const boxes = [];

      if (measurements.length > 0) {
        const items = measurements
          .map(
            (m) =>
              `<li><span>${escapeHtml(m.name || m.propertyID)}</span><strong class="num">${formatQuantity(m)}</strong></li>`
          )
          .join('\n');
        boxes.push(`<div class="step-box">
          <p class="step-box-title">Messwerte</p>
          <ul class="kv-list">${items}</ul>
        </div>`);
      }

      if (settings.length > 0) {
        const items = settings
          .map(
            (s) =>
              `<li><span>${escapeHtml(s.name || s.propertyID)}</span><strong class="num">${formatQuantity(s)}</strong></li>`
          )
          .join('\n');
        boxes.push(`<div class="step-box">
          <p class="step-box-title">Prozess- &amp; Druckparameter</p>
          <ul class="kv-list">${items}</ul>
        </div>`);
      }

      if (additions.length > 0) {
        const items = additions
          .map(
            (a) =>
              `<li><span>${escapeHtml(a.name)}</span><strong class="num">${formatQuantity(a.quantity)}</strong></li>`
          )
          .join('\n');
        boxes.push(`<div class="step-box">
          <p class="step-box-title">Zugaben in diesem Schritt</p>
          <ul class="kv-list">${items}</ul>
        </div>`);
      }

      const stepPhotosHtml =
        stepImages.length > 0
          ? `<div class="step-photo-row">
              ${stepImages
            .map(
              (img) => `<button type="button" class="step-photo-card" data-lightbox-src="../${escapeHtml(img.contentUrl)}" data-lightbox-caption="${escapeHtml(img.caption)}">
                    <img src="../${escapeHtml(img.contentUrl)}" alt="${escapeHtml(img.caption)}" loading="lazy">
                    <span class="photo-caption">${escapeHtml(img.caption)}</span>
                  </button>`
            )
            .join('\n')}
            </div>`
          : '';

      return `<li class="timeline-step">
        <div class="step-header">
          <div class="step-title-group">
            <span class="step-badge">Schritt ${step.position} · ${escapeHtml(phaseLabel)}</span>
            <h3>${escapeHtml(step.name)}</h3>
          </div>
          <time class="step-time" datetime="${escapeHtml(step.datePublished)}">${formatDate(step.datePublished)}</time>
        </div>
        ${step.description ? `<p>${escapeHtml(step.description)}</p>` : ''}
        ${directions ? `<ul class="step-directions">${directions}</ul>` : ''}
        ${boxes.length > 0 ? `<div class="step-subgrid">${boxes.join('\n')}</div>` : ''}
        ${stepPhotosHtml}
      </li>`;
    })
    .join('\n');

  const keywordsHtml = (batch.keywords || [])
    .map((kw) => `<li class="tag-item">#${escapeHtml(kw)}</li>`)
    .join('\n');

  return `<!DOCTYPE html>
<html lang="de">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${escapeHtml(batch.identifier)} · ${escapeHtml(batch.name)} — Braupause</title>
  <meta name="description" content="${escapeHtml(batch.description || batch.name)}">
  ${renderIconLinks('../')}
  <link rel="stylesheet" href="../styles.css">
  <script type="application/ld+json">
${cleanJsonLd(batch)}
  </script>
</head>
<body>
  <div class="container">
    <header class="site-header">
      ${renderBrandLockup('../', brewerName ? `Nach Blaupause gebraut · ${escapeHtml(brewerName)}` : 'Nach Blaupause gebraut · Sud- &amp; Gärprotokoll')}
      <div class="header-meta">
        <a class="btn-secondary" href="../index.html">← Alle Sude</a>
      </div>
    </header>

    <main>
      <article>
        <div class="batch-hero">
          <div class="blueprint-banner">
            <div class="banner-group">
              <span class="code-id code-id-light">${escapeHtml(batch.identifier)}</span>
              <span class="status-pill">${escapeHtml(batch['brew:statusLabel'] || batch['brew:status'])}</span>
            </div>
            ${brewerName ? `<span class="banner-brewer">Gebraut von ${escapeHtml(brewerName)}</span>` : ''}
          </div>

          <div class="hero-body-grid${heroImages.length > 0 ? '' : ' no-photos'}">
            <div>
              <h1>${escapeHtml(batch.name)}</h1>
              ${batch.description ? `<p>${escapeHtml(batch.description)}</p>` : ''}

              <div class="hero-meta-chips">
                <div class="hero-meta-chip">
                  <span class="hero-meta-label">Kategorie</span>
                  <span class="hero-meta-value">${escapeHtml(batch.recipeCategory)}</span>
                </div>
                ${batch['brew:style']?.name
      ? `<div class="hero-meta-chip">
                        <span class="hero-meta-label">Stil</span>
                        <span class="hero-meta-value">${escapeHtml(batch['brew:style'].name)}</span>
                      </div>`
      : ''
    }
                <div class="hero-meta-chip">
                  <span class="hero-meta-label">Angestellt</span>
                  <span class="hero-meta-value num"><time datetime="${escapeHtml(batch.dateCreated)}">${formatDate(batch.dateCreated)}</time></span>
                </div>
                ${batch.dateModified
      ? `<div class="hero-meta-chip">
                        <span class="hero-meta-label">Letzter Stand</span>
                        <span class="hero-meta-value num"><time datetime="${escapeHtml(batch.dateModified)}">${formatDate(batch.dateModified)}</time></span>
                      </div>`
      : ''
    }
              </div>

              ${keywordsHtml ? `<ul class="tag-list" aria-label="Schlagwörter">${keywordsHtml}</ul>` : ''}
            </div>

            ${heroImages.length > 0
      ? `<div class="hero-photo-deck" aria-label="Fotos zum Sud">
                    ${heroPhotosHtml}
                  </div>`
      : ''
    }
          </div>
        </div>

        <div class="spec-grid">
          <section class="spec-panel" aria-labelledby="heading-vitals">
            <h2 id="heading-vitals">Kennzahlen &amp; Gärdaten</h2>
            <dl class="dl-ledger">
              <dt>Stammwürze (OG)</dt>
              <dd>${formatQuantity(vs.originalGravity)}</dd>
              ${vs.originalGravity?.description ? `<dd class="dl-note">${escapeHtml(vs.originalGravity.description)}</dd>` : ''}

              <dt>Aktueller Messwert</dt>
              <dd>${formatQuantity(vs.currentGravity || vs.finalGravity)}</dd>
              ${(vs.currentGravity?.description || vs.finalGravity?.description) ? `<dd class="dl-note">${escapeHtml((vs.currentGravity || vs.finalGravity).description)}</dd>` : ''}

              ${latestRealExtract
      ? `<dt>Tatsächlicher Restextrakt</dt>
                     <dd>${formatQuantity(latestRealExtract)}</dd>`
      : ''
    }

              <dt>Alkoholgehalt (ABV)</dt>
              <dd>${formatQuantity(vs.abv)}</dd>

              ${latestPressure
      ? `<dt>Spunddruck (aktuell)</dt>
                     <dd>${formatQuantity(latestPressure)}</dd>`
      : ''
    }
            </dl>
          </section>

          <section class="spec-panel" aria-labelledby="heading-volumes">
            <h2 id="heading-volumes">Volumen-Bilanz &amp; Fass-Schauglas</h2>
            <div class="volume-meter">
              <div class="volume-bar-track" role="progressbar" aria-valuenow="${currVolVal}" aria-valuemin="0" aria-valuemax="${initVolVal}">
                <div class="volume-bar-fill" style="width: ${fillPercent}%;"></div>
              </div>
              <div class="volume-caption">
                <span>Aktuell im Fass: <strong>${formatQuantity(vols.currentVolume)}</strong> (${fillPercent}\u00A0%)</span>
                <span>Startvolumen: ${formatQuantity(vols.initialVolume)}</span>
              </div>
            </div>

            ${ledgerRowsHtml
      ? `<div class="table-wrap">
                    <table aria-label="Volumen-Buchungsjournal">
                      <thead>
                        <tr>
                          <th class="num">Datum</th>
                          <th>Vorgang</th>
                          <th>Ziel</th>
                          <th class="num">Differenz</th>
                        </tr>
                      </thead>
                      <tbody>
                        ${ledgerRowsHtml}
                      </tbody>
                    </table>
                  </div>`
      : ''
    }
          </section>
        </div>

        <section aria-labelledby="heading-ingredients">
          <h2 id="heading-ingredients">Zutaten &amp; Gaben</h2>
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Zutat</th>
                  <th>Kategorie</th>
                  <th>Phase</th>
                  <th class="num">Menge</th>
                </tr>
              </thead>
              <tbody>
                ${ingredientsHtml}
              </tbody>
            </table>
          </div>
          ${toolsHtml
      ? `<div class="equipment-wrap">
                  <p class="equipment-heading">Verwendetes Equipment</p>
                  <ul class="tag-list">${toolsHtml}</ul>
                </div>`
      : ''
    }
        </section>

        <section aria-labelledby="heading-timeline">
          <h2 id="heading-timeline">Chronologisches Brau- &amp; Gärprotokoll</h2>
          <ol class="timeline">
            ${stepsHtml}
          </ol>
        </section>
      </article>
    </main>

    <footer class="site-footer">
      <span><a href="../index.html">← Zurück zum Kellerbuch</a> · <a href="../datenschutz.html">Datenschutz</a> · <a href="../kontakt.html">Kontakt</a></span>
      <span>Sud-ID: <code class="code-id">${escapeHtml(batch.identifier)}</code>${brewerName ? ` · Gebraut von ${escapeHtml(brewerName)}` : ''}</span>
    </footer>
  </div>

  <dialog id="photo-lightbox" class="lightbox-dialog" aria-labelledby="lightbox-caption">
    <div id="lightbox-stage"></div>
    <div class="lightbox-bar">
      <p id="lightbox-caption" class="lightbox-caption"></p>
      <button id="lightbox-close" type="button" class="btn-secondary">Schließen</button>
    </div>
  </dialog>

  <script>
    (function () {
      const dialog = document.getElementById('photo-lightbox');
      const stage = document.getElementById('lightbox-stage');
      const capEl = document.getElementById('lightbox-caption');
      const closeBtn = document.getElementById('lightbox-close');

      for (const trigger of document.querySelectorAll('[data-lightbox-src]')) {
        trigger.addEventListener('click', () => {
          const src = trigger.getAttribute('data-lightbox-src') || '';
          const cap = trigger.getAttribute('data-lightbox-caption') || '';
          if (stage) {
            stage.textContent = '';
            const img = document.createElement('img');
            img.className = 'lightbox-img';
            img.src = src;
            img.alt = cap;
            stage.appendChild(img);
          }
          if (capEl) capEl.textContent = cap;
          if (dialog && typeof dialog.showModal === 'function') {
            dialog.showModal();
          }
        });
      }

      closeBtn?.addEventListener('click', () => dialog?.close());
      dialog?.addEventListener('click', (e) => {
        if (e.target === dialog) dialog.close();
      });
    })();
  </script>
</body>
</html>`;
}

async function copyAssetDirectory(srcDir, destDir, allowedPattern) {
  let entries;
  try {
    entries = await fs.readdir(srcDir, { withFileTypes: true });
  } catch (err) {
    if (err.code === 'ENOENT') return;
    throw err;
  }
  await fs.mkdir(destDir, { recursive: true });
  for (const entry of entries) {
    const srcPath = path.join(srcDir, entry.name);
    const destPath = path.join(destDir, entry.name);
    if (entry.isDirectory()) {
      await copyAssetDirectory(srcPath, destPath, allowedPattern);
    } else if (entry.isFile() && allowedPattern.test(entry.name)) {
      await fs.copyFile(srcPath, destPath);
    }
  }
}

async function main() {
  await fs.rm(DIST_DIR, { recursive: true, force: true });
  await fs.mkdir(path.join(DIST_DIR, 'batches'), { recursive: true });
  await fs.mkdir(path.join(DIST_DIR, 'api', 'batches'), { recursive: true });
  await fs.mkdir(path.join(DIST_DIR, 'schemas'), { recursive: true });

  const entries = await fs.readdir(BATCHES_DIR, { withFileTypes: true });
  const jsonFiles = entries
    .filter((e) => e.isFile() && e.name.endsWith('.json'))
    .map((e) => e.name)
    .sort();

  const batches = [];

  for (const fileName of jsonFiles) {
    const filePath = path.join(BATCHES_DIR, fileName);
    const raw = await fs.readFile(filePath, 'utf8');
    const batch = JSON.parse(raw);
    batches.push(batch);

    // Write detail HTML
    const detailHtml = renderBatchDetailPage(batch);
    await fs.writeFile(path.join(DIST_DIR, 'batches', `${batch.identifier}.html`), detailHtml, 'utf8');

    // Write clean JSON-LD API file
    await fs.writeFile(
      path.join(DIST_DIR, 'api', 'batches', `${batch.identifier}.json`),
      cleanJsonLd(batch) + '\n',
      'utf8'
    );
  }

  // Sort newest brew date first for index page
  batches.sort((a, b) => Date.parse(b.dateCreated) - Date.parse(a.dateCreated));

  // Write index.html, styles.css, .nojekyll, schema, and api/batches.json
  await fs.writeFile(path.join(DIST_DIR, 'index.html'), renderIndexPage(batches), 'utf8');
  await fs.writeFile(path.join(DIST_DIR, 'styles.css'), buildStyles(), 'utf8');
  await fs.writeFile(path.join(DIST_DIR, '.nojekyll'), '', 'utf8');
  await fs.copyFile(SCHEMA_PATH, path.join(DIST_DIR, 'schemas', 'brew-batch.schema.json'));

  // Statische Seiten (z. B. Datenschutz, Kontakt) unverändert nach dist/ kopieren
  await copyAssetDirectory(PAGES_DIR, DIST_DIR, /\.html$/i);

  // Copy web-ready logo & batch media files (SVG/PNG/JPG/WebP)
  await copyAssetDirectory(LOGO_DIR, path.join(DIST_DIR, 'assets', 'logo'), /\.(svg|png)$/i);
  await copyAssetDirectory(BATCH_ASSETS_DIR, path.join(DIST_DIR, 'assets', 'batches'), /\.(svg|png|jpe?g|webp)$/i);

  await fs.writeFile(
    path.join(DIST_DIR, 'api', 'batches.json'),
    JSON.stringify(
      batches.map((b) => ({
        identifier: b.identifier,
        name: b.name,
        recipeCategory: b.recipeCategory,
        style: b['brew:style']?.name,
        status: b['brew:status'],
        statusLabel: b['brew:statusLabel'],
        dateCreated: b.dateCreated,
        dateModified: b.dateModified,
        url: `./batches/${b.identifier}.html`,
        jsonUrl: `./api/batches/${b.identifier}.json`
      })),
      null,
      2
    ) + '\n',
    'utf8'
  );

  console.log(`🏗️  Statische Seite erfolgreich nach dist/ gebaut (${batches.length} Sud-Dokumente).`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
