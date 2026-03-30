use crate::scene::card::SceneCard;

// MVP placeholder for future raycast-based hit testing.
pub fn nearest_card(cards: &[SceneCard]) -> Option<u64> {
    cards
        .iter()
        .max_by(|a, b| a.focus_weight.total_cmp(&b.focus_weight))
        .map(|card| card.id)
}
