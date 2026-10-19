# gscan

<!-- HARD CAP: 50 lines. This file is a router, not a manual.
     If you are adding detail here, it belongs in docs/ instead. -->

Single-repository Rust CLI for surveying local Git repositories. Build from this
root with Cargo. The current code is a starter; README.md defines the first milestone.

## Read on demand, not up front

| Need | File |
|---|---|
| Scope, intended usage and current implementation status | `README.md` |
| CLI behaviour, edge cases and safety contract | `docs/design.md` |
| How we work (the loop, roles, parallelism) | `docs/workflow.md` |
| Code style, naming, commit format | `docs/conventions.md` |
| What is open, in priority order | `docs/plan/PLAN.md` |
| What was already built — scan the index first | `docs/plan/HISTORY-INDEX.md` |
| One task's full contract | `docs/plan/tasks/<id>.md` |
| System shape and boundaries | `docs/architecture.md` |

Read only the documents needed for the task; verify implementation claims against code.

## Roles

Use the kit's roles when following its task workflow: explorer and architect
inspect; planner scopes work; implementer writes code; scribe maintains docs;
code-reviewer and reviewer assess changes; tester verifies behaviour.
Shared role definitions live in `~/.claude/agents/`.

## The loop

Plan → implement → review → verify → record. See `docs/workflow.md`.
Claude Kit slash commands are harness-specific; use available tools to perform
the same steps when those commands are unavailable.

## Verification

`cargo fmt -- --check`, `cargo clippy -- -D warnings`, and `cargo test`.
Test repository scanning against temporary Git repositories, including dirty,
unborn and detached HEAD states and missing upstreams, once implemented.

## Rules that hold everywhere

1. Keep task ownership explicit; isolate concurrent work in separate worktrees.
2. Preserve unrelated changes and keep edits within the requested task's scope.
3. Prefer `rg` and targeted reads; cite paths and lines in findings.
4. Use Git commands and the Rust standard library for the first milestone.
5. Scanning is read-only; never fetch or modify inspected repositories implicitly.
6. Keep credentials, tokens and private repository contents out of docs and logs.
