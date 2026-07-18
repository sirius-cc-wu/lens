# Iteration: P10 YAML Frontmatter Rendering

Status: completed

Phase Intent:

- Turn a leading metadata convention into safe, readable document content.

Goal:

- Show valid YAML frontmatter as structured metadata and give authors a clear
  correction path without losing readable Markdown when it is invalid.

Risks Addressed:

- `R-02`: user-controlled metadata must not become executable document markup.

Artifacts to Start:

- `ADR-014`, YAML frontmatter rendering:
  [`docs/decisions/adr-014-yaml-frontmatter-rendering.md`](../decisions/adr-014-yaml-frontmatter-rendering.md) - defines the leading-delimiter, value-structure, and error boundaries.

Artifacts to Refine:

- Supplementary specification, README, proposal list, risk list, and
  documentation index:
  [`docs/supplementary-specification.md`](../supplementary-specification.md),
  [`README.md`](../../README.md), [`docs/improvement-proposals.md`](../improvement-proposals.md),
  [`docs/risk-list.md`](../risk-list.md), and [`docs/index.md`](../index.md) - state the user-visible metadata and escaping behavior.
- Browser fixture suite:
  [`tests/browser/lens.spec.mjs`](../../tests/browser/lens.spec.mjs) - exercise valid nested metadata and malformed YAML from the compiled command.

Artifacts Consulted:

- Markdown renderer and escaping boundary:
  [`src/markdown.rs`](../../src/markdown.rs)
- Browser security constraints:
  [`docs/supplementary-specification.md`](../supplementary-specification.md)

Decisions Recorded:

- `ADR-014`: accept only a leading, delimiter-bounded YAML mapping and render
  every value through the existing HTML-escaping boundary.

Trace:

- Proposal 10 -> `ADR-014` -> Markdown renderer -> unit and browser checks

Test-Driven Evidence:

- Oracle: proposal 10 requires valid metadata, both delimiters, nested unknown
  fields, and an actionable invalid-YAML result.
- Discrimination: the new metadata and alert browser scenarios failed before
  the renderer change because neither element existed; the focused unit test
  also failed because a leading header produced no metadata section.
- Green evidence: focused Rust renderer tests and the frontmatter browser
  scenarios pass after the implementation.

Exit Criteria:

- Valid leading YAML metadata appears before the Markdown body without raw
  delimiters.
- `---` and `...` can close the header.
- Nested and unknown values remain visible but escaped.
- Malformed YAML exposes a correction message while the body stays readable.
- Formatter, tests, Clippy, package verification, and browser tests pass.

Results:

- Added a structured YAML-value renderer backed by `serde_yaml`; it preserves
  arbitrary mapping, sequence, and scalar values while escaping strings.
- Added an alert for malformed, non-mapping, and unclosed headers, retaining
  readable document content in each case.
- Added unit and end-to-end browser coverage for nested metadata, the alternate
  delimiter, malformed YAML, and HTML escaping.

Artifact Outcomes:

- started: `ADR-014`, frontmatter rendering - records the safe metadata
  contract.
- refined: user and quality documentation, risk mitigation, and browser
  fixture evidence - make the rendering behavior reviewable and executable.
