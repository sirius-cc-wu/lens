import { spawn } from "node:child_process";
import { once } from "node:events";
import { chmod, mkdir, mkdtemp, realpath, rm, symlink, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";

import { expect, test } from "@playwright/test";

test("known_markdown_link_then_displays_linked_document", async ({ page }) => {
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

test("known_markdown_link_then_browser_history_returns_to_initial_document", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    await page.goto(fixture.lens.url);
    await page.getByRole("link", { name: "Open guide" }).click();
    await expect(page.getByRole("heading", { level: 1, name: "Guide page" })).toBeVisible();

    // Act
    await page.goBack();

    // Assert
    expect(new URL(page.url()).pathname).toBe("/");
    await expect(page.getByRole("heading", { level: 1, name: "Browser fixture" })).toBeVisible();
  } finally {
    await fixture.stop();
  }
});

test("direct_plantuml_target_then_displays_diagram_without_navigation_pane", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ targetRelativePath: "architecture.puml" });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.locator("article")).toContainText("Standalone PlantUML file.");
    await expect(page.getByRole("navigation", { name: "Discovered documents" })).toHaveCount(0);
    await expect.poll(() => fixture.renderer.requests).toBe(1);
    await expect
      .poll(() =>
        page.locator("img[data-diagram]").evaluate((image) => image.complete && image.naturalWidth > 0),
      )
      .toBe(true);
  } finally {
    await fixture.stop();
  }
});

test("known_plantuml_route_then_displays_authorized_diagram", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    // Act
    const response = await page.goto(`${fixture.lens.url}/documents/architecture.puml`);

    // Assert
    expect(response?.status()).toBe(200);
    await expect(page.locator("article")).toContainText("Standalone PlantUML file.");
    await expect(page.getByRole("navigation", { name: "Discovered documents" })).toHaveCount(0);
    await expect.poll(() => fixture.renderer.requests).toBe(1);
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

test("document_view_then_shows_compact_repository_relative_path", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    await page.goto(fixture.lens.url);

    // Act
    await page.getByRole("link", { name: "Open guide" }).click();

    // Assert
    const documentPath = page.locator(".document-header").getByRole("heading", { level: 1 });
    await expect(documentPath).toHaveText("guides/guide.md");
    await expect(page).toHaveTitle("Lens: guides/guide.md");
    expect((await documentPath.boundingBox()).height).toBeLessThan(32);
  } finally {
    await fixture.stop();
  }
});

test("valid_frontmatter_then_renders_compact_semantic_metadata_table_without_delimiters", async ({
  page,
}) => {
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
    const table = page.getByRole("table", { name: "Document metadata" });
    await expect(table).toBeVisible();
    await expect(table.locator("tbody > tr").first().locator("th, td")).toHaveCount(4);
    const tagItems = metadata.locator("li");
    expect(await tagItems.first().evaluate((item) => getComputedStyle(item).listStyleType)).toBe(
      "none",
    );
    expect(
      await tagItems.evaluateAll((items) =>
        items.map((item) => getComputedStyle(item, "::after").content),
      ),
    ).toEqual(['","', "none"]);
    const tagSpacing = await tagItems.evaluateAll(([first, second]) =>
      Math.round(second.getBoundingClientRect().left - first.getBoundingClientRect().right),
    );
    expect(tagSpacing).toBeLessThan(8);
    await expect(page.getByRole("heading", { level: 1, name: "Browser fixture" })).toBeVisible();
    await expect(page.locator("article")).not.toContainText("tags:");
  } finally {
    await fixture.stop();
  }
});

test("wide_markdown_table_then_remains_readable_with_local_horizontal_scrolling", async ({
  page,
}) => {
  // Arrange
  const fixture = await startBrowserFixture({
    readme: [
      "# Risk list",
      "",
      "| ID | Risk | Type | Likelihood | Impact | Mitigation |",
      "|---|---|---|---|---|---|",
      "| `R-01` | Renderer availability changes unexpectedly. | Technical | Medium | High | Retain a local rendering path and visible failure controls. |",
      "| `R-02` | Unsafe content reaches the browser. | Security | Low | High | Escape document content and keep a restrictive content security policy. |",
    ].join("\n"),
  });
  await page.setViewportSize({ width: 390, height: 844 });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    const tableRegion = page.locator(".markdown-table");
    const table = tableRegion.getByRole("table");
    await expect(table).toBeVisible();
    await expect(tableRegion).toHaveAttribute("tabindex", "0");

    const presentation = await tableRegion.evaluate((region) => {
      const renderedTable = region.querySelector("table");
      const [firstRow, secondRow] = renderedTable.tBodies[0].rows;
      return {
        tableScrollsLocally: region.scrollWidth > region.clientWidth,
        pageFitsViewport: document.documentElement.scrollWidth === window.innerWidth,
        headerIsDistinct:
          getComputedStyle(renderedTable.tHead.rows[0].cells[0]).backgroundColor !==
          getComputedStyle(firstRow).backgroundColor,
        rowsAreAlternating:
          getComputedStyle(firstRow).backgroundColor !== getComputedStyle(secondRow).backgroundColor,
        firstColumnStaysOnOneLine: getComputedStyle(firstRow.cells[0]).whiteSpace === "nowrap",
        cellsAlignAtTop: getComputedStyle(firstRow.cells[firstRow.cells.length - 1]).verticalAlign === "top",
      };
    });
    expect(presentation).toEqual({
      tableScrollsLocally: true,
      pageFitsViewport: true,
      headerIsDistinct: true,
      rowsAreAlternating: true,
      firstColumnStaysOnOneLine: true,
      cellsAlignAtTop: true,
    });
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

test("document_page_at_narrow_and_wide_viewports_then_uses_single_reading_column", async ({
  page,
}) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    await page.goto(fixture.lens.url);

    for (const viewport of [
      { width: 390, height: 844 },
      { width: 1440, height: 900 },
    ]) {
      // Act
      await page.setViewportSize(viewport);

      // Assert
      await expect(page.getByRole("navigation", { name: "Discovered documents" })).toHaveCount(0);
      await expect(page.getByRole("searchbox", { name: "Search discovered documents" })).toHaveCount(0);
      await expect(page.getByRole("button", { name: /^(Hide|Show) documents$/ })).toHaveCount(0);
      const layout = await page.locator("main").evaluate((main) => {
        const content = main.querySelector(".document-content");
        return {
          mainWidth: main.getBoundingClientRect().width,
          contentWidth: content.getBoundingClientRect().width,
          pageFitsViewport: document.documentElement.scrollWidth === window.innerWidth,
          collapsedAttribute: main.hasAttribute("data-document-navigation-collapsed"),
          navigationStorage: sessionStorage.getItem("lens.documentNavigationCollapsed"),
        };
      });
      expect(layout.contentWidth).toBeCloseTo(layout.mainWidth, 0);
      expect(layout.pageFitsViewport).toBe(true);
      expect(layout.collapsedAttribute).toBe(false);
      expect(layout.navigationStorage).toBeNull();
    }
  } finally {
    await fixture.stop();
  }
});

test("document_page_with_catalog_query_then_ignores_query_and_page", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    const knownDocumentUrl = `${fixture.lens.url}/documents/guides/guide.md`;
    const ordinaryResponse = await page.request.get(knownDocumentUrl);

    // Act
    const responseWithCatalogQuery = await page.request.get(
      `${knownDocumentUrl}?query=README&page=99`,
    );

    // Assert
    expect(responseWithCatalogQuery.status()).toBe(200);
    expect(await responseWithCatalogQuery.text()).toBe(await ordinaryResponse.text());
  } finally {
    await fixture.stop();
  }
});

test("undiscovered_document_path_then_returns_404_guidance_without_its_source", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ hiddenDocument: "Confidential source" });

  try {
    // Act
    const response = await page.goto(`${fixture.lens.url}/documents/.private.md`);

    // Assert
    expect(response?.status()).toBe(404);
    await expect(
      page.getByRole("heading", { level: 1, name: "Document unavailable" }),
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

test("direct_file_link_outside_parent_then_displays_repository_document", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ targetRelativePath: "guides/guide.md" });

  try {
    await page.goto(fixture.lens.url);
    await expect(page.getByRole("heading", { level: 1, name: "Guide page" })).toBeVisible();

    // Act
    await page.getByRole("link", { name: "Iteration evidence" }).click();

    // Assert
    expect(new URL(page.url()).pathname).toBe("/documents/iterations/evidence.md");
    await expect(page.getByRole("heading", { level: 1, name: "Iteration evidence" })).toBeVisible();
    await expect(page.locator("article")).toContainText("Repository-scoped document.");
  } finally {
    await fixture.stop();
  }
});

test("directory_target_link_outside_directory_then_displays_repository_document", async ({
  page,
}) => {
  // Arrange
  const fixture = await startBrowserFixture({ targetRelativePath: "guides" });

  try {
    await page.goto(fixture.lens.url);
    await expect(page.getByRole("heading", { level: 1, name: "Guide page" })).toBeVisible();

    // Act
    await page.getByRole("link", { name: "Iteration evidence" }).click();

    // Assert
    expect(new URL(page.url()).pathname).toBe("/documents/iterations/evidence.md");
    await expect(page.getByRole("heading", { level: 1, name: "Iteration evidence" })).toBeVisible();
  } finally {
    await fixture.stop();
  }
});

test("current_directory_link_outside_directory_then_displays_repository_document", async ({
  page,
}) => {
  // Arrange
  const fixture = await startBrowserFixture({ currentDirectoryRelativePath: "guides" });

  try {
    await page.goto(fixture.lens.url);
    await expect(page.getByRole("heading", { level: 1, name: "Guide page" })).toBeVisible();

    // Act
    await page.getByRole("link", { name: "Iteration evidence" }).click();

    // Assert
    expect(new URL(page.url()).pathname).toBe("/documents/iterations/evidence.md");
    await expect(page.getByRole("heading", { level: 1, name: "Iteration evidence" })).toBeVisible();
  } finally {
    await fixture.stop();
  }
});

test("target_scoped_directory_link_outside_directory_then_returns_guidance_without_source", async ({
  page,
}) => {
  // Arrange
  const fixture = await startBrowserFixture({
    targetRelativePath: "guides",
    scope: "target",
  });

  try {
    await page.goto(fixture.lens.url);

    // Act
    const [response] = await Promise.all([
      page.waitForResponse((candidate) => candidate.request().isNavigationRequest()),
      page.getByRole("link", { name: "Iteration evidence" }).click(),
    ]);

    // Assert
    expect(response.status()).toBe(404);
    await expect(
      page.getByRole("heading", { level: 1, name: "Document unavailable" }),
    ).toBeVisible();
    await expect(page.locator("article")).not.toContainText("Repository-scoped document.");
  } finally {
    await fixture.stop();
  }
});

test("direct_file_link_outside_repository_then_returns_guidance_without_source", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ targetRelativePath: "guides/guide.md" });

  try {
    await page.goto(fixture.lens.url);

    // Act
    const [response] = await Promise.all([
      page.waitForResponse((candidate) => candidate.request().isNavigationRequest()),
      page.getByRole("link", { name: "Outside repository" }).click(),
    ]);

    // Assert
    expect(response.status()).toBe(404);
    await expect(
      page.getByRole("heading", { level: 1, name: "Document unavailable" }),
    ).toBeVisible();
    await expect(page.locator("article")).not.toContainText("Outside repository source");
  } finally {
    await fixture.stop();
  }
});

test("source_link_inside_root_then_renders_accessible_vscode_destination", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ sourceLinks: true });

  try {
    await page.goto(fixture.lens.url);
    const initialUrl = page.url();
    const sourceLink = page.getByRole("link", {
      name: "Source file (opens in VS Code)",
    });
    const sourceLineLink = page.getByRole("link", {
      name: "Source line (opens in VS Code)",
    });
    const spacedSourceLink = page.getByRole("link", {
      name: "Source with space (opens in VS Code)",
    });
    const expectedSourceUrl = vscodeUrl(
      await realpath(join(fixture.repository.directory, "src", "example.rs")),
    );
    const expectedSpacedSourceUrl = vscodeUrl(
      await realpath(join(fixture.repository.directory, "src", "example file.rs")),
    );

    // Act
    await sourceLink.hover();

    // Assert
    await expect(sourceLink).toHaveAttribute("href", expectedSourceUrl);
    await expect(sourceLineLink).toHaveAttribute("href", `${expectedSourceUrl}:1:1`);
    await expect(spacedSourceLink).toHaveAttribute("href", expectedSpacedSourceUrl);
    await expect(sourceLink.locator(".source-link-indicator")).toHaveText(" (opens in VS Code)");
    await expect(sourceLink.locator(".source-link-indicator")).toBeVisible();
    expect(page.url()).toBe(initialUrl);
  } finally {
    await fixture.stop();
  }
});

test("changed_source_link_document_then_refreshes_browser_without_navigation", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ sourceLinks: true });

  try {
    await page.goto(fixture.lens.url);
    const initialUrl = page.url();

    // Act
    await writeFile(
      fixture.repository.readmePath,
      `${fixture.repository.sourceLinksMarkdown}\n\nRefreshed source-link page.\n`,
    );

    // Assert
    await expect(page.getByText("Refreshed source-link page.")).toBeVisible();
    expect(page.url()).toBe(initialUrl);
  } finally {
    await fixture.stop();
  }
});

test("disallowed_source_links_then_preserve_authored_destinations", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ sourceLinks: true });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    const authoredDestinations = new Map([
      ["Hidden source", ".hidden/secret.rs"],
      ["Symbolic source", "src/linked.rs"],
      ["Missing source", "src/missing.rs"],
      ["Source directory", "src/directory"],
      ["Outside source", `../${basename(fixture.repository.outsideDocument)}`],
      ["Absolute source", join(fixture.repository.directory, "src", "example.rs")],
    ]);
    for (const [name, destination] of authoredDestinations) {
      const link = page.getByRole("link", { name });
      await expect(link).toHaveAttribute("href", destination);
      await expect(link.locator(".source-link-indicator")).toHaveCount(0);
    }
  } finally {
    await fixture.stop();
  }
});

test("document_external_and_fragment_links_then_preserve_browser_destinations", async ({
  page,
}) => {
  // Arrange
  const fixture = await startBrowserFixture({ sourceLinks: true });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.getByRole("link", { name: "Guide document" })).toHaveAttribute(
      "href",
      "/documents/guides/guide.md",
    );
    await expect(page.getByRole("link", { name: "PlantUML document" })).toHaveAttribute(
      "href",
      "/documents/architecture.puml",
    );
    await expect(page.getByRole("link", { name: "External site" })).toHaveAttribute(
      "href",
      "https://example.com/",
    );
    await expect(page.getByRole("link", { name: "Authored VS Code link" })).toHaveAttribute(
      "href",
      "vscode://file/tmp/authored.rs",
    );
    await expect(page.getByRole("link", { name: "Same-document section" })).toHaveAttribute(
      "href",
      "#source-links",
    );
    await expect(
      page.getByRole("link", { name: "Authored VS Code link" }).locator(".source-link-indicator"),
    ).toHaveCount(0);
  } finally {
    await fixture.stop();
  }
});

test("source_link_then_does_not_add_source_content_route", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture({ sourceLinks: true });

  try {
    await page.goto(fixture.lens.url);

    // Act
    const sourceRoute = await page.request.get(
      `${fixture.lens.url}/source?path=src%2Fexample.rs`,
    );
    const documentRoute = await page.request.get(
      `${fixture.lens.url}/documents/src/example.rs`,
    );

    // Assert
    expect(sourceRoute.status()).toBe(404);
    expect(documentRoute.status()).toBe(404);
    expect(await sourceRoute.text()).not.toContain("Browser source fixture");
    expect(await documentRoute.text()).not.toContain("Browser source fixture");
  } finally {
    await fixture.stop();
  }
});

test("plantuml server fails before client script loads then reveals the source", async ({ page }) => {
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

test("document page then omits rendering status and disable control", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.getByText("PlantUML server rendering")).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "Disable diagram rendering for this session" }),
    ).toHaveCount(0);
    await expect(page.locator(".diagram-disabled")).toHaveCount(0);
    await expect.poll(() => fixture.renderer.requests).toBe(1);
  } finally {
    await fixture.stop();
  }
});

test("plantuml server failure then retry button loads the diagram", async ({ page }) => {
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

test("renderer disable request then returns not found", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    await page.goto(fixture.lens.url);
    await expect.poll(() => fixture.renderer.requests).toBe(1);

    // Act
    const response = await page.request.post(`${fixture.lens.url}/renderer/disable`);

    // Assert
    expect(response.status()).toBe(404);
    expect(fixture.renderer.requests).toBe(1);
  } finally {
    await fixture.stop();
  }
});

async function startBrowserFixture({
  hiddenDocument,
  readme,
  rendererStatus,
  rendererStatuses,
  targetRelativePath,
  currentDirectoryRelativePath,
  scope,
  sourceLinks,
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
      repository && (() => rm(repository.outsideDocument, { force: true })),
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
    repository = await createDocumentationRepository({
      hiddenDocument,
      readme,
      sourceLinks,
    });
    renderer = await startRenderer({ status: rendererStatus, statuses: rendererStatuses });
    lens = await startLens(
      repository,
      renderer.url,
      targetRelativePath,
      currentDirectoryRelativePath,
      scope,
    );
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

async function createDocumentationRepository({
  hiddenDocument,
  readme,
  sourceLinks = false,
} = {}) {
  const directory = await mkdtemp(join(tmpdir(), "lens-browser-"));
  const outsideDocument = `${directory}-outside.md`;
  const binDirectory = join(directory, "bin");
  const readmePath = join(directory, "README.md");
  const sourceLinksMarkdown = [
    "# Source links",
    "",
    "[Source file](src/example.rs)",
    "",
    "[Source line](src/example.rs#L1)",
    "",
    "[Source with space](src/example%20file.rs)",
    "",
    "[Guide document](guides/guide.md)",
    "",
    "[PlantUML document](architecture.puml)",
    "",
    "[Hidden source](.hidden/secret.rs)",
    "",
    "[Symbolic source](src/linked.rs)",
    "",
    "[Missing source](src/missing.rs)",
    "",
    "[Source directory](src/directory)",
    "",
    `[Outside source](../${basename(outsideDocument)})`,
    "",
    `[Absolute source](${join(directory, "src", "example.rs")})`,
    "",
    "[External site](https://example.com/)",
    "",
    "[Authored VS Code link](vscode://file/tmp/authored.rs)",
    "",
    "[Same-document section](#source-links)",
  ].join("\n");
  let files = [];
  try {
    await mkdir(join(directory, "guides"), { recursive: true });
    await mkdir(join(directory, "iterations"));
    await mkdir(join(directory, ".git"));
    await mkdir(binDirectory);
    if (sourceLinks) {
      await mkdir(join(directory, "src", "directory"), { recursive: true });
      await mkdir(join(directory, ".hidden"));
    }
    files = [
      writeFile(
        readmePath,
        readme ??
          (sourceLinks
            ? sourceLinksMarkdown
            : "# Browser fixture\n\nA **rendered** document.\n\n[Open guide](guides/guide.md)\n\n```plantuml\n@startuml\nAlice -> Bob: browser fixture\n@enduml\n```\n"),
      ),
      writeFile(
        join(directory, "guides", "guide.md"),
        `# Guide page\n\nThe guide is a discovered document.\n\n[Iteration evidence](../iterations/evidence.md)\n\n[Outside repository](../../${basename(outsideDocument)})\n`,
      ),
      writeFile(
        join(directory, "iterations", "evidence.md"),
        "# Iteration evidence\n\nRepository-scoped document.\n",
      ),
      writeFile(outsideDocument, "# Outside\n\nOutside repository source.\n"),
      writeFile(
        join(directory, "architecture.puml"),
        "@startuml\nAlice -> Bob: standalone fixture\n@enduml\n",
      ),
      writeFile(join(binDirectory, "xdg-open"), "#!/bin/sh\nexit 0\n"),
    ];
    if (sourceLinks) {
      files.push(
        writeFile(join(directory, "src", "example.rs"), "Browser source fixture"),
        writeFile(join(directory, "src", "example file.rs"), "Spaced browser source fixture"),
        writeFile(join(directory, ".hidden", "secret.rs"), "Hidden browser source fixture"),
        symlink(
          join(directory, "src", "example.rs"),
          join(directory, "src", "linked.rs"),
        ),
      );
    }
    if (hiddenDocument) {
      files.push(writeFile(join(directory, ".private.md"), hiddenDocument));
    }
    await Promise.all(files);
    await chmod(join(binDirectory, "xdg-open"), 0o755);
    return {
      binDirectory,
      directory,
      outsideDocument,
      readmePath,
      sourceLinksMarkdown,
    };
  } catch (error) {
    await Promise.allSettled(files);
    try {
      await rm(directory, { force: true, recursive: true });
      await rm(outsideDocument, { force: true });
    } catch (cleanupError) {
      throw new AggregateError([error, cleanupError], "Repository setup and cleanup failed");
    }
    throw error;
  }
}

function vscodeUrl(path) {
  const normalized = path.replaceAll("\\", "/");
  const rooted = normalized.startsWith("/") ? normalized : `/${normalized}`;
  return `vscode://file${encodeURI(rooted)}`;
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

async function startLens(
  repository,
  rendererUrl,
  targetRelativePath,
  currentDirectoryRelativePath,
  scope,
) {
  const lensBinary = process.env.LENS_BROWSER_TEST_BINARY;
  if (!lensBinary) {
    throw new Error("Playwright global setup did not provide the Lens executable path");
  }
  const commandArguments = currentDirectoryRelativePath
    ? []
    : [targetRelativePath ? join(repository.directory, targetRelativePath) : repository.directory];
  if (scope) {
    commandArguments.push("--scope", scope);
  }
  const child = spawn(lensBinary, commandArguments, {
    cwd: currentDirectoryRelativePath
      ? join(repository.directory, currentDirectoryRelativePath)
      : undefined,
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
