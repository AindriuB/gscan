# Installing gscan from a release

## Requirements and downloads

No Rust toolchain is needed. Install Git first and confirm `git --version` works.
Choose the archive matching your OS and CPU; these are native builds, not universal
binaries. Linux ARM64 and Intel Mac archives are not provided in v0.1.0.

| Target in filename | Runtime baseline | Format |
|---|---|---|
| `x86_64-unknown-linux-gnu` | Linux x86_64, glibc 2.35+ (Ubuntu 22.04+) | `.tar.gz` |
| `aarch64-apple-darwin` | macOS 14+, Apple Silicon | `.tar.gz` |
| `x86_64-pc-windows-msvc` | Windows 10/11 x86_64, Git for Windows | `.zip` |

Windows builds link the MSVC C runtime statically. Unix timeout cleanup requires
`/bin/kill`; Windows uses `taskkill`. These are standard OS utilities, not Rust tools.

The repository and releases are private. Download your archive and `SHA256SUMS`
from the authenticated GitHub Releases page:
https://github.com/AindriuB/gscan/releases

Draft releases are visible to repository maintainers with the relevant access.
Once published within the private repository, release downloads still require
repository access. Nothing is published to crates.io by this workflow.

With authenticated GitHub CLI, for example:

```sh
gh release download v0.1.0 --repo AindriuB/gscan \
  --pattern 'gscan-0.1.0-x86_64-unknown-linux-gnu.tar.gz' \
  --pattern SHA256SUMS
```

Use the matching filename for macOS or Windows. You can also download through the
browser without installing GitHub CLI. Checksums detect file changes; they are not
cryptographic signatures or a substitute for downloading from the trusted repository.

## Linux

From the download directory, verify the matching entry before extracting:

```sh
archive=gscan-0.1.0-x86_64-unknown-linux-gnu.tar.gz
awk -v file="$archive" '$2 == file {print}' SHA256SUMS > selected.sha256
test -s selected.sha256 && sha256sum --check selected.sha256
tar -xzf "$archive"
mkdir -p "$HOME/.local/bin"
install -m 755 gscan-0.1.0-x86_64-unknown-linux-gnu/gscan "$HOME/.local/bin/gscan"
"$HOME/.local/bin/gscan" --version
```

Only extract/install if verification reports `OK`. Ensure `$HOME/.local/bin` is
on PATH (add `export PATH="$HOME/.local/bin:$PATH"` to your shell startup file if
needed), then run `gscan /path/to/projects`.

## macOS (Apple Silicon)

```sh
archive=gscan-0.1.0-aarch64-apple-darwin.tar.gz
awk -v file="$archive" '$2 == file {print}' SHA256SUMS > selected.sha256
test -s selected.sha256 && shasum -a 256 --check selected.sha256
tar -xzf "$archive"
mkdir -p "$HOME/.local/bin"
install -m 755 gscan-0.1.0-aarch64-apple-darwin/gscan "$HOME/.local/bin/gscan"
"$HOME/.local/bin/gscan" --version
```

Only continue after checksum verification succeeds. Add `$HOME/.local/bin` to
PATH in `~/.zshrc` if necessary. Archives are not Developer ID signed or notarized;
macOS may require you to approve the downloaded executable through its normal
Privacy & Security controls. Do not disable Gatekeeper globally.

## Windows (PowerShell)

From the directory containing the ZIP and checksum file:

```powershell
$archive = 'gscan-0.1.0-x86_64-pc-windows-msvc.zip'
$pattern = '^[0-9a-f]{64}  ' + [regex]::Escape($archive) + '$'
$entries = @(Get-Content .\SHA256SUMS | Where-Object { $_ -match $pattern })
if ($entries.Count -ne 1) { throw 'Missing or duplicate checksum entry' }
$expected = ($entries[0] -split '  ', 2)[0]
if ((Get-FileHash $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) {
    throw 'Checksum mismatch'
}
Expand-Archive $archive -DestinationPath .\gscan-extracted
$installDir = Join-Path $env:LOCALAPPDATA 'Programs\gscan'
New-Item -ItemType Directory -Force $installDir | Out-Null
Copy-Item .\gscan-extracted\gscan-0.1.0-x86_64-pc-windows-msvc\gscan.exe $installDir
& (Join-Path $installDir 'gscan.exe') --version
```

Add that installation directory to your user PATH through Windows Environment
Variables, then open a new terminal. The binary is not Authenticode-signed;
Windows may show its normal downloaded-program warning. Retain the archive's
LICENSE and documentation with your downloaded copy.

## Upgrading and removing

Check `gscan --version` and `command -v gscan` (PowerShell: `Get-Command gscan`)
to identify the copy you run. Download and verify the new archive, stop any running
gscan process, then replace that executable. Keep the previous version if you
want an easy rollback. Existing repositories and Git configuration are untouched.

For Unix removal, delete only the executable you installed:
`rm "$HOME/.local/bin/gscan"`. On Windows remove
`$env:LOCALAPPDATA\Programs\gscan\gscan.exe`, then remove that PATH entry if the
directory is no longer used. Delete downloaded/extracted archives separately.
If installed using Cargo instead, use `cargo uninstall gscan`.
