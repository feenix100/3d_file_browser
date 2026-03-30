use crate::fs::entries::FsEntry;

pub fn filter_entries(entries: Vec<FsEntry>, query: &str) -> Vec<FsEntry> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return entries;
    }

    entries
        .into_iter()
        .filter(|entry| {
            entry.name.to_lowercase().contains(&q)
                || entry
                    .extension
                    .as_ref()
                    .map(|ext| ext.to_lowercase().contains(&q))
                    .unwrap_or(false)
        })
        .collect()
}
