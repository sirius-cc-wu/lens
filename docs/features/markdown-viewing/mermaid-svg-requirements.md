---
type: "Feature Specification"
title: "Mermaid Diagram Standalone SVG View Requirements"
description: "Product requirements, user stories, example mapping, and acceptance criteria for viewing rendered Mermaid diagrams in a standalone scalable SVG tab."
id: "FEAT-01-REQ-MERMAID-SVG"
status: "active"
scope: "Lens"
tags: [requirements, specification, mermaid, svg]
---

# Mermaid Diagram Standalone SVG View Requirements

## 1. Problem Statement & User Value

In Lens, Markdown documents are styled for prose readability with a centered column layout constrained to a maximum width of 920 pixels (`min(920px, calc(100% - 2rem))`). While this layout ensures comfortable reading line lengths for text, Mermaid diagrams rendered inside the document are constrained to this column width.

Complex diagrams—such as large architectural graphs, multi-lane sequence diagrams, extensive class hierarchies, or state machine charts—are scaled down proportionally to fit inside the 920-pixel column. As a result:
- Text labels, node descriptions, and edge annotations become too small to read comfortably.
- On large monitors (1080p, 1440p, 4K), screen real estate is underutilized while diagrams remain cramped.
- Users cannot easily isolate or export the pure vector graphic without page chrome and text surrounding it.

### Product Goal

Enable developers and technical writers to open any successfully rendered Mermaid diagram in a dedicated browser tab or window as a standalone, scalable SVG document (`image/svg+xml`). In this standalone view, the diagram is liberated from document width constraints, dynamically scales with window resizing, supports native browser zoom, and provides a clean vector asset that can be inspected or saved directly.

---

## 2. User Story

**As a** developer or technical writer reviewing architecture and technical documentation in Lens,  
**I want to** open a rendered Mermaid diagram in a separate browser tab as a standalone scalable SVG,  
**So that** I can resize the browser window and zoom freely to read diagram labels clearly, regardless of diagram complexity or screen size.

---

## 3. Collaborative Requirements Workshop (Three Amigos / Example Mapping)

To clarify requirements, uncover boundary scenarios, and align on acceptance criteria, product, engineering, and quality contributors collaborate in a requirements discovery workshop (Three Amigos discovery using example mapping). The structured visual map below illustrates the user story, underlying business rules, and concrete examples.

### User Story
> *Open a rendered Mermaid diagram in a standalone SVG tab to inspect and resize it without document column constraints.*

```
                       ┌────────────────────────────────────────────────────────┐
                       │                       User Story                       │
                       │  Open rendered Mermaid diagram in standalone SVG tab.  │
                       └──────────────────────────┬─────────────────────────────┘
                                                  │
         ┌───────────────────┬────────────────────┼───────────────────┬───────────────────┐
         ▼                   ▼                    ▼                   ▼                   ▼
    ┌──────────┐       ┌───────────┐        ┌───────────┐       ┌───────────┐       ┌───────────┐
    │  Rule 1  │       │  Rule 2   │        │  Rule 3   │       │  Rule 4   │       │  Rule 5   │
    │ Gated    │       │ Isolated  │        │ Full      │       │ High      │       │ 100%      │
    │ Control  │       │ New Tab   │        │ Scaling   │       │ Contrast  │       │ Offline   │
    └────┬─────┘       └─────┬─────┘        └─────┬─────┘       └─────┬─────┘       └─────┬─────┘
         │                   │                    │                   │                   │
         ├─────────┐         ├─────────┐          ├─────────┐         │                   │
         ▼         ▼         ▼         ▼          ▼         ▼         ▼                   ▼
     [Ex 1.1]  [Ex 1.2]  [Ex 2.1]  [Ex 2.2]   [Ex 3.1]  [Ex 3.2]  [Ex 4.1]            [Ex 5.1]
      Valid    Syntax     Opens    Document   Expands   Browser    White bg            Zero net
     diagram   error      _blank   untouched  to window   zoom     in dark             traffic
      shows    hides      tab      scroll     width       works    viewer
      link     link
```

### Business Rules & Concrete Examples

#### Rule 1: Control Gating & Availability
*The "Open SVG" action must only be presented on Mermaid diagram blocks that have rendered successfully.*
- **Example 1.1 (Valid diagram):** A Markdown document contains a valid Mermaid diagram ````mermaid\ngraph TD;\nA-->B;\n````. Once client-side rendering succeeds, an accessible "Open SVG" action is displayed on the diagram figure.
- **Example 1.2 (Syntax error):** A document contains a malformed Mermaid diagram ````mermaid\ngraph TD;\nA--->;\n````. Rendering fails; Lens displays the error message and the expandable source code disclosure. The "Open SVG" action remains hidden.
- **Example 1.3 (Initial loading):** Before client-side diagram rendering has completed, the "Open SVG" action is hidden so users cannot navigate to an unrendered or blank diagram.
- **Example 1.4 (Mixed document):** A document contains one valid diagram and one invalid diagram. The valid diagram displays the "Open SVG" action; the invalid diagram suppresses it.

#### Rule 2: Independent Browsing Context & Non-Destructive Navigation
*Activating the action must open the diagram in a separate browser tab or window without modifying or navigating away from the current Markdown document view.*
- **Example 2.1 (New tab):** Clicking "Open SVG" opens a new tab (`target="_blank"`, `rel="noopener noreferrer"`). The user's original document tab remains active and preserves its exact scroll position.
- **Example 2.2 (Multi-diagram isolation):** A document contains 3 distinct Mermaid diagrams. Clicking "Open SVG" on diagram #2 opens specifically diagram #2 in a new tab. Diagram #1 and #3 remain unaffected.
- **Example 2.3 (Tab closing):** Closing the standalone SVG tab does not disrupt the document viewing session or terminal process.

#### Rule 3: Unconstrained Responsive Scalability
*In the standalone view, the diagram must not be constrained by article column boundaries or hardcoded inline width caps, expanding smoothly with viewport resizing.*
- **Example 3.1 (Full window expansion):** A wide sequence diagram (intrinsic width 2200px) was shrunk to 920px in the Markdown document. When opened in a new tab on a 1920x1080 monitor, the diagram expands across the full available viewport width, making all message labels legible.
- **Example 3.2 (Dynamic window resize):** The user resizes the standalone browser window from 800px to 1600px width. The SVG dynamically scales proportionally without clipping, fixed width ceilings, or layout distortions.
- **Example 3.3 (Native browser zoom):** The user presses `Ctrl +` (or `Cmd +`) in the standalone tab. The browser natively zooms the vector SVG with sharp lines and crisp typography.

#### Rule 4: Visual Contrast & Dark-Mode Viewer Legibility
*The standalone SVG must provide legible contrast across various browser image viewer canvas backgrounds.*
- **Example 4.1 (Dark canvas theme):** A user whose browser displays standalone images over a dark grey or black background opens the SVG. Because the standalone SVG carries a clean light/white background style, dark diagram lines and text remain fully legible and high-contrast.

#### Rule 5: 100% Offline & Pure Client Operation
*Standalone SVG viewing must operate completely offline without external network dependencies, remote server calls, or persistent server-side caching.*
- **Example 5.1 (No internet connection):** Lens runs on an air-gapped machine with no internet connection. The user opens a Markdown document and activates "Open SVG". The standalone SVG tab opens and renders completely offline.

#### Rule 6: Document Auto-Refresh Coexistence
*When an authorized document's source file is updated and saved, automatic refresh must seamlessly update both the in-document diagram and the standalone view target.*
- **Example 6.1 (File edit & refresh):** The user edits a diagram in `architecture.md` and saves the file. Lens detects the change and refreshes the document view. The in-document diagram updates, and the "Open SVG" action points to the newly updated diagram. Any previously opened standalone tab remains readable as a static snapshot.

---

## 4. Use Case Specification (`UC-11`)

### Use Case Identifier: `UC-11` (or `FEAT-01: UC-11`)
*(Note: Globally within the Lens repository, `UC-11` is also used in `FEAT-04: Background Viewer Service`; in the context of `FEAT-01: Markdown and Diagram Viewing`, this use case specifies Standalone SVG Viewing.)*

- **Primary Actor:** Developer or technical writer reading repository documentation.
- **Goal:** View a rendered Mermaid diagram in a dedicated browser tab as a standalone scalable SVG document.
- **Preconditions:**
  1. Lens is running an active viewing session for an authorized Markdown document.
  2. The document contains at least one fenced ````mermaid block.
  3. The Mermaid diagram has been successfully parsed and rendered in the browser.
- **Trigger:** The user activates the "Open SVG" control associated with a rendered Mermaid diagram.

### Main Success Scenario
1. The user views a Markdown document containing a rendered Mermaid diagram.
2. Lens presents an accessible "Open SVG" control on the rendered diagram container.
3. The user activates the "Open SVG" control.
4. The browser opens a new browser tab displaying the diagram as a standalone SVG document (`image/svg+xml`).
5. The standalone SVG is unconstrained by the document article column width, scaling to fit the new viewport with full vector fidelity.
6. The user resizes the browser window or applies native browser zoom to inspect fine diagram details legibly.
7. The user may natively save or copy the SVG asset using standard browser controls (e.g., right-click "Save image as...").
8. The original Markdown document remains open, interactive, and at its current scroll position in the primary browser tab.

### Extensions & Alternative Scenarios
- **1a. Rendering failure / syntax error:**
  - If a Mermaid block contains invalid syntax, Lens displays the actionable error banner and raw source disclosure `<details>`.
  - The "Open SVG" control is suppressed (hidden) so users cannot navigate to a broken view.
- **1b. Diagram awaiting render:**
  - Before client-side rendering completes, the "Open SVG" control remains hidden.
- **3a. Multiple diagrams in one document:**
  - Each rendered diagram has its own independent "Open SVG" control. Activating it opens only that specific diagram.
- **4a. Closing the standalone tab:**
  - Closing the standalone tab has no effect on the primary document tab or the Lens background process.
- **4b. Theme/Canvas contrast:**
  - If the browser's native image viewer uses a dark canvas background, the SVG provides a neutral light background style ensuring dark text and lines remain readable.
- **8a. Markdown source file updated on disk:**
  - Automatic refresh re-renders the document and updates the diagram. The "Open SVG" control is refreshed to provide the updated diagram on subsequent clicks. Previously opened tabs retain their static snapshot.

---

## 5. Observable Acceptance Criteria

| ID | Criterion | Verification Method |
|:---|:---|:---|
| **AC-1** | **Control Visibility on Valid Diagrams:** Every successfully rendered Mermaid diagram displays a visible, accessible "Open SVG" action link. | Automated browser test verifies element presence and visibility after Mermaid rendering completes. |
| **AC-2** | **Control Suppression on Invalid Diagrams:** If a Mermaid block fails to render due to syntax errors, the "Open SVG" control remains hidden while the error message and source code remain visible. | Automated test verifies `[data-mermaid-open]` element has `hidden` attribute when given invalid Mermaid syntax. |
| **AC-3** | **Standalone Tab Navigation:** Clicking "Open SVG" opens a new browser tab/window (`_blank`) presenting a pure SVG document (`image/svg+xml`). | Automated test verifies link attributes `target="_blank"` and `rel="noopener noreferrer"`, and verifies document MIME type in new page. |
| **AC-4** | **Unconstrained Vector Scaling:** When viewed in the standalone tab, the SVG expands to fit the viewport width without being capped by the document column (920px) or Mermaid inline max-width limits. | Automated browser test asserts SVG bounding rect widens when window is resized from 800px to 1600px. |
| **AC-5** | **High Contrast Legibility:** The standalone SVG includes an explicit background style ensuring legibility across light and dark browser image viewer canvases. | Inspect root SVG styles for background color declaration. |
| **AC-6** | **Session Isolation & Token Safety:** Opening the standalone SVG does not leak session tokens into external or public URLs, nor corrupt URL parameters during navigation. | Verify opened URL structure and verify `app.js` link interceptor ignores standalone Blob URLs. |
| **AC-7** | **Zero Regression:** All existing PlantUML rendering, document navigation, source link handoff, and automatic refresh capabilities continue to function without regression. | Run full test suite (`cargo test --locked` and browser integration test suite). |
| **AC-8** | **Offline Operation:** The standalone SVG capability operates completely without internet or remote network access. | Verify zero external network requests during rendering and standalone viewing. |

---

## 6. Non-Goals & Scope Boundaries

To maintain lean, focused product delivery and avoid scope creep:
- **No in-place interactive diagram editor:** Editing Mermaid diagram source in the browser is out of scope; source edits continue to happen in the user's editor (e.g. VS Code) with Lens providing automatic refresh.
- **No standalone `.mmd` / `.mermaid` file viewing in this milestone:** Target scope remains Markdown documents containing fenced ````mermaid blocks (similar to `FEAT-01` initial scope).
- **No PlantUML standalone SVG changes in this scope:** PlantUML diagrams continue to use their established server-rendered image pipeline and per-diagram retry mechanisms.
- **No custom canvas zoom/pan UI widgets:** Users leverage native browser window resizing, native browser zoom (`Ctrl +` / `Ctrl -`), and scrollbars in the standalone tab rather than a heavyweight custom JavaScript pan-and-zoom library.
- **Implementation Independence:** Technical implementation decisions (e.g. DOMParser post-processing, Blob URL lifecycle, Axum routing vs pure client memory) are delegated to the Software Architect and Software Engineer specifications.

---

## 7. Open Product Questions for Sirius

1. **Standalone Viewing for PlantUML Diagrams:**  
   Currently, PlantUML diagrams are rendered as raster images or SVG via the configured PlantUML server. Does Sirius want a similar "Open Standalone Image/SVG" capability for PlantUML diagrams in a subsequent release, or is this requirement strictly focused on Mermaid?
2. **Action Presentation Preference:**  
   The current requirement defines a textual link labeled "Open SVG" positioned adjacent to the rendered diagram. Does Sirius prefer this explicit text label, or would an iconography-based control (e.g., an expand/popout icon) be preferred?
3. **In-Document Zoom Controls vs Standalone Tab:**  
   The requirement directly addresses Sirius's request to "show the diagrams in another browser page as a SVG so that I can resize it." Would in-document pan-and-zoom controls (e.g. dragging and mouse-wheel zoom directly inside the Markdown reading pane) be of interest for a future roadmap milestone, or is the standalone tab approach sufficient?
