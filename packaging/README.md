# gscan

A command-line overview of local Git repositories. No Rust toolchain is required
for this binary; Git must be installed and available on PATH.

```text
gscan /path/to/projects
gscan --dirty /path/to/projects
gscan --json --recursive --jobs 4 /path/to/projects
gscan --fetch --timeout 60 /path/to/projects
```

Default scans are read-only and use locally cached upstream refs. Only `--fetch`
contacts remotes and writes objects/refs. `--recursive` stops at repository boundaries.

Read INSTALL.md for supported platforms, checksum verification, installation,
upgrades and removal. A Debian/Ubuntu amd64 package is also available for
installation through APT. BUILD.json records this archive's version, target, source
commit and compiler. The MIT license is in LICENSE.

Find source and documentation in the repository that provided this release.

JSON text fields use reversible ASCII byte escapes; consult docs/design.md in
the source repository before decoding paths in a script. Timeouts apply per Git
command, not to the whole scan. Descendant cleanup is best effort for detached
processes, using /bin/kill on Unix and taskkill on Windows.

Exit codes: 0 successful scan (including dirty/empty results), 1 partial or fatal
scan/output failure, 2 invalid arguments. Run `gscan --help` for all options.
