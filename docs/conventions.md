# Conventions

## Rust

Use stable Rust, edition 2024, and rustfmt defaults. Files/modules/functions use
snake_case; types use PascalCase. Use small explicit types for branch states and
scan results. Keep paths as `Path`/`PathBuf` and OS arguments as `OsString`.
Do not use lossy conversion for filesystem access or line splitting for NUL data.

Production code must not panic on user input, Git failures or filesystem races.
Return errors with operation and path context. `unwrap`/`expect` are acceptable
in test fixture setup with useful messages. Avoid unsafe code and additional
Rust dependencies in the first milestone.

Invoke Git using `std::process::Command` argument arrays. Never construct shell
commands from a repository name. Keep parsing, filesystem discovery, formatting
and terminal I/O separate. Document non-obvious protocol handling briefly.

## Tests and verification

Run `cargo fmt -- --check`, `cargo clippy --offline -- -D warnings`, and
`cargo test --offline`. Add behaviour tests for meaningful changes; documentation
edits need link/content checks, not new Rust tests.

Tests create unique temporary directories and clean them up through a fixture
guard. Set local Git identity in fixtures, use explicit initial branches and
isolate global/system Git configuration. No sleeps, network, user repository
mutations or reliance on the user's Git identity. Do not change process-wide
environment variables in parallel tests; set each child command's environment.

Use table-driven parser fixtures for record edge cases and real local Git repos
for subprocess and CLI behaviour. Test exit codes and stderr as well as stdout.
Check read-only behaviour against relevant refs, index and config before/after;
this is a smoke test, not proof against every possible external Git behaviour.

## Changes and commits

For planned work use `task/<id>-<slug>` branches and explicit task ownership.
Commit as `<id>: <imperative summary>`; small unnumbered maintenance can use
`docs:`, `fix:` or `chore:`. Preserve unrelated edits and stage explicit paths.
Keep implementation tasks sequential until their dependencies are committed.
Shared docs are updated by the scribe role after behaviour is verified.

## Documentation

Describe actual state separately from intended behaviour. README is the entry
point; design.md defines the CLI contract; architecture.md explains boundaries.
Keep AGENTS.md and CLAUDE.md below 51 lines and consistent on project facts.
Keep history entries and their index together. Record observed validation only,
including the commit used for any test baseline. Never claim zero tests proves
behaviour. Avoid secrets, private file contents and raw log dumps in docs.
