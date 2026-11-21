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
fn cli_checks_and_runs_the_factorial_example() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = TestDir::new("factorial");
    let example = workspace.join("examples/factorial.isk");
    let check = dir.isk("check", &example);
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(!dir.0.join(".isk-build").exists());

    let factorial = dir.isk("run", &example);
    assert!(factorial.status.success());
    assert_eq!(String::from_utf8_lossy(&factorial.stdout).trim(), "3628800");
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
    fs::write(dir.0.join("invalid.isk"), "fn main() { missing() }\n").unwrap();

    for command in ["check", "build", "run"] {
        let output = dir.isk(command, "invalid.isk");
        assert!(!output.status.success(), "{command} unexpectedly succeeded");
        assert!(output.stdout.is_empty(), "{command} wrote to stdout");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("invalid.isk: 1:13: ISK-NAME-002: unknown function `missing`"),
            "{stderr}"
        );
        assert!(!dir.0.join(".isk-build").exists());
    }
}

#[test]
fn cli_reports_runtime_arithmetic_failures() {
    let dir = TestDir::new("arithmetic");
    for (expression, diagnostic) in [
        ("18446744073709551615 + 1", "ISK integer overflow"),
        ("1 / 0", "ISK division by zero"),
    ] {
        fs::write(
            dir.0.join("arithmetic.isk"),
            format!("fn main() {{ print({expression}) }}"),
        )
        .unwrap();

        let check = dir.isk("check", "arithmetic.isk");
        assert!(check.status.success());
        let run = dir.isk("run", "arithmetic.isk");
        assert!(!run.status.success());
        assert!(run.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(stderr.contains(diagnostic), "{stderr}");
    }
}
