# Packaging and release maintenance

## Artifacts and compatibility

Version `0.1.1` is defined in Cargo.toml; tags must be exactly `v<version>`.
The project is MIT-licensed and Cargo publishing is disabled. Packaging does
not change repository visibility.

| Target | Runner | Artifact | Runtime baseline |
|---|---|---|---|
| `x86_64-unknown-linux-gnu` | Ubuntu 22.04 | `.tar.gz` | glibc 2.35+ |
| `aarch64-apple-darwin` | macOS 14 | `.tar.gz` | Apple Silicon, macOS 14+ |
| `x86_64-pc-windows-msvc` | Windows 2022 | `.zip` | Windows 10/11 x86_64 |

Linux packages are GNU/glibc builds, not musl builds. Windows uses
`-C target-feature=+crt-static`. macOS sets `MACOSX_DEPLOYMENT_TARGET=14.0`.
These are the tested baselines, not a promise that older platforms will work.
Intel Mac, Linux ARM64, platform signing/notarization, an APT repository, and
Homebrew/WinGet distribution remain outside this release.

Each archive has one `gscan-<version>-<target>/` directory containing:

- `gscan` or `gscan.exe`.
- `LICENSE` (MIT).
- A standalone `README.md` from packaging/README.md.
- `INSTALL.md` from docs/installation.md.
- `BUILD.json` with version, target, source commit, compiler and source timestamp.

Archive metadata is normalized for identical inputs. Compiler updates and target
changes can change binaries; we do not claim reproducible builds across toolchains.
SHA256SUMS verifies download integrity; it is not a signing mechanism.

## Local preparation

Python 3.11+, stable Rust and Git are needed for building packages. Only Git and
standard OS utilities are needed to run an extracted binary. Install the Rust target
and use a matching native machine:

```sh
rustup target add x86_64-unknown-linux-gnu
cargo build --release --locked --offline --target x86_64-unknown-linux-gnu
python3 scripts/package.py build --target x86_64-unknown-linux-gnu --tag v0.1.1
python3 -m unittest discover -s tests -p 'test_*.py' -v
```

A local build inherits the host's libc/OS baseline. Only the pinned CI runner
builds are the official release assets. Output is in the ignored `dist/` directory.

The build script creates an archive and per-archive `.sha256`, then verifies its
checksum, exact contents, version and a real repository scan after extraction.
Rust-related PATH entries are removed for that check. Both the archive verifier
and tests reject missing/extra entries, traversal paths, checksum changes and
metadata drift. The full native Rust suite runs before package construction.

## CI and the publication gate

The Packages workflow runs on branch pushes, PRs, tags and manual dispatch.
Every platform builds and tests its archive. A read-only assembly job verifies
all three target archives and the Debian package against the same source commit and creates SHA256SUMS.
CI artifacts are retained for 14 days; release assets persist with the release.

Only a pushed tag beginning with `v` enables the final draft job. It has
`contents: write`; build and assembly jobs have only `contents: read`.
Actions are pinned to commits and checkout does not persist credentials.

The draft helper rechecks the tag/version/source commit, archive hashes and
repository visibility (public or private). It creates a draft with exactly three archives, one Debian package
and SHA256SUMS. A retry can update that draft's expected assets, but it refuses
unexpected user assets or any already published release.

**Publishing is deliberately manual.** Inspect the draft notes and assets, then
use GitHub's Publish release button when authorized. The workflow never publishes
or changes visibility.

## Creating the next release

1. Update Cargo.toml and Cargo.lock to the selected version. Add matching notes
   at docs/releases/v<version>.md; update versioned installation examples.
2. Commit and push. Wait for CI and Packages to pass on that exact source commit.
3. Create an annotated tag pointing at that verified commit and push it:

   ```sh
   git tag -a v0.1.1 -m "gscan v0.1.1"
   git push origin v0.1.1
   ```

   Substitute the new version. Never reuse or move an existing release tag.
4. Wait for the tag's Packages workflow to finish. Download the draft assets and
   verify SHA256SUMS and BUILD.json's source commit. Confirm the release is a draft.
5. Review/publish separately. If a build failed before draft creation, diagnose
   and rerun the failed job. Do not overwrite a published release to repair it;
   make a new version instead.

From a clone, use GitHub CLI with repository access to inspect a draft:

```sh
gh release view v0.1.1
gh release download v0.1.1 --dir downloaded-release
```

The [installation guide](installation.md) covers user installation, verification,
upgrades and removal. Release notes also state unsigned binaries and runtime limits.

## Debian package

The Linux build also runs `python3 scripts/debian.py` after archive creation.
Build tools are `dpkg-deb` and `dpkg-shlibdeps` (the `dpkg-dev` package).
Dependencies are derived from the actual executable, with Git added explicitly.
The official Ubuntu 22.04 build provides the release baseline; local builds may
require newer libraries. The package is named `gscan_<version>-1_amd64.deb`;
pre-release hyphens become Debian `~` separators.

Only `/usr/bin/gscan` and `/usr/share/doc/gscan` files are installed, owned by root.
No maintainer scripts, services, configuration files or shell changes are allowed.
Validation compares every file against the verified Linux archive and checks
package identity, dependencies, permissions and ownership. SHA256SUMS covers the
three archives and the `.deb`; the release helper expects five assets total.

The Linux CI job uses a disposable Ubuntu 22.04 container to check dependency
installation, upgrading from an older package version, reinstalling, scanning
without Rust, and removing gscan while preserving unrelated files and Git.
`tests/debian_lifecycle.sh` must only run inside that disposable container.
