# History

Completed work only, newest first. Consult HISTORY-INDEX.md before reading entries.

## 2026-10-19 — Debian package implementation and verification

Task 07 implements an amd64 `.deb` for v0.1.1, installing to `/usr/bin/gscan`
with Git and detected runtime dependencies. The release gate checks package
metadata, root ownership, permissions and exact Linux archive payload parity.
SHA256SUMS and draft creation include the `.deb` alongside existing archives.
The implementation is verified and submitted for merge; v0.1.1 is not released.

**Validation:** `18b3bdb` passes 30 local Rust tests, formatting, Clippy and 19
Python tests. Native package workflow 39241887388 passed Linux, macOS, Windows
and combined checksum verification. Disposable Ubuntu 22.04 (CI) and Ubuntu 24.04
(local) tests passed dependency installation, upgrade, reinstall, scanning without
Rust, and removal while preserving unrelated files and Git.

**Cost:** Debian packaging requires dpkg build tools and native Linux verification.
Local binaries can require newer libc than the official Ubuntu 22.04 build.
Downloads are individual packages; no APT update repository is configured.
Existing user-installed copies may take precedence on PATH and are not removed.

## 2026-10-19 — Portable documentation and release preparation

Replaced machine-specific paths and repository examples with generic examples,
removed shared-workspace setup instructions and account-specific links, and
made release documentation usable from any clone. Draft creation supports public
and private repositories while retaining tag checks and the manual publishing gate.
Repository visibility and existing release tags/assets were not changed.

**Validation:** all 14 Python release and packaging tests pass, including public
repository draft creation and refusal to modify published releases. Tracked-file
reference and Markdown link checks pass. Git history is retained unchanged.

## 2026-10-19 — Verified packages and private draft release

Task 06 delivers MIT-licensed v0.1.0 archives for Linux x86_64 GNU, macOS Apple
Silicon and Windows x86_64 MSVC, with installation guidance, source metadata
and SHA256SUMS. A tag-triggered workflow creates private drafts only; crates.io
publishing remains disabled. Source `3145975` is tagged `v0.1.0`.

**Validation:** the tag workflow
passes native Rust checks, 13 Python packaging tests, extracted-binary smoke
checks without Rust on PATH, and aggregate checksum verification. Linux passes
30 Rust tests. The private draft contains exactly the three archives and
SHA256SUMS. Downloaded hashes, contents and source commits all match; the
downloaded Linux binary also passes version and Git fixture scanning.

**Cost:** Windows checkout line endings require normalized archive documents.
Older procps on the Ubuntu 22.04 baseline requires an explicit `--` before the
negative process-group ID during timeout cleanup. Packages are unsigned, require
Git at runtime, and support only the documented architectures and OS baselines.
The release remains a draft pending a separate publication decision.

## 2026-10-19 — Extended scanning and native platform verification

Task 05 adds JSON output, recursive discovery, bounded inspection workers,
per-command deadlines and explicit sequential fetching. Results remain sorted
and default scans remain read-only. Task 06 records packaging as backlog only.

**Validation:** implementation `8169379` passes 30 Linux tests, formatting and
all-target Clippy. An independent Python parser verifies JSON schema, escaping,
nullable counts and partial failures. Native Linux, macOS and Windows CI passes
on the recorded implementation.
A live parallel JSON scan reported 13 repositories and no errors. Fetch tests
used temporary local remotes and verified worktree/index/local branch preservation.

**Cost:** macOS rejects invalid-byte filename fixtures and does not provide
`/dev/full`; those tests are Linux-only, with Unicode paths checked on all platforms.
Windows capture handles must be released even when process spawning fails.
Deadlines use capture files to avoid pipe EOF hangs; descendant cleanup requires
OS helpers and is best effort for detached processes. Fetch writes are serial
because linked worktrees may share refs.

## 2026-10-19 — Dirty repository filter

Task 04 adds `--dirty` to show only repositories with changed paths. Clean
repositories with unpushed commits are excluded. Filtering preserves inspection
errors and exit codes, and distinguishes empty roots from clean-only scans.

**Validation:** implementation `69b7104` passes 23 tests, formatting and all-target
Clippy with warnings denied. Three new integration tests failed before the feature
was implemented and now pass. The live local scan returned two dirty repositories.

**Cost:** filtering must happen after inspection, so a hidden clean row cannot hide
a failure elsewhere. This option reduces output, not scan work or runtime.

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
unchanged by scanning. A live scan of a local projects directory returned 13 repositories.

**Cost:** Git's rename records consume an extra NUL-delimited path; treating each
record as one file would overcount. Keep byte parsing separate from ASCII output.
Sequential buffered Git processes have no timeout in v1; very large or slow
repositories remain a documented limitation.

## 2026-10-19 — Scaffold and first-version design

Created the Rust crate in commit `b7354bd`. Added contributor guidance.
Documented the first-version CLI contract, architecture,
conventions and three sequential implementation tasks. The executable remains
the starter greeting; repository scanning has not been implemented.

**Validation:** baseline `b7354bd` passes Cargo tests (zero tests), rustfmt check
and Clippy with warnings denied. Documentation links and guide lengths are
checked separately when this documentation is written.

**Cost:** kept build and repository-topology guidance scoped to this single crate.
