use crate::app::state::AppState;

pub fn overlay_lines(state: &AppState) -> Vec<String> {
    let mut out = Vec::new();
    out.push(format!(
        "Path Spine: {}",
        state.navigation.current_path.display()
    ));
    out.push(format!("Mode: {:?}", state.ui.mode));
    out.push(format!(
        "Filter Beam: {}",
        if state.filter.query.is_empty() {
            "(none)"
        } else {
            state.filter.query.as_str()
        }
    ));
    out.push(format!(
        "Deck: {} visible / cap {}",
        state.scene.visible_cards.len(),
        state.scene.max_visible_cards
    ));
    if let Some(note) = state.notifications.back() {
        out.push(format!("Notification: {}", note.message));
    }
    out
}
