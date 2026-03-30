use std::time::UNIX_EPOCH;

use crate::fs::entries::{EntryKind, FsEntry};

pub fn selected_entry_summary(entry: Option<&FsEntry>) -> String {
    let Some(entry) = entry else {
        return "No selection".to_string();
    };

    let kind = match entry.kind {
        EntryKind::File => "file",
        EntryKind::Directory => "directory",
        EntryKind::Symlink => "symlink",
        EntryKind::Other => "other",
    };
    let modified = entry
        .modified
        .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let extension = entry.extension.as_deref().unwrap_or("-");

    format!(
        "{} | kind={} | ext={} | size={} bytes | modified={} | hidden={}",
        entry.name, kind, extension, entry.size, modified, entry.hidden
    )
}
