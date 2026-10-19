"""Packaging contract tests; no network and no compiler required."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import package


class PackagingTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="gscan-package-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.version = package.version()
        self.commit = "a" * 40

    def make(self, target, mutate=None):
        binary = self.root / package.executable(target)
        binary.write_bytes(b"test binary")
        files = package.contents(binary, target, self.version, self.commit, "rustc test", 1700000000)
        if mutate:
            mutate(files)
        archive = self.root / package.filename(self.version, target)
        package.write_archive(archive, package.stem(self.version, target), files,
                              package.executable(target), 1700000000)
        self.checksum(archive)
        return archive

    def checksum(self, archive):
        archive.with_name(archive.name + ".sha256").write_text(f"{package.digest(archive)}  {archive.name}\n")

    def test_all_archive_formats_have_expected_contents_and_metadata(self):
        for target in package.TARGETS:
            with self.subTest(target=target):
                archive = self.make(target)
                files = package.read_archive(archive, target, self.version, self.commit)
                self.assertEqual(len(files), 5)
                prefix = package.stem(self.version, target)
                metadata = json.loads(files[f"{prefix}/BUILD.json"])
                self.assertEqual(metadata["target"], target)
                self.assertEqual(files[f"{prefix}/{package.executable(target)}"], b"test binary")

    def test_windows_checkout_line_endings_are_normalized(self):
        checkout = self.root / "checkout"
        (checkout / "packaging").mkdir(parents=True)
        (checkout / "docs").mkdir()
        (checkout / "LICENSE").write_bytes(b"MIT License\r\nCopyright\r\n")
        (checkout / "packaging/README.md").write_bytes(b"README\r\n")
        (checkout / "docs/installation.md").write_bytes(b"INSTALL\r\n")
        binary = self.root / "gscan.exe"
        binary.write_bytes(b"binary")
        files = package.contents(binary, "x86_64-pc-windows-msvc", self.version,
                                 self.commit, "rustc test", 1700000000, root=checkout)
        self.assertEqual(files["LICENSE"], b"MIT License\nCopyright\n")
        self.assertEqual(files["README.md"], b"README\n")
        self.assertEqual(files["INSTALL.md"], b"INSTALL\n")

    def test_corrupted_download_fails_checksum(self):
        target = next(iter(package.TARGETS))
        archive = self.make(target)
        with archive.open("ab") as stream:
            stream.write(b"corruption")
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            package.read_archive(archive, target, self.version)

    def test_missing_license_and_extra_traversal_entry_are_rejected(self):
        for target in package.TARGETS:
            for mutate in (lambda files: files.pop("LICENSE"), lambda files: files.update({"../escape": b"bad"})):
                with self.subTest(target=target):
                    archive = self.make(target, mutate)
                    with self.assertRaisesRegex(ValueError, "archive has unexpected"):
                        package.read_archive(archive, target, self.version)

    def test_wrong_source_commit_is_rejected(self):
        target = next(iter(package.TARGETS))
        archive = self.make(target)
        with self.assertRaisesRegex(ValueError, "different source commit"):
            package.read_archive(archive, target, self.version, "b" * 40)

    def test_wrong_binary_metadata_version_is_rejected(self):
        target = next(iter(package.TARGETS))
        def mutate(files):
            metadata = json.loads(files["BUILD.json"])
            metadata["version"] = "99.0.0"
            files["BUILD.json"] = json.dumps(metadata).encode()
        archive = self.make(target, mutate)
        with self.assertRaisesRegex(ValueError, "metadata mismatch"):
            package.read_archive(archive, target, self.version)

    def test_tag_must_match_cargo_version(self):
        package.validate_tag(f"v{self.version}", self.version)
        for tag in [self.version, "v99.0.0", "v../../bad"]:
            with self.assertRaisesRegex(ValueError, "does not match"):
                package.validate_tag(tag, self.version)

    def test_archive_metadata_is_deterministic_for_identical_inputs(self):
        for target in package.TARGETS:
            first = self.make(target).read_bytes()
            self.assertEqual(first, self.make(target).read_bytes())


if __name__ == "__main__":
    unittest.main()
