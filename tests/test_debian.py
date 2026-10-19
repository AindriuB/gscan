"""Validate Debian payload contracts with real dpkg archives, without installation."""
import argparse
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import debian
import package


@unittest.skipUnless(platform.system() == "Linux" and shutil.which("dpkg-deb"), "requires dpkg-deb on Linux")
class DebianTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.version = package.version()
        self.commit = "a" * 40
        self.stage = self.root / "stage"
        self.output = self.root / "dist"
        self.output.mkdir()
        binary = self.root / "gscan"
        binary.write_bytes(b"fixture executable")
        for target in package.TARGETS:
            files = package.contents(binary, target, self.version, self.commit, "rustc test", 1700000000)
            archive = self.output / package.filename(self.version, target)
            package.write_archive(archive, package.stem(self.version, target), files, package.executable(target), 1700000000)
            self.checksum(archive)
        self.archive = self.output / package.filename(self.version, debian.TARGET)
        files = package.read_archive(self.archive, debian.TARGET, self.version)
        for name, data in debian.payload(files, self.version).items():
            path = self.stage / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            path.chmod(0o755 if name == "usr/bin/gscan" else 0o644)
        self.control = self.stage / "DEBIAN/control"
        self.control.parent.mkdir()
        self.control.write_text(f"Package: gscan\nVersion: {debian.deb_version(self.version)}\n"
                                "Architecture: amd64\nMaintainer: gscan contributors\n"
                                "Depends: git, libc6 (>= 2.35), libgcc-s1\nDescription: Test fixture\n")
        self.deb = self.output / debian.filename(self.version)

    def checksum(self, path):
        path.with_name(path.name + ".sha256").write_text(f"{package.digest(path)}  {path.name}\n")

    def build(self):
        subprocess.run(["dpkg-deb", "--root-owner-group", "--build", str(self.stage), str(self.deb)],
                       check=True, capture_output=True)
        self.checksum(self.deb)

    def verify(self):
        debian.verify(self.deb, self.archive, self.version, self.commit)

    def test_valid_payload_and_combined_manifest(self):
        self.build()
        self.verify()
        # Only Git lookup is stubbed; all dpkg and archive validation is real.
        run = package.run
        with patch.object(package, "run", side_effect=lambda args, **kw:
                          self.commit if args[:2] == ["git", "rev-parse"] else run(args, **kw)):
            package.manifest(argparse.Namespace(output=self.output, tag=f"v{self.version}"))
        lines = (self.output / "SHA256SUMS").read_text().splitlines()
        self.assertEqual(len(lines), 4)
        self.assertIn(f"{package.digest(self.deb)}  {self.deb.name}", lines)
        self.deb.unlink()
        with self.assertRaisesRegex(ValueError, "one Debian package"):
            package.manifest(argparse.Namespace(output=self.output, tag=""))

    def test_rejects_wrong_identity_dependencies_and_payload(self):
        for field, old, new in [("Version", debian.deb_version(self.version), "99.0.0-1"),
                                ("Architecture", "amd64", "arm64"), ("Depends", "git, libc6 (>= 2.35), libgcc-s1", "git")]:
            with self.subTest(field=field):
                original = self.control.read_text()
                self.control.write_text(original.replace(f"{field}: {old}", f"{field}: {new}"))
                self.build()
                with self.assertRaises(ValueError):
                    self.verify()
                self.control.write_text(original)
        (self.stage / "usr/bin/gscan").write_bytes(b"different binary")
        self.build()
        with self.assertRaisesRegex(ValueError, "differs"):
            self.verify()

    def test_rejects_hooks_symlinks_and_wrong_permissions(self):
        hook = self.stage / "DEBIAN/postinst"
        hook.write_text("#!/bin/sh\nexit 0\n")
        hook.chmod(0o755)
        self.build()
        with self.assertRaisesRegex(ValueError, "maintainer scripts"):
            self.verify()
        hook.unlink()
        binary = self.stage / "usr/bin/gscan"
        original = binary.read_bytes()
        binary.unlink()
        binary.symlink_to("/bin/sh")
        self.build()
        with self.assertRaisesRegex(ValueError, "contents"):
            self.verify()
        binary.unlink()
        binary.write_bytes(original)
        binary.chmod(0o644)
        self.build()
        with self.assertRaisesRegex(ValueError, "permissions"):
            self.verify()

    def test_rejects_corrupt_deb(self):
        self.build()
        with self.deb.open("ab") as stream:
            stream.write(b"changed")
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            self.verify()


class VersionTests(unittest.TestCase):
    def test_debian_prerelease_ordering(self):
        self.assertEqual(debian.deb_version("0.1.1-rc.1"), "0.1.1~rc.1-1")


if __name__ == "__main__":
    unittest.main()
