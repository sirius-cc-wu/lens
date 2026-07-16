import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const url = process.argv[2];
if (!url) {
  console.error('usage: node scripts/browser-interaction.mjs URL');
  process.exit(2);
}

const browser = process.env.CHROME_BIN || 'google-chrome';
const userDataDir = mkdtempSync(join(tmpdir(), 'lens-chrome-'));
const chrome = spawn(
  browser,
  [
    '--headless=new',
    '--no-sandbox',
    '--disable-gpu',
    '--disable-dev-shm-usage',
    '--remote-debugging-port=9223',
    `--user-data-dir=${userDataDir}`,
    url,
  ],
  { stdio: ['ignore', 'ignore', 'pipe'] },
);

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function waitForTarget() {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    try {
      const response = await fetch('http://127.0.0.1:9223/json/list');
      const targets = await response.json();
      const target = targets.find((item) => item.type === 'page' && item.webSocketDebuggerUrl);
      if (target) return target.webSocketDebuggerUrl;
    } catch {}
    await sleep(100);
  }
  throw new Error('Chrome remote debugging target did not start');
}

class CdpClient {
  constructor(socket) {
    this.socket = socket;
    this.nextId = 1;
    this.pending = new Map();
    socket.addEventListener('message', (event) => {
      const message = JSON.parse(String(event.data));
      const pending = this.pending.get(message.id);
      if (!pending) return;
      this.pending.delete(message.id);
      if (message.error) pending.reject(new Error(message.error.message));
      else pending.resolve(message.result);
    });
  }

  send(method, params = {}) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.socket.send(JSON.stringify({ id, method, params }));
    });
  }
}

async function main() {
  const webSocketUrl = await waitForTarget();
  const socket = new WebSocket(webSocketUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });
  const cdp = new CdpClient(socket);
  await cdp.send('Page.enable');
  await cdp.send('Runtime.enable');
  await sleep(1200);

  const evaluate = async (expression) => {
    const result = await cdp.send('Runtime.evaluate', {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (result.exceptionDetails) throw new Error('browser evaluation failed');
    return result.result.value;
  };

  assert.equal(await evaluate('document.title'), 'Lens');
  assert.equal(await evaluate('document.querySelectorAll(".tree-entry").length > 0'), true);
  await evaluate(`(async () => {
    const button = [...document.querySelectorAll('[data-path]')]
      .find((entry) => entry.dataset.path === 'README.md');
    if (!button) throw new Error('README fixture was not listed');
    button.click();
    await new Promise((resolve) => setTimeout(resolve, 600));
  })()`);
  assert.equal(await evaluate('document.querySelectorAll(".diagram-card").length'), 1);
  assert.equal(await evaluate('document.querySelector(".diagram-card .block-source").hidden'), true);
  await evaluate('document.querySelector(".diagram-card .secondary-button").click()');
  assert.equal(await evaluate('document.querySelector(".diagram-card .block-source").hidden'), false);
  assert.equal(await evaluate('document.querySelector(".diagram-card .block-source").textContent.includes("Alice -> Bob")'), true);
  console.log('browser interaction test passed');
  socket.close();
}

try {
  await main();
} finally {
  chrome.kill('SIGTERM');
  await new Promise((resolve) => {
    if (chrome.exitCode !== null) resolve();
    else chrome.once('close', resolve);
  });
  rmSync(userDataDir, { recursive: true, force: true });
}
