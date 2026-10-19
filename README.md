# gscan

Survey local Git repositories from the command line. See where unfinished work
and unpushed commits are waiting before ending a development session.

**Status:** design and Cargo scaffold only. Scanning is not implemented yet.

## Planned usage

```sh
gscan /srv/dev/projects
gscan                     # scan immediate children of the current directory
gscan --help
gscan --version
```

Illustrative output, not a result from the current program:

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
The implementation will use the Rust standard library and installed Git, with
no third-party Rust dependencies in v1.

```sh
cargo run
cargo test --offline
cargo fmt -- --check
cargo clippy --offline -- -D warnings
```

Currently `cargo run` prints the Cargo starter greeting. Once implemented, use
`cargo run -- /srv/dev/projects`, or `cargo install --path .` to install locally.

## Project documents

- [Design and CLI contract](docs/design.md)
- [Architecture](docs/architecture.md)
- [Conventions](docs/conventions.md)
- [Workflow](docs/workflow.md)
- [Implementation plan](docs/plan/PLAN.md)

Claude Kit is installed locally in `.claude/` and is not versioned. To recreate
it from the existing shared kit, run `~/.claude-kit/install.sh /path/to/gscan`.
