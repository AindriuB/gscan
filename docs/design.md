# Design: local repository overview

## Purpose

Answer “where have I left work?” with one command over a projects directory.
Default scanning remains read-only and offline. Optional recursion, concurrency,
JSON and explicit fetch extend the same command, without a server or database.

## Arguments

`gscan [OPTIONS] [DIRECTORY]` scans children of DIRECTORY, defaulting to `.`.
Accept `--help`/`-h` and `--version` only as standalone invocations. `--` ends
option parsing. Unknown options, multiple paths, missing numeric values, zero,
negative, fractional, non-ASCII or out-of-range numbers exit 2 with usage on stderr.
Use OS-native paths. Boolean flags repeat harmlessly; the last numeric value wins.
Options work before or after DIRECTORY until `--`. Numeric values accept either
`--jobs 4` or `--jobs=4` syntax.

| Option | Default | Contract |
|---|---|---|
| `--dirty` | Off | Retain rows with changed paths greater than zero |
| `--json` | Off | Emit the versioned JSON object below |
| `--recursive` | Off | Descend ordinary directories; stop at repositories |
| `--jobs N` | 1 | 1–64 concurrent inspections, deterministic output |
| `--timeout N` | 30 | 1–3600 seconds per Git subprocess |
| `--fetch` | Off | Sequential explicit remote refresh before inspection |

Dirty filtering occurs after inspection and fetching. Ahead/behind alone does not
make a repository dirty. Filtering never suppresses failures or changes exit codes.

## Discovery and parallelism

The supplied root itself is excluded. Inspect immediate real child directories
with a `.git` file or directory, including linked worktrees. Broken `.git` metadata
is a candidate whose Git inspection can fail. Skip child symlinks and `.git` itself.
With recursion, descend ordinary directories using an iterative walk, stopping at
every candidate boundary. Do not traverse bare stores recognized by HEAD, objects
and refs. This conservative filesystem check may also skip an ordinary directory
with that exact store layout. Do not inspect nested repositories or submodules
inside a discovered repository. Filesystem contents can change during discovery;
this is a best-effort live overview, not an atomic snapshot.

An unreadable root is fatal. Child traversal/inspection failures retain other
results. Sort candidates and final rows by native path order, independent of worker
completion order. Table names are paths relative to the scan root, distinguishing
nested repositories with the same basename. Bound concurrent inspections by jobs;
if worker creation fails, available workers finish the queue. Fetches stay serial
to avoid simultaneous writers when linked worktrees share a Git directory.

## Git status

Use `git --no-optional-locks -C PATH status --porcelain=v2 --branch -z
--untracked-files=all`. Count normal, rename/copy, conflict and untracked records
once each. A staged-and-unstaged path counts once; consume a rename's additional
NUL-delimited source path without counting it. Ignored files do not count.
Untracked directories count their files individually. Submodules count at most
once per reported path, respecting normal Git submodule-ignore configuration.

Branch headers supply names and comparison counts. Detached HEAD uses
`detached@` plus 12 object-ID characters in the table. An unborn branch shows
`NAME (unborn)`. Missing upstream or comparison headers produce dashes, not zeroes.
Keep upstream absence distinct from unavailable comparison data internally.
Conflicts are changed paths, not scan errors.

## Explicit fetch

Only `--fetch` permits network activity and Git writes. Before inspection, run
`git fetch --all --no-tags --no-prune --no-prune-tags --no-recurse-submodules
--no-auto-maintenance --no-write-fetch-head` once per candidate. It uses configured
remote URLs and refspecs, so custom refspecs determine which refs change. Repositories
with no remotes require no refresh. Linked worktrees may repeat the same fetch;
serial execution favours simplicity over deduplication. Authentication must already
be configured; stdin is closed, terminal prompts and standard askpass are disabled.
Custom credential helpers may still start processes; the command deadline applies.

No checkout, reset, clean, working-file edits or Git configuration writes are
performed. Fetch may write objects and refs, and a failed fetch can have partially
updated them. On failure, record an error and still attempt status so cached data
can be shown. Such a scan exits 1 even if every status command succeeds.

## Process isolation and deadlines

Invoke Git through argument arrays, never a shell. Clear inherited `GIT_*` variables
in the child, then set explicit prompt controls and `LC_ALL=C`. Preserve normal
repository/global Git configuration and ownership checks. Never add safe.directory
overrides. Do not mutate the parent environment.

Each Git subprocess has its own deadline, including fetch. Capture stdout/stderr
in a unique temporary directory (mode 0700 on Unix), avoiding pipe-reader hangs
when descendants retain output handles. Poll the child and capture sizes; reject
output above 32 MiB per stream. Temporary disk usage can overshoot that threshold
between polls. Cleanup removes capture files after reading or failure.

On timeout, terminate the Unix process group with `/bin/kill`, or the Windows tree
with `taskkill`, then kill and reap the direct child. Descendant cleanup is best
effort: missing OS helpers or deliberately detached processes can escape it.
Windows may temporarily retain capture files if an escaped process holds a handle.
Process startup, filesystem I/O and cleanup overhead are not a strict real-time
guarantee. Directory traversal is not timed. Each repository can consume a fetch
and a status deadline; no whole-scan timeout is claimed.

## Table, JSON and errors

The table columns are REPOSITORY, BRANCH, CHANGES, AHEAD, BEHIND. Align names left
and counts right, sizing from rendered ASCII text. No colour or truncation.
Escape literal backslashes as `\\` and control/non-ASCII bytes as lowercase
`\xNN`; apply this to path/name fields and diagnostics to prevent terminal escapes.

Empty roots print `No repositories found.`. Clean-only filtered scans print
`No dirty repositories found.`. If errors exist with no matching rows, print no
success/empty message. Successful rows remain visible alongside errors; a fetch
failure can accompany a row with cached status. Table-mode errors go to stderr.

JSON emits one object followed by newline, even for fatal root errors:

```json
{
  "schema_version": 1,
  "text_encoding": "escaped-os-bytes",
  "repositories": [{
    "path": "/projects/example",
    "name": "example",
    "branch": {"kind": "named", "name": "main", "oid": null},
    "changes": 0,
    "upstream": "origin/main",
    "ahead": 0,
    "behind": 1
  }],
  "errors": []
}
```

`branch.kind` is named, unborn or detached. Detached branches have a full `oid`
and null name; others have a name and null oid. Upstream/ahead/behind are nullable.
An error contains `path` and `message`. Filtering affects only repositories.
Arrays are empty when no rows/errors exist. No table or empty-result prose appears
in JSON mode. Scan errors appear only in the JSON errors array; usage errors and
failed output writes still go to stderr. Exit codes remain authoritative.

All path/Git-name/message values first use the ASCII byte representation above,
then standard JSON quoting. Consumers parse JSON and, if needed, decode `\\` and
`\xNN`. For example a literal filename containing `\xff` is distinguished from a
filename containing byte FF by its escaped backslash. OS path bytes are native to
the platform (Rust OsStr encoded bytes); they are not a portable serialized file
handle. On Linux they preserve arbitrary filename bytes. Use paths on their
originating platform. Never pass decoded text to a shell; use argument arrays.

| Exit | Meaning |
|---|---|
| 0 | Successful scan, including dirty, empty and clean-only results; help/version |
| 1 | Root, traversal, Git, timeout, fetch or output failure; results may be partial |
| 2 | Invalid command-line usage |

## Verification and future work

Temporary local repositories cover branch states, counts, conflicts, worktrees,
submodules, recursion, deterministic worker output, byte/Unicode paths, JSON,
failed inspections and explicit fetch with a local bare remote. A subprocess
fixture verifies deadlines with output exceeding a pipe buffer. Python independently
parses JSON. Native CI covers Linux, macOS and Windows; Linux-only tests exercise
invalid filename bytes and `/dev/full`.

Packaging is implemented by task 06 with a manual publication gate; see
[packaging.md](packaging.md). Cancellation signals, whole-scan
budgets, fetch deduplication and colour are not part of this feature set.
