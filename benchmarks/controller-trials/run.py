"""Linux/OpenCode controller pilot. Run from a full clone; see README.md."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import subprocess
import sys
import tempfile
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ORACLE = ROOT / "benchmarks/controller-rename"
PIN = "6b68ec828713ab3072ee34ba1624f64270fc4f98"
MODEL = "openai/gpt-6.2-astra"
SEED = 221126
LIMIT = 240
STEPS = 10
PROFILE = """---
mode: primary
steps: 1
permissions:
  - action: "*"
    resource: "*"
    effect: deny
---
You are an ISK maintenance agent. Use only the JSON action interface described
by the experiment driver. Return a JSON array of actions, no other text.
"""
COMMON = """Use JSON arrays of actions, executed left to right. Operations:
["read"] returns main.isk. ["replace",before,after] replaces one unique literal
substring; use context for repeated text. ["callers",["name",...]] writes callers.json.
["check"] and ["run"] invoke the ISK CLI. ["submit"] freezes your current files
for grading; one correction is allowed if rejected. Submit must end the batch.
"""
EXTRA = """Additional operations: ["inspect",name] returns the compiler's JSON
symbol/caller report. ["rename",old,new] applies the compiler's validated edit.
"""
REPAIR = """The controller violates the Ready interlock. Diagnose and repair
main.isk so Ready moves Starting to Running only with a clear interlock.
Preserve all other transitions, the public transition signature, and the
demo. Keep the change minimal. Submit main.isk."""


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT).decode().strip()


def original():
    return subprocess.check_output(["git", "show", PIN + ":showcase/controller/src/main.isk"], cwd=ROOT)


def repair_starter():
    src = original()
    old = b"if interlock_clear { 2 } else { state }"
    assert src.count(old) == 1
    return src.replace(old, b"2")


def call(work, *args):
    """Only the workdir is writable; no host home, credentials, repo or network."""
    cmd = ["bwrap", "--unshare-all", "--die-with-parent", "--new-session",
           "--ro-bind", "/usr", "/usr", "--symlink", "usr/bin", "/bin",
           "--symlink", "usr/lib", "/lib", "--symlink", "usr/lib64", "/lib64",
           "--ro-bind", str(SYSROOT), "/rust", "--ro-bind", str(ISK), "/isk",
           "--dev", "/dev", "--proc", "/proc", "--tmpfs", "/tmp",
           "--bind", str(work), "/work", "--chdir", "/work", "--clearenv",
           "--setenv", "PATH", "/rust/bin:/usr/bin", "--setenv", "HOME", "/tmp",
           "prlimit", "--cpu=8", "--fsize=33554432", "--as=4294967296",
           "/isk", *args]
    result = subprocess.run(cmd, capture_output=True, timeout=15)
    return result.returncode, (result.stdout + result.stderr)[:32768].decode(errors="replace")


def matrix(source):
    # Replace the demo with probes of all documented IDs, one unknown ID per
    # domain, both interlock values and representative consecutive transitions.
    body = ["fn main() {"]
    for state in [0, 1, 2, 3, 4, 99]:
        for event in [0, 1, 2, 3, 4, 99]:
            for gate in ["true", "false"]:
                body.append(f"let v{len(body)} = print(transition({state}, {event}, {gate}));")
    body.extend(["let started = transition(0, 0, true);",
                 "let waiting = transition(started, 1, false);",
                 "print(transition(waiting, 1, true))", "}"])
    return source.split(b"fn main()", 1)[0] + "\n".join(body).encode()


def trace(source):
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp)
        (path / "main.isk").write_bytes(source)
        return call(path, "run", "main.isk")


def grade(task, work):
    if task == "rename":
        p = subprocess.run([sys.executable, ORACLE / "evaluate.py", "grade", work], capture_output=True)
        return json.loads(p.stdout)
    source = (work / "main.isk").read_bytes()
    return {"demo_ok": trace(source) == DEMO, "transitions_ok": trace(matrix(source)) == MATRIX}


def enact(action, work, semantic, counters):
    match action:
        case ["read"]:
            return (work / "main.isk").read_text()
        case ["replace", str(old), str(new)] if old:
            p = work / "main.isk"
            text = p.read_text()
            if text.count(old) != 1:
                return "error: expected one matching substring"
            p.write_text(text.replace(old, new))
            return "edited"
        case ["callers", list(names)] if all(isinstance(n, str) for n in names):
            (work / "callers.json").write_text(json.dumps(names))
            return "written"
        case ["check"] | ["run"]:
            code, output = call(work, action[0], "main.isk")
            counters["validation_cycles"] += action[0] == "check"
            counters["test_cycles"] += action[0] == "run"
            if code:
                counters["failed_tests" if action[0] == "run" else "failed_builds"] += 1
            return [code, output]
        case ["inspect", str(name)] if semantic:
            return call(work, "inspect", "main.isk", name, "--json")
        case ["rename", str(old), str(new)] if semantic:
            code, out = call(work, "rename", "main.isk", old, new)
            if not code:
                (work / "main.isk").write_text(out)
            return code, "applied" if not code else out
        case _:
            return "unknown or unavailable action"


def trial(task, group, number, out):
    folder = out / f"{task}-{group}-{number}"
    folder.mkdir()
    semantic = group == "semantic"
    submissions = []
    usage = dict(input=0, output=0)
    counters = dict(tool_calls=0, validation_cycles=0, test_cycles=0, failed_builds=0, failed_tests=0)
    turns = 0
    sid = None
    error = None
    accounted = True
    tools = ["read", "replace", "callers", "check", "run", "submit"] + (["inspect", "rename"] if semantic else [])
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp) / "participant"
        if task == "rename":
            subprocess.run([sys.executable, ORACLE / "evaluate.py", "prepare", workspace], check=True)
            assignment = (workspace / "task.md").read_text()
        else:
            workspace.mkdir()
            assignment = REPAIR
            (workspace / "main.isk").write_bytes(repair_starter())
        # Work on files only through the allowlisted broker; the model process
        # runs in a different directory and has all native tools denied.
        client = Path(tmp) / "client"
        profiles = client / ".opencode/agents"
        profiles.mkdir(parents=True)
        (profiles / "participant.md").write_text(PROFILE)
        prompt = assignment + "\n" + COMMON + (EXTRA if semantic else "")
        started = time.monotonic()
        for turns in range(1, STEPS + 1):
            try:
                cmd = ["opencode", "run", "--agent", "participant", "--model", MODEL,
                       "--format", "json", "--archived"]
                if sid:
                    cmd += ["--session", sid]
                p = subprocess.run(cmd + [prompt], cwd=client, capture_output=True,
                                   timeout=max(.1, LIMIT - (time.monotonic() - started)))
                events = [json.loads(line) for line in p.stdout.splitlines()]
                if p.returncode or not events or any(e["type"] in ("error", "tool_use") for e in events):
                    raise RuntimeError("model invocation failed or attempted native tool")
                sid = events[0]["sessionID"]
                texts = [e["part"]["text"] for e in events if e["type"] == "text"]
                finishes = [e["part"] for e in events if e["type"] == "step_finish"]
                accounted &= bool(finishes)
                for part in finishes:
                    t = part["tokens"]
                    usage["input"] += t["input"] + sum(t.get("cache", {}).values())
                    usage["output"] += t["output"] + t.get("reasoning", 0)
                text = "\n".join(texts).strip()
                if text.startswith("```") and text.endswith("```"):
                    text = text.split("\n", 1)[1][:-3].strip()
                replies = []
                actions = None
                try:
                    actions = json.loads(text)
                    if not isinstance(actions, list) or len(actions) > 16:
                        raise ValueError
                except ValueError:
                    replies.append("expected a JSON array of up to 16 actions")
                    actions = []
                for a in actions:
                    counters["tool_calls"] += 1
                    if a == ["submit"]:
                        snap = folder / f"submission-{len(submissions)+1}"
                        snap.mkdir()
                        for name in ["main.isk", "callers.json"]:
                            if (workspace / name).exists():
                                (snap / name).write_bytes((workspace / name).read_bytes())
                        verdict = grade(task, snap)
                        submissions.append(verdict)
                        replies.append(verdict)
                        break
                    replies.append(enact(a, workspace, semantic, counters))
                with (folder / "turns.jsonl").open("a") as log:
                    print(json.dumps(dict(turn=turns, response=text, replies=replies, tokens=usage.copy())), file=log)
                if submissions and (all(submissions[-1].values()) or len(submissions) == 2):
                    break
                prompt = json.dumps(replies) + "\nContinue with the next JSON array."
            except (ValueError, KeyError, RuntimeError, subprocess.TimeoutExpired) as exc:
                error = type(exc).__name__
                accounted = False
                break
        elapsed = (time.monotonic() - started) * 1000
    record = dict(schema_version=2, benchmark=dict(name="controller-maintenance", version="1", revision=git("rev-parse","HEAD")),
                  task=dict(id="controller-"+task, suite="controller", language="isk", toolchain=PIN+"; "+RUST),
                  candidate=group, trial=number, environment=dict(os=platform.system()+" "+platform.release(),
                  architecture=platform.machine(), cpu=platform.processor() or platform.machine(),
                  memory_bytes=os.sysconf("SC_PAGE_SIZE")*os.sysconf("SC_PHYS_PAGES")),
                  model=dict(name=MODEL, version=MODEL.split("/")[-1], configuration={"sampling":"provider default","client":CLIENT}, tokenizer="provider-reported"),
                  measurements=dict(input_tokens=usage["input"] if accounted else None, output_tokens=usage["output"] if accounted else None,
                  tool_calls=counters["tool_calls"], failed_builds=counters["failed_builds"], correction_iterations=max(0,len(submissions)-1),
                  wall_time_ms=round(elapsed,2), runtime_ns=None, peak_memory_bytes=None, binary_size_bytes=None),
                  authoring=dict(prompt=assignment+"\n"+COMMON+(EXTRA if semantic else ""),tools=tools,
                  budget=dict(turns=STEPS,seconds=LIMIT,submissions=2,actions_per_turn=16),iterations=turns,failed_tests=counters["failed_tests"],
                  first_attempt_correct=bool(submissions and all(submissions[0].values())),context_tokens=None,cost_usd=None,
                  validation_cycles=counters["validation_cycles"],test_cycles=counters["test_cycles"]),
                  outcome=dict(completed=bool(submissions),tests_passed=bool(submissions and all(submissions[-1].values())),
                               notes=[error] if error else ([] if submissions else ["no submission"])))
    with (out / "results.jsonl").open("a") as log:
        print(json.dumps(record), file=log)
    print(task, group, number, record["authoring"]["first_attempt_correct"], error, flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("isk", type=Path, help="absolute CLI built at PIN (see README)")
    parser.add_argument("output", type=Path, help="new output directory")
    args = parser.parse_args()
    ISK = args.isk.resolve(strict=True)
    SYSROOT = Path(subprocess.check_output(["rustc","--print","sysroot"],text=True).strip())
    RUST = subprocess.check_output(["rustc","--version"],text=True).strip()
    CLIENT = subprocess.check_output(["opencode","--version"],text=True).strip()
    DEMO = trace(original())
    MATRIX = trace(matrix(original()))
    assert DEMO[0] == MATRIX[0] == 0, (DEMO,MATRIX)
    assert trace(matrix(repair_starter())) != MATRIX
    out = args.output.resolve()
    out.mkdir()
    (out / "binary.sha256").write_text(hashlib.sha256(ISK.read_bytes()).hexdigest()+"\n")
    rng = random.Random(SEED)
    for task in ["rename", "repair"]:
        order = [(g,n) for n in range(1,6) for g in ["text","semantic"]]
        rng.shuffle(order)
        for group, number in order:
            trial(task, group, number, out)
