# Plan

## Now — first usable local scan

Design is documented; implementation has not started. Deliver the CLI contract
in [design.md](../design.md) with no third-party Rust dependencies or network calls.

| ID | Task | Depends on | Status |
|---|---|---|---|
| 01 | [Git status model and inspection](tasks/01-git-status.md) | None | Not started |
| 02 | [Discovery and table formatting](tasks/02-scan-output.md) | 01 | Not started |
| 03 | [CLI integration and end-to-end checks](tasks/03-cli.md) | 02 | Not started |

Baseline: `b7354bd`, measured 2026-10-19. Formatting, Clippy and Cargo tests pass;
there are zero tests and main prints a greeting. Re-measure after source changes.

## Next

Use v1 on the homelab and decide whether `--dirty` or JSON would save real work.
No feature beyond the first scan is committed for implementation.

## Someday

Recursive scanning, explicit fetch, concurrency and portable terminal presentation.
Each needs a separate scope decision and tests.
