# 06 — Packaging and releases

**Status:** In progress

**Depends on:** 05

**Owns:** `Cargo.toml`, `LICENSE`, `.gitignore`, `scripts/package.py`,
`scripts/draft_release.py`, `packaging/README.md`, `tests/test_*.py`,
`.github/workflows/release.yml`, `README.md`, `docs/**`.

## Agreed scope

Prepare `v0.1.0` using MIT licensing, with repository/releases remaining private.
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
published releases, refuse public repositories, and allow safe draft retries.

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
