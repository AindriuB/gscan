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

## Extended features — complete

| ID | Task | Depends on | Status |
|---|---|---|---|
| 05 | [JSON, deadlines, recursion, workers, fetch and portability](tasks/05-extended-scanning.md) | 04 | Complete / merged |

Implementation `8169379` passes 30 Linux Rust tests, formatting, all-target Clippy
and independent Python JSON validation. Native CI passes on Linux, macOS and
Windows ([run](https://github.com/AindriuB/gscan/actions/runs/39233162807)).
Platform-specific fixtures account for different test counts. One ignored test
function is an explicitly launched subprocess fixture, not missing coverage.

## Packaging — complete

| ID | Task | Depends on | Status |
|---|---|---|---|
| 06 | [Packaging and releases](tasks/06-packaging.md) | 05 | Complete / merged |

Task 06 delivered MIT-licensed v0.1.0 archives, SHA256SUMS, installation docs
and tag-triggered private draft releases. Publishing a draft remains a separate
maintainer action.

## Now

Tasks 01–06 are complete. The private v0.1.0 draft and downloaded checksums
are verified; see [task 06](tasks/06-packaging.md) for evidence.
Publishing the draft remains a separate maintainer decision.

## Someday

Whole-scan deadlines, signal cancellation, fetch deduplication and colour.
Each needs a separate scope decision and tests.
