use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn entry_path(input: PathBuf) -> Result<PathBuf, String> {
    if !input.is_dir() {
        return Ok(input);
    }
    let manifest = input.join("isk.toml");
    let error = |message: &str| format!("{}: ISK-PROJECT-001: {message}", manifest.display());
    let root = input
        .canonicalize()
        .map_err(|err| error(&format!("cannot resolve project directory: {err}")))?;
    if !fs::symlink_metadata(&manifest).is_ok_and(|metadata| metadata.is_file()) {
        return Err(error(
            "expected a regular isk.toml file in the project directory",
        ));
    }
    let source = fs::read_to_string(&manifest)
        .map_err(|err| error(&format!("cannot read manifest: {err}")))?;
    let document = toml::from_str::<toml::Table>(&source)
        .map_err(|err| error(&format!("invalid TOML: {err}")))?;
    let package = document
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| error("expected a [package] table"))?;
    if let Some(key) = document.keys().find(|key| *key != "package") {
        return Err(error(&format!("unknown manifest key `{key}`")));
    }
    if let Some(key) = package
        .keys()
        .find(|key| !matches!(key.as_str(), "name" | "version" | "entry"))
    {
        return Err(error(&format!("unknown package key `{key}`")));
    }
    let string_field = |key| {
        package
            .get(key)
            .and_then(toml::Value::as_str)
            .ok_or_else(|| error(&format!("expected a string for package.{key}")))
    };
    let name = string_field("name")?;
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Err(error(
            "package.name must be nonempty and contain no control characters",
        ));
    }
    semver::Version::parse(string_field("version")?)
        .map_err(|err| error(&format!("invalid package.version: {err}")))?;
    let entry = Path::new(if package.contains_key("entry") {
        string_field("entry")?
    } else {
        "src/main.isk"
    });
    if entry.extension().and_then(|ext| ext.to_str()) != Some("isk")
        || !entry
            .components()
            .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
    {
        return Err(error(
            "package.entry must be a relative .isk path without `..`",
        ));
    }
    let entry = root.join(entry);
    let resolved = entry
        .canonicalize()
        .map_err(|err| error(&format!("cannot resolve entry {}: {err}", entry.display())))?;
    if !resolved.starts_with(&root) || !resolved.is_file() {
        return Err(error(
            "package.entry must resolve to a regular file inside the project",
        ));
    }
    Ok(resolved)
}
