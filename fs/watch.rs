use std::path::Path;

// Extension point for live filesystem watching in a later pass.
pub fn watch_path(_path: &Path) {
    // Intentionally empty in MVP pass.
}
