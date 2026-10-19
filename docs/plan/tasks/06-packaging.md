# 06 — Packaging and releases

**Status:** Complete / merged

**Depends on:** 05

**Owns:** `src/process.rs` (Unix timeout cleanup compatibility), `Cargo.toml`, `LICENSE`, `.gitignore`, `scripts/package.py`,
`scripts/draft_release.py`, `packaging/README.md`, `tests/test_*.py`,
`.github/workflows/release.yml`, `README.md`, `docs/**`.

## Agreed scope

Prepare `v0.1.0` using MIT licensing, without changing repository visibility.
Create a draft after archive verification; publishing the draft is a separate
maintainer action. Do not publish to crates.io or change repository visibility.

| Target | Build runner | Runtime baseline |
|---|---|---|
| Linux x86_64 GNU | Ubuntu 22.04 | glibc 2.35+ |
| macOS Apple Silicon | macOS 14 | macOS 14+ |
| Windows x86_64 MSVC | Windows 2022 | Windows 10/11; static MSVC CRT |

Archives contain the executable, standalone README, MIT LICENSE, INSTALL.md and
BUILD.json (version, target, source commit, compiler and source timestamp).
Produce SHA256SUMS covering all three archives. Linux/macOS use tar.gz; Windows
uses ZIP. Package scripts use only Python's standard library.

## Automation and publishing gate

Branch/PR/manual workflows build, verify and retain CI artifacts. Only a pushed
`v*` tag can trigger draft creation. Require exact Cargo/tag/binary version and
source-commit matches. Restrict contents-write permission to the draft job; pin
actions to commits and avoid persisted checkout credentials. Refuse to modify
published releases, support public and private repositories, and allow safe draft retries.

## Acceptance

- Every native runner runs formatting, Clippy, Rust tests and packaging tests.
- Extract each archive, verify its checksum and exact contents, run --version,
  and scan a temporary Git fixture with Rust removed from PATH.
- Reject corrupted archives, missing files, unsafe paths and version/commit drift.
- Independently aggregate verified per-target hashes into SHA256SUMS.
- Draft v0.1.0 contains all three archives and SHA256SUMS; confirm download hashes.
- Document authenticated downloads, compatibility, installation, upgrades/removal,
  unsigned platform binaries and the separate publication step.

Baseline: `930b99a`; main's three-platform CI passed before this task.

## Compatibility correction discovered during packaging

Ubuntu 22.04's older procps can misparse a negative process-group argument without
an option terminator. Timeout cleanup now uses `kill -KILL -- -PGID`; native
package tests must pass on the baseline runner before the release is tagged.

## Completion evidence

Released source: `31459751f03eae012bccad7e4cc1f23af43ae97b`, tagged `v0.1.0`.
The tag workflow
passed native formatting, Clippy, Rust and 13 packaging tests, archive smoke
checks and checksum assembly on all three platforms. Linux runs 30 Rust tests;
platform-specific fixtures account for different counts elsewhere.

The draft
contains exactly three archives and SHA256SUMS. All downloaded archive hashes,
contents and source metadata were independently checked on 2026-10-19; the
downloaded Linux executable also passed version and fixture scanning without
Rust on PATH. The release remains a draft. Publishing is a separate action.
