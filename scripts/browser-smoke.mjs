import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';

const url = process.argv[2];
if (!url) {
  console.error('usage: node scripts/browser-smoke.mjs URL');
  process.exit(2);
}

const browser = process.env.CHROME_BIN || 'google-chrome';
const result = spawnSync(
  browser,
  [
    '--headless=new',
    '--no-sandbox',
    '--disable-gpu',
    '--disable-dev-shm-usage',
    '--virtual-time-budget=3000',
    '--dump-dom',
    url,
  ],
  { encoding: 'utf8', timeout: 15000 },
);

if (result.error) throw result.error;
assert.equal(result.status, 0, result.stderr);
for (const required of ['<title>Lens</title>', 'id="tree"', 'id="content"']) {
  assert(result.stdout.includes(required), `browser DOM is missing: ${required}`);
}
console.log('browser smoke test passed');
