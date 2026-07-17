# Repository Agent Guidance

## Module Boundaries

- Prefer cohesive modules over size targets. A source file should have one primary reason to change.
- Before adding another responsibility to a file over roughly 500 non-test lines, evaluate whether it should be split.
- Split when at least two of these signals are present:
  - The file contains concerns that can change independently.
  - A concern has its own types, helpers, dependencies, and tests.
  - Imports are used by only one substantial section of the file.
  - A crate root or module entry point mixes public API declarations with substantial implementation.
  - Understanding or testing one concern requires navigating unrelated sections.
- Split along domain or capability boundaries, not into similarly sized chunks.
- Keep tightly coupled types and helpers together. Place tests with the module that owns the behavior.
- Make structural extraction mechanical: preserve behavior and public paths, using re-exports where necessary.
- Do not combine a file split with unrelated renaming, redesign, or abstraction changes.
- Do not create tiny pass-through modules or one-file-per-type structures solely to reduce line count.
- Once multiple implementation modules exist, keep `lib.rs` and `mod.rs` focused on module declarations and intentional public re-exports.
- After a Rust module split, run `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --locked --all-targets --all-features -- -D warnings`.

## Technical Documentation

- Introduce project-specific concepts and specialized technical or process jargon in plain language on first use, followed by the canonical term in parentheses.
- Name test scenarios for the behavior being verified, not jargon such as "happy path," "gating," or "soak."
- Prefer the concrete resource or operation over generic phrases such as "resource acquisition," while retaining exact identifiers needed to match code and logs.

## Behavior-Oriented Tests

- Name tests `<condition_or_action>_then_<observable_result>`, such as `missing_shutdown_ack_then_times_out_and_marks_rescan_failed`.
- Structure tests as setup, one primary action, and verification (Arrange-Act-Assert, or 3A), labeling each phase with `// Arrange`, `// Act`, and `// Assert` or the equivalent comment syntax.
- Verify one coherent behavior per test. Do not force 3A when concurrency or event-driven behavior requires actions and assertions to alternate.
