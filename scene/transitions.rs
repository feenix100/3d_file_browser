use crate::scene::card::SceneCard;
use std::collections::HashMap;

pub fn ease_cards(cards: &mut [SceneCard], alpha: f32) {
    for card in cards {
        card.position = card.position.lerp(card.target_position, alpha);
    }
}

pub fn apply_temporal_positions(
    cards: &mut [SceneCard],
    previous_positions: &HashMap<u64, glam::Vec3>,
    reduced_motion: bool,
) {
    let alpha = if reduced_motion { 1.0 } else { 0.22 };
    for card in cards {
        let from = previous_positions
            .get(&card.id)
            .copied()
            .unwrap_or_else(|| card.target_position + glam::vec3(0.0, 0.0, 1.6));
        card.position = from.lerp(card.target_position, alpha);
    }
}
