#!/usr/bin/env python3
"""Create/update a draft only, after validating all target archives."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

import package
import debian


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--output", type=Path, default=package.ROOT / "dist")
    args = parser.parse_args()
    try:
        release_version = package.version()
        if not args.tag:
            raise ValueError("a release tag is required")
        package.validate_tag(args.tag, release_version)
        commit = package.run(["git", "rev-parse", "HEAD"], cwd=package.ROOT)
        tagged = package.run(["git", "rev-parse", f"refs/tags/{args.tag}^{{commit}}"], cwd=package.ROOT)
        if tagged != commit:
            raise ValueError("release tag must point at the checked-out source commit")
        package.manifest(args)
        notes = package.ROOT / "docs/releases" / f"{args.tag}.md"
        if not notes.is_file():
            raise ValueError("release notes are missing")
        visibility = package.run(["gh", "repo", "view", args.repo, "--json", "visibility", "--jq", ".visibility"])
        if visibility not in {"PRIVATE", "PUBLIC"}:
            raise ValueError("unsupported repository visibility")
        assets = [args.output / package.filename(release_version, target) for target in package.TARGETS]
        assets.extend([args.output / debian.filename(release_version), args.output / "SHA256SUMS"])
        allowed = {path.name for path in assets}
        query = subprocess.run(["gh", "release", "view", args.tag, "--repo", args.repo,
                                "--json", "isDraft,assets"], capture_output=True, text=True)
        if query.returncode == 0:
            existing = json.loads(query.stdout)
            if not existing["isDraft"]:
                raise ValueError("refusing to modify a published release")
            if {asset["name"] for asset in existing["assets"]} - allowed:
                raise ValueError("draft contains unexpected assets; inspect it manually")
            package.run(["gh", "release", "edit", args.tag, "--repo", args.repo,
                         "--title", f"gscan {args.tag}", "--notes-file", str(notes)])
            package.run(["gh", "release", "upload", args.tag, "--repo", args.repo,
                         "--clobber", *map(str, assets)])
        else:
            package.run(["gh", "release", "create", args.tag, "--repo", args.repo,
                         "--verify-tag", "--target", commit, "--draft",
                         "--title", f"gscan {args.tag}", "--notes-file", str(notes), *map(str, assets)])
        release = json.loads(package.run(["gh", "release", "view", args.tag, "--repo", args.repo,
                                         "--json", "isDraft,assets,url"]))
        if not release["isDraft"] or {asset["name"] for asset in release["assets"]} != allowed:
            raise ValueError("release must remain a draft with exactly the expected assets")
        print(f"Draft ready: {release['url']}")
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Draft release failed: {error}\n")


if __name__ == "__main__":
    main()
