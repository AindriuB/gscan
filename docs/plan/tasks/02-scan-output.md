# 02 — Discovery and table formatting

**Status:** Not started

**Depends on:** 01

**Owns:** `src/scan.rs`, `src/output.rs`, `src/lib.rs`

## Context

Read docs/design.md, docs/architecture.md and docs/conventions.md.
Baseline is recorded in docs/plan/PLAN.md; re-measure before implementation.

## Work and acceptance

Implement immediate-child discovery and sorted inspection results using task 01.
Render successful statuses as the specified ASCII table. Keep errors separate
from successful rows for the CLI to report. These modules do not print directly.

- Skip ordinary directories, symlinked children and bare repos; include linked worktrees.
- Empty roots succeed; invalid roots fail; broken candidate metadata yields a
  partial report while valid siblings still appear.
- Sorting is deterministic and changed counts align with their headings.
- Escape backslashes, control bytes and non-ASCII bytes in names and diagnostics.
- Test unborn/detached labels and unavailable comparison dashes.
- Tests cover mixed results, empty roots, escaping and ordering.
- Formatting, Clippy and tests pass.
