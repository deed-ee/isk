"""Print trial counts and descriptive statistics; never drop failed trials."""
import json
from pathlib import Path
from statistics import median
import sys

batch = Path(sys.argv[1])
records = [json.loads(line) for line in (batch / "results.jsonl").read_text().splitlines()]
print("| Task | Arm | First | Final | Seconds (median; range) | Total tokens (median; range) | Corrections | Used semantic ops |")
print("|---|---|---:|---:|---:|---:|---:|---:|")
for task in ("rename", "repair", "parcel"):
    for group in ("text", "semantic"):
        rows = [r for r in records if r["task"]["id"] == "controller-"+task and r["candidate"] == group]
        if not rows:
            continue
        seconds = []
        tokens = []
        used = 0
        for r in rows:
            m = r["measurements"]
            seconds.append(m["wall_time_ms"]/1000)
            if m["input_tokens"] is not None and m["output_tokens"] is not None:
                tokens.append(m["input_tokens"]+m["output_tokens"])
            turns = batch / f"{task}-{group}-{r['trial']}" / "turns.jsonl"
            actions = []
            for line in turns.read_text().splitlines() if turns.exists() else []:
                try:
                    response = json.loads(json.loads(line)["response"])
                except ValueError:
                    continue
                if isinstance(response, list):
                    actions.extend(response[:response.index(["submit"])] if ["submit"] in response else response)
            used += group == "semantic" and any(isinstance(a, list) and a and a[0] in ("inspect","rename") for a in actions)

        def spread(values, digits):
            if len(values) != len(rows):
                return "unavailable"
            return f"{median(values):.{digits}f}; {min(values):.{digits}f}–{max(values):.{digits}f}"

        first = sum(r["authoring"]["first_attempt_correct"] is True for r in rows)
        final = sum(r["outcome"]["tests_passed"] is True for r in rows)
        corrections = sum(r["measurements"]["correction_iterations"] for r in rows)
        print(f"| {task} | {group} | {first}/{len(rows)} | {final}/{len(rows)} | "
              f"{spread(seconds,1)} | {spread(tokens,0)} | {corrections} | {used}/{len(rows)} |")
