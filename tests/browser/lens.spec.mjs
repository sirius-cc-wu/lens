import { spawn } from "node:child_process";
import { once } from "node:events";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { expect, test } from "@playwright/test";

test("controlled renderer and discovered link then display rendered documents", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    // Act
    await page.goto(fixture.lens.url);
    await expect(page.getByRole("heading", { level: 1, name: "Browser fixture" })).toBeVisible();
    await expect.poll(() => fixture.renderer.requests).toBe(1);
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
    await fixture.stop();
  }
});

test("undiscovered document path then returns 404 guidance without its source", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ hiddenDocument: "Confidential source" });

  try {
    // Act
    const response = await page.goto(`${fixture.lens.url}/documents/.private.md`);

    // Assert
    expect(response?.status()).toBe(404);
    await expect(
      page.getByRole("heading", { level: 1, name: "Document navigation unavailable" }),
    ).toBeVisible();
    await expect(page.locator("article")).toContainText(
      "requested document is not part of this viewing session",
    );
    await expect(page.locator("article")).not.toContainText("Confidential source");
    await expect(page.getByRole("link", { name: "Return to the initial document" })).toBeVisible();
  } finally {
    await fixture.stop();
  }
});

test("renderer fails before client script loads then reveals the source", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ rendererStatus: 503 });

  try {
    await page.route("**/app.js", async (route) => {
      await expect
        .poll(() =>
          page
            .locator("img[data-diagram]")
            .evaluate((image) => image.complete && image.naturalWidth === 0),
        )
        .toBe(true);
      await route.continue();
    });

    // Act
    await page.goto(fixture.lens.url);
    await expect.poll(() => fixture.renderer.requests).toBe(1);

    // Assert
    await expect(page.getByText("PlantUML rendering failed. The source is shown below.")).toBeVisible();
    await expect(page.locator(".diagram-source")).toHaveJSProperty("open", true);
    await expect(page.locator("article")).toContainText("A rendered document.");
    await expect(page.locator(".diagram-source")).toContainText("Alice -> Bob: browser fixture");
  } finally {
    await fixture.stop();
  }
});

async function startBrowserFixture({ hiddenDocument, rendererStatus } = {}) {
  let repository;
  let renderer;
  let lens;
  const stop = async () => {
    const errors = [];
    for (const cleanup of [
      lens && (() => lens.stop()),
      renderer && (() => renderer.stop()),
      repository && (() => rm(repository.directory, { force: true, recursive: true })),
    ]) {
      if (!cleanup) {
        continue;
      }
      try {
        await cleanup();
      } catch (error) {
        errors.push(error);
      }
    }
    if (errors.length > 0) {
      throw new AggregateError(errors, "Could not stop the browser test fixture");
    }
  };

  try {
    repository = await createDocumentationRepository({ hiddenDocument });
    renderer = await startRenderer({ status: rendererStatus });
    lens = await startLens(repository, renderer.url);
    return { lens, renderer, stop };
  } catch (error) {
    try {
      await stop();
    } catch (cleanupError) {
      throw new AggregateError([error, cleanupError], "Browser test fixture setup and cleanup failed");
    }
    throw error;
  }
}

async function createDocumentationRepository({ hiddenDocument } = {}) {
  const directory = await mkdtemp(join(tmpdir(), "lens-browser-"));
  const binDirectory = join(directory, "bin");
  let files = [];
  try {
    await mkdir(join(directory, "guides"), { recursive: true });
    await mkdir(binDirectory);
    files = [
      writeFile(
        join(directory, "README.md"),
        "# Browser fixture\n\nA **rendered** document.\n\n[Open guide](guides/guide.md)\n\n```plantuml\n@startuml\nAlice -> Bob: browser fixture\n@enduml\n```\n",
      ),
      writeFile(
        join(directory, "guides", "guide.md"),
        "# Guide page\n\nThe guide is a discovered document.\n",
      ),
      writeFile(join(binDirectory, "xdg-open"), "#!/bin/sh\nexit 0\n"),
    ];
    if (hiddenDocument) {
      files.push(writeFile(join(directory, ".private.md"), hiddenDocument));
    }
    await Promise.all(files);
    await chmod(join(binDirectory, "xdg-open"), 0o755);
    return { binDirectory, directory };
  } catch (error) {
    await Promise.allSettled(files);
    try {
      await rm(directory, { force: true, recursive: true });
    } catch (cleanupError) {
      throw new AggregateError([error, cleanupError], "Repository setup and cleanup failed");
    }
    throw error;
  }
}

async function startRenderer({ status = 200 } = {}) {
  let requests = 0;
  const server = createServer((_request, response) => {
    requests += 1;
    if (status === 200) {
      response.writeHead(200, { "content-type": "image/svg+xml" });
      response.end('<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>');
      return;
    }
    response.writeHead(status, { "content-type": "text/plain; charset=utf-8" });
    response.end("Controlled renderer failure");
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
  const lensBinary = process.env.LENS_BROWSER_TEST_BINARY;
  if (!lensBinary) {
    throw new Error("Playwright global setup did not provide the Lens executable path");
  }
  const child = spawn(lensBinary, [repository.directory], {
    env: {
      ...process.env,
      LENS_PLANTUML_SERVER: rendererUrl,
      PATH: `${repository.binDirectory}:${process.env.PATH}`,
    },
    stdio: ["ignore", "pipe", "pipe"],
  });
  const stop = async () => {
    if (child.exitCode !== null || child.signalCode !== null || child.pid === undefined) {
      return;
    }
    const closed = once(child, "close");
    child.kill("SIGKILL");
    await closed;
  };
  try {
    const url = await waitForLoopbackUrl(child);
    return { url, stop };
  } catch (error) {
    await stop();
    throw error;
  }
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
