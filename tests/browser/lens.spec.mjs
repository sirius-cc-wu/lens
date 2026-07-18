import { spawn } from "node:child_process";
import { once } from "node:events";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { expect, test } from "@playwright/test";

test("controlled_renderer_and_navigation_pane_link_then_displays_selected_document", async ({ page }) => {
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
    await fixture.stop();
  }
});

test("standalone plantuml link then displays its rendered diagram", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    await page.goto(fixture.lens.url);
    await expect.poll(() => fixture.renderer.requests).toBe(1);

    // Act
    await page.getByRole("link", { name: "architecture.puml" }).click();

    // Assert
    expect(new URL(page.url()).pathname).toBe("/documents/architecture.puml");
    await expect(page.locator("article")).toContainText("Standalone PlantUML file.");
    await expect.poll(() => fixture.renderer.requests).toBe(2);
    await expect
      .poll(() =>
        page.locator("img[data-diagram]").evaluate((image) => image.complete && image.naturalWidth > 0),
      )
      .toBe(true);
  } finally {
    await fixture.stop();
  }
});

test("save displayed document then refreshes browser view automatically", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    await page.goto(fixture.lens.url);
    await expect(page.getByRole("heading", { level: 1, name: "Browser fixture" })).toBeVisible();
    const revision = await page.request.get(`${fixture.lens.url}/revisions/README.md`);
    expect(revision.status()).toBe(200);
    expect(await revision.text()).toBe("0");

    // Act
    await writeFile(
      join(fixture.repository.directory, "README.md"),
      "# Refreshed browser fixture\n\nChanged saved content.\n",
    );

    // Assert
    await expect(page.getByRole("heading", { level: 1, name: "Refreshed browser fixture" })).toBeVisible();
    await expect(page.locator("article")).toContainText("Changed saved content.");
    expect(new URL(page.url()).pathname).toBe("/");
  } finally {
    await fixture.stop();
  }
});

test("valid frontmatter then renders nested metadata without markdown delimiters", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({
    readme: "---\ntitle: Browser metadata\ntags:\n  - browser\n  - docs\npublication:\n  audience: maintainers\n...\n# Browser fixture\n\nA rendered document.\n",
  });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    const metadata = page.locator(".document-metadata");
    await expect(metadata).toContainText("title");
    await expect(metadata).toContainText("Browser metadata");
    await expect(metadata).toContainText("browser");
    await expect(metadata).toContainText("audience");
    await expect(metadata).toContainText("maintainers");
    await expect(page.getByRole("heading", { level: 1, name: "Browser fixture" })).toBeVisible();
    await expect(page.locator("article")).not.toContainText("tags:");
  } finally {
    await fixture.stop();
  }
});

test("malformed frontmatter then explains correction and renders markdown body", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({
    readme: "---\ntitle: [missing bracket\n---\n# Browser fixture\n\nA rendered document.\n",
  });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.getByRole("alert")).toContainText("Could not parse YAML frontmatter.");
    await expect(page.getByRole("alert")).toContainText(
      "Fix the YAML between the opening and closing delimiters.",
    );
    await expect(page.getByRole("heading", { level: 1, name: "Browser fixture" })).toBeVisible();
  } finally {
    await fixture.stop();
  }
});

test("document_navigation_pane_then_lists_authorized_documents_and_marks_current", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ hiddenDocument: "Confidential source" });

  try {
    // Act
    await page.goto(fixture.lens.url);

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
    await fixture.stop();
  }
});

test("filter_document_navigation_then_limits_visible_authorized_documents", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    await page.goto(fixture.lens.url);
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

test("disabled renderer then preserves plantuml source without a diagram request", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ rendererMode: "disabled" });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.locator(".diagram-disabled")).toBeVisible();
    await expect(page.locator("img[data-diagram]")).toHaveCount(0);
    await expect(page.locator(".diagram-source")).toContainText("Alice -> Bob: browser fixture");
    await expect.poll(() => fixture.renderer.requests).toBe(0);
  } finally {
    await fixture.stop();
  }
});

test("renderer failure then retry button loads the diagram", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ rendererStatuses: [503, 200] });

  try {
    await page.goto(fixture.lens.url);
    await expect(page.getByText("PlantUML rendering failed. The source is shown below.")).toBeVisible();

    // Act
    await page.getByRole("button", { name: "Retry diagram rendering" }).click();

    // Assert
    await expect.poll(() => fixture.renderer.requests).toBe(2);
    await expect
      .poll(() =>
        page.locator("img[data-diagram]").evaluate((image) => image.complete && image.naturalWidth > 0),
      )
      .toBe(true);
    await expect(page.getByText("PlantUML rendering failed. The source is shown below.")).toBeHidden();
  } finally {
    await fixture.stop();
  }
});

test("disable renderer control then blocks further rendering for the session", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    await page.goto(fixture.lens.url);
    await expect.poll(() => fixture.renderer.requests).toBe(1);
    await expect(page.getByText("Diagram renderer: public.")).toBeVisible();

    // Act
    await page.getByRole("button", { name: "Disable diagram rendering for this session" }).click();

    // Assert
    await expect(page.getByText("Diagram rendering is disabled for this viewing session.")).toBeVisible();
    await expect(page.locator(".diagram-disabled")).toBeVisible();
    await expect(page.getByRole("button", { name: "Disable diagram rendering for this session" })).toHaveCount(0);
    const diagramResponse = await page.request.get(`${fixture.lens.url}/diagrams/0/0`);
    expect(diagramResponse.status()).toBe(503);
    expect(fixture.renderer.requests).toBe(1);
  } finally {
    await fixture.stop();
  }
});

async function startBrowserFixture({
  hiddenDocument,
  readme,
  rendererMode,
  rendererStatus,
  rendererStatuses,
} = {}) {
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
    repository = await createDocumentationRepository({ hiddenDocument, readme });
    renderer = await startRenderer({ status: rendererStatus, statuses: rendererStatuses });
    lens = await startLens(repository, renderer.url, rendererMode);
    return { lens, renderer, repository, stop };
  } catch (error) {
    try {
      await stop();
    } catch (cleanupError) {
      throw new AggregateError([error, cleanupError], "Browser test fixture setup and cleanup failed");
    }
    throw error;
  }
}

async function createDocumentationRepository({ hiddenDocument, readme } = {}) {
  const directory = await mkdtemp(join(tmpdir(), "lens-browser-"));
  const binDirectory = join(directory, "bin");
  let files = [];
  try {
    await mkdir(join(directory, "guides"), { recursive: true });
    await mkdir(binDirectory);
    files = [
      writeFile(
        join(directory, "README.md"),
        readme ??
          "# Browser fixture\n\nA **rendered** document.\n\n[Open guide](guides/guide.md)\n\n```plantuml\n@startuml\nAlice -> Bob: browser fixture\n@enduml\n```\n",
      ),
      writeFile(
        join(directory, "guides", "guide.md"),
        "# Guide page\n\nThe guide is a discovered document.\n",
      ),
      writeFile(
        join(directory, "architecture.puml"),
        "@startuml\nAlice -> Bob: standalone fixture\n@enduml\n",
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

async function startRenderer({ status = 200, statuses } = {}) {
  let requests = 0;
  const server = createServer((_request, response) => {
    const responseStatus = (statuses ?? [status])[Math.min(requests, (statuses ?? [status]).length - 1)];
    requests += 1;
    if (responseStatus === 200) {
      response.writeHead(200, { "content-type": "image/svg+xml" });
      response.end('<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>');
      return;
    }
    response.writeHead(responseStatus, { "content-type": "text/plain; charset=utf-8" });
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

async function startLens(repository, rendererUrl, rendererMode) {
  const lensBinary = process.env.LENS_BROWSER_TEST_BINARY;
  if (!lensBinary) {
    throw new Error("Playwright global setup did not provide the Lens executable path");
  }
  const commandArguments = [repository.directory];
  if (rendererMode) {
    commandArguments.push("--renderer", rendererMode);
  }
  const child = spawn(lensBinary, commandArguments, {
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
