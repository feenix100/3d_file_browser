use crate::fs::entries::{EntryKind, FsEntry};
use crate::scene::card::{CardCategory, SceneCard};

pub fn curved_deck_layout(
    entries: &[FsEntry],
    selected_index: usize,
    max_visible: usize,
    query: &str,
    hovered_index: Option<usize>,
) -> Vec<SceneCard> {
    if entries.is_empty() {
        return Vec::new();
    }

    let selected_index = selected_index.min(entries.len() - 1);
    let half = max_visible / 2;
    let start = selected_index.saturating_sub(half);
    let end = (selected_index + half + 1).min(entries.len());

    entries[start..end]
        .iter()
        .enumerate()
        .map(|(local_idx, entry)| {
            let absolute_idx = start + local_idx;
            let offset = absolute_idx as i32 - selected_index as i32;
            let distance = offset.unsigned_abs() as f32;

            // Rolodex arc around a hidden spindle to keep depth ordering legible.
            let step = 0.28f32;
            let theta = (offset as f32 * step).clamp(-1.10, 1.10);
            let radius = 8.8f32;
            let x = theta.sin() * radius;
            let z = (1.0 - theta.cos()) * radius + distance * 0.24;
            let y = -0.044 * distance + (offset as f32 * 0.65).sin() * 0.08;
            // Keep cards readable by biasing orientation back toward camera.
            let yaw = -theta * 0.62;
            let scale = (1.42 - 0.11 * distance).max(0.72);
            let focused = if offset == 0 { 1.0 } else { 0.0 };
            let hovered = if Some(absolute_idx) == hovered_index { 1.0 } else { 0.0 };

            let mut opacity = (1.0 - 0.12 * distance).max(0.25);
            if !query.trim().is_empty()
                && !entry
                    .name
                    .to_lowercase()
                    .contains(&query.trim().to_lowercase())
            {
                opacity *= 0.35;
            }

            let category = classify_entry(entry);
            let (panel_size, shape_kind) = match category {
                CardCategory::Folder => (glam::vec2(1.16, 1.16), 1.0), // explicit square folder tile
                CardCategory::Executable => (glam::vec2(1.20, 1.08), 2.0),
                CardCategory::File => (glam::vec2(1.08, 1.08), 0.0), // square file tile
                CardCategory::Symlink => (glam::vec2(1.06, 1.06), 0.0),
                CardCategory::Other => (glam::vec2(1.06, 1.06), 0.0),
            };
            SceneCard {
                id: entry.id,
                label: entry.name.clone(),
                category,
                position: glam::vec3(x, y, z),
                rotation: glam::vec3(0.0, yaw, 0.0),
                scale,
                panel_size,
                opacity,
                focus_weight: focused,
                hover_weight: hovered,
                shape_kind,
                target_position: glam::vec3(x, y, z),
            }
        })
        .collect()
}

fn classify_entry(entry: &FsEntry) -> CardCategory {
    match entry.kind {
        EntryKind::Directory => CardCategory::Folder,
        EntryKind::Symlink => CardCategory::Symlink,
        EntryKind::File => {
            let ext = entry
                .extension
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase();
            if matches!(ext.as_str(), "exe" | "bat" | "cmd" | "com" | "msi" | "lnk") {
                CardCategory::Executable
            } else {
                CardCategory::File
            }
        }
        EntryKind::Other => CardCategory::Other,
    }
}
