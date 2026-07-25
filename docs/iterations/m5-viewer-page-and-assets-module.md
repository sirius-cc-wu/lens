# Iteration: M5 Viewer Page and Assets Module

Status: completed

Phase Intent:

- Separate server-rendered page composition and browser assets from transport
  routing while preserving exact browser-visible output.

Goal:

- Move page, navigation, renderer-control, guidance, and security-policy markup
  into `viewer::page`, and store JavaScript and CSS as dedicated files embedded
  into the binary at compile time.

Risks Addressed:

- `R-02`: moving markup could change escaping or the restrictive content
  security policy.
- `R-03`: navigation composition could include an unknown document or change
  the catalog query/page links.
- Asset extraction could introduce runtime file access, change HTTP body bytes,
  or break refresh, renderer controls, and navigation-pane persistence.

Artifacts to Start:

- This M5 iteration record: captures presentation and asset invariants.
- Compile-time assets:
  [`src/viewer/assets/app.js`](../../src/viewer/assets/app.js) and
  [`src/viewer/assets/app.css`](../../src/viewer/assets/app.css)

Artifacts to Refine:

- Page composition and owned tests:
  [`src/viewer/page.rs`](../../src/viewer/page.rs)
- Viewer response coordination:
  [`src/viewer/routes.rs`](../../src/viewer/routes.rs) - delegate markup composition and serve embedded assets.

Artifacts Consulted:

- `ADR-006` and `ADR-016`, navigation presentation and tab-local visibility:
  [`docs/decisions/adr-006-document-navigation-pane.md`](../decisions/adr-006-document-navigation-pane.md) and
  [`docs/decisions/adr-016-collapsible-document-navigation-pane.md`](../decisions/adr-016-collapsible-document-navigation-pane.md)
- `DCD-01`, target page-module ownership:
  [`docs/features/markdown-viewing/uml-design.md`](../features/markdown-viewing/uml-design.md)

Decisions to Record:

- Keep presentation as stateless functions over catalog results and session
  status; do not make the page module own or mutate `ViewerState`.
- Embed dedicated asset files with `include_str!`. Remove only the repository
  files' final newline when serving so the previous raw-string HTTP bodies stay
  byte-for-byte unchanged.

Trace:

- Proposal 13 -> `DCD-01` page module -> navigation/security decisions -> page
  unit tests -> all presentation, refresh, and renderer browser scenarios

Exit Criteria:

- All page, navigation, renderer-control, guidance, and security-policy output
  retains its previous escaping, links, attributes, and text.
- JavaScript and CSS contents match the previous embedded raw strings and are
  compiled into the binary without runtime filesystem reads.
- Presentation tests reside with `viewer::page`, and all required checks pass.

Results:

- Moved page-shell, navigation, pagination, renderer-control, deferred-guidance,
  and content-security-policy composition into `src/viewer/page.rs`.
- Moved the exact JavaScript and CSS bodies into dedicated asset files and load
  them with `include_str!`; shell `cmp` checks against the previous committed
  raw strings passed for both assets.
- Moved six presentation tests to the page module and changed their setup to
  supply catalog results directly, keeping the page module independent of
  session ownership.
- Focused verification passed: `cargo test --locked viewer::page::tests` (six
  tests).
- Full verification passed: `cargo fmt --check`, `cargo test --locked` (53
  library tests and three CLI tests), `cargo clippy --locked --all-targets
  --all-features -- -D warnings`, and all 14 `npm run test:browser` scenarios.

Artifact Outcomes:

- started: dedicated JavaScript and CSS assets - compile-time inputs with no
  runtime deployment dependency.
- started: `viewer::page` - owns stateless page composition and presentation
  tests.
- refined: viewer response coordination - passes catalog and session values to
  page functions while retaining all response headers and routes.
