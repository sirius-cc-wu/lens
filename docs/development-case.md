# Development Case

Lens will use a lightweight, iterative Unified Process tailored to a small,
single-product CLI. Each iteration selects a narrow user outcome or top risk,
updates only the durable artifacts needed to support that decision, implements a
vertical slice where appropriate, and records evidence in an iteration record.

## Roles

| Role | Responsibility |
|---|---|
| Product owner | Confirms user workflows, scope, and release trade-offs. |
| Developer | Investigates risks, updates artifacts, implements slices, and maintains tests. |
| Reviewer | Checks behavior, security boundaries, and traceability from requirements to evidence. |

One person may hold multiple roles.

## Required Artifacts by Need

| Need | Artifact |
|---|---|
| Product scope and value | [Vision](vision.md) |
| User goals and scenario detail | [Feature/use-case artifact](features/markdown-viewing.md) |
| Architectural quality constraints | [Supplementary specification](supplementary-specification.md) |
| Shared vocabulary | [Glossary](glossary.md) |
| Uncertainty and mitigation | [Risk list](risk-list.md) |
| Iteration objective, trace, and results | One record under [iterations/](iterations/) |
| Significant accepted technical decision | A decision record when a decision is made and needs future context |

Use cases lead later analysis and design: significant scenarios receive a system
sequence diagram and operation contract before detailed collaboration or class

## Change and Quality Practices

- Keep current durable artifacts at their canonical paths; iteration records
  link to them rather than copying them.
- Treat documentation as evolving evidence, not as a phase gate.
- Add automated tests alongside implementation for resolved behavior and
  security-sensitive boundaries.
- Run the repository's formatter, tests, and linter before an iteration closes.
- Record a decision separately when its alternatives and consequences would be
  unclear from code alone.
