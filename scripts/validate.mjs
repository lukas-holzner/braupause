import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, '..');
const SCHEMA_PATH = path.join(ROOT_DIR, 'schemas', 'brew-batch.schema.json');
const BATCHES_DIR = path.join(ROOT_DIR, 'batches');

function toLiters(qv) {
  if (!qv || typeof qv.value !== 'number') return null;
  if (qv.unitText === 'L') return qv.value;
  if (qv.unitText === 'mL') return qv.value / 1000;
  return null;
}

async function main() {
  const schemaRaw = await fs.readFile(SCHEMA_PATH, 'utf8');
  const schema = JSON.parse(schemaRaw);

  const ajv = new Ajv2020({ allErrors: true, strict: false });
  addFormats(ajv);
  const validateSchema = ajv.compile(schema);

  const entries = await fs.readdir(BATCHES_DIR, { withFileTypes: true });
  const jsonFiles = entries
    .filter((e) => e.isFile() && e.name.endsWith('.json'))
    .map((e) => e.name)
    .sort();

  if (jsonFiles.length === 0) {
    console.error('❌ Keine Batch-Dateien in batches/ gefunden.');
    process.exit(1);
  }

  let hasErrors = false;
  const seenIds = new Set();

  for (const fileName of jsonFiles) {
    const filePath = path.join(BATCHES_DIR, fileName);
    const expectedId = path.basename(fileName, '.json');
    const fileErrors = [];

    let data;
    try {
      const raw = await fs.readFile(filePath, 'utf8');
      data = JSON.parse(raw);
    } catch (err) {
      console.error(`❌ [${fileName}] Ungültiges JSON: ${err.message}`);
      hasErrors = true;
      continue;
    }

    const valid = validateSchema(data);
    if (!valid && validateSchema.errors) {
      for (const err of validateSchema.errors) {
        fileErrors.push(
          `Schema-Fehler bei "${err.instancePath || '/'}": ${err.message} (${JSON.stringify(err.params)})`
        );
      }
    }

    // Domain Check 1: identifier matches filename & uniqueness
    if (data.identifier !== expectedId) {
      fileErrors.push(
        `Dateiname "${fileName}" stimmt nicht mit identifier "${data.identifier}" überein (erwartet: "${expectedId}").`
      );
    }
    if (seenIds.has(data.identifier)) {
      fileErrors.push(`Doppelter identifier "${data.identifier}".`);
    }
    seenIds.add(data.identifier);

    // Domain Check 2: @id matches urn:batch:<identifier>
    if (data['@id'] && data['@id'] !== `urn:batch:${data.identifier}`) {
      fileErrors.push(
        `"@id" sollte "urn:batch:${data.identifier}" lauten (aktuell: "${data['@id']}").`
      );
    }

    // Domain Check 3: Sequential step positions & chronological order
    if (Array.isArray(data.recipeInstructions)) {
      let prevTime = -Infinity;
      data.recipeInstructions.forEach((step, idx) => {
        const expectedPos = idx + 1;
        if (step.position !== expectedPos) {
          fileErrors.push(
            `recipeInstructions[${idx}]: position ist ${step.position}, erwartet wurde ${expectedPos}.`
          );
        }
        const ts = Date.parse(step.datePublished);
        if (!Number.isNaN(ts)) {
          if (ts < prevTime) {
            fileErrors.push(
              `recipeInstructions[${idx}] ("${step.name}"): datePublished (${step.datePublished}) liegt zeitlich vor dem vorherigen Schritt.`
            );
          }
          prevTime = ts;
        }
      });
    }

    // Domain Check 4: Volume ledger balance check
    if (data['brew:volumes']?.initialVolume && data['brew:volumes']?.currentVolume) {
      const initL = toLiters(data['brew:volumes'].initialVolume);
      const currL = toLiters(data['brew:volumes'].currentVolume);
      const ledger = data['brew:volumes'].ledger || [];

      if (initL !== null && currL !== null && ledger.length > 0) {
        let deltaSum = 0;
        for (const entry of ledger) {
          const dL = toLiters(entry.delta);
          if (dL !== null) deltaSum += dL;
        }
        const expectedCurrent = Number((initL + deltaSum).toFixed(3));
        const diff = Math.abs(expectedCurrent - currL);
        if (diff > 0.05) {
          fileErrors.push(
            `Volumenbilanz geht nicht auf: initialVolume (${initL} L) + Summe ledger (${deltaSum.toFixed(2)} L) = ${expectedCurrent} L, aber currentVolume ist ${currL} L (Differenz: ${diff.toFixed(2)} L).`
          );
        }
      }
    }

    // Domain Check 5: Fotos – Datei vorhanden & stepPosition verweist auf existierenden Schritt
    const stepCount = Array.isArray(data.recipeInstructions) ? data.recipeInstructions.length : 0;
    const imageRefs = [
      ...(data.image || []).map((img, i) => ({ img, where: `image[${i}]` })),
      ...(data.recipeInstructions || []).flatMap((step, s) =>
        (step.image || []).map((img, i) => ({ img, where: `recipeInstructions[${s}].image[${i}]` }))
      )
    ];
    for (const { img, where } of imageRefs) {
      const url = img?.contentUrl;
      if (typeof url === 'string' && url && !url.includes('://')) {
        try {
          const stat = await fs.stat(path.join(ROOT_DIR, url));
          if (!stat.isFile()) throw new Error('keine Datei');
        } catch {
          fileErrors.push(`${where}: Bilddatei "${url}" existiert nicht.`);
        }
      }
      if (Number.isInteger(img?.stepPosition) && img.stepPosition > stepCount) {
        fileErrors.push(`${where}: stepPosition ${img.stepPosition} verweist auf keinen existierenden Schritt.`);
      }
    }

    if (fileErrors.length > 0) {
      hasErrors = true;
      console.error(`❌ [${fileName}] ${fileErrors.length} Fehler gefunden:`);
      for (const e of fileErrors) {
        console.error(`   • ${e}`);
      }
    } else {
      console.log(
        `✅ [${fileName}] Gültig (${data.name} — Status: ${data['brew:statusLabel'] || data['brew:status']})`
      );
    }
  }

  if (hasErrors) {
    console.error('\n💥 Validierung fehlgeschlagen.');
    process.exit(1);
  } else {
    console.log(`\n🎉 Alle ${jsonFiles.length} Sud-Dateien erfolgreich validiert!`);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
