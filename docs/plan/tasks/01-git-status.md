# 01 — Git status model and inspection

**Status:** Complete / merged (verified together at `445a232`)

**Depends on:** None

**Owns:** `src/git.rs`, `src/lib.rs`

## Context

Read docs/design.md, docs/architecture.md and docs/conventions.md.
Baseline is recorded in docs/plan/PLAN.md; re-measure before implementation.

## Work and acceptance

Define RepoStatus and branch states from architecture.md. Implement direct Git
invocation with controlled child environment and porcelain-v2 NUL parsing.
Expose inspection by path for task 02. Add parser tests and local repository
fixtures inside git.rs; task 02 may reuse public status types.

- Count normal, rename, conflict and untracked records correctly, including
  names with whitespace/newlines; reject malformed records without panicking.
- Represent named, unborn and detached states and optional upstream/counts.
- Real temporary repositories confirm clean/dirty, rename and detached behaviour.
- A local bare remote fixture verifies ahead and behind counts without network.
- Inherited Git environment overrides cannot redirect inspection.
- Formatting, Clippy and tests pass. Scanner/CLI behaviour is not claimed yet.
