"""Check and measure the checked-in reference programs, not agent submissions."""

import argparse
import hashlib
import json
import math
import os
import platform
import shlex
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TASK = ROOT / "benchmarks/gcd"
LANGUAGES = {"isk": "isk", "rust": "rs", "python": "py"}
TASKS = ("gcd-u64", "gcd-coprime")
EXE = ".exe" if os.name == "nt" else ""
TIMEOUT = 30


def timed(command, cwd, timeout=TIMEOUT, env=None):
    start = time.perf_counter_ns()
    result = subprocess.run(
        command, cwd=cwd, capture_output=True, text=True, timeout=timeout, env=env
    )
    elapsed = time.perf_counter_ns() - start
    if result.returncode:
        raise RuntimeError(
            f"{shlex.join(map(str, command))} exited {result.returncode}: "
            f"{result.stderr.strip() or result.stdout.strip()}"
        )
    return result.stdout, elapsed


def revision():
    files = [
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src/lib.rs", "src/main.rs",
        "benchmarks/pilot.py", "benchmarks/result.schema.json", "benchmarks/README.md",
        "benchmarks/gcd/cases.json",
        *[
            f"benchmarks/gcd/{name}.{extension}"
            for name in ("reference", "coprime")
            for extension in LANGUAGES.values()
        ],
    ]
    digest = hashlib.sha256()
    for name in sorted(files):
        digest.update(f"{name}\0".encode())
        digest.update(hashlib.sha256((ROOT / name).read_bytes()).digest())
    return f"sha256:{digest.hexdigest()}"


def driver(language, cases, task):
    expressions = [f"gcd({a}, {b})" for a, b, _ in cases]
    python_prelude = ""
    if task == "gcd-coprime":
        if language == "python":
            python_prelude = (
                "def pilot_bool(value):\n"
                "    if type(value) is not bool:\n"
                "        raise TypeError('coprime must return bool')\n"
                "    return int(value)\n\n"
            )
        for a, b, _ in cases:
            call = f"coprime({a}, {b})"
            if language == "isk":
                expressions.append(f"if {call} {{ 1 }} else {{ 0 }}")
            elif language == "rust":
                expressions.append(f"u64::from({call})")
            else:
                expressions.append(f"pilot_bool({call})")
    if language == "python":
        return python_prelude + "\n".join(f"print({expression})" for expression in expressions)
    if language == "rust":
        calls = "\n".join(f'    println!("{{}}", {expression});' for expression in expressions)
    else:
        calls = "\n".join(
            f"    let case{index} = print({expression});"
            for index, expression in enumerate(expressions)
        )
    return f"fn main() {{\n{calls}\n}}"


def environment():
    cpu = platform.processor()
    cpuinfo = Path("/proc/cpuinfo")
    if not cpu and cpuinfo.is_file():
        for line in cpuinfo.read_text().splitlines():
            key, _, value = line.partition(":")
            if key.strip() == "model name":
                cpu = value.strip()
                break
    memory = None
    if hasattr(os, "sysconf") and {"SC_PHYS_PAGES", "SC_PAGE_SIZE"} <= os.sysconf_names.keys():
        pages = os.sysconf("SC_PHYS_PAGES")
        page_size = os.sysconf("SC_PAGE_SIZE")
        if pages > 0 and page_size > 0:
            memory = pages * page_size
    return {
        "os": f"{platform.system()} {platform.release()}",
        "architecture": platform.machine(),
        "cpu": cpu or "unknown",
        "memory_bytes": memory,
    }


def trial(root, isk, task, language, cases, expected, number, toolchain, benchmark, host, build_env):
    cwd = root / f"{task}-{number}-{language}"
    cwd.mkdir()
    extension = LANGUAGES[language]
    source = cwd / f"gcd.{extension}"
    reference = (TASK / f"reference.{extension}").read_text(encoding="utf-8")
    if task == "gcd-coprime":
        reference += "\n" + (TASK / f"coprime.{extension}").read_text(encoding="utf-8")
    source.write_text(f"{reference}\n{driver(language, cases, task)}\n", encoding="utf-8")
    build_ns = binary_size = runtime_ns = tests_passed = None
    try:
        if language == "isk":
            binary, build_ns = timed([isk, "build", source.name], cwd, env=build_env)
            binary = cwd / binary.strip()
        elif language == "rust":
            binary = cwd / f"gcd{EXE}"
            _, build_ns = timed(
                ["rustc", "--edition=2021", source.name, "-o", binary], cwd, env=build_env
            )
        if language == "python":
            command = [sys.executable, "-I", source]
        else:
            binary_size = binary.stat().st_size
            command = [binary]

        warm_stdout, _ = timed(command, cwd)
        stdout, runtime_ns = timed(command, cwd)
        tests_passed = warm_stdout == stdout == expected
        notes = [] if tests_passed else [
            f"expected stdout {expected!r}, got {warm_stdout!r} and {stdout!r}"
        ]
        completed = True
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        notes = [str(error)]
        completed = False

    return {
        "schema_version": 1,
        "benchmark": benchmark,
        "task": {"id": task, "suite": "numerical", "language": language, "toolchain": toolchain},
        "candidate": "reference",
        "trial": number,
        "environment": host,
        "model": None,
        "measurements": {
            "input_tokens": None, "output_tokens": None, "tool_calls": None,
            "failed_builds": None, "correction_iterations": None,
            "wall_time_ms": None if build_ns is None else build_ns / 1_000_000,
            "runtime_ns": runtime_ns, "peak_memory_bytes": None, "binary_size_bytes": binary_size,
        },
        "outcome": {"completed": completed, "tests_passed": tests_passed, "notes": notes},
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--trials", type=int, default=3, help="repetitions per language")
    parser.add_argument("--task", choices=(*TASKS, "all"), default=TASKS[0])
    args = parser.parse_args()
    if args.trials < 1:
        parser.error("--trials must be positive")

    cases = json.loads((TASK / "cases.json").read_text(encoding="utf-8"))
    if not cases or any(
        not 0 <= a < 2**64 or not 0 <= b < 2**64 or expected != math.gcd(a, b)
        for a, b, expected in cases
    ):
        parser.error("invalid u64 GCD fixtures")
    values = [value for _, _, value in cases]
    benchmark = {"name": "isk-numerical-pilot", "version": "2", "revision": revision()}
    host = environment()
    passed = True
    try:
        with tempfile.TemporaryDirectory(prefix="isk-pilot-") as directory:
            root = Path(directory)
            target = root / "compiler"
            timed(["cargo", "build", "--locked", "--quiet", "--target-dir", target], ROOT, 120)
            sysroot, _ = timed(["rustc", "--print", "sysroot"], ROOT)
            build_env = os.environ.copy()
            # The trial directories are outside the repository's rustup override.
            build_env["PATH"] = os.pathsep.join(
                [str(Path(sysroot.strip()) / "bin"), os.environ.get("PATH", os.defpath)]
            )
            rustc, _ = timed(["rustc", "--version"], root, env=build_env)
            isk = target / f"debug/isk{EXE}"
            toolchains = {
                "isk": f"ISK source {benchmark['revision']}; backend {rustc.strip()}",
                "rust": rustc.strip(),
                "python": f"{platform.python_implementation()} {platform.python_version()}",
            }
            languages = tuple(LANGUAGES)
            for task in TASKS if args.task == "all" else (args.task,):
                outputs = values if task == "gcd-u64" else values + [int(v == 1) for v in values]
                expected = "".join(f"{value}\n" for value in outputs)
                for number in range(1, args.trials + 1):
                    offset = (number - 1) % len(languages)
                    for language in languages[offset:] + languages[:offset]:
                        record = trial(
                            root, isk, task, language, cases, expected, number,
                            toolchains[language], benchmark, host, build_env,
                        )
                        print(json.dumps(record), flush=True)
                        passed &= record["outcome"]["tests_passed"] is True
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        print(f"pilot setup failed: {error}", file=sys.stderr)
        return 1
    if not passed:
        print("pilot failed: see outcome.notes in the JSON Lines output", file=sys.stderr)
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
