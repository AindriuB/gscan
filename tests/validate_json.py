"""Independent JSON contract check, using only Python and temporary local Git repos."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

binary = str(Path(sys.argv[1]).resolve())
with tempfile.TemporaryDirectory(prefix="gscan-json-") as temp:
    root = Path(temp)
    name = 'quote"back\\line\n' if os.name != "nt" else "repo café"
    repo = root / name
    subprocess.run(["git", "init", "-b", "main", str(repo)], check=True, capture_output=True)
    (repo / "file").write_text("untracked", encoding="utf-8")
    result = subprocess.run([binary, "--json", str(root)], check=True, capture_output=True)
    assert result.stderr == b""
    document = json.loads(result.stdout)
    assert document["schema_version"] == 1
    assert document["text_encoding"] == "escaped-os-bytes"
    row, = document["repositories"]
    expected_name = ''.join(chr(b) if 32 <= b <= 126 and b != 92 else '\\\\' if b == 92 else f'\\x{b:02x}' for b in name.encode())
    assert row["name"] == expected_name, row
    assert row["changes"] == 1 and row["ahead"] is None and row["behind"] is None
    assert row["branch"] == {"kind": "unborn", "name": "main", "oid": None}
    assert document["errors"] == []
    if sys.platform == "linux":
        odd = os.fsencode(root) + b"/bad\xff"
        os.mkdir(odd)
        subprocess.run([b"git", b"init", b"-b", b"main", odd], check=True, capture_output=True)
        document = json.loads(subprocess.check_output([binary, "--json", str(root)]))
        assert any(row["name"] == "bad\\xff" for row in document["repositories"])
    broken = root / "broken"
    broken.mkdir()
    (broken / ".git").write_text("invalid", encoding="utf-8")
    result = subprocess.run([binary, "--json", "--jobs=4", str(root)], capture_output=True)
    assert result.returncode == 1 and result.stderr == b""
    document = json.loads(result.stdout)
    assert document["repositories"] and len(document["errors"]) == 1
    assert document["errors"][0]["path"].endswith("broken")
print("JSON schema, escaping, values and partial failures verified with Python.")
