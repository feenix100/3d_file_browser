use crate::fs::entries::FsEntry;
use crate::scene::card::SceneCard;
use crate::scene::layout::curved_deck_layout;
use crate::scene::transitions::{apply_temporal_positions, ease_cards};
use std::collections::HashMap;

pub fn rebuild_scene_deck(
    entries: Vec<FsEntry>,
    selected_index: usize,
    hovered_index: Option<usize>,
    max_visible: usize,
    query: &str,
    previous_positions: &HashMap<u64, glam::Vec3>,
    reduced_motion: bool,
) -> Vec<SceneCard> {
    let mut cards = curved_deck_layout(&entries, selected_index, max_visible, query, hovered_index);
    apply_temporal_positions(&mut cards, previous_positions, reduced_motion);
    ease_cards(&mut cards, if reduced_motion { 1.0 } else { 0.82 });
    cards
}
