//! Explicit single-entry applications. No manifest or directory traversal.
use std::fs;
use std::path::{Path, PathBuf};

/// Resolve the scaffold entry under an explicitly selected application root.
pub fn application_entry(root: &Path) -> Result<PathBuf, String> {
    let entry = root.join("src").join("main.isk");
    let resolve = |path: &Path| {
        fs::canonicalize(path)
            .map_err(|error| format!("cannot resolve {}: {error}", path.display()))
    };
    let base = resolve(root)?;
    let actual = resolve(&entry)?;
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
