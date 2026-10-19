# Design: local repository overview

## Purpose and scope

Answer “where have I left work?” with one command over a projects directory.
V1 is a synchronous, read-only CLI. No server, database, background process,
network calls, fetching, cleanup, recursive discovery or automatic Git fixes.
This contract is implemented in v1 and verified by unit and integration tests.

## Arguments

`gscan [--dirty] [DIRECTORY]` scans the supplied directory, defaulting to the current one.
Accept `--help`, `-h`, and `--version` as standalone invocations. Support `--`
before a positional path beginning with a hyphen. Reject unknown options and
extra positional arguments with a short diagnostic and usage on stderr.
Use OS-native arguments and paths rather than requiring UTF-8 filesystem names.

`--dirty` filters successful rows to changed-path counts greater than zero; ahead
or behind commits alone do not qualify. It may appear before or after DIRECTORY,
but before `--`, and repeating it is harmless. `gscan -- --dirty` scans a directory
named `--dirty`. Help/version remain standalone. Every candidate is inspected;
filtering does not suppress failures or change exit codes.

## Discovery

Read immediate children of DIRECTORY. Inspect real directories with a `.git`
file or directory; the file form supports linked worktrees and separate Git dirs.
Skip symlinked children, regular files, ordinary directories and bare repos.
The supplied directory itself is not inspected. Sort candidates by raw directory
name on Linux, independent of filesystem enumeration order.

An unreadable scan root is fatal. An unreadable directory entry or a candidate
whose Git metadata is broken produces a diagnostic and scanning continues for
other entries. An empty scan succeeds with `No repositories found.` on stdout.
A root containing only failed candidates must not be described as empty.
With `--dirty`, an error-free scan with repositories but no matching rows prints
`No dirty repositories found.`. A genuinely empty root keeps the existing empty
message. If any inspection fails and no rows match, stdout has no success/empty
message; stderr and exit 1 report the failure.

## Repository status

Use machine-readable Git output, not human-readable `git status` text:

```sh
git --no-optional-locks -C PATH status --porcelain=v2 --branch -z --untracked-files=all
```

Count ordinary, renamed/copied, unmerged and untracked status records once each.
A staged and unstaged change to the same path counts once. A rename record has
an extra NUL-delimited original path; consume it without counting it as a file.
Ignored files do not count. Directories of untracked files count each file.
A submodule counts at most once if Git reports its path as changed; do not scan
its contents separately. Follow Git's normal submodule-ignore configuration.

Use branch headers for branch name and ahead/behind counts. For detached HEAD,
show `detached@` followed by the first 12 characters of the reported object ID.
For an unborn branch, show its branch name followed by ` (unborn)` and use dashes
for ahead/behind. Missing upstream or missing comparison headers also produce
dashes, not fabricated zeroes. Record upstream absence separately from comparison
unavailability internally. A repository with configured upstream but unavailable
counts remains a successful row. Conflicts are changed paths, not scan errors.

## Output and exit codes

Print a plain table on stdout: REPOSITORY, BRANCH, CHANGES, AHEAD, BEHIND.
Left-align names, right-align counts; size columns from headers and rendered rows.
Use no colour, terminal detection, truncation or dependency for Unicode widths.
Escape control and non-ASCII path/name bytes as `\xNN` and literal backslashes as
`\\`, giving stable ASCII output and preventing terminal control sequences.
Apply the same escaping to paths and Git diagnostics on stderr.

Successful rows remain visible when other repositories fail. Print one concise
error per failed candidate on stderr, prefixed with its escaped path. Do not
print repository contents or environment variables. Do not show a success row
for a failed candidate.

| Exit | Meaning |
|---|---|
| 0 | Scan completed without errors, including dirty or empty results; help/version |
| 1 | Root, Git, output, or per-repository inspection failure; results may be partial |
| 2 | Invalid command-line usage |

## Safety and boundaries

Invoke Git directly with argument arrays, never a shell. Set `GIT_TERMINAL_PROMPT=0`
and `LC_ALL=C`. Clear inherited `GIT_*` variables in the child environment before
setting explicit ones, so caller repository overrides cannot redirect inspection.
Do not change the parent environment. Disable optional locks to avoid status
refresh writes. Respect Git's ownership checks; never add safe.directory overrides.
No checkout, reset, clean, fetch or configuration writes are part of scanning.

V1 runs Git sequentially and buffers each status response. Very large untracked
sets can consume memory and slow scans. Process deadlines, cancellation and
streaming are deferred; document this limit rather than claiming bounded runtime.
Changes during a scan can produce a mixed-time snapshot; this is not an audit log.

## Verification cases

Use disposable local repositories for clean, staged, unstaged, untracked,
renamed, conflicted, unborn, detached, linked-worktree and missing-upstream cases.
Use a temporary local bare remote for ahead/behind cases; no network credentials.
Exercise spaces, newline/control bytes, non-UTF-8 names, malformed Git output,
broken `.git` metadata, skipped symlinks, empty roots and mixed success/failure.
Test that hostile inherited Git environment variables do not redirect the scan.

## Deferred features

JSON output, recursive discovery, colour, parallel scans and explicit
fetch are separate future decisions. First ship and use the small table command.
