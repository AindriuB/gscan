# 06 — Packaging and releases

**Status:** Backlog — not started

**Depends on:** 05

## Outcome

Install gscan from a versioned release without needing a Rust toolchain.

## Scope to settle before implementation

- Choose the release version and software license; confirm whether releases stay
  private. Do not publish to crates.io or change repository visibility implicitly.
- Select tested platform/architecture targets: Linux x86_64 first, then verified
  macOS and Windows targets. Decide libc compatibility and native versus cross builds.
- Build versioned archives containing the executable, README and license.
- Generate SHA-256 checksums and document download, verification, installation,
  upgrade and removal. Include shell completion/man-page support only if useful.
- Add tag-driven release automation with least-privilege permissions and an
  explicit publishing gate. Keep distribution credentials out of the repository.

## Acceptance for a future task

A fresh machine can install a selected archive, verify its checksum, run
`gscan --version` and scan a fixture repository. CI tests the archive contents and
version consistency. Release notes describe compatibility and known limitations.
No packaging or publication is performed by task 05.
