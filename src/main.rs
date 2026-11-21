use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

fn usage() -> &'static str {
    "Usage: isk <check|build|run> <file.isk>\n       isk inspect <file.isk> <function>\n       isk rename <file.isk> <old-name> <new-name>"
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn compile_file(path: &Path) -> Result<String, String> {
    let source = read_source(path)?;
    isk::compile(&source).map_err(|error| format!("{}: {error}", path.display()))
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
    fs::write(&rust_path, rust).map_err(|error| error.to_string())?;
    let result = Command::new("rustc")
        .arg("--edition=2021")
        .args(["--crate-name", "isk_program"])
        .arg(&rust_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .map_err(|error| format!("cannot run rustc: {error}"))?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into_owned());
    }
    Ok(binary_path)
}

fn execute() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let command = args.next().and_then(|arg| arg.into_string().ok());
    let path = args.next().map(PathBuf::from).ok_or_else(usage)?;
    if command.as_deref() == Some("inspect") {
        let name = args.next().and_then(|arg| arg.into_string().ok());
        let (Some(name), None) = (name, args.next()) else {
            return Err(usage().to_owned());
        };
        let source = read_source(&path)?;
        let function = isk::inspect_function(&source, &name)
            .map_err(|error| format!("{}: {error}", path.display()))?;
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
        for range in function.calls {
            println!("call\t{}\t{}", range.start, range.end);
        }
        return Ok(());
    }
    if command.as_deref() == Some("rename") {
        let from = args.next().and_then(|arg| arg.into_string().ok());
        let to = args.next().and_then(|arg| arg.into_string().ok());
        let (Some(from), Some(to), None) = (from, to, args.next()) else {
            return Err(usage().to_owned());
        };
        let source = read_source(&path)?;
        let edited = isk::rename_function(&source, &from, &to)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        print!("{edited}");
        return Ok(());
    }
    if args.next().is_some() {
        return Err(usage().to_owned());
    }
    match command.as_deref() {
        Some("check") => {
            compile_file(&path)?;
            println!("{}: ok", path.display());
            Ok(())
        }
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
