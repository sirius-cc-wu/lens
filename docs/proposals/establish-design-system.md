---
type: "Improvement Proposal"
title: "Establish a Lens Design System"
description: "Explores visual directions in Stitch, selects one through explicit criteria, and records the resulting interface rules in a repository-level DESIGN.md."
status: "proposed"
tags: [proposal, design-system, user-interface]
---

# Establish a Lens Design System

Status: proposed

## Summary

Establish a design system—a shared set of visual rules and reusable interface
patterns—for Lens. Create a dedicated Stitch project to explore several
distinct visual directions against the same Lens workflows, compare those
directions with explicit criteria, and select one before changing the
production interface.

Record the selected direction in a repository-level `DESIGN.md`. That file will
be the durable source of truth for Lens's visual foundations, layouts,
components, interaction states, responsive behavior, and accessibility
expectations. Stitch will support exploration; the repository will retain the
decision and all rules needed to implement and maintain it.

## Motivation

Lens already has a recognizable browser interface, including a warm reading
surface, document navigation, Markdown typography, metadata tables, diagram
controls, and responsive behavior. Those decisions currently live primarily in
the embedded `APP_STYLESHEET` in `src/viewer.rs`. There is no single artifact
that explains why the interface looks and behaves as it does or how a new
feature should fit into it.

Without a design system, each interface change must rediscover choices such as
color, typography, spacing, borders, status treatment, responsive layout, and
control hierarchy. This increases visual drift and makes review depend on
individual taste. A documented system will make future changes more consistent,
give contributors and coding agents concrete constraints, and make visual
decisions reviewable before they become implementation details.

Generating alternatives before selecting a direction also creates a deliberate
decision point. It allows Lens to retain the strongest parts of its current
editorial character while considering other expressions appropriate for a
local technical-documentation viewer.

## Goals

- Give Lens a coherent and recognizable visual identity suited to reading
  technical documentation.
- Make long-form Markdown, source code, tables, metadata, and diagrams readable
  across supported viewport sizes.
- Define reusable foundations and component states instead of accumulating
  isolated CSS values.
- Preserve semantic HTML, keyboard operation, visible focus, and existing
  no-JavaScript navigation behavior.
- Make the selected direction implementable without a runtime design framework
  or an external network dependency.
- Give future interface proposals a stable `DESIGN.md` against which they can
  be evaluated.

## Proposed Exploration

### Shared Stitch brief

Create one Stitch project for Lens. Stitch is used here as an interface concept
exploration tool, not as a runtime dependency or the canonical design record.
Every alternative should use the same content, controls, states, and viewport
sizes so that reviewers compare design decisions rather than different feature
sets.

The shared brief should describe Lens as:

- a local, browser-based reader for repository Markdown and PlantUML;
- optimized for sustained reading and architecture discovery;
- technical and trustworthy without resembling a general-purpose editor;
- responsive from a narrow mobile viewport to a wide desktop viewport; and
- self-contained, with no externally hosted fonts, scripts, styles, or images
  required at runtime.

### Alternatives

Create at least three meaningfully different directions. Initial hypotheses are:

1. **Editorial Reader** — evolve the current warm, typographic reading
   experience with clearer hierarchy and more systematic components.
2. **Technical Workbench** — favor compact information density, strong
   navigation, and utilitarian controls for frequent repository use.
3. **Quiet Documentation** — use a restrained neutral surface that keeps
   authored documents and diagrams visually dominant.

These names are starting points, not required outcomes. Each direction should
make a distinct claim about Lens's character rather than present only a new
palette.

### Required concept surfaces

Each direction should show the same representative states:

- a desktop document with the navigation pane visible;
- the same document with the navigation pane hidden;
- a narrow viewport with navigation, document content, and controls;
- navigation search results, pagination, and the current document;
- headings, paragraphs, links, lists, block quotes, inline code, code blocks,
  and a wide table;
- YAML frontmatter metadata and a malformed-frontmatter error;
- a successfully rendered diagram with disclosed source;
- renderer status, diagram failure with retry, and rendering-disabled states;
  and
- long document identifiers and content that wraps or scrolls.

The exploration may use a fixture document assembled specifically for design
review. It must not omit difficult states merely because a simpler mockup is
more visually attractive.

## Selection Criteria

Review the alternatives using one shared rubric:

| Criterion | Question |
|---|---|
| Reading quality | Can users comfortably scan and read long technical documents? |
| Information hierarchy | Are document identity, authored content, navigation, metadata, and system status clearly distinguished? |
| Repository navigation | Are search, pagination, current location, and hide/show controls easy to understand? |
| State clarity | Are normal, focused, selected, disabled, warning, and error states distinguishable without relying only on color? |
| Responsive behavior | Does the direction remain useful at narrow and wide viewports without hiding essential information? |
| Accessibility | Can the design support sufficient contrast, visible keyboard focus, meaningful control labels, and reduced-motion preferences? |
| Implementation fit | Can Lens implement it with semantic server-rendered HTML, its existing small JavaScript layer, and self-hosted assets? |
| Product character | Does it feel specific to a focused technical-documentation viewer rather than a generic dashboard or marketing site? |

The review should record:

- the selected direction and the reasons it best fits Lens;
- useful elements adopted from other directions;
- rejected directions and the main trade-off for each; and
- any issue that must be proven in browser code rather than inferred from a
  static concept.

Static mockups cannot verify keyboard behavior, browser reflow, contrast, or
assistive-technology semantics. Those qualities remain implementation and
verification responsibilities.

## Repository Design Record

After a direction is selected, add `DESIGN.md` at the repository root. It should
be understandable without access to the Stitch project and should cover:

1. **Purpose and principles** — the user experience Lens is trying to create
   and the qualities that guide trade-offs.
2. **Selected direction** — representative images, the selection rationale, and
   links to any retained exploration artifacts.
3. **Visual foundations** — named reusable values (design tokens) for color,
   typography, spacing, widths, borders, corner treatment, and elevation where
   used.
4. **Page structure** — document width, navigation relationship, header
   hierarchy, and narrow-viewport behavior.
5. **Content styling** — Markdown headings, prose, links, lists, quotations,
   code, tables, metadata, and PlantUML diagrams.
6. **Components and states** — buttons, inputs, navigation items, pagination,
   notices, errors, renderer controls, diagram source disclosure, and visible
   focus.
7. **Interaction and accessibility** — keyboard expectations, semantic HTML,
   contrast targets, motion policy, overflow behavior, and behavior when
   JavaScript is unavailable.
8. **Usage guidance** — concise examples of correct use, combinations to avoid,
   and the process for extending the system.

The file should distinguish stable rules from examples. It should name
implementation-facing values precisely enough that code review can identify
unintentional deviations, without turning `DESIGN.md` into a copy of the CSS.

## Artifact Ownership

The Stitch project may remain useful for future exploration, but Lens must not
depend on a mutable external project as its only design history.

- Retain representative images for all reviewed alternatives under
  `docs/design/alternatives/` if their export format and usage rights permit
  repository storage.
- Retain the selected representative images under `docs/design/`.
- Record the selection rationale in `DESIGN.md`; a Stitch link may supplement
  but must not replace that record.
- Do not commit generated application code merely because Stitch produced it.
  Production markup and styles must be adapted to Lens's behavior, security
  boundary, and repository conventions.
- Record the origin and applicable license of any generated or third-party
  visual asset before including it in the product.

## Implementation Approach

Selecting the system and implementing it are separate decisions. Accepting this
proposal authorizes the exploration and design record; production restyling
should begin only after the selected direction and `DESIGN.md` are reviewed.

Implementation should proceed in narrow slices:

1. Introduce the selected foundations and page shell.
2. Apply the system to document navigation and controls.
3. Apply content styles to Markdown, metadata, tables, code, and diagrams.
4. Complete status, failure, disabled, focus, overflow, and responsive states.
5. Remove obsolete visual rules only after equivalent states are verified.

The existing `/app.css` route and content security policy should remain stable.
If the stylesheet is materially expanded or reorganized, first make a
behavior-preserving extraction of the embedded stylesheet from `viewer.rs` into
a cohesive UI asset module or file. Keep that structural change separate from
the visual redesign so regressions can be attributed and reviewed clearly.

Reusable values should be expressed as CSS custom properties where that makes
their relationship visible in code. A token should exist because multiple
elements share a design decision, not merely to give every literal value a
name.

## Compatibility and Constraints

The design-system implementation must preserve:

- the loopback-only viewer and current content security policy;
- the authorized document set and existing route behavior;
- server-rendered document navigation, search, and pagination when JavaScript
  is unavailable;
- semantic control labels and expanded-state information;
- Markdown meaning and source-safety behavior;
- diagram source fallback, retry, and session-disable behavior;
- readable overflow for code, tables, long identifiers, and large diagrams;
  and
- operation without downloading runtime fonts or design-system packages.

No selected concept may require a client-side application framework merely to
reproduce its appearance.

## Feasibility and Effort Shape

The exploration is feasible without changing production code. Lens already
exposes the major interface states needed to build representative concepts.
The design effort is small to medium: one focused exploration and selection
iteration followed by a documentation pass.

Implementation effort depends on the selected direction. An evolutionary
direction may mostly replace stylesheet values and clarify markup classes. A
direction that substantially changes information architecture should return as
a separate behavior proposal rather than silently expanding this visual-system
work.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Generated concepts optimize a static screenshot but fail with real Markdown. | Use one demanding shared fixture and require all listed states in every direction. |
| Selection becomes a subjective palette preference. | Score each direction against the shared product and accessibility criteria and record the rationale. |
| The external Stitch project changes or becomes unavailable. | Keep the chosen rules and permitted representative artifacts in the repository. |
| A visually polished result weakens keyboard, no-JavaScript, or overflow behavior. | Treat existing behavior as a constraint and verify it in the real browser implementation. |
| `DESIGN.md` and CSS drift apart. | Require UI changes to update the design record when they introduce or revise a shared rule. |
| Restyling a large embedded stylesheet obscures regressions. | Perform any structural extraction mechanically and separately before applying visual changes in slices. |
| Generated assets introduce unclear rights or network dependencies. | Review asset provenance and keep runtime assets local and intentionally licensed. |

## Acceptance Criteria

This proposal is complete when:

- one Lens Stitch project contains at least three distinct directions based on
  the same brief, states, content, and viewport sizes;
- the alternatives are reviewed using the selection criteria above;
- the selected direction, adopted ideas, rejected alternatives, and unresolved
  implementation questions are recorded;
- a repository-level `DESIGN.md` documents the selected foundations, layouts,
  content styles, components, states, responsive behavior, accessibility rules,
  and extension process;
- `DESIGN.md` is sufficient to guide implementation without access to Stitch;
- permitted representative artifacts are retained in the repository, with
  their origin recorded; and
- a follow-up implementation plan divides the production work into verifiable
  slices without changing Lens's functional scope.

## Verification for a Later Implementation

Automated tests should continue to verify behavior rather than exact incidental
CSS strings. Add targeted checks only for durable design-system requirements
that can be evaluated mechanically, such as required asset routes, semantic
states, focus hooks, or the absence of external runtime resources.

### Manual end-to-end test

- **Setup:** Build Lens and prepare documents containing long prose, all
  heading levels, links, lists, block quotes, inline and block code, a wide
  table, valid and invalid YAML frontmatter, a valid diagram, and a diagram
  failure. Prepare enough documents to exercise navigation search and
  pagination.
- **Actions:** Open the fixture at narrow and wide viewports. Use the keyboard
  to search, paginate, hide and restore navigation, retry a diagram, disclose
  source, and disable rendering. Repeat document search and pagination with
  JavaScript disabled. Inspect wrapping and overflow with long identifiers and
  large content.
- **Expected result:** The interface matches the selected direction and
  `DESIGN.md`; authored content remains readable; controls and states are clear
  and visibly focused; no essential meaning depends only on color; narrow
  layouts do not lose controls or content; overflow remains usable; and all
  existing workflows behave as before.

## Out of Scope

- Implementing a new Lens feature or changing document behavior as part of
  visual exploration.
- Turning the Stitch project or generated code into a production dependency.
- Adding a general-purpose frontend framework.
- Creating a hosted multi-user Lens application.
- Reworking PlantUML rendering, document authorization, or automatic refresh.
- Requiring a logo, custom illustration set, dark theme, or externally hosted
  font unless one is proposed and evaluated separately.
