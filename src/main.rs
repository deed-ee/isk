use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use serde_json::json;
use sha2::{Digest, Sha256};

fn usage() -> &'static str {
    concat!(
        "Usage: isk new <directory>\n",
        "       isk check <file.isk> [--diagnostics=tsv|--json]\n",
        "       isk <build|run> <file.isk>\n",
        "       isk inspect <file.isk> <function> [--callers|--json]\n",
        "       isk rename <file.isk> <old-name> <new-name> [--plan | --apply <snapshot>]\n",
        "       isk affected-tests <file.isk> <changed-function> [<test-root> ...] [--json]\n",
        "       isk test <file.isk> <test-root> ..."
    )
}

fn new_project(root: &Path) -> Result<(), String> {
    let io_error = |path: &Path, error| format!("cannot create {}: {error}", path.display());
    fs::create_dir(root).map_err(|error| io_error(root, error))?;
    let src = root.join("src");
    fs::create_dir(&src).map_err(|error| io_error(&src, error))?;
    let entry = src.join("main.isk");
    let mut file = fs::File::create_new(&entry).map_err(|error| io_error(&entry, error))?;
    file.write_all(b"fn main() {\n    print(42)\n}\n")
        .map_err(|error| io_error(&entry, error))?;
    println!("{}", entry.display());
    Ok(())
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn compile_file(path: &Path) -> Result<String, String> {
    let source = read_source(path)?;
    isk::compile(&source).map_err(|error| format!("{}: {error}", path.display()))
}

fn tsv_field(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        if ch == '\\' || ch.is_control() {
            escaped.extend(ch.escape_default());
        } else {
            escaped.push(ch);
        }
    }
    escaped
}

fn check_file(path: &Path, tsv: bool) -> Result<(), String> {
    let source = read_source(path)?;
    isk::check_source(&source).map_err(|diagnostic| {
        if tsv {
            format!(
                "error\t{}\t{}\t{}\t{}\t{}",
                diagnostic.code,
                diagnostic.line,
                diagnostic.column,
                tsv_field(&path.to_string_lossy()),
                tsv_field(&diagnostic.message)
            )
        } else {
            format!("{}: {diagnostic}", path.display())
        }
    })?;
    if !tsv {
        println!("{}: ok", path.display());
    }
    Ok(())
}

fn invalid_source_json(path: &Path, diagnostic: isk::Diagnostic) -> String {
    json!({
        "schema_version": 1, "ok": false,
        "error": {
            "code": diagnostic.code, "message": diagnostic.message,
            "path": path.to_string_lossy(), "line": diagnostic.line,
            "column": diagnostic.column
        }
    })
    .to_string()
}

fn inspect_json(path: &Path, source: &str, name: &str) -> Result<(), String> {
    let program =
        isk::CheckedProgram::validate(source).map_err(|error| invalid_source_json(path, error))?;
    let function = program.inspect_function(name).map_err(|error| {
        json!({
            "schema_version": 1, "ok": false,
            "error": {
                "code": isk::UnknownFunction::CODE, "message": error.message(),
                "path": path.to_string_lossy()
            }
        })
        .to_string()
    })?;
    let parameters: Vec<_> = function
        .parameters
        .iter()
        .map(|(name, ty)| json!({"name": name, "type": ty}))
        .collect();
    let calls: Vec<_> = program
        .call_sites(name)
        .expect("inspected function belongs to this snapshot")
        .map(|call| {
            json!({
                "start": call.callee_range.start, "end": call.callee_range.end,
                "caller_id": call.caller_id, "caller_name": call.caller_name
            })
        })
        .collect();
    println!(
        "{}",
        json!({
            "schema_version": 1, "ok": true,
            "function": {
                "id": function.id, "name": function.name, "parameters": parameters,
                "result": function.result,
                "declaration": {"start": function.declaration.start, "end": function.declaration.end}
            },
            "calls": calls
        })
    );
    Ok(())
}

fn build_file(path: &Path) -> Result<PathBuf, String> {
    let rust = compile_file(path)?;
    let output_dir = PathBuf::from(".isk-build");
    fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "input path must have a valid file name".to_owned())?;
    let rust_path = output_dir.join(format!("{stem}.rs"));
    let binary_path = output_dir.join(if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        stem.to_owned()
    });
    compile_native(&rust, &rust_path, &binary_path)?;
    Ok(binary_path)
}

fn compile_native(rust: &str, rust_path: &Path, binary_path: &Path) -> Result<(), String> {
    fs::write(rust_path, rust).map_err(|error| error.to_string())?;
    let result = Command::new("rustc")
        .arg("--edition=2021")
        .args(["--crate-name", "isk_program"])
        .arg(rust_path)
        .arg("-o")
        .arg(binary_path)
        .output()
        .map_err(|error| format!("cannot run rustc: {error}"))?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into_owned());
    }
    Ok(())
}

fn run_tests(path: &Path, roots: &[&str]) -> Result<bool, String> {
    let source = read_source(path)?;
    let program =
        isk::CheckedProgram::validate(&source).map_err(|error| invalid_source_json(path, error))?;
    let (tests, rust) = program
        .compile_test_harness(roots)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let temp =
        tempfile::tempdir().map_err(|error| format!("cannot stage test harness: {error}"))?;
    let binary = temp.path().join(if cfg!(windows) {
        "isk-tests.exe"
    } else {
        "isk-tests"
    });
    compile_native(&rust, &temp.path().join("tests.rs"), &binary)?;

    let mut results = Vec::with_capacity(tests.len());
    let mut passed = true;
    for (index, test) in tests.iter().enumerate() {
        let output = Command::new(&binary)
            .arg(index.to_string())
            .output()
            .map_err(|error| format!("cannot run test `{}`: {error}", test.name))?;
        let status = match output.status.code() {
            Some(0) => "passed",
            Some(2) => "returned_false",
            _ => "runtime_error",
        };
        passed &= status == "passed";
        results.push(json!({
            "symbol": {
                "id": test.id, "name": test.name,
                "declaration": {"start": test.declaration.start, "end": test.declaration.end}
            },
            "status": status,
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }));
    }
    println!(
        "{}",
        json!({"schema_version": 1, "ok": passed, "tests": results})
    );
    Ok(passed)
}

enum RenameMode {
    Preview,
    Plan,
    Apply(String),
}

fn editable_file(path: &Path) -> Result<fs::Metadata, String> {
    let io_error = |error| format!("cannot update {}: {error}", path.display());
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if !metadata.is_file() || metadata.permissions().readonly() {
        return Err("ISK-EDIT-006: rename apply requires a writable regular file".to_owned());
    }
    Ok(metadata)
}

fn write_rename(path: &Path, source: &str, edited: &str) -> Result<(), String> {
    let io_error = |error| format!("cannot update {}: {error}", path.display());
    let metadata = editable_file(path)?;
    if edited == source {
        return Ok(());
    }
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(io_error)?;
    staged.write_all(edited.as_bytes()).map_err(io_error)?;
    staged
        .as_file()
        .set_permissions(metadata.permissions())
        .map_err(io_error)?;
    staged.as_file().sync_all().map_err(io_error)?;
    editable_file(path)?;
    if read_source(path)? != source {
        return Err("ISK-EDIT-005: source changed since the rename was planned".to_owned());
    }
    staged
        .persist(path)
        .map_err(|error| io_error(error.error))?;
    Ok(())
}

fn rename_file(path: &Path, from: &str, to: &str, mode: RenameMode) -> Result<(), String> {
    if matches!(mode, RenameMode::Apply(_)) {
        editable_file(path)?;
    }
    let source = read_source(path)?;
    let fingerprint = || format!("sha256:{:x}", Sha256::digest(source.as_bytes()));
    if let RenameMode::Apply(expected) = &mode {
        if *expected != fingerprint() {
            return Err(format!(
                "{}: ISK-EDIT-005: source does not match the planned snapshot",
                path.display()
            ));
        }
    }
    let with_path = |error| format!("{}: {error}", path.display());
    let program = isk::CheckedProgram::new(&source).map_err(with_path)?;
    let plan = program.plan_rename(from, to).map_err(with_path)?;
    match mode {
        RenameMode::Preview => print!("{}", plan.apply(&source).map_err(with_path)?),
        RenameMode::Plan => {
            println!("snapshot\t{}", fingerprint());
            for range in plan.edits() {
                println!(
                    "replace\t{}\t{}\t{}",
                    range.start,
                    range.end,
                    plan.replacement()
                );
            }
        }
        RenameMode::Apply(_) => {
            let edited = plan.apply(&source).map_err(with_path)?;
            write_rename(path, &source, &edited)?;
        }
    }
    Ok(())
}

fn execute() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let command = args.next().and_then(|arg| arg.into_string().ok());
    let path = args.next().map(PathBuf::from).ok_or_else(usage)?;
    if command.as_deref() == Some("new") {
        if args.next().is_some() {
            return Err(usage().to_owned());
        }
        return new_project(&path);
    }
    if command.as_deref() == Some("test") {
        let roots = args
            .map(|name| name.into_string())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| usage())?;
        if roots.is_empty() {
            return Err(usage().to_owned());
        }
        let roots: Vec<_> = roots.iter().map(String::as_str).collect();
        if !run_tests(&path, &roots)? {
            process::exit(1);
        }
        return Ok(());
    }
    if command.as_deref() == Some("affected-tests") {
        let changed = args
            .next()
            .and_then(|name| name.into_string().ok())
            .ok_or_else(usage)?;
        let mut roots = args
            .map(|name| name.into_string())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| usage())?;
        let json_output = roots.last().is_some_and(|arg| arg == "--json");
        if json_output {
            roots.pop();
        }
        if changed == "--json" || roots.iter().any(|root| root == "--json") {
            return Err(usage().to_owned());
        }
        let roots: Vec<_> = roots.iter().map(String::as_str).collect();
        let source = read_source(&path)?;
        let program = isk::CheckedProgram::validate(&source).map_err(|error| {
            if json_output {
                invalid_source_json(&path, error)
            } else {
                format!("{}: {error}", path.display())
            }
        })?;
        match (program.affected_tests(&changed, &roots), json_output) {
            (isk::TestSelection::Focused(roots), true) => println!(
                "{}",
                json!({"schema_version": 1, "ok": true, "scope": "focused", "tests": roots})
            ),
            (isk::TestSelection::Full(reason), true) => println!(
                "{}",
                json!({"schema_version": 1, "ok": true, "scope": "full", "reason": reason.code()})
            ),
            (isk::TestSelection::Focused(roots), false) => {
                println!("scope\tfocused");
                for root in roots {
                    println!("test\t{root}");
                }
            }
            (isk::TestSelection::Full(reason), false) => println!("scope\tfull\t{}", reason.code()),
        }
        return Ok(());
    }
    if command.as_deref() == Some("inspect") {
        let name = args.next().and_then(|arg| arg.into_string().ok());
        let Some(name) = name else {
            return Err(usage().to_owned());
        };
        let include_callers = match args.next() {
            None => false,
            Some(flag) if flag == "--callers" && args.next().is_none() => true,
            Some(flag) if flag == "--json" && args.next().is_none() => {
                return inspect_json(&path, &read_source(&path)?, &name);
            }
            Some(_) => return Err(usage().to_owned()),
        };
        let source = read_source(&path)?;
        let with_path = |error| format!("{}: {error}", path.display());
        let program = isk::CheckedProgram::new(&source).map_err(with_path)?;
        let function = program
            .inspect_function(&name)
            .map_err(|error| with_path(error.to_string()))?;
        let call_sites = if include_callers {
            Some(program.call_sites(&name).map_err(with_path)?)
        } else {
            None
        };
        let params = function
            .parameters
            .iter()
            .map(|(name, ty)| format!("{name}: {ty}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "function\t{}\t{}\t{}\t{}({params}) -> {}",
            function.id,
            function.declaration.start,
            function.declaration.end,
            function.name,
            function.result
        );
        if let Some(sites) = call_sites {
            for site in sites {
                println!(
                    "call\t{}\t{}\t{}\t{}",
                    site.callee_range.start,
                    site.callee_range.end,
                    site.caller_id,
                    site.caller_name
                );
            }
        } else {
            for range in function.calls {
                println!("call\t{}\t{}", range.start, range.end);
            }
        }
        return Ok(());
    }
    if command.as_deref() == Some("rename") {
        let from = args.next().and_then(|arg| arg.into_string().ok());
        let to = args.next().and_then(|arg| arg.into_string().ok());
        let (Some(from), Some(to)) = (from, to) else {
            return Err(usage().to_owned());
        };
        let mode = match args.next() {
            None => RenameMode::Preview,
            Some(flag) if flag == "--plan" && args.next().is_none() => RenameMode::Plan,
            Some(flag) if flag == "--apply" => {
                let expected = args.next().and_then(|arg| arg.into_string().ok());
                let (Some(expected), None) = (expected, args.next()) else {
                    return Err(usage().to_owned());
                };
                RenameMode::Apply(expected)
            }
            Some(_) => return Err(usage().to_owned()),
        };
        return rename_file(&path, &from, &to, mode);
    }
    let extra = args.next();
    if command.as_deref() == Some("check") {
        return match extra {
            None => check_file(&path, false),
            Some(flag) if flag == "--diagnostics=tsv" && args.next().is_none() => {
                check_file(&path, true)
            }
            Some(flag) if flag == "--json" && args.next().is_none() => {
                let source = read_source(&path)?;
                isk::check_source(&source).map_err(|error| invalid_source_json(&path, error))?;
                println!("{}", json!({"schema_version": 1, "ok": true}));
                Ok(())
            }
            Some(_) => Err(usage().to_owned()),
        };
    }
    if extra.is_some() {
        return Err(usage().to_owned());
    }
    match command.as_deref() {
        Some("build") => {
            let binary = build_file(&path)?;
            println!("{}", binary.display());
            Ok(())
        }
        Some("run") => {
            let binary = build_file(&path)?;
            let status = Command::new(binary)
                .status()
                .map_err(|error| format!("cannot run compiled program: {error}"))?;
            if !status.success() {
                process::exit(status.code().unwrap_or(1));
            }
            Ok(())
        }
        _ => Err(usage().to_owned()),
    }
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        process::exit(1);
    }
}
