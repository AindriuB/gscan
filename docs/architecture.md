# Architecture

## Current state

One Rust binary crate, edition 2024, with no third-party dependencies.
The CLI, scanner, Git inspection and formatting modules implement the extended CLI contract.

## Components

| File | Responsibility |
|---|---|
| `src/main.rs` | Arguments, orchestration, stdout/stderr and exit codes |
| `src/lib.rs` | Public module exports for integration tests |
| `src/git.rs` | Git invocation, porcelain parsing, repository status types |
| `src/scan.rs` | Immediate-child discovery, sorting and partial-error collection |
| `src/output.rs` | Byte escaping, table rendering and JSON serialization |
| `src/process.rs` | Per-command deadlines, capture limits and child cleanup |
| `tests/cli.rs` | Executable-level tests using temporary Git repositories |
| `tests/validate_json.py` | Independent JSON schema/escaping checks |
| `.github/workflows/ci.yml` | Native Linux, macOS and Windows checks |

Data flow: arguments → discovery → optional serial fetch → bounded Git inspection
workers → sorted statuses/errors → table or JSON → exit status. Workers share an
atomic candidate index and return owned batches; no async runtime is needed.
Each Git subprocess uses the capture/deadline helper. Pass paths explicitly; do
not change the process working directory.

`RepoStatus` contains a `PathBuf`, a branch state (named, unborn or detached),
changed-path count, optional upstream name, and optional ahead/behind pair.
Keep Git names as bytes until rendering. `ScanReport` contains successful statuses
and path-associated failures so one broken repository does not hide other rows.
Separate parser errors from failed Git execution and attach path context at the
inspection boundary. Formatting functions return strings; callers own I/O.

## Repository topology

`/srv/dev/projects/gscan` is a single Git repository on `main`, tracking
`origin/main` in the private GitHub repository `AindriuB/gscan`. Clone with
`git clone git@github.com:AindriuB/gscan.git`. Sibling directories are scan inputs, not dependencies or
repositories that must be cloned to build gscan.

Cargo source, lockfile, README, agent guides and docs are versioned.
`target/`, `.claude/` and `.worktrees/` are local generated directories.
The shared Claude Kit lives at `~/.claude-kit`; reinstall project support with
`~/.claude-kit/install.sh /path/to/gscan`. Shared roles live under `~/.claude/`.

## Decisions — 2026-10-19

- Use Git CLI rather than libgit2: retain Git's own worktree and status semantics
  without adding a second Git implementation to a beginner project.
- Use porcelain v2 with NUL delimiters: filenames can contain whitespace and newlines.
- Scan one directory level: predictable scope and no accidental disk-wide traversal.
- Keep v1 read-only and offline: cached upstream comparisons are sufficient for an overview.
- Use only the standard library initially: a small interface does not need a framework.
- Add native Linux/macOS/Windows CI; claim verification only from observed runs.

The precise behavioural contract and known limits are in [design.md](design.md).

## Extended-scan decisions — 2026-10-19

- Keep one inspection worker by default, with explicit bounded parallelism.
- Stop recursive discovery at repository boundaries to avoid scanning build trees.
- Serialize fetches because worktrees can share refs; parallelize read-only inspection.
- Use temporary files rather than output pipes so deadline cleanup does not depend
  on reader threads reaching EOF after an inherited handle escapes.
- Keep JSON serialization dependency-free; independently parse outputs with Python.
- Treat packaging as a separate backlog task, not an implicit public release.
