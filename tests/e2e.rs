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
        Command::new(env!("CARGO_BIN_EXE_isk"))
            .arg(command)
            .arg(source.as_ref())
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
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
fn cli_rejects_invalid_programs_without_build_artifacts() {
    let dir = TestDir::new("invalid");
    fs::write(dir.0.join("invalid.isk"), "fn main() { missing() }\n").unwrap();

    for command in ["check", "build", "run"] {
        let output = dir.isk(command, "invalid.isk");
        assert!(!output.status.success(), "{command} unexpectedly succeeded");
        assert!(output.stdout.is_empty(), "{command} wrote to stdout");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("invalid.isk:"), "{stderr}");
        assert!(stderr.contains("unknown function `missing`"), "{stderr}");
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
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(stderr.contains(diagnostic), "{stderr}");
    }
}
