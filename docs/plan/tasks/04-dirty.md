# 04 — Filter repositories with unfinished changes

**Status:** Complete / merged (verified at `69b7104`)

**Depends on:** 03 (merged)

**Owns:** `src/main.rs`, `tests/cli.rs`, `README.md`, `docs/design.md`,
`docs/plan/PLAN.md`, `docs/plan/HISTORY.md`, `docs/plan/HISTORY-INDEX.md`, this task.

## Contract

Accept `--dirty` before or after DIRECTORY, before `--`. Repeating it is harmless.
Only display successfully inspected repositories with CHANGES greater than zero.
Ahead/behind counts alone do not make a repository dirty. Inspect every candidate
so filtering never hides Git failures. Preserve exit codes and stderr diagnostics.

An empty root still prints `No repositories found.`. An error-free scan containing
repositories but no changed paths prints `No dirty repositories found.`. When a
scan has errors and no matching rows, print no success/empty message on stdout.
`-- --dirty` refers to a directory named `--dirty`. Help/version stay standalone.
No changes to Git inspection, dependencies or output table format are needed.

## Acceptance

- Mixed clean/dirty scans show only dirty rows, including untracked/unborn repos.
- A clean repository with unpushed commits is excluded.
- Dirty rows retain cached comparison counts and the existing table columns.
- Empty roots, clean-only roots and failures have distinct correct output.
- Flags work in both positions, repeat safely, and obey the `--` separator.
- Errors remain visible and return exit 1 even if all successful rows are filtered.
- Formatting, all-target Clippy and the full test suite pass.

Baseline: `97ffbff` contains the previously verified 20-test implementation.
