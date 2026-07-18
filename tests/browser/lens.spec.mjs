import { spawn } from "node:child_process";
import { once } from "node:events";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { expect, test } from "@playwright/test";

const lensBinary = resolve("target/debug/lens");

test("controlled_renderer_and_navigation_pane_link_then_displays_selected_document", async ({ page }) => {
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
    await page.getByRole("link", { name: "guides/guide.md" }).click();

    // Assert
    expect(new URL(page.url()).pathname).toBe("/documents/guides/guide.md");
    await expect(page.getByRole("heading", { level: 1, name: "Guide page" })).toBeVisible();
    await expect(page.locator("article")).toContainText("The guide is a discovered document.");
    await expect(
      page.getByRole("navigation", { name: "Discovered documents" }).getByRole("link", {
        name: "guides/guide.md",
      }),
    ).toHaveAttribute("aria-current", "page");
  } finally {
    await lens.stop();
    await renderer.stop();
    await rm(repository.directory, { force: true, recursive: true });
  }
});

test("document_navigation_pane_then_lists_authorized_documents_and_marks_current", async ({ page }) => {
  // Arrange
  const repository = await createDocumentationRepository({ hiddenDocument: "Confidential source" });
  const renderer = await startRenderer();
  const lens = await startLens(repository, renderer.url);

  try {
    // Act
    await page.goto(lens.url);

    // Assert
    const navigation = page.getByRole("navigation", { name: "Discovered documents" });
    await expect(navigation.getByRole("link", { name: "README.md" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    await expect(navigation.getByRole("link", { name: "guides/guide.md" })).toBeVisible();
    await expect(navigation).not.toContainText(".private.md");
    await expect(navigation).not.toContainText("Confidential source");
  } finally {
    await lens.stop();
    await renderer.stop();
    await rm(repository.directory, { force: true, recursive: true });
  }
});

test("filter_document_navigation_then_limits_visible_authorized_documents", async ({ page }) => {
  // Arrange
  const repository = await createDocumentationRepository();
  const renderer = await startRenderer();
  const lens = await startLens(repository, renderer.url);

  try {
    await page.goto(lens.url);
    const navigation = page.getByRole("navigation", { name: "Discovered documents" });
    const filter = page.getByRole("searchbox", { name: "Filter discovered documents" });

    // Act
    await filter.fill("guide");

    // Assert
    await expect(navigation.getByRole("link", { name: "README.md" })).toBeHidden();
    await expect(navigation.getByRole("link", { name: "guides/guide.md" })).toBeVisible();
    expect(new URL(page.url()).pathname).toBe("/");

    // Act
    await filter.fill("no-match");

    // Assert
    await expect(navigation.getByText("No discovered documents match the filter.")).toBeVisible();
  } finally {
    await lens.stop();
    await renderer.stop();
    await rm(repository.directory, { force: true, recursive: true });
  }
});

test("undiscovered document path then displays guidance without its source", async ({ page }) => {
  // Arrange
  const repository = await createDocumentationRepository({ hiddenDocument: "Confidential source" });
  const renderer = await startRenderer();
  const lens = await startLens(repository, renderer.url);

  try {
    // Act
    await page.goto(`${lens.url}/documents/.private.md`);

    // Assert
    await expect(
      page.getByRole("heading", { level: 1, name: "Document navigation unavailable" }),
    ).toBeVisible();
    await expect(page.locator("article")).toContainText(
      "requested document is not part of this viewing session",
    );
    await expect(page.locator("article")).not.toContainText("Confidential source");
    await expect(page.getByRole("link", { name: "Return to the initial document" })).toBeVisible();
  } finally {
    await lens.stop();
    await renderer.stop();
    await rm(repository.directory, { force: true, recursive: true });
  }
});

test("renderer failure then reveals the source while keeping the document readable", async ({ page }) => {
  // Arrange
  const repository = await createDocumentationRepository();
  const renderer = await startRenderer({ status: 503 });
  const lens = await startLens(repository, renderer.url);

  try {
    // Act
    await page.goto(lens.url);
    await expect.poll(() => renderer.requests).toBe(1);

    // Assert
    await expect(page.getByText("PlantUML rendering failed. The source is shown below.")).toBeVisible();
    await expect(page.locator(".diagram-source")).toHaveJSProperty("open", true);
    await expect(page.locator("article")).toContainText("A rendered document.");
    await expect(page.locator(".diagram-source")).toContainText("Alice -> Bob: browser fixture");
  } finally {
    await lens.stop();
    await renderer.stop();
    await rm(repository.directory, { force: true, recursive: true });
  }
});

async function createDocumentationRepository({ hiddenDocument } = {}) {
  const directory = await mkdtemp(join(tmpdir(), "lens-browser-"));
  const binDirectory = join(directory, "bin");
  await mkdir(join(directory, "guides"), { recursive: true });
  await mkdir(binDirectory);
  const files = [
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
