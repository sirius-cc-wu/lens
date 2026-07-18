import { spawn } from "node:child_process";
import { once } from "node:events";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { expect, test } from "@playwright/test";

const lensBinary = resolve("target/debug/lens");

test("controlled renderer and discovered link then display rendered documents", async ({ page }) => {
  // Arrange
  const repository = await createDocumentationRepository();
  const renderer = await startRenderer();
  const lens = await startLens(repository, renderer.url);

  try {
    // Act
    await page.goto(lens.url);
    await expect(page.getByRole("heading", { level: 1, name: "Browser fixture" })).toBeVisible();
    await expect.poll(() => renderer.requests).toBe(1);
    await expect
      .poll(() =>
        page.locator("img[data-diagram]").evaluate((image) => image.complete && image.naturalWidth > 0),
      )
      .toBe(true);
    await page.getByRole("link", { name: "Open guide" }).click();

    // Assert
    expect(new URL(page.url()).pathname).toBe("/documents/guides/guide.md");
    await expect(page.getByRole("heading", { level: 1, name: "Guide page" })).toBeVisible();
    await expect(page.locator("article")).toContainText("The guide is a discovered document.");
  } finally {
    await lens.stop();
    await renderer.stop();
    await rm(repository.directory, { force: true, recursive: true });
  }
});

async function createDocumentationRepository() {
  const directory = await mkdtemp(join(tmpdir(), "lens-browser-"));
  const binDirectory = join(directory, "bin");
  await mkdir(join(directory, "guides"), { recursive: true });
  await mkdir(binDirectory);
  await Promise.all([
    writeFile(
      join(directory, "README.md"),
      "# Browser fixture\n\nA **rendered** document.\n\n[Open guide](guides/guide.md)\n\n```plantuml\n@startuml\nAlice -> Bob: browser fixture\n@enduml\n```\n",
    ),
    writeFile(
      join(directory, "guides", "guide.md"),
      "# Guide page\n\nThe guide is a discovered document.\n",
    ),
    writeFile(join(binDirectory, "xdg-open"), "#!/bin/sh\nexit 0\n"),
  ]);
  await chmod(join(binDirectory, "xdg-open"), 0o755);
  return { binDirectory, directory };
}

async function startRenderer() {
  let requests = 0;
  const server = createServer((_request, response) => {
    requests += 1;
    response.writeHead(200, { "content-type": "image/svg+xml" });
    response.end('<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>');
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  if (address === null || typeof address === "string") {
    throw new Error("Controlled renderer did not expose a TCP address");
  }
  return {
    get requests() {
      return requests;
    },
    url: `http://127.0.0.1:${address.port}`,
    stop: () => new Promise((resolve, reject) => server.close((error) => error ? reject(error) : resolve())),
  };
}

async function startLens(repository, rendererUrl) {
  const child = spawn(lensBinary, [repository.directory], {
    env: {
      ...process.env,
      LENS_PLANTUML_SERVER: rendererUrl,
      PATH: `${repository.binDirectory}:${process.env.PATH}`,
    },
    stdio: ["ignore", "pipe", "pipe"],
  });
  const url = await waitForLoopbackUrl(child);
  return {
    url,
    async stop() {
      if (child.exitCode !== null) {
        return;
      }
      child.kill("SIGINT");
      await once(child, "exit");
    },
  };
}

function waitForLoopbackUrl(child) {
  return new Promise((resolveUrl, reject) => {
    let output = "";
    const timeout = setTimeout(() => reject(new Error(`Lens did not print a loopback URL: ${output}`)), 10_000);
    child.stdout.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      output += chunk;
      const match = output.match(/at (http:\/\/127\.0\.0\.1:\d+)/);
      if (match) {
        clearTimeout(timeout);
        resolveUrl(match[1]);
      }
    });
    child.once("error", (error) => {
      clearTimeout(timeout);
      reject(error);
    });
    child.once("exit", (code) => {
      clearTimeout(timeout);
      reject(new Error(`Lens exited before serving the fixture (status ${code}): ${output}`));
    });
  });
}
