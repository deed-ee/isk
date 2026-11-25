//! Entry resolution for explicitly selected single-file applications.
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

const DEFAULT_ENTRY: &str = "src/main.isk";

/// Resolve the entry of an explicitly selected application root.
pub fn application_entry(root: &Path) -> Result<PathBuf, String> {
    let base = resolve(root)?;
    let manifest = base.join("isk.toml");
    let error = |message: &str| format!("{}: ISK-PROJECT-001: {message}", manifest.display());
    match fs::symlink_metadata(&manifest) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return resolve_entry(&base, Path::new(DEFAULT_ENTRY));
        }
        Err(err) => return Err(error(&format!("cannot inspect manifest: {err}"))),
        Ok(metadata) if !metadata.is_file() => {
            return Err(error(
                "expected a regular isk.toml file in the project directory",
            ))
        }
        Ok(_) => {}
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
        DEFAULT_ENTRY
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
    resolve_entry(&base, entry).map_err(|message| error(&message))
}

fn resolve(path: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|error| format!("cannot resolve {}: {error}", path.display()))
}

fn resolve_entry(base: &Path, entry: &Path) -> Result<PathBuf, String> {
    let entry = base.join(entry);
    let actual = fs::canonicalize(&entry)
        .map_err(|error| format!("cannot resolve entry {}: {error}", entry.display()))?;
    if !actual.starts_with(base) {
        return Err(format!(
            "application entry {} resolves outside its project",
            entry.display()
        ));
    }
    if !actual.is_file() {
        return Err(format!(
            "application entry {} must be a regular file",
            entry.display()
        ));
    }
    Ok(actual)
}
