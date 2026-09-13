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
    const response = await page.goto(fixture.lens.urlFor("/documents/architecture.puml"));

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
    const revision = await page.request.get(fixture.lens.urlFor("/revisions/README.md"));
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
    const knownDocumentUrl = fixture.lens.urlFor("/documents/guides/guide.md");
    const ordinaryResponse = await page.request.get(knownDocumentUrl);

    // Act
    const responseWithCatalogQuery = await page.request.get(
      `${knownDocumentUrl}&query=README&page=99`,
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
    const response = await page.goto(fixture.lens.urlFor("/documents/.private.md"));

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
      `/documents/guides/guide.md?token=${fixture.lens.token}`,
    );
    await expect(page.getByRole("link", { name: "PlantUML document" })).toHaveAttribute(
      "href",
      `/documents/architecture.puml?token=${fixture.lens.token}`,
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
      `${fixture.lens.origin}/source?token=${fixture.lens.token}&path=src%2Fexample.rs`,
    );
    const documentRoute = await page.request.get(
      fixture.lens.urlFor("/documents/src/example.rs"),
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
    const response = await page.request.post(fixture.lens.urlFor("/renderer/disable"));

    // Assert
    expect(response.status()).toBe(404);
    expect(fixture.renderer.requests).toBe(1);
  } finally {
    await fixture.stop();
  }
});

test("unauthenticated_request_without_capability_then_receives_unauthorized", async ({ page }) => {
  // Arrange
  const fixture = await startBrowserFixture();

  try {
    // Act
    const unauthenticatedInitial = await page.request.get(fixture.lens.origin);
    const unauthenticatedDocument = await page.request.get(
      `${fixture.lens.origin}/documents/guides/guide.md`,
    );
    const unauthenticatedRevision = await page.request.get(
      `${fixture.lens.origin}/revisions/README.md`,
    );
    const unauthenticatedDiagram = await page.request.get(
      `${fixture.lens.origin}/diagrams/0/0`,
    );
    const unauthenticatedAsset = await page.request.get(
      `${fixture.lens.origin}/app.js`,
    );

    // Assert
    expect(unauthenticatedInitial.status()).toBe(401);
    expect(unauthenticatedDocument.status()).toBe(401);
    expect(unauthenticatedRevision.status()).toBe(401);
    expect(unauthenticatedDiagram.status()).toBe(401);
    expect(unauthenticatedAsset.status()).toBe(401);
  } finally {
    await fixture.stop();
  }
});

test("cross_session_request_then_cannot_access_other_session", async ({ page }) => {
  // Arrange
  const firstFixture = await startBrowserFixture();
  const secondFixture = await startBrowserFixture({
    readme: "# Second isolated fixture",
  });

  try {
    // Act
    const crossInitial = await page.request.get(
      `${firstFixture.lens.origin}/?token=${secondFixture.lens.token}`,
    );
    const crossDiagram = await page.request.get(
      `${firstFixture.lens.origin}/diagrams/0/0?token=${secondFixture.lens.token}`,
    );

    // Assert
    expect(crossInitial.status()).toBe(401);
    expect(crossDiagram.status()).toBe(401);
  } finally {
    await secondFixture.stop();
    await firstFixture.stop();
  }
});

test("shared_browser_context_then_two_sessions_isolate_and_refresh_independently", async ({
  context,
}) => {
  // Arrange
  const firstFixture = await startBrowserFixture({
    readme: "# First document\n\nInitial first content.\n\n[Open first guide](guides/guide.md)\n",
  });
  const secondFixture = await startBrowserFixture({
    readme: "# Second document\n\nInitial second content.\n\n[Open second guide](guides/guide.md)\n",
  });

  const firstPage = await context.newPage();
  const secondPage = await context.newPage();

  try {
    // Act
    await firstPage.goto(firstFixture.lens.url);
    await expect(firstPage.getByRole("heading", { level: 1, name: "First document" })).toBeVisible();
    await expect(firstPage.locator("article")).toContainText("Initial first content.");
    await firstPage.getByRole("link", { name: "Open first guide" }).click();
    await expect(firstPage.getByRole("heading", { level: 1, name: "Guide page" })).toBeVisible();
    await firstPage.goBack();
    await expect(firstPage.getByRole("heading", { level: 1, name: "First document" })).toBeVisible();

    await secondPage.goto(secondFixture.lens.url);
    await expect(secondPage.getByRole("heading", { level: 1, name: "Second document" })).toBeVisible();
    await expect(secondPage.locator("article")).toContainText("Initial second content.");
    await secondPage.getByRole("link", { name: "Open second guide" }).click();
    await expect(secondPage.getByRole("heading", { level: 1, name: "Guide page" })).toBeVisible();
    await secondPage.goBack();
    await expect(secondPage.getByRole("heading", { level: 1, name: "Second document" })).toBeVisible();

    // Update both documents to verify live refresh isolation
    await writeFile(
      join(firstFixture.repository.directory, "README.md"),
      "# First document\n\nUpdated first content.\n",
    );
    await writeFile(
      join(secondFixture.repository.directory, "README.md"),
      "# Second document\n\nUpdated second content.\n",
    );

    // Assert
    await expect(firstPage.locator("article")).toContainText("Updated first content.");
    await expect(firstPage.locator("article")).not.toContainText("Second");

    await expect(secondPage.locator("article")).toContainText("Updated second content.");
    await expect(secondPage.locator("article")).not.toContainText("First");
  } finally {
    await firstPage.close();
    await secondPage.close();
    await secondFixture.stop();
    await firstFixture.stop();
  }
});

test("second_port_navigation_then_no_capability_transmitted_and_unauthenticated_replay_fails", async ({
  page,
}) => {
  // Arrange
  const fixture = await startBrowserFixture();
  let capturedHeaders = null;
  const captureServer = createServer((req, res) => {
    if (req.url === "/capture") {
      capturedHeaders = req.headers;
    }
    res.writeHead(200, { "Content-Type": "text/html" });
    res.end("<h1>Captured</h1>");
  });
  await new Promise((resolve) => captureServer.listen(0, "127.0.0.1", resolve));
  const capturePort = captureServer.address().port;
  const captureUrl = `http://127.0.0.1:${capturePort}/capture`;

  try {
    // Act
    await page.goto(fixture.lens.url);
    await expect(page.getByRole("heading", { level: 1, name: "Browser fixture" })).toBeVisible();

    await page.goto(captureUrl);
    await expect(page.getByRole("heading", { level: 1, name: "Captured" })).toBeVisible();

    // Assert
    expect(capturedHeaders.cookie).toBeUndefined();
    expect(capturedHeaders.referer).toBeUndefined();
    expect(JSON.stringify(capturedHeaders)).not.toContain(fixture.lens.token);

    const replayDocument = await page.request.get(fixture.lens.origin);
    const replayDiagram = await page.request.get(`${fixture.lens.origin}/diagrams/0/0`);
    expect(replayDocument.status()).toBe(401);
    expect(replayDiagram.status()).toBe(401);
  } finally {
    await new Promise((resolve) => captureServer.close(resolve));
    await fixture.stop();
  }
});

test("mermaid_diagram_with_font_family_override_then_does_not_alter_document_body_styles", async ({
  page,
}) => {
  // Arrange
  const readme = [
    "# Mermaid Security Fixture",
    "",
    "Text before diagram.",
    "",
    "```mermaid",
    '%%{init: {"fontFamily": "x;a{b} :not(&){display:none !important} c{d}"}}%%',
    "flowchart LR",
    "A-->B",
    "```",
    "",
    "Text after diagram.",
    "",
    "```mermaid",
    "flowchart LR",
    "C-->D",
    "```",
  ].join("\n");
  const fixture = await startBrowserFixture({ readme });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.getByRole("heading", { level: 1, name: "Mermaid Security Fixture" })).toBeVisible();
    await expect(page.locator("article")).toContainText("Text before diagram.");
    await expect(page.locator("article")).toContainText("Text after diagram.");

    const bodyDisplay = await page.evaluate(() => getComputedStyle(document.body).display);
    expect(bodyDisplay).toBe("block");

    const diagrams = page.locator("[data-mermaid-container]");
    await expect(diagrams).toHaveCount(2);
    await expect(diagrams.nth(0).locator(".mermaid-target svg")).toBeVisible();
    await expect(diagrams.nth(1).locator(".mermaid-target svg")).toBeVisible();
  } finally {
    await fixture.stop();
  }
});

test("mermaid_gantt_with_all_weekdays_excluded_then_does_not_hang_and_reveals_source", async ({
  page,
}) => {
  // Arrange
  const readme = [
    "# Mermaid Gantt Fixture",
    "",
    "```mermaid",
    "gantt",
    "  excludes monday,tuesday,wednesday,thursday,friday,saturday,sunday",
    "  Task :2025-01-01, 1d",
    "```",
    "",
    "```mermaid",
    "flowchart LR",
    "A-->B",
    "```",
  ].join("\n");
  const fixture = await startBrowserFixture({ readme });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.getByRole("heading", { level: 1, name: "Mermaid Gantt Fixture" })).toBeVisible();

    const diagrams = page.locator("[data-mermaid-container]");
    await expect(diagrams).toHaveCount(2);

    const invalidGantt = diagrams.nth(0);
    await expect(invalidGantt.locator(".diagram-error")).toBeVisible();
    await expect(invalidGantt.locator(".diagram-source")).toHaveAttribute("open", "");
    await expect(invalidGantt.locator(".diagram-source code")).toContainText("excludes monday,tuesday");

    const validFlowchart = diagrams.nth(1);
    await expect(validFlowchart.locator(".mermaid-target svg")).toBeVisible();
    await expect(validFlowchart.locator(".diagram-error")).toBeHidden();
  } finally {
    await fixture.stop();
  }
});

test("rendered_mermaid_diagram_then_displays_standalone_open_svg_link", async ({
  page,
}) => {
  // Arrange
  const readme = [
    "# Mermaid Standalone SVG Fixture",
    "",
    "```mermaid",
    "flowchart LR",
    "  Alpha[Alpha Service] --> Beta[Beta Service]",
    "```",
  ].join("\n");
  const fixture = await startBrowserFixture({ readme });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.getByRole("heading", { level: 1, name: "Mermaid Standalone SVG Fixture" })).toBeVisible();

    const diagram = page.locator("[data-mermaid-container]");
    await expect(diagram).toHaveCount(1);
    await expect(diagram.locator(".mermaid-target svg")).toBeVisible();

    const openLink = diagram.locator("[data-mermaid-open]");
    await expect(openLink).toBeVisible();
    await expect(openLink).toHaveAttribute("target", "_blank");
    await expect(openLink).toHaveAttribute("rel", "noopener noreferrer");
    await expect(openLink).toHaveText("Open SVG");

    const href = await openLink.getAttribute("href");
    expect(href).toMatch(/^blob:/);
  } finally {
    await fixture.stop();
  }
});

test("blob_url_creation_fails_then_in_document_mermaid_renders_and_open_link_remains_hidden", async ({
  page,
}) => {
  // Arrange
  const readme = [
    "# Mermaid Standalone SVG Fixture",
    "",
    "```mermaid",
    "flowchart LR",
    "  Alpha[Alpha Service] --> Beta[Beta Service]",
    "```",
  ].join("\n");
  const fixture = await startBrowserFixture({ readme });

  try {
    await page.addInitScript(() => {
      window.URL.createObjectURL = () => {
        throw new Error("Simulated Blob URL creation failure");
      };
    });

    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.getByRole("heading", { level: 1, name: "Mermaid Standalone SVG Fixture" })).toBeVisible();

    const diagram = page.locator("[data-mermaid-container]");
    await expect(diagram).toHaveCount(1);
    await expect(diagram.locator(".mermaid-target svg")).toBeVisible();
    await expect(diagram.locator(".diagram-error")).toBeHidden();
    await expect(diagram.locator("[data-mermaid-open]")).toBeHidden();
  } finally {
    await fixture.stop();
  }
});

test("invalid_mermaid_syntax_then_suppresses_open_svg_link_and_reveals_source", async ({
  page,
}) => {
  // Arrange
  const readme = [
    "# Invalid Mermaid Fixture",
    "",
    "```mermaid",
    "flowchart TD",
    "  [invalid syntax",
    "```",
  ].join("\n");
  const fixture = await startBrowserFixture({ readme });

  try {
    // Act
    await page.goto(fixture.lens.url);

    // Assert
    await expect(page.getByRole("heading", { level: 1, name: "Invalid Mermaid Fixture" })).toBeVisible();

    const diagram = page.locator("[data-mermaid-container]");
    await expect(diagram).toHaveCount(1);
    await expect(diagram.locator(".diagram-error")).toBeVisible();
    await expect(diagram.locator("[data-mermaid-open]")).toBeHidden();
    await expect(diagram.locator(".diagram-source")).toHaveAttribute("open", "");
    await expect(diagram.locator(".diagram-source code")).toContainText("[invalid syntax");
  } finally {
    await fixture.stop();
  }
});

test("open_svg_link_clicked_then_navigates_to_uncorrupted_blob_url_and_scales_dynamically", async ({
  page,
}) => {
  // Arrange
  const readme = [
    "# Mermaid Standalone SVG Fixture",
    "",
    "```mermaid",
    "flowchart LR",
    "  Alpha[Alpha Service] --> Beta[Beta Service]",
    "```",
  ].join("\n");
  const fixture = await startBrowserFixture({ readme });

  try {
    await page.goto(fixture.lens.url);
    await expect(page.getByRole("heading", { level: 1, name: "Mermaid Standalone SVG Fixture" })).toBeVisible();

    const diagram = page.locator("[data-mermaid-container]");
    await expect(diagram.locator(".mermaid-target svg")).toBeVisible();
    const openLink = diagram.locator("[data-mermaid-open]");
    await expect(openLink).toBeVisible();

    // Act
    const pagePromise = page.context().waitForEvent("page");
    await openLink.click();
    const svgPage = await pagePromise;
    await svgPage.waitForLoadState();

    // Assert
    const linkHref = await openLink.getAttribute("href");
    expect(linkHref).toMatch(/^blob:/);
    expect(linkHref).not.toContain("token=");

    const svgUrl = svgPage.url();
    expect(svgUrl).toMatch(/^blob:/);
    expect(svgUrl).not.toContain("token=");

    const standaloneSvg = svgPage.locator("svg");
    await expect(standaloneSvg).toBeVisible();
    await expect(standaloneSvg).toHaveAttribute("width", "100%");
    await expect(standaloneSvg).toHaveAttribute("height", "100%");

    const maxWidth = await standaloneSvg.evaluate((el) => el.style.maxWidth);
    expect(maxWidth).toBe("");

    const backgroundColor = await standaloneSvg.evaluate((el) => el.style.backgroundColor);
    expect(["rgb(255, 255, 255)", "#ffffff"]).toContain(backgroundColor);

    // Verify dynamic viewport scaling
    await svgPage.setViewportSize({ width: 600, height: 400 });
    const initialBox = await standaloneSvg.boundingBox();

    await svgPage.setViewportSize({ width: 1200, height: 800 });
    const expandedBox = await standaloneSvg.boundingBox();

    expect(expandedBox.width).toBeGreaterThan(initialBox.width);
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
  const runtimeDirectory = join(directory, "runtime");
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
    await mkdir(runtimeDirectory);
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
    await chmod(runtimeDirectory, 0o700);
    return {
      binDirectory,
      directory,
      outsideDocument,
      readmePath,
      runtimeDirectory,
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
  const environment = {
    ...process.env,
    LENS_PLANTUML_SERVER: rendererUrl,
    PATH: `${repository.binDirectory}:${process.env.PATH}`,
    XDG_RUNTIME_DIR: repository.runtimeDirectory,
  };
  const service = spawn(lensBinary, ["--lens-background-service"], {
    env: environment,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const stop = async () => {
    if (service.exitCode !== null || service.signalCode !== null || service.pid === undefined) {
      return;
    }
    const closed = once(service, "close");
    service.kill("SIGKILL");
    await closed;
  };
  try {
    await waitForServiceReady(service);
    const readyUrl = await runLensClient(lensBinary, commandArguments, {
      cwd: currentDirectoryRelativePath
        ? join(repository.directory, currentDirectoryRelativePath)
        : undefined,
      env: environment,
    });
    const parsed = new URL(readyUrl);
    const token = parsed.searchParams.get("token") || "";
    return {
      url: readyUrl,
      origin: parsed.origin,
      token,
      urlFor: (pathname) => `${parsed.origin}${pathname}?token=${token}`,
      stop,
    };
  } catch (error) {
    await stop();
    throw error;
  }
}

function runLensClient(lensBinary, commandArguments, options) {
  const child = spawn(lensBinary, commandArguments, {
    ...options,
    stdio: ["ignore", "pipe", "pipe"],
  });
  return new Promise((resolveUrl, reject) => {
    let stdout = "";
    let stderr = "";
    const timeout = setTimeout(
      () => reject(new Error(`Lens client did not exit after its ready acknowledgment: ${stdout}${stderr}`)),
      10_000,
    );
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.once("error", (error) => {
      clearTimeout(timeout);
      reject(error);
    });
    child.once("close", (status, signal) => {
      clearTimeout(timeout);
      if (status !== 0) {
        reject(new Error(`Lens client failed (status ${status}, signal ${signal}): ${stdout}${stderr}`));
        return;
      }
      const match = stdout.match(/at (http:\/\/127\.0\.0\.1:\d+\S*)/);
      if (!match) {
        reject(new Error(`Lens client did not print a loopback URL: ${stdout}${stderr}`));
        return;
      }
      resolveUrl(match[1]);
    });
  });
}

function waitForServiceReady(child) {
  return new Promise((resolve, reject) => {
    let stdout = "";
    let stderr = "";
    const timeout = setTimeout(
      () => reject(new Error(`Lens background service did not become ready: ${stdout}${stderr}`)),
      10_000,
    );
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
      if (stdout.includes("Lens background service is ready")) {
        clearTimeout(timeout);
        resolve();
      }
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.once("error", (error) => {
      clearTimeout(timeout);
      reject(error);
    });
    child.once("exit", (status, signal) => {
      clearTimeout(timeout);
      reject(
        new Error(
          `Lens background service exited before readiness (status ${status}, signal ${signal}): ${stdout}${stderr}`,
        ),
      );
    });
  });
}
