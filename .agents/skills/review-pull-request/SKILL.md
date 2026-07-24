---
name: review-pull-request
description: Review repository changes across the full affected dependency cone and report every actionable correctness, compatibility, performance, security, maintainability, concurrency, and test-coverage finding in severity order. Use when the user asks to review a pull request, branch, commit, patch, diff, or uncommitted changes, including requests to run or perform `/review`.
---

# Review Pull Requests

Perform an evidence-based code review. Inspect all code and data paths that can
influence or be influenced by the change (the affected dependency cone), not
only the edited lines.

## Establish the Review Scope

1. Read the user's requested scope and all repository instructions that govern
   the affected files.
2. Use the environment's standard `/review` workflow when available; otherwise
   perform the equivalent review directly.
3. Determine the exact base and head for the review from the pull request,
   explicit commit range, branch merge base, or worktree state. State a
   consequential assumption about the range.
4. Inspect the complete diff and changed-file list before evaluating individual
   hunks. Do not modify code, post comments, or change pull request state during
   a review unless the user explicitly requests it.

## Trace the Affected Dependency Cone

For each changed behavior, inspect every relevant:

- definition, caller, and callee;
- public API and compatibility boundary;
- serialization, deserialization, persistence, and migration path;
- concurrency interaction, shared state, ordering assumption, and failure path;
- external or cross-package consumer;
- existing test, fixture, mock, and test helper.

Follow the behavior far enough to determine its actual preconditions,
postconditions, side effects, and externally observable results.

## Evaluate the Change

Review the complete affected dependency cone for:

- correctness and edge cases;
- race conditions and other concurrency defects;
- API and data compatibility;
- performance regressions;
- security issues;
- maintainability problems with a concrete impact;
- missing or inadequate tests.

Confirm each suspected issue against the repository before reporting it. Prefer
specific, reproducible defects over speculative concerns or style preferences.
Do not report pre-existing behavior as introduced by the change unless the
change makes it newly reachable or materially worse.

Use focused tests or static checks when they can validate a finding without
changing tracked files. A test passing does not replace tracing the behavior.
Never claim a check was run when it was not.

## Rank Findings

Rank every actionable finding in this order:

1. **Critical**: Enables catastrophic compromise, unrecoverable data loss, or
   widespread service failure.
2. **High**: Causes a major security, correctness, compatibility, or
   availability failure in realistic use.
3. **Medium**: Causes an incorrect result or meaningful regression under a
   plausible but limited condition.
4. **Low**: Has limited impact but is still a concrete defect or maintainability
   risk worth fixing in this change.

Within a severity, order findings by impact and likelihood. Do not inflate
severity because a category sounds serious.

## Report the Review

Lead with the findings. Number them continuously in severity order. Include all
of the following for every finding:

- severity;
- exact file path and line number;
- explanation of the defect and its impact;
- proposed fix;
- missing or recommended test coverage.

Use this compact shape:

```text
1. [High] Short finding title — path/to/file.ext:123
   Explanation and impact: ...
   Proposed fix: ...
   Test coverage: ...
```

If there are no actionable findings, say so explicitly and briefly identify
any residual risk or validation gap. Do not post findings as GitHub comments
unless the user explicitly requests comments.
