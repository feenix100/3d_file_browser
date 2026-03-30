use crate::fs::entries::FsEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Modified,
    Size,
    Type,
}

#[derive(Debug, Clone)]
pub struct SortState {
    pub key: SortKey,
    pub ascending: bool,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            key: SortKey::Name,
            ascending: true,
        }
    }
}

impl SortState {
    pub fn toggle_direction(&mut self) {
        self.ascending = !self.ascending;
    }
}

pub fn sort_entries(entries: &mut [FsEntry], sort: &SortState) {
    entries.sort_by(|a, b| {
        let ord = match sort.key {
            SortKey::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortKey::Modified => a.modified.cmp(&b.modified),
            SortKey::Size => a.size.cmp(&b.size),
            SortKey::Type => a
                .extension
                .as_deref()
                .unwrap_or_default()
                .cmp(b.extension.as_deref().unwrap_or_default()),
        };
        if sort.ascending { ord } else { ord.reverse() }
    });
}
