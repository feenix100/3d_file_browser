use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

use crate::platform::os::normalize_path;

pub fn create_folder(parent: &Path, name: &str) -> anyhow::Result<String> {
    let normalized = normalize_path(parent);
    let new_folder = normalized.join(name);
    if new_folder.exists() {
        bail!("folder already exists: {:?}", new_folder);
    }
    std::fs::create_dir_all(&new_folder)
        .with_context(|| format!("failed to create {:?}", new_folder))?;
    Ok(format!("Created folder {}", new_folder.display()))
}

pub fn rename_entry(from: &Path, to_name: &str) -> anyhow::Result<String> {
    let from = normalize_path(from);
    let Some(parent) = from.parent() else {
        bail!("cannot rename root path");
    };
    let to = parent.join(to_name);
    std::fs::rename(&from, &to).with_context(|| format!("failed to rename {:?}", from))?;
    Ok(format!("Renamed to {}", to.display()))
}

pub fn delete_entry(path: &Path) -> anyhow::Result<String> {
    let path = normalize_path(path);
    if !path.exists() {
        bail!("path does not exist: {:?}", path);
    }

    let md = std::fs::metadata(&path).with_context(|| format!("failed to stat {:?}", path))?;
    if md.is_dir() {
        std::fs::remove_dir_all(&path).with_context(|| format!("failed to remove {:?}", path))?;
    } else {
        std::fs::remove_file(&path).with_context(|| format!("failed to remove {:?}", path))?;
    }
    Ok(format!("Deleted {}", path.display()))
}

pub fn safe_child(parent: &Path, child: &Path) -> Option<PathBuf> {
    let parent = normalize_path(parent);
    let child = normalize_path(child);
    if child.starts_with(&parent) {
        Some(child)
    } else {
        None
    }
}
