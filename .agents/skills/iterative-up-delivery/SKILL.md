---
name: iterative-up-delivery
description: Carry a repository proposal or feature through risk-driven Unified Process (UP) analysis, design, construction, and transition; deliver thin iterations, make exactly one commit per completed iteration, then push and open a GitHub pull request. Use when the user asks to "iterative UP" a proposal, specification, feature, or repository-scoped work item; requests delivery one iteration at a time; requires one commit per iteration; or wants iterative work finished as a PR.
---

# Iterative UP Delivery

Coordinate analysis, design, implementation, verification, commit history, and
PR publication as one persistent workflow. Treat an iteration as a coherent,
risk-driven outcome—not as an arbitrary batch of edits.

## Compose Specialized Skills

Load and follow these skills when they are available:

- `iterative-up-analysis-design` for phase intent, artifact lifecycle, trace,
  and iteration records.
- The relevant use-case, contract, modeling, realization, and
  language-adaptation skills for artifacts selected by the current risk.
- `test-driven-implementation` for production behavior.
- `behavior-preserving-refactoring` for structural changes that should not
  alter behavior.
- `commit` for each iteration boundary.
- `create-pr` for final publication.

Use this skill to coordinate their boundaries. Follow repository governance
when it is stricter than this workflow.

## Preserve the Delivery Invariant

Maintain this mapping for commits created by the workflow:

```text
one completed UP iteration <-> one scoped commit
```

- Include an iteration's record, canonical artifacts, code, tests, and
  documentation in that iteration's single commit.
- Do not create separate planning, implementation, formatting, lint-fix, or
  documentation commits for the same iteration.
- Do not combine two iteration objectives in one commit.
- Do not push between iterations. Preserve unpublished local commits so an
  omission belonging to the most recent iteration can be repaired by amending
  that commit after revalidation.
- Never amend a published commit or hide a genuinely new objective in an
  existing commit. Create a new, named iteration when new risk or scope
  warrants one.

## 1. Establish Scope and Repository State

1. Read the work item completely, including linked acceptance criteria, risks,
   design artifacts, and verification expectations.
2. Read `AGENTS.md`, repository indexes, development guidance, neighboring
   iteration records, contribution rules, and commit or PR conventions.
3. Inspect the worktree, current branch, worktrees, remotes, default branch,
   recent log, and existing PRs before editing.
4. Preserve unrelated changes. Stop when overlapping or uncommitted work makes
   clean iteration commits unsafe.
5. Create or select a descriptive feature branch without force-switching a
   branch owned by another worktree. If the intended branch is active in
   another worktree, inspect that worktree and coordinate with its owner rather
   than editing or publishing from a stale checkout. If the owner cannot be
   identified, stop and report the branch, worktree path, and status to the
   user for direction.
6. Record the starting commit. Use it later to audit only the commits created
   by this delivery workflow, while reporting any pre-existing branch commits
   separately.
7. Treat an explicit delivery request as authorization to work within the
   proposal's stated scope. Surface a material scope expansion instead of
   silently accepting it.

## 2. Shape a Risk-Driven Iteration Sequence

1. Identify the current UP phase intent and the highest unresolved risk or
   highest-value executable behavior.
2. Propose the smallest coherent sequence that can satisfy the work item.
   Refine the later sequence as evidence arrives; do not force every UP phase
   or artifact into the plan.
3. Prefer thin outcomes such as:
   - resolve a security-sensitive operation and its architecture decision;
   - implement one end-to-end behavior slice with focused executable evidence;
   - integrate remaining acceptance behavior and close user documentation.
4. Give each iteration one objective, explicit exit criteria, and a repository-
   conventional ID. Inspect phase plans, existing record filenames, and record
   frontmatter to select the next unclaimed ID. Never reuse an ID. If the
   repository has no clear allocation convention, use a unique descriptive
   slug and avoid inventing a numeric series.
5. Create one historical iteration record per iteration and link canonical,
   evolving artifacts rather than copying them into the record.
6. Keep a live task plan whose items correspond to planned iterations and
   final PR publication. Do not make a planning-only commit.

## 3. Execute One Iteration

Repeat this section until the proposal's acceptance criteria are satisfied.

### Plan the iteration

- Mark the iteration record active.
- State the objective, selected risks, artifacts to start or refine, trace,
  verification evidence, and exit criteria.
- Choose only artifacts that reduce current uncertainty or enable the behavior
  slice.

### Analyze and design

- Drive design from actor goals, scenarios, system events, operation effects,
  and architectural risks.
- Keep each durable artifact at one canonical path and update trace links.
- Record durable cross-cutting decisions separately when required.
- Validate materially changed PlantUML through the repository's configured
  server. If unavailable, record the skipped validation accurately.

### Implement and verify

- Work test-first when executable behavior changes.
- Follow repository test naming and setup/action/verification conventions.
- Run focused checks while developing, then all checks required to close this
  iteration.
- Keep refactoring inside the iteration only when it directly enables the
  objective; preserve unrelated cleanup for later work.

### Close the iteration

- Re-read the objective and exit criteria against actual evidence.
- Complete the iteration record's results and artifact outcomes without
  duplicating canonical artifacts.
- Update affected decisions, risks, requirements, indexes, user guidance, and
  proposal status when the evidence changes them.
- Record residual risk and the next objective. Do not claim manual or external
  validation that was not performed.

### Commit the iteration

1. Inspect status, unstaged diff, staged diff, and recent commit conventions.
2. Ensure every changed file belongs to this iteration and that no expected
   artifact or evidence is missing.
3. Run the iteration's closing validation before staging.
4. Stage only the iteration files and review the staged diff.
5. Create one convention-aware commit describing the iteration outcome.
6. Verify the commit, the worktree, and the live task plan before starting the
   next iteration.

## 4. Run the Delivery Gate

After the last planned behavior slice:

1. Check every proposal acceptance criterion and every iteration exit
   criterion against code, artifacts, and evidence.
2. Run the repository's full required formatter, tests, linter, browser checks,
   documentation checks, and diagram validation as applicable.
3. Perform proposal-required manual acceptance checks. A skipped required check
   blocks PR publication unless the user explicitly approves the exception;
   record any approved exception and its impact in the PR. Validation that
   repository governance expressly allows to be skipped when an external
   service is unavailable may remain skipped if reported accurately.
4. Confirm all iteration records are completed, canonical artifacts agree with
   implementation, and user-facing status is current.
5. Review the complete branch diff for security, compatibility, accidental
   files, unrelated changes, and missing documentation.
6. If the gate reveals work, place it in a named final integration iteration
   with its own record, evidence, and single commit. Do not add an anonymous
   cleanup commit.
7. Require a clean worktree before publication.

## 5. Audit Iterations Against Commits

Review commits from the recorded starting commit through `HEAD` in chronological
order. Build a mapping containing:

- iteration ID and objective;
- commit SHA and subject;
- iteration record path;
- principal validation evidence.

Confirm every workflow-created commit maps to exactly one completed iteration
and every completed iteration maps to exactly one commit. Resolve any mismatch
before pushing.

## 6. Publish the Pull Request

1. Resolve the intended base and remote; fetch and inspect the complete
   base-to-head commit list and diff.
2. Check upstream state and existing open PRs. Report an existing PR instead of
   creating a duplicate unless the user explicitly requests another.
3. Push the completed branch once. Stop on behind or diverged state rather than
   rewriting shared history without authorization.
4. Prepare a PR title from repository conventions and the delivered outcome.
5. Include in the PR body:
   - the proposal goal and delivered behavior;
   - the iteration-to-commit mapping;
   - design and compatibility decisions;
   - automated, manual, diagram, and skipped validation;
   - residual risks or intentionally deferred scope.
6. Open a draft PR by default unless the user requests ready-for-review or
   repository policy clearly says otherwise.
7. Finish only after returning the PR URL, base and head branches, included
   iterations and commits, validation result, and draft or ready status.

## Stop Conditions

Stop and request direction when:

- the next change materially exceeds the authorized proposal;
- unresolved overlapping work prevents isolated iteration commits;
- a required external decision changes the architecture or user-visible scope;
- publication would require rewriting shared history; or
- required validation fails and cannot be resolved within the authorized work.

Do not treat ordinary implementation difficulty, newly discovered in-scope
work, or a long-running check as a reason to abandon the workflow.
