# 07 — Debian packaging

**Status:** Implemented; awaiting PR/native CI

**Depends on:** 06

**Owns:** Cargo version, release scripts/workflow, packaging tests and documentation.

Provide an amd64 `.deb` alongside the existing archives in v0.1.1. Install the
binary to `/usr/bin/gscan` and documentation/licence to `/usr/share/doc/gscan`.
Declare Git and the executable's actual shared-library dependencies. Do not add
custom installers, maintainer scripts, services or shell-profile changes.

## Acceptance

- Build on Ubuntu 22.04 using dpkg tools with root-owned package entries.
- Verify identity, ownership, permissions and byte-for-byte Linux archive parity.
- Include the `.deb` in SHA256SUMS and draft assets; reject missing packages.
- Exercise dependency installation, upgrade, reinstall and removal in a disposable
  Ubuntu container, preserving unrelated files and running without Rust.
- Keep archives available for other platforms; document APT install/removal and
  precedence of any existing Cargo installation.
- Use a pull request and required CI; leave release publication as a separate step.

Local validation: 30 Rust tests, formatting, Clippy and 19 Python tests pass.
Debian-specific tests require Linux/dpkg and are skipped on other platforms.
