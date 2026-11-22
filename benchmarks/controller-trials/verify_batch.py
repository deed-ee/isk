"""Cross-check the twenty raw records and their submission evidence."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("batch", type=Path)
parser.add_argument("--parcel", action="store_true")
args = parser.parse_args()
rows = [json.loads(line) for line in (args.batch / "results.jsonl").read_text().splitlines()]
tasks = ("parcel",) if args.parcel else ("rename", "repair")
expected = {(task, arm, i) for task in tasks for arm in ("semantic", "text") for i in range(1, 6)}
for row in rows:
    task = row["task"]["id"].removeprefix("controller-")
    arm = row["candidate"]
    i = row["trial"]
    expected.remove((task, arm, i))  # Reject unexpected or repeated allocations.
    work = args.batch / f"{task}-{arm}-{i}"
    log = work / "turns.jsonl"
    verdicts = []
    for line in log.read_text().splitlines() if log.exists() else []:
        turn = json.loads(line)
        verdicts.extend(item for item in turn["replies"] if isinstance(item, dict))
    snapshots = sorted(work.glob("submission-*"))
    assert len(snapshots) == len(verdicts) <= 2
    for snapshot, verdict in zip(snapshots, verdicts):
        assert (snapshot / "main.isk").is_file()
        if task == "rename":
            result = subprocess.run(
                [sys.executable, HERE.parent / "controller-rename/evaluate.py", "grade", snapshot],
                capture_output=True,
            )
            assert verdict == json.loads(result.stdout)
        if task == "parcel":
            from run import parcel_grade
            assert verdict == parcel_grade(snapshot)
    assert bool(verdicts) == row["outcome"]["completed"]
    assert bool(verdicts and all(verdicts[0].values())) == row["authoring"]["first_attempt_correct"]
    assert bool(verdicts and all(verdicts[-1].values())) == row["outcome"]["tests_passed"]
    assert max(0,len(verdicts)-1) == row["measurements"]["correction_iterations"]
assert not expected
print(f"{len(rows)} distinct allocations; first/final verdicts and submission evidence agree.")
