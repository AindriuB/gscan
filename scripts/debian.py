#!/usr/bin/env python3
"""Build and verify the amd64 Debian package using native dpkg tools."""
import argparse
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile

import package

TARGET = "x86_64-unknown-linux-gnu"


def deb_version(version):
    # Debian sorts pre-releases before the corresponding stable version.
    return version.replace("-", "~", 1) + "-1"


def filename(version):
    return f"gscan_{deb_version(version)}_amd64.deb"


def payload(files, version):
    prefix = package.stem(version, TARGET)
    mapping = {"gscan": "usr/bin/gscan", "LICENSE": "usr/share/doc/gscan/copyright",
               "README.md": "usr/share/doc/gscan/README.md", "INSTALL.md": "usr/share/doc/gscan/INSTALL.md",
               "BUILD.json": "usr/share/doc/gscan/BUILD.json"}
    return {destination: files[f"{prefix}/{source}"] for source, destination in mapping.items()}


def verify(deb, archive, version, commit):
    package.check_checksum(deb)
    expected = payload(package.read_archive(archive, TARGET, version, commit), version)
    control = package.run(["dpkg-deb", "--field", str(deb)])
    fields = dict(line.split(": ", 1) for line in control.splitlines() if ": " in line and not line.startswith(" "))
    if (fields.get("Package"), fields.get("Version"), fields.get("Architecture")) != ("gscan", deb_version(version), "amd64"):
        raise ValueError("Debian package identity mismatch")
    dependencies = fields.get("Depends", "").split(", ")
    if not all(any(item == name or item.startswith(name + " (") for item in dependencies)
               for name in ("git", "libc6", "libgcc-s1")):
        raise ValueError("Debian runtime dependencies missing")
    for flag, allowed in (("--ctrl-tarfile", {"control"}), ("--fsys-tarfile", set(expected))):
        data = subprocess.run(["dpkg-deb", flag, str(deb)], check=True, capture_output=True).stdout
        with tarfile.open(fileobj=io.BytesIO(data)) as source:
            seen = set()
            for entry in source:
                name = entry.name.removeprefix("./")
                if entry.uid != 0 or entry.gid != 0:
                    raise ValueError("Debian payload must be root-owned")
                if entry.isdir():
                    if name not in ("", ".") and not any(path.startswith(name.rstrip("/") + "/") for path in allowed):
                        raise ValueError("unexpected Debian directory")
                    continue
                if not entry.isfile() or name not in allowed or name in seen:
                    raise ValueError("unexpected Debian contents or maintainer scripts")
                seen.add(name)
                mode = 0o755 if name == "usr/bin/gscan" else 0o644
                if entry.mode != mode:
                    raise ValueError("incorrect Debian file permissions")
                if flag == "--fsys-tarfile" and source.extractfile(entry).read() != expected[name]:
                    raise ValueError("Debian payload differs from verified Linux archive")
            if seen != allowed:
                raise ValueError("missing Debian contents")
    print(f"Verified {deb.name}: metadata, dependencies and matching Linux payload")


def build(output, tag):
    version = package.version()
    package.validate_tag(tag, version)
    package.native(TARGET)
    commit = package.run(["git", "rev-parse", "HEAD"], cwd=package.ROOT)
    archive = output / package.filename(version, TARGET)
    files = package.read_archive(archive, TARGET, version, commit)
    metadata = json.loads(files[f"{package.stem(version, TARGET)}/BUILD.json"])
    output = output.resolve()
    with tempfile.TemporaryDirectory(prefix="gscan-deb-") as temp:
        root = Path(temp)
        staging = root / "package"
        for name, data in payload(files, version).items():
            path = staging / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            path.chmod(0o755 if name == "usr/bin/gscan" else 0o644)
        # dpkg-shlibdeps discovers the ABI requirements of the actual binary.
        (root / "debian").mkdir()
        (root / "debian/control").write_text("Source: gscan\n\nPackage: gscan\nArchitecture: amd64\n")
        dependencies = package.run(["dpkg-shlibdeps", "-O", "-e" + str(staging / "usr/bin/gscan")], cwd=root)
        dependencies = next(line.removeprefix("shlibs:Depends=") for line in dependencies.splitlines()
                            if line.startswith("shlibs:Depends="))
        installed_size = sum((path.stat().st_size + 1023) // 1024 for path in staging.rglob("*") if path.is_file())
        control = staging / "DEBIAN/control"
        control.parent.mkdir()
        control.write_text(f"Package: gscan\nVersion: {deb_version(version)}\nArchitecture: amd64\n"
                           "Section: utils\nPriority: optional\nMaintainer: Andrew Bannister\n"
                           f"Depends: git, {dependencies}\nInstalled-Size: {installed_size}\n"
                           "Description: Survey local Git repositories\n"
                           " Show changed paths and cached upstream status across local repositories.\n")
        control.chmod(0o644)
        for directory in staging.rglob("*"):
            if directory.is_dir():
                directory.chmod(0o755)
        env = dict(os.environ, SOURCE_DATE_EPOCH=str(metadata["source_date_epoch"]))
        deb = output / filename(version)
        package.run(["dpkg-deb", "--root-owner-group", "-Zxz", "--build", str(staging), str(deb)], env=env)
        deb.with_name(deb.name + ".sha256").write_text(f"{package.digest(deb)}  {deb.name}\n")
    verify(deb, archive, version, commit)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=package.ROOT / "dist")
    parser.add_argument("--tag", default="")
    args = parser.parse_args()
    try:
        build(args.output, args.tag)
    except (ValueError, OSError, StopIteration, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Debian packaging failed: {error}\n")


if __name__ == "__main__":
    main()
