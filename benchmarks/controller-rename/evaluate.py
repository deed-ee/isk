"""Organizer-side preparation and exact grading. Never execute a submission."""

import argparse
import json
from pathlib import Path
import subprocess


HERE = Path(__file__).resolve().parent
REVISION = "119eaa8aaeab98758dd7086f6003d1311b9252b1"
SOURCE = "showcase/controller/src/main.isk"


def replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError(f"expected one fixture anchor: {before!r}")
    return source.replace(before, after)


def starter():
    data = subprocess.check_output(["git", "show", f"{REVISION}:{SOURCE}"], cwd=HERE)
    # Rename one unrelated local and both its uses, preserving demo behavior.
    data = replace_once(data, b"let blocked =", b"let advance =")
    data = replace_once(data, b"transition(blocked,", b"transition(advance,")
    return replace_once(data, b"print(blocked)", b"print(advance)")


def expected():
    data = replace_once(starter(), b"fn advance(", b"fn advance_without_fault(")
    return replace_once(data, b"    advance(", b"    advance_without_fault(")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["prepare", "grade"])
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    directory = args.directory
    if args.operation == "prepare":
        directory.mkdir()  # Never overwrite a participant's work.
        (directory / "main.isk").write_bytes(starter())
        (directory / "task.md").write_bytes((HERE / "task.md").read_bytes())
        return 0

    try:
        source_ok = (directory / "main.isk").read_bytes() == expected()
    except FileNotFoundError:
        source_ok = False
    try:
        callers = json.loads((directory / "callers.json").read_text(encoding="utf-8"))
    except (FileNotFoundError, ValueError):
        callers = None
    result = {"source_ok": source_ok, "callers_ok": callers == ["transition"]}
    print(json.dumps(result))
    return int(not all(result.values()))


if __name__ == "__main__":
    raise SystemExit(main())
