import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const asset = readFileSync(new URL('../assets/index.html', import.meta.url), 'utf8');

for (const required of [
  '<title>Lens</title>',
  'loadFile(path, startLine = null)',
  'PlantUML block',
  'Go to line',
]) {
  assert(asset.includes(required), `browser asset is missing: ${required}`);
}

assert(!asset.includes('innerHTML'), 'browser asset must not use innerHTML');
assert(!/javascript\s*:/i.test(asset), 'browser asset must not allow javascript URLs');
console.log('browser asset checks passed');
