# gscan

Survey local Git repositories from the command line. See where unfinished work
and unpushed commits are waiting before ending a development session.

**Status:** v1 implemented and verified on Linux.

## Usage

```sh
gscan /srv/dev/projects
gscan                     # scan immediate children of the current directory
gscan --help
gscan --version
```

Example output (repository names and counts are illustrative):

```text
REPOSITORY     BRANCH          CHANGES  AHEAD  BEHIND
agent-gateway  feature/tools         3      2       0
data-prism     main                  0      0       0
jresolve      main                  1      -       -
```

CHANGES counts changed paths, including untracked files. A dash means no upstream
comparison is available. Counts use locally cached Git refs, never an automatic
fetch. A dirty repository is a normal result, not an error.

V1 inspects immediate child directories, including linked worktrees. It skips
symlinks, ordinary directories and bare repositories. It does not recurse or
modify repositories. See [the CLI contract](docs/design.md) for edge cases.

## Development

Requires stable Rust (edition 2024) and Git on PATH. Linux is the initial target.
The implementation uses the Rust standard library and installed Git, with
no third-party Rust dependencies in v1.

```sh
cargo run -- /srv/dev/projects
cargo test --offline
cargo fmt -- --check
cargo clippy --offline -- -D warnings
```

Use `cargo install --path . --offline` to install `gscan` locally.

Verification covers 20 unit and integration tests, including local upstreams,
linked worktrees, merge conflicts, submodules, unusual filenames and partial
failures. The first homelab smoke scan inspected 13 repositories successfully.

Counts are based on cached refs and may be stale until you fetch separately.
V1 runs Git sequentially without process timeouts and buffers each status response;
very large repositories can take time and memory. Linux is the tested platform.

## Project documents

- [Design and CLI contract](docs/design.md)
- [Architecture](docs/architecture.md)
- [Conventions](docs/conventions.md)
- [Workflow](docs/workflow.md)
- [Implementation plan](docs/plan/PLAN.md)

Claude Kit is installed locally in `.claude/` and is not versioned. To recreate
it from the existing shared kit, run `~/.claude-kit/install.sh /path/to/gscan`.
