# 03 — CLI integration and end-to-end checks

**Status:** Not started

**Depends on:** 02

**Owns:** `src/main.rs`, `tests/cli.rs`

## Context

Read docs/design.md, docs/architecture.md and docs/conventions.md.
Baseline is recorded in docs/plan/PLAN.md; re-measure before implementation.

## Work and acceptance

Wire argument parsing, scanning, output and exit codes. Exercise the built
binary against disposable local repositories. Documentation close-out is a
separate scribe step after verification; do not claim it in the code task.

- No argument scans current-directory children; a single path scans that root.
- Help/version succeed, `--` accepts hyphen-prefixed paths, invalid usage exits 2.
- Clean and dirty scans exit 0; root/Git/per-repository errors exit 1.
- Successful stdout rows survive another candidate's failure; errors use stderr.
- Empty-root output follows design.md; failed candidates are not called empty.
- Test detached/unborn states, unavailable upstream, linked worktrees, odd paths
  and hostile inherited Git environment variables through the executable.
- Inspect fixture refs, index and configuration before/after a scan to detect
  accidental writes. Use no network or real project mutations.
- Formatting, Clippy and the full test suite pass.
