"""Release-gate tests. All GitHub calls are mocked; nothing is uploaded."""
from contextlib import redirect_stderr, redirect_stdout
import io
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import draft_release
import package
import debian


class DraftReleaseTests(unittest.TestCase):
    def exercise(self, existing, visibility="PRIVATE", tag_commit=None):
        commit = "a" * 40
        version = package.version()
        assets = [package.filename(version, target) for target in package.TARGETS] + [debian.filename(version), "SHA256SUMS"]
        def run(args, **kwargs):
            if args[:3] == ["git", "rev-parse", "HEAD"]:
                return commit
            if args[:2] == ["git", "rev-parse"]:
                return tag_commit or commit
            if args[:3] == ["gh", "repo", "view"]:
                return visibility
            if args[:3] == ["gh", "release", "view"]:
                return json.dumps({"isDraft": True, "assets": [{"name": name} for name in assets], "url": "https://example.invalid/draft"})
            return ""
        query = subprocess.CompletedProcess([], 1 if existing is None else 0,
                                            stdout="" if existing is None else json.dumps(existing))
        with patch.object(sys, "argv", ["draft_release.py", "--repo", "example/gscan", "--tag", f"v{version}"]), \
             patch.object(package, "manifest") as manifest, \
             patch.object(package, "run", side_effect=run) as calls, \
             patch.object(draft_release.subprocess, "run", return_value=query), \
             redirect_stderr(io.StringIO()), redirect_stdout(io.StringIO()):
            try:
                draft_release.main()
                code = 0
            except SystemExit as error:
                code = error.code
            return code, [call.args[0] for call in calls.call_args_list], manifest.called

    def test_creates_only_a_verified_draft(self):
        code, calls, validated = self.exercise(None)
        self.assertEqual(code, 0)
        self.assertTrue(validated)
        create, = [args for args in calls if args[:3] == ["gh", "release", "create"]]
        self.assertIn("--draft", create)
        self.assertIn("--verify-tag", create)

    def test_refuses_to_modify_published_release(self):
        code, calls, _ = self.exercise({"isDraft": False, "assets": []})
        self.assertEqual(code, 1)
        self.assertFalse(any(args[:2] == ["gh", "release"] for args in calls))

    def test_creates_draft_in_public_repository(self):
        code, calls, _ = self.exercise(None, visibility="PUBLIC")
        self.assertEqual(code, 0)
        create, = [args for args in calls if args[:3] == ["gh", "release", "create"]]
        self.assertIn("--draft", create)

    def test_refuses_unknown_visibility(self):
        code, calls, _ = self.exercise(None, visibility="UNKNOWN")
        self.assertEqual(code, 1)
        self.assertFalse(any(args[:2] == ["gh", "release"] for args in calls))

    def test_refuses_wrong_tag_commit(self):
        code, calls, validated = self.exercise(None, tag_commit="b" * 40)
        self.assertEqual(code, 1)
        self.assertFalse(validated)
        self.assertFalse(any(args[0] == "gh" for args in calls))

    def test_draft_retry_updates_expected_assets_only(self):
        code, calls, _ = self.exercise({"isDraft": True, "assets": []})
        self.assertEqual(code, 0)
        self.assertTrue(any(args[:3] == ["gh", "release", "upload"] for args in calls))
        self.assertFalse(any(args[:3] == ["gh", "release", "create"] for args in calls))
        code, calls, _ = self.exercise({"isDraft": True, "assets": [{"name": "user-upload.txt"}]})
        self.assertEqual(code, 1)
        self.assertFalse(any(args[:2] == ["gh", "release"] for args in calls))


if __name__ == "__main__":
    unittest.main()
