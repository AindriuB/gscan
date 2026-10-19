# Plan

## First milestone — complete

V1 is implemented and merged into main. The CLI follows [design.md](../design.md)
with no third-party Rust dependencies, automatic fetch or repository mutations.

| ID | Task | Depends on | Status |
|---|---|---|---|
| 01 | [Git status model and inspection](tasks/01-git-status.md) | None | Complete / merged |
| 02 | [Discovery and table formatting](tasks/02-scan-output.md) | 01 | Complete / merged |
| 03 | [CLI integration and end-to-end checks](tasks/03-cli.md) | 02 | Complete / merged |

Verified implementation: `445a232`, measured 2026-10-19. All 20 tests pass;
rustfmt and Clippy (all targets, warnings denied) pass. A live read-only scan
successfully inspected 13 immediate child repositories in `/srv/dev/projects`.

## Follow-up — complete

| ID | Task | Depends on | Status |
|---|---|---|---|
| 04 | [Dirty repository filter](tasks/04-dirty.md) | 03 | Complete / merged |

Verified implementation: `69b7104`, measured 2026-10-19. All 23 tests,
formatting and all-target Clippy pass. The live filtered scan returned two
repositories with changed paths and excluded clean repositories.

## Now

Use `--dirty` for a quick unfinished-work check. No implementation task is in progress.

## Next

Consider JSON output if scripts need to consume scan results. It is not yet
committed for implementation.

## Someday

Recursive scanning, explicit fetch, concurrency, process deadlines and portability.
Each needs a separate scope decision and tests.
