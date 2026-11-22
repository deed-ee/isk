"""Run from a full checkout with Cargo and Python 3."""

import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def grade(work, source_ok, callers_ok):
    result = subprocess.run(
        [sys.executable, HERE / "evaluate.py", "grade", work], capture_output=True
    )
    if result.returncode != int(not (source_ok and callers_ok)) or json.loads(
        result.stdout
    ) != {"source_ok": source_ok, "callers_ok": callers_ok}:
        raise RuntimeError(f"unexpected grader result: {result}")


def main():
    with TemporaryDirectory() as temp:
        work = Path(temp) / "participant"
        subprocess.run([sys.executable, HERE / "evaluate.py", "prepare", work], check=True)
        source = work / "main.isk"
        report = work / "callers.json"
        original = source.read_bytes()
        cli = ["cargo", "run", "--locked", "--quiet", "--manifest-path", ROOT / "Cargo.toml", "--"]

        def isk(*args):
            return subprocess.check_output([*cli, *args], cwd=work)

        baseline = isk("run", source).splitlines()
        if baseline != b"0 1 1 2 3 3 3 4 1 4 4 3 4".split():
            raise RuntimeError("starter changed the controller demo trace")
        inspection = json.loads(isk("inspect", source, "advance", "--json"))
        callers = list(dict.fromkeys(call["caller_name"] for call in inspection["calls"]))
        report.write_text(json.dumps(callers), encoding="utf-8")
        edited = isk("rename", source, "advance", "advance_without_fault")
        source.write_bytes(edited)
        grade(work, True, True)
        if isk("run", source).splitlines() != baseline:
            raise RuntimeError("reference edit changed the controller demo trace")
        source.write_bytes(original.replace(b"advance", b"advance_without_fault"))
        grade(work, False, True)
        source.write_bytes(edited)
        report.write_text('["main"]', encoding="utf-8")
        grade(work, True, False)
    print("Reference edit and demo trace accepted; both negative controls rejected.")


if __name__ == "__main__":
    main()
