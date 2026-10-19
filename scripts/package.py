#!/usr/bin/env python3
"""Build, verify and combine release archives. Python 3.11+, standard library only."""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tarfile
import tempfile
import time
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
TARGETS = {
    "x86_64-unknown-linux-gnu": ("Linux", {"x86_64", "amd64"}),
    "aarch64-apple-darwin": ("Darwin", {"arm64", "aarch64"}),
    "x86_64-pc-windows-msvc": ("Windows", {"amd64", "x86_64"}),
}


def run(args, **kwargs):
    return subprocess.run(args, check=True, capture_output=True, text=True, **kwargs).stdout.strip()


def version(root=ROOT):
    with (root / "Cargo.toml").open("rb") as stream:
        package = tomllib.load(stream)["package"]
    value = package["version"]
    if package["name"] != "gscan" or package.get("license") != "MIT":
        raise ValueError("expected gscan with MIT license")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?", value):
        raise ValueError("unsupported release version")
    return value


def validate_tag(tag, release_version):
    if tag and tag != f"v{release_version}":
        raise ValueError(f"tag {tag!r} does not match Cargo version v{release_version}")


def stem(release_version, target):
    if target not in TARGETS:
        raise ValueError(f"unsupported target: {target}")
    return f"gscan-{release_version}-{target}"


def filename(release_version, target):
    return stem(release_version, target) + (".zip" if target.endswith("windows-msvc") else ".tar.gz")


def executable(target):
    return "gscan.exe" if target.endswith("windows-msvc") else "gscan"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def contents(binary, target, release_version, commit, compiler, epoch, root=ROOT):
    metadata = {
        "name": "gscan", "version": release_version, "target": target,
        "commit": commit, "rustc": compiler, "source_date_epoch": epoch,
    }
    return {
        executable(target): binary.read_bytes(),
        "LICENSE": (root / "LICENSE").read_bytes(),
        "README.md": (root / "packaging/README.md").read_bytes(),
        "INSTALL.md": (root / "docs/installation.md").read_bytes(),
        "BUILD.json": (json.dumps(metadata, indent=2) + "\n").encode(),
    }


def write_archive(path, prefix, files, binary_name, epoch):
    if path.suffix == ".zip":
        date = time.gmtime(max(epoch, 315532800))[:6]
        with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for name, data in sorted(files.items()):
                info = zipfile.ZipInfo(f"{prefix}/{name}", date_time=date)
                info.create_system = 3
                info.external_attr = (0o100755 if name == binary_name else 0o100644) << 16
                archive.writestr(info, data, compress_type=zipfile.ZIP_DEFLATED)
    else:
        with path.open("wb") as stream:
            with gzip.GzipFile(filename="", mode="wb", fileobj=stream, mtime=epoch) as compressed:
                with tarfile.open(fileobj=compressed, mode="w") as archive:
                    for name, data in sorted(files.items()):
                        info = tarfile.TarInfo(f"{prefix}/{name}")
                        info.size = len(data)
                        info.mode = 0o755 if name == binary_name else 0o644
                        info.mtime = epoch
                        archive.addfile(info, io.BytesIO(data))


def check_checksum(archive):
    expected_line = f"{digest(archive)}  {archive.name}\n"
    if archive.with_name(archive.name + ".sha256").read_text() != expected_line:
        raise ValueError(f"checksum mismatch: {archive.name}")
    return expected_line


def read_archive(archive, target, release_version, expected_commit=None):
    check_checksum(archive)
    prefix = stem(release_version, target)
    required = {f"{prefix}/{name}" for name in [executable(target), "LICENSE", "README.md", "INSTALL.md", "BUILD.json"]}
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as source:
            infos = source.infolist()
            names = [info.filename for info in infos]
            if set(names) != required or len(names) != len(required):
                raise ValueError("archive has unexpected, missing or duplicate entries")
            if any((info.external_attr >> 16) & 0o170000 != 0o100000 for info in infos):
                raise ValueError("archive entries must be regular files")
            files = {name: source.read(name) for name in names}
    else:
        with tarfile.open(archive) as source:
            infos = source.getmembers()
            names = [info.name for info in infos]
            if set(names) != required or len(names) != len(required) or not all(i.isfile() for i in infos):
                raise ValueError("archive has unexpected, missing, duplicate or non-file entries")
            binary_info = source.getmember(f"{prefix}/{executable(target)}")
            if binary_info.mode & 0o111 != 0o111:
                raise ValueError("archive binary is not executable")
            files = {name: source.extractfile(name).read() for name in names}
    metadata = json.loads(files[f"{prefix}/BUILD.json"])
    if (metadata.get("name"), metadata.get("version"), metadata.get("target")) != ("gscan", release_version, target):
        raise ValueError("archive metadata mismatch")
    if not re.fullmatch(r"[0-9a-f]{40}", metadata.get("commit", "")):
        raise ValueError("invalid source commit")
    if expected_commit and metadata["commit"] != expected_commit:
        raise ValueError("archive built from a different source commit")
    if not files[f"{prefix}/LICENSE"].startswith(b"MIT License\n"):
        raise ValueError("MIT license missing from archive")
    return files


def native(target):
    system, machines = TARGETS[target]
    if platform.system() != system or platform.machine().lower() not in machines:
        raise ValueError(f"cannot execute {target} on {platform.system()}/{platform.machine()}")


def smoke(archive, target, release_version, commit=None):
    native(target)
    files = read_archive(archive, target, release_version, commit)
    with tempfile.TemporaryDirectory(prefix="gscan-package-") as temp:
        root = Path(temp)
        for name, data in files.items():
            destination = root / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        binary = root / stem(release_version, target) / executable(target)
        binary.chmod(0o755)
        env = dict(os.environ)
        # Prove the extracted program does not rely on the developer toolchain.
        env["PATH"] = os.pathsep.join(
            entry for entry in env.get("PATH", "").split(os.pathsep)
            if entry and not any(shutil.which(tool, path=entry) for tool in ("rustc", "cargo", "rustup"))
        )
        if shutil.which("rustc", path=env["PATH"]) or shutil.which("cargo", path=env["PATH"]):
            raise ValueError("Rust tools still visible in smoke-test PATH")
        git = shutil.which("git", path=env["PATH"])
        if not git:
            raise ValueError("Git must remain available without Rust on PATH")
        for key in list(env):
            if key.startswith("GIT_"):
                del env[key]
        home = root / "home"
        home.mkdir()
        env.update(HOME=str(home), XDG_CONFIG_HOME=str(home), GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
        if run([str(binary), "--version"], env=env) != f"gscan {release_version}":
            raise ValueError("packaged binary version mismatch")
        projects = root / "projects"
        projects.mkdir()
        repo = projects / "fixture"
        run([git, "init", "-b", "main", str(repo)], env=env)
        (repo / "file").write_text("untracked\n")
        result = json.loads(run([str(binary), "--json", "--jobs=2", str(projects)], env=env))
        if result["errors"] or len(result["repositories"]) != 1 or result["repositories"][0]["changes"] != 1:
            raise ValueError("extracted binary could not scan a Git fixture")
    print(f"Verified {archive.name}: checksum, contents, version and scan without Rust on PATH")


def build(args):
    release_version = version()
    validate_tag(args.tag, release_version)
    native(args.target)
    binary = ROOT / "target" / args.target / "release" / executable(args.target)
    if run([str(binary), "--version"]) != f"gscan {release_version}":
        raise ValueError("release binary version does not match Cargo.toml")
    commit = run(["git", "rev-parse", "HEAD"], cwd=ROOT)
    epoch = int(run(["git", "show", "-s", "--format=%ct", "HEAD"], cwd=ROOT))
    files = contents(binary, args.target, release_version, commit, run(["rustc", "--version"]), epoch)
    args.output.mkdir(parents=True, exist_ok=True)
    archive = args.output / filename(release_version, args.target)
    write_archive(archive, stem(release_version, args.target), files, executable(args.target), epoch)
    archive.with_name(archive.name + ".sha256").write_text(f"{digest(archive)}  {archive.name}\n", newline="\n")
    smoke(archive, args.target, release_version, commit)


def manifest(args):
    release_version = version()
    validate_tag(args.tag, release_version)
    commit = run(["git", "rev-parse", "HEAD"], cwd=ROOT)
    expected = {filename(release_version, target) for target in TARGETS}
    found = {p.name for p in args.output.iterdir() if p.name.endswith((".tar.gz", ".zip"))}
    if found != expected:
        raise ValueError(f"expected exactly three target archives, found {sorted(found)}")
    lines = []
    for target in sorted(TARGETS):
        archive = args.output / filename(release_version, target)
        read_archive(archive, target, release_version, commit)
        lines.append(check_checksum(archive))
    (args.output / "SHA256SUMS").write_text("".join(lines), newline="\n")
    print("Verified all target archives and wrote SHA256SUMS")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("build", "manifest"):
        item = sub.add_parser(name)
        item.add_argument("--output", type=Path, default=ROOT / "dist")
        item.add_argument("--tag", default="")
        if name == "build":
            item.add_argument("--target", choices=TARGETS, required=True)
    args = parser.parse_args()
    try:
        {"build": build, "manifest": manifest}[args.command](args)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Packaging failed: {error}\n")


if __name__ == "__main__":
    main()
