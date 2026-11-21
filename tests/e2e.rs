use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn cli_runs_programs_and_checks_integer_overflow() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let temp_dir = std::env::temp_dir().join(format!("isk-e2e-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let factorial = Command::new(env!("CARGO_BIN_EXE_isk"))
        .args(["run", "examples/factorial.isk"])
        .current_dir(&workspace)
        .output()
        .unwrap();
    assert!(factorial.status.success());
    assert_eq!(String::from_utf8_lossy(&factorial.stdout).trim(), "3628800");

    let overflow_source = temp_dir.join("overflow.isk");
    fs::write(
        &overflow_source,
        "fn main() { print(18446744073709551615 + 1) }",
    )
    .unwrap();
    let overflow = Command::new(env!("CARGO_BIN_EXE_isk"))
        .args(["run", overflow_source.to_str().unwrap()])
        .current_dir(&temp_dir)
        .output()
        .unwrap();
    assert!(!overflow.status.success());
    assert!(String::from_utf8_lossy(&overflow.stderr).contains("ISK integer overflow"));

    fs::remove_dir_all(temp_dir).unwrap();
}
