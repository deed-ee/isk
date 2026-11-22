use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Output};

struct TestDir(PathBuf);

impl TestDir {
    fn new(case: &str) -> Self {
        let path = std::env::temp_dir().join(format!("isk-e2e-{}-{case}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn isk(&self, command: &str, source: impl AsRef<Path>) -> Output {
        self.command(command, source).output().unwrap()
    }

    fn command(&self, command: &str, source: impl AsRef<Path>) -> Command {
        let mut cli = Command::new(env!("CARGO_BIN_EXE_isk"));
        cli.arg(command).arg(source.as_ref()).current_dir(&self.0);
        cli
    }

    fn rename(&self, source: &str, from: &str, to: &str) -> Output {
        self.command("rename", source)
            .args([from, to])
            .output()
            .unwrap()
    }
}

#[test]
fn cli_previews_valid_renames_and_does_not_touch_the_input() {
    let dir = TestDir::new("rename");
    let source = "fn helper(n: u64) -> u64 { n + 1 }\n\
                      fn other() {}\n\
                      fn main() { let value = 41; print(helper(value)) }\n";
    let path = dir.0.join("original.isk");
    fs::write(&path, source).unwrap();

    let conflict = dir.rename("original.isk", "helper", "other");
    assert!(!conflict.status.success());
    assert!(conflict.stdout.is_empty());
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("ISK-EDIT-004"));
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    assert!(!dir.0.join(".isk-build").exists());

    // A local with the same name and a Rust-only keyword are both valid ISK targets.
    for target in ["value", "match"] {
        let edited = dir.rename("original.isk", "helper", target);
        assert!(
            edited.status.success(),
            "{}",
            String::from_utf8_lossy(&edited.stderr)
        );
        let edited = String::from_utf8(edited.stdout).unwrap();
        assert_eq!(edited, source.replace("helper", target));
        fs::write(dir.0.join("edited.isk"), edited).unwrap();
        let run = dir.isk("run", "edited.isk");
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "42");
    }
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cli_creates_and_runs_a_starter_without_overwriting_existing_projects() {
    let dir = TestDir::new("new-project");
    let created = dir.isk("new", "my app");
    assert!(created.status.success(), "{created:?}");
    let entry = Path::new("my app").join("src/main.isk");
    assert_eq!(
        Path::new(String::from_utf8(created.stdout).unwrap().trim_end()),
        entry
    );
    let run = dir.isk("run", &entry);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(String::from_utf8(run.stdout).unwrap().trim_end(), "42");

    fs::write(dir.0.join(&entry), "fn main() { print(7) }\n").unwrap();
    let existing = dir.isk("new", "my app");
    assert_eq!(existing.status.code(), Some(1), "{existing:?}");
    assert!(existing.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(dir.0.join(entry)).unwrap(),
        "fn main() { print(7) }\n"
    );

    let extra = dir.command("new", "unused").arg("extra").output().unwrap();
    assert_eq!(extra.status.code(), Some(1), "{extra:?}");
    assert!(!dir.0.join("unused").exists());
}

#[test]
fn cli_json_reports_symbols_and_structured_errors() {
    let dir = TestDir::new("json");
    let path = dir.0.join("source.isk");
    fs::write(
        &path,
        "fn helper(n: u64) -> u64 { n }\nfn main() { print(helper(42)) }",
    )
    .unwrap();
    let run = |args: &[&str]| {
        dir.command(args[0], "source.isk")
            .args(&args[1..])
            .output()
            .unwrap()
    };
    let check = run(&["check", "--json"]);
    assert!(check.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&check.stdout).unwrap(),
        serde_json::json!({"schema_version": 1, "ok": true})
    );
    let inspect = run(&["inspect", "helper", "--json"]);
    assert!(inspect.status.success(), "{inspect:?}");
    let result: serde_json::Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(
        result["function"]["parameters"],
        serde_json::json!([{"name": "n", "type": "u64"}])
    );
    assert_eq!(
        result["function"]["declaration"],
        serde_json::json!({"start": 3, "end": 9})
    );
    assert_eq!(result["calls"][0]["caller_name"], "main");

    let missing = run(&["inspect", "missing", "--json"]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(missing.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&missing.stderr).unwrap();
    assert_eq!(error["error"]["code"], "ISK-INSPECT-001");
    assert!(error["error"].get("line").is_none());

    fs::write(path, "fn main() {\n missing()\n}").unwrap();
    for args in [&["check", "--json"][..], &["inspect", "main", "--json"]] {
        let failure = run(args);
        assert_eq!(failure.status.code(), Some(1));
        assert!(failure.stdout.is_empty());
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&failure.stderr).unwrap(),
            serde_json::json!({
                "schema_version": 1, "ok": false, "error": {
                    "code": "ISK-NAME-002", "message": "unknown function `missing`",
                    "path": "source.isk", "line": 2, "column": 2
                }
            })
        );
    }
    assert!(!dir.0.join(".isk-build").exists());
}

#[test]
fn cli_suggests_transitive_test_roots_or_requests_full_verification() {
    let dir = TestDir::new("affected-tests");
    fs::write(
        dir.0.join("suite.isk"),
        "fn helper(n: u64) -> u64 { if n == 0 { 0 } else { helper(n - 1) } }\n\
         fn bridge() -> u64 { helper(1) }\n\
         fn test_independent() -> bool { true }\n\
         fn test_indirect() -> bool { bridge() == 0 }\n\
         fn test_direct() -> bool { helper(0) == 0 }\n\
         fn main() {}",
    )
    .unwrap();
    for (args, expected, expected_json) in [
        (
            &[
                "helper",
                "test_direct",
                "test_independent",
                "test_indirect",
                "test_direct",
            ][..],
            "scope\tfocused\ntest\ttest_indirect\ntest\ttest_direct",
            serde_json::json!({"scope": "focused", "tests": ["test_indirect", "test_direct"]}),
        ),
        (
            &["helper"],
            "scope\tfull\tno-roots",
            serde_json::json!({"scope": "full", "reason": "no-roots"}),
        ),
        (
            &["removed", "test_direct"],
            "scope\tfull\tunknown-change",
            serde_json::json!({"scope": "full", "reason": "unknown-change"}),
        ),
        (
            &["helper", "unmapped"],
            "scope\tfull\tunknown-root",
            serde_json::json!({"scope": "full", "reason": "unknown-root"}),
        ),
        (
            &["helper", "test_independent"],
            "scope\tfull\tno-reachable-root",
            serde_json::json!({"scope": "full", "reason": "no-reachable-root"}),
        ),
    ] {
        let output = dir
            .command("affected-tests", "suite.isk")
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim_end(),
            expected
        );
        let output = dir
            .command("affected-tests", "suite.isk")
            .args(args)
            .arg("--json")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stderr.is_empty());
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let mut expected_json = expected_json;
        expected_json["schema_version"] = 1.into();
        expected_json["ok"] = true.into();
        assert_eq!(json, expected_json);
    }

    let misplaced = dir
        .command("affected-tests", "suite.isk")
        .args(["helper", "--json", "test_direct"])
        .output()
        .unwrap();
    assert_eq!(misplaced.status.code(), Some(1), "{misplaced:?}");
    assert!(misplaced.stdout.is_empty());
    assert!(misplaced.stderr.starts_with(b"Usage:"));

    fs::write(dir.0.join("suite.isk"), "fn main() { missing() }").unwrap();
    let invalid = dir
        .command("affected-tests", "suite.isk")
        .args(["helper", "--json"])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1), "{invalid:?}");
    assert!(invalid.stdout.is_empty());
    let json: serde_json::Value = serde_json::from_slice(&invalid.stderr).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "ISK-NAME-002");
    assert_eq!(json["error"]["path"], "suite.isk");
    assert_eq!(json["error"]["line"], 1);
    assert_eq!(json["error"]["column"], 13);
    assert!(!dir.0.join(".isk-build").exists());
}

#[test]
fn cli_runs_explicit_tests_and_reports_results_by_symbol() {
    let dir = TestDir::new("test-roots");
    let source = concat!(
        "fn fails() -> bool { false }\n",
        "fn crashes() -> bool { 1 / 0 == 0 }\n",
        "fn passes() -> bool { let logged = print(42); true }\n",
        "fn main() { print(99) }\n",
    );
    fs::write(dir.0.join("suite.isk"), source).unwrap();
    let run = |roots: &[&str]| {
        dir.command("test", "suite.isk")
            .args(roots)
            .output()
            .unwrap()
    };
    let all = run(&["passes", "crashes", "fails", "passes"]);
    assert_eq!(all.status.code(), Some(1), "{all:?}");
    assert!(all.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&all.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["ok"], false);
    let tests = report["tests"].as_array().unwrap();
    assert_eq!(tests.len(), 3);
    for (id, (name, status)) in [
        ("fails", "returned_false"),
        ("crashes", "runtime_error"),
        ("passes", "passed"),
    ]
    .into_iter()
    .enumerate()
    {
        let start = source.find(name).unwrap();
        assert_eq!(
            tests[id]["symbol"],
            serde_json::json!({
                "id": id, "name": name, "declaration": {"start": start, "end": start + name.len()}
            })
        );
        assert_eq!(tests[id]["status"], status);
    }
    assert!(tests[1]["stderr"]
        .as_str()
        .unwrap()
        .contains("ISK division by zero"));
    assert_eq!(tests[2]["stdout"].as_str().unwrap().trim_end(), "42");

    let passed = run(&["passes"]);
    assert_eq!(passed.status.code(), Some(0), "{passed:?}");
    let report: serde_json::Value = serde_json::from_slice(&passed.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["tests"].as_array().unwrap().len(), 1);

    for (roots, diagnostic) in [
        (&["passes", "missing"][..], "unknown function `missing`"),
        (&["passes", "main"][..], "must have signature"),
    ] {
        let invalid = run(roots);
        assert_eq!(invalid.status.code(), Some(1), "{invalid:?}");
        assert!(invalid.stdout.is_empty());
        assert!(String::from_utf8_lossy(&invalid.stderr).contains(diagnostic));
    }
    assert!(!dir.0.join(".isk-build").exists());
}

#[test]
fn cli_plans_and_applies_a_validated_rename_workflow() {
    let dir = TestDir::new("rename-workflow");
    let source = concat!(
        "fn step(n: u64) -> u64 { if n <= 1 { 1 } else { n * step(n - 1) } }\n",
        "fn report(step: u64) -> u64 { step }\n",
        "fn main() { let step = 5; print(report(step) + step(5)) }\n",
    );
    let path = dir.0.join("workflow.isk");
    fs::write(&path, source).unwrap();
    let inspect = dir
        .command("inspect", "workflow.isk")
        .args(["step", "--callers"])
        .output()
        .unwrap();
    assert!(inspect.status.success());
    let report = String::from_utf8(inspect.stdout).unwrap();
    let callers: Vec<_> = report
        .lines()
        .skip(1)
        .map(|line| line.rsplit('\t').next().unwrap())
        .collect();
    assert_eq!(callers, ["step", "main"]);
    let plan = dir
        .command("rename", "workflow.isk")
        .args(["step", "factorial", "--plan"])
        .output()
        .unwrap();
    assert!(plan.status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    let plan = String::from_utf8(plan.stdout).unwrap();
    let mut records = plan.lines();
    let stamp = records.next().unwrap().strip_prefix("snapshot\t").unwrap();
    let offsets = [
        source.find("step(n:").unwrap(),
        source.find("step(n -").unwrap(),
        source.find("step(5)").unwrap(),
    ];
    let expected = offsets.map(|offset| format!("replace\t{offset}\t{}\tfactorial", offset + 4));
    assert_eq!(records.collect::<Vec<_>>(), expected);

    let apply = |target| {
        dir.command("rename", "workflow.isk")
            .args(["step", target, "--apply", stamp])
            .output()
            .unwrap()
    };
    let conflict = apply("report");
    assert!(!conflict.status.success());
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("ISK-EDIT-004"));
    assert_eq!(fs::read_to_string(&path).unwrap(), source);

    let changed = format!("{source}\n");
    fs::write(&path, &changed).unwrap();
    let stale = apply("factorial");
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("ISK-EDIT-005"));
    assert_eq!(fs::read_to_string(&path).unwrap(), changed);

    fs::write(&path, source).unwrap();
    let applied = apply("factorial");
    assert!(
        applied.status.success(),
        "{}",
        String::from_utf8_lossy(&applied.stderr)
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        source
            .replace("fn step(n:", "fn factorial(n:")
            .replace("step(n -", "factorial(n -")
            .replace("step(5)", "factorial(5)")
    );
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);

    let checked = dir
        .command("check", "workflow.isk")
        .arg("--diagnostics=tsv")
        .output()
        .unwrap();
    assert!(checked.status.success());
    let run = dir.isk("run", "workflow.isk");
    assert!(run.status.success());
    assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "125");
}

#[cfg(unix)]
#[test]
fn cli_rename_apply_does_not_replace_a_symlink() {
    let dir = TestDir::new("rename-symlink");
    let source = "fn helper() {} fn main() { helper() }\n";
    let path = dir.0.join("target.isk");
    fs::write(&path, source).unwrap();
    std::os::unix::fs::symlink(&path, dir.0.join("link.isk")).unwrap();
    let plan = dir
        .command("rename", "target.isk")
        .args(["helper", "new_name", "--plan"])
        .output()
        .unwrap();
    assert!(plan.status.success());
    let plan = String::from_utf8(plan.stdout).unwrap();
    let stamp = plan
        .lines()
        .next()
        .unwrap()
        .strip_prefix("snapshot\t")
        .unwrap();
    let write = dir
        .command("rename", "link.isk")
        .args(["helper", "new_name", "--apply", stamp])
        .output()
        .unwrap();
    assert!(!write.status.success());
    assert!(String::from_utf8_lossy(&write.stderr).contains("ISK-EDIT-006"));
    assert!(fs::symlink_metadata(dir.0.join("link.isk"))
        .unwrap()
        .is_symlink());
    assert_eq!(fs::read_to_string(path).unwrap(), source);
}

#[test]
fn cli_inspects_only_resolved_calls_and_reports_utf8_byte_ranges() {
    let dir = TestDir::new("inspect");
    let source = "fn step(x: u64, flag: bool) -> u64 { if flag { x } else { step(x, true) } }\r\n\
                  fn idle() {}\r\n\
                  fn main() { let step = 0; print(\u{2003}step(step(1, false), true) + step) }\r\n";
    fs::write(dir.0.join("query.isk"), source).unwrap();
    let inspect = |name: &str| {
        dir.command("inspect", "query.isk")
            .arg(name)
            .output()
            .unwrap()
    };
    let result = inspect("step");
    assert!(result.status.success(), "{:?}", result.stderr);
    let stdout = String::from_utf8(result.stdout).unwrap();
    let offsets = [
        source.find("step(x, true)").unwrap(),
        source.find("step(step(1").unwrap(),
        source.find("step(1, false)").unwrap(),
    ];
    let expected = format!(
        "function\t0\t3\t7\tstep(x: u64, flag: bool) -> u64\n\
         call\t{}\t{}\ncall\t{}\t{}\ncall\t{}\t{}\n",
        offsets[0],
        offsets[0] + 4,
        offsets[1],
        offsets[1] + 4,
        offsets[2],
        offsets[2] + 4,
    );
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        expected.lines().collect::<Vec<_>>()
    );
    assert_eq!(inspect("step").stdout, stdout.as_bytes());

    let callers = dir
        .command("inspect", "query.isk")
        .args(["step", "--callers"])
        .output()
        .unwrap();
    assert!(callers.status.success());
    let enriched = String::from_utf8(callers.stdout).unwrap();
    let mut lines = enriched.lines();
    assert_eq!(lines.next(), stdout.lines().next());
    for (offset, id, name) in [
        (offsets[0], 0, "step"),
        (offsets[1], 2, "main"),
        (offsets[2], 2, "main"),
    ] {
        assert_eq!(
            lines.next(),
            Some(format!("call\t{offset}\t{}\t{id}\t{name}", offset + 4).as_str())
        );
    }
    assert_eq!(lines.next(), None);

    let idle = inspect("idle");
    assert!(idle.status.success());
    let idle_at = source.find("idle").unwrap();
    assert_eq!(
        String::from_utf8(idle.stdout).unwrap().trim_end(),
        format!("function\t1\t{idle_at}\t{}\tidle() -> ()", idle_at + 4)
    );

    let missing = inspect("missing");
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("ISK-INSPECT-001"));
    fs::write(
        dir.0.join("query.isk"),
        source.replace("step(1, false)", "step(1, 2)"),
    )
    .unwrap();
    let invalid = inspect("step");
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("ISK-TYPE-006"));
    assert!(!dir.0.join(".isk-build").exists());
}

#[test]
fn cli_checks_and_runs_repository_examples() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for (example, expected) in [
        ("factorial", "3628800"),
        ("coprime", "true"),
        ("guarded-start", "false"),
    ] {
        let dir = TestDir::new(example);
        let source = workspace.join(format!("examples/{example}.isk"));
        let check = dir.isk("check", &source);
        assert_eq!(check.status.code(), Some(0), "{check:?}");
        assert!(!dir.0.join(".isk-build").exists());

        let run = dir.isk("run", &source);
        assert_eq!(run.status.code(), Some(0), "{run:?}");
        assert!(run.stderr.is_empty(), "{run:?}");
        let stdout = String::from_utf8(run.stdout).unwrap();
        assert_eq!(stdout.replace("\r\n", "\n"), format!("{expected}\n"));
    }
}

#[test]
fn controller_showcase_runs_guarded_transitions_and_latches_faults() {
    let dir = TestDir::new("controller");
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("showcase/controller/src/main.isk");
    let run = dir.isk("run", source);
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    assert!(run.stderr.is_empty(), "{run:?}");
    let stdout = String::from_utf8(run.stdout).unwrap();
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [
            "0", "1", "1", "2", // Guarded start and ready.
            "3", "3", "3",
            "4", // Fault, stop and failed reset cannot clear it; valid reset can.
            "1", "4", "4", "3", // Restart, cancel, unknown event, invalid state.
            "4", // Normal stop from Running with a blocked interlock.
        ]
    );
}

#[test]
fn cli_respects_arithmetic_precedence_and_grouping() {
    let dir = TestDir::new("arithmetic-precedence");
    let source = "arithmetic-precedence.isk";
    fs::write(
        dir.0.join(source),
        "fn main() {
    let ungrouped = print(2 + 3 * 4);
    print((2 + 3) * 4)
}
",
    )
    .unwrap();

    let run = dir.isk("run", source);
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    let stdout = String::from_utf8(run.stdout).unwrap();
    assert_eq!(stdout.lines().collect::<Vec<_>>(), ["14", "20"]);
}

#[test]
fn cli_builds_literal_arithmetic_and_preserves_u64_values() {
    let dir = TestDir::new("literal-arithmetic");
    fs::write(
        dir.0.join("literal-arithmetic.isk"),
        "fn main() {
    let arithmetic = print((2 + 3) * (8 - 2) / 2);
    let max = print(18446744073709551615);
    print(4294967296 < 18446744073709551615)
}
",
    )
    .unwrap();

    let build = dir.isk("build", "literal-arithmetic.isk");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let binary = PathBuf::from(String::from_utf8(build.stdout).unwrap().trim());
    assert_eq!(binary.parent(), Some(Path::new(".isk-build")));
    assert!(dir.0.join(".isk-build/literal-arithmetic.rs").is_file());

    let program = Command::new(dir.0.join(binary))
        .current_dir(&dir.0)
        .output()
        .unwrap();
    assert!(
        program.status.success(),
        "{}",
        String::from_utf8_lossy(&program.stderr)
    );
    let stdout = String::from_utf8(program.stdout).unwrap();
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        ["15", "18446744073709551615", "true"]
    );
}

#[test]
fn cli_rejects_invalid_programs_without_build_artifacts() {
    let dir = TestDir::new("invalid");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

    for (fixture, diagnostic) in [
        (
            "unknown-function.isk",
            "1:13: ISK-NAME-002: unknown function `missing`",
        ),
        ("parse-error.isk", "3:1: ISK-PARSE-004: expected `;`"),
        ("type-error.isk", "2:5: ISK-TYPE-002: binding annotation"),
        ("duplicate-function.isk", "2:1: ISK-SEMA-001:"),
    ] {
        let source = fixtures.join(fixture);
        for command in ["check", "build", "run"] {
            let output = dir.isk(command, &source);
            assert_eq!(output.status.code(), Some(1), "{command}: {output:?}");
            assert!(output.stdout.is_empty(), "{command} wrote to stdout");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains(&format!("{fixture}: {diagnostic}")),
                "{stderr}"
            );
            assert!(!dir.0.join(".isk-build").exists());
        }
    }
}

#[test]
fn cli_check_reports_structured_source_diagnostics_on_request() {
    let dir = TestDir::new("diagnostics");
    let filename = "source\tfile.isk";
    // Windows does not permit a tab in a file name.
    let (filename, escaped) = if cfg!(windows) {
        ("source-file.isk", "source-file.isk")
    } else {
        (filename, r"source\tfile.isk")
    };
    let check = |source| {
        fs::write(dir.0.join(filename), source).unwrap();
        dir.command("check", filename)
            .arg("--diagnostics=tsv")
            .output()
            .unwrap()
    };
    let invalid = check("fn main() {\n\u{2003}missing()\n}\n");
    assert_eq!(invalid.status.code(), Some(1));
    assert!(invalid.stdout.is_empty());
    assert_eq!(
        String::from_utf8(invalid.stderr).unwrap().trim_end(),
        format!("error\tISK-NAME-002\t2\t2\t{escaped}\tunknown function `missing`")
    );
    let lexical = check("fn main() { \\ }");
    assert_eq!(lexical.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(lexical.stderr).unwrap().trim_end(),
        format!("error\tISK-LEX-002\t1\t13\t{escaped}\tunexpected character `\\\\`")
    );
    let valid = check("fn main() {}");
    assert!(valid.status.success());
    assert!(valid.stdout.is_empty() && valid.stderr.is_empty());
    assert!(!dir.0.join(".isk-build").exists());
}

#[test]
fn cli_reports_runtime_arithmetic_failures() {
    let dir = TestDir::new("arithmetic");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for (fixture, diagnostic) in [
        ("integer-overflow.isk", "ISK integer overflow"),
        ("divide-by-zero.isk", "ISK division by zero"),
    ] {
        let source = fixtures.join(fixture);

        let check = dir.isk("check", &source);
        assert_eq!(check.status.code(), Some(0), "{check:?}");
        let run = dir.isk("run", &source);
        assert_eq!(run.status.code(), Some(101), "{run:?}");
        assert!(run.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(stderr.contains(diagnostic), "{stderr}");
    }
}
