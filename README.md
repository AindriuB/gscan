# gscan

Survey local Git repositories from the command line. See where unfinished work
and unpushed commits are waiting before ending a development session.

## Usage

```sh
gscan /path/to/projects
gscan --dirty /path/to/projects
gscan --json --recursive --jobs 4 /path/to/projects
gscan --fetch --timeout 60 /path/to/projects
gscan --help
```

Without a directory, scan children of the current directory. Defaults are
read-only, one directory level, one worker, and a 30-second timeout per Git command.

| Option | Behaviour |
|---|---|
| `--dirty` | Show only repositories with changed paths; retain all errors |
| `--json` | Emit schema-versioned JSON instead of a table |
| `--recursive` | Descend ordinary directories, stopping at repositories |
| `--jobs N` | Inspect with 1–64 workers; default 1, output always sorted |
| `--timeout N` | Allow 1–3600 seconds per Git command; default 30 |
| `--fetch` | Explicitly refresh configured remotes before inspecting |

Numeric options also accept `--jobs=4` and `--timeout=60`. Options may precede or
follow the directory. Use `--` before a directory whose name begins with `-`.

Example output (names and counts are illustrative):

```text
REPOSITORY     BRANCH          CHANGES  AHEAD  BEHIND
example-api    feature/tools         3      2       0
example-web   main                  0      0       0
example-cli   main                  1      -       -
```

CHANGES counts changed paths, including untracked files and conflicts. A dash
means no upstream comparison is available. Dirty repositories are normal results,
not errors. `--dirty` excludes clean repositories with unpushed commits.

Recursive scans show paths relative to the scan root, skip symlinked children and
bare stores, and stop at repository boundaries. They do not descend into submodules
or nested repositories inside an already discovered repository.

`--fetch` contacts configured remotes and writes Git objects/tracking refs according
to their configured refspecs. Fetches are sequential, with no pruning, tags,
submodule recursion or automatic maintenance. A failed fetch is reported even if
cached status can still be inspected. Default scans never fetch.

## JSON and scripts

`--json` emits one object with `schema_version`, `text_encoding`, `repositories`
and `errors`, including empty or partially failed scans. Branch state is explicit;
missing comparisons use `null`. Exit 0 means a successful scan, 1 a scan/fetch/output
failure, and 2 invalid usage. JSON scan errors are in the object rather than duplicated
on stderr; usage and output errors still use stderr.

Text fields use reversible ASCII byte escapes (`\\` for a backslash, `\xNN` for
control/non-ASCII bytes), then normal JSON quoting. See [the schema and full CLI
contract](docs/design.md) before consuming paths in a script.

## Development and verification

Requires stable Rust (edition 2024) and Git on PATH. There are no third-party Rust
dependencies. Native CI runs on Linux, macOS and Windows; platform-specific
filesystem tests run where their fixtures are supported. Python 3.11+ is required
for JSON validation and packaging checks.

```sh
cargo run -- --dirty /path/to/projects
cargo test --offline
cargo fmt -- --check
cargo clippy --offline --all-targets -- -D warnings
cargo build --offline
python tests/validate_json.py target/debug/gscan
```

On Windows the executable path ends in `.exe`. Install from source with
`cargo install --path . --offline`, or use the verified release archives described
in the [installation guide](docs/installation.md). Archives cover Linux x86_64
(glibc 2.35+), macOS 14+ Apple Silicon and Windows x86_64; Rust is not needed to
run them. Git is still required. Release automation creates drafts for
review; it does not publish them automatically.

MIT-licensed; see [LICENSE](LICENSE).
For build scripts, checksum verification and the tag workflow, see
[packaging maintenance](docs/packaging.md).

Timeout cleanup uses `/bin/kill` for Unix process groups and `taskkill` for Windows
process trees, then kills/reaps the direct child. These helpers must be available
for descendant cleanup. Detached helpers may escape cleanup. Capture files are
temporary; output above 32 MiB per stream is rejected. Filesystem discovery and OS
cleanup are outside the per-command timeout; this is not a whole-scan deadline.

## Project documents

- [Design and CLI contract](docs/design.md)
- [Architecture](docs/architecture.md)
- [Conventions](docs/conventions.md)
- [Workflow](docs/workflow.md)
- [Implementation plan and backlog](docs/plan/PLAN.md)
- [Installation](docs/installation.md) and [release maintenance](docs/packaging.md)
