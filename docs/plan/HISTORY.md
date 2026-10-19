# History

Completed work only, newest first. Consult HISTORY-INDEX.md before reading entries.

## 2026-10-19 — First usable repository scan

Tasks 01–03 are merged at `445a232`. The CLI scans immediate child repositories,
prints changed-path and cached upstream counts, handles detached/unborn branches,
and preserves successful rows when another repository fails. Filenames and errors
are escaped before terminal output. Scanning does not fetch or request Git index
refresh writes.

**Validation:** 20 unit/integration tests pass; rustfmt, all-target Clippy with
warnings denied, and diff checks pass. Tests include conflicts, submodules,
linked worktrees, malformed status records, non-UTF-8 paths, inherited Git
repository overrides and output errors. A fixture's refs, index and config were
unchanged by scanning. A live scan of `/srv/dev/projects` returned 13 repositories.

**Cost:** Git's rename records consume an extra NUL-delimited path; treating each
record as one file would overcount. Keep byte parsing separate from ASCII output.
Sequential buffered Git processes have no timeout in v1; very large or slow
repositories remain a documented limitation.

## 2026-10-19 — Scaffold and first-version design

Created the Rust crate in commit `b7354bd`. Installed local Claude Kit support and
added agent guidance. Documented the first-version CLI contract, architecture,
conventions and three sequential implementation tasks. The executable remains
the starter greeting; repository scanning has not been implemented.

**Validation:** baseline `b7354bd` passes Cargo tests (zero tests), rustfmt check
and Clippy with warnings denied. Documentation links and guide lengths are
checked separately when this documentation is written.

**Cost:** the kit templates described a multi-repository workspace. Adapted them
to this single crate to avoid incorrect build and repository-topology guidance.
