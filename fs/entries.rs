use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl EntryKind {
    pub fn is_dir(self) -> bool {
        matches!(self, EntryKind::Directory)
    }
}

#[derive(Debug, Clone)]
pub struct FsEntry {
    pub id: u64,
    pub path: PathBuf,
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub extension: Option<String>,
    pub hidden: bool,
}

#[derive(Debug, Clone)]
pub struct DirectorySnapshot {
    pub path: PathBuf,
    pub entries: Vec<FsEntry>,
    pub generated_at: SystemTime,
}

pub fn read_directory_snapshot(path: &Path) -> anyhow::Result<DirectorySnapshot> {
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(path).with_context(|| format!("failed to read {:?}", path))?;

    for entry_res in read_dir {
        let entry = match entry_res {
            Ok(entry) => entry,
            Err(err) => {
                tracing::warn!("skipping unreadable dir entry: {err}");
                continue;
            }
        };

        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(md) => md,
            Err(err) => {
                tracing::warn!("failed metadata for {:?}: {err}", path);
                continue;
            }
        };
        let file_type = metadata.file_type();
        let kind = if file_type.is_dir() {
            EntryKind::Directory
        } else if file_type.is_file() {
            EntryKind::File
        } else if file_type.is_symlink() {
            EntryKind::Symlink
        } else {
            EntryKind::Other
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let extension = path.extension().map(|s| s.to_string_lossy().to_string());
        let hidden = crate::fs::metadata::is_hidden(&path, &metadata);
        let id = stable_id(&path);

        entries.push(FsEntry {
            id,
            path,
            name,
            kind,
            size: metadata.len(),
            modified: metadata.modified().ok(),
            extension,
            hidden,
        });
    }

    Ok(DirectorySnapshot {
        path: path.to_path_buf(),
        entries,
        generated_at: SystemTime::now(),
    })
}

fn stable_id(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}
