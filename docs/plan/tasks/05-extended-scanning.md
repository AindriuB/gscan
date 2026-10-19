# 05 — Extended scanning and portability

**Status:** Complete / merged (implementation `8169379`)

**Depends on:** 04

**Owns:** `src/**`, `tests/**`, `.github/workflows/**`, `README.md`,
`AGENTS.md`, `CLAUDE.md`, `docs/**`. No packaging implementation in this task.

## Contract

- `--json`: schema-versioned object with repositories and errors, including empty
  results and fatal root failures. Paths and Git names use the documented reversible
  ASCII byte representation, rather than lossy Unicode. Counts use JSON numbers/null.
- `--recursive`: descend ordinary child directories, stop at repository boundaries;
  exclude symlinks, `.git` and bare repository stores. Root itself stays excluded.
  Show relative paths in tables so equally named nested repositories are distinct.
- `--jobs N`: 1–64 bounded inspection workers, default 1; deterministic sorted output.
- `--timeout SECONDS`: positive integer 1–3600, default 30, per Git subprocess.
  Capture output without pipe deadlocks; terminate timed-out children and report
  failures without losing other repositories. Document process-tree limitations.
- `--fetch`: explicit sequential refresh of configured remotes before inspections,
  no tags/pruning/submodule recursion/automatic maintenance. A failed fetch is an
  error but successful inspection may still report cached status. No external
  fetch is needed to test this feature; use temporary local remotes.
- Portability: remove unconditional Unix test dependencies, add Linux/macOS/Windows
  CI, and state which platforms have actually been verified.
- Packaging: create a separate backlog contract for release archives, checksums,
  installation instructions, version/license decisions and release automation.

## Verification

Extend real-repository integration tests for JSON, recursion, worker ordering,
local-remote fetch changes/failures and argument bounds. Use controlled subprocess
fixtures for deadlines and large output. Run formatting, all-target Clippy and
all tests. Validate JSON with an independent parser. Run the platform CI matrix
when available, and never claim unobserved platform results.

Baseline `01030ff`: 23 tests passed before this task; no external Rust dependencies.

## Verification result — 2026-10-19

Linux: 30 Rust tests passed, plus one ignored subprocess fixture invoked by the
deadline test. Formatting and all-target Clippy passed. Native GitHub CI passed
on Linux, macOS and Windows, including independent Python JSON validation.
A parallel JSON scan of 13 local repositories returned no errors. Fetch checks
used local temporary remotes; no external remote was fetched for verification.
