# Workflow

Use plan → implement → review → verify → record. V1 tasks are sequential because
the scanner consumes the Git model and the CLI consumes both. Parallel agents
are optional; separate concurrent writers into distinct worktrees and file scopes.

## Plan

Read README.md and docs/design.md, then the relevant architecture/conventions.
Each task in docs/plan/tasks declares owned files, dependencies and acceptance.
The task list is the working checklist; new features go into later tasks rather
than silently expanding the current task. Resolve routine implementation choices
within the documented contract; surface changes that alter user-visible scope.

## Implement

Use a task branch and isolated worktree for implementation. Keep changes within
the task's declared ownership and preserve unrelated work. Write parser tests
and temporary-repository fixtures alongside code. A single agent can perform the
roles sequentially; use available harness tools rather than assuming slash commands.

## Review and verify

Review the committed diff for concrete incorrect inputs and contract mismatches.
Run formatting, Clippy and tests on that revision. Do not review a working tree
while another process or agent mutates it. Report command results and remaining
limitations briefly; fix failures before calling the task complete.

## Record

Update PLAN.md only when status changes. Record completed behaviour and its
validation in HISTORY.md with a matching HISTORY-INDEX.md row. Keep intended
features out of completion history. The scribe commits documentation changes
before reporting completion; never include unrelated user edits.

## Kit integration

Claude Kit supports `/plan`, `/fanout`, `/review`, `/verify`, `/record`, `/recon`
and `/design` in its harness. Shared roles are in `~/.claude/agents/`; project
scripts/settings are in the ignored `.claude/` directory. Other agents follow
the same phases using their own tools. Small fixes and documentation work can
be done directly without inventing an implementation task or parallel work.
