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

## Now

Use v1 on the homelab. No additional implementation task is in progress.

## Next

Decide whether `--dirty` or JSON output would save real work after using the table.
Neither is committed for implementation.

## Someday

Recursive scanning, explicit fetch, concurrency, process deadlines and portability.
Each needs a separate scope decision and tests.
