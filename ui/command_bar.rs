use crate::app::state::UiMode;

pub fn mode_hint(mode: UiMode) -> &'static str {
    match mode {
        UiMode::Normal => "Left/Right move | Enter open | Backspace up | Alt+Left/Right history | Ctrl+F search | Ctrl+N new folder | F2 rename | Delete request | 1/2/3/4 sort | Mouse wheel move | Top-left style button (click) | F8 panel | F9 text | F10 outline | F11 bg boxes",
        UiMode::Search => "Search mode | Esc cancel | Left/Right move | Enter open",
        UiMode::Rename => "Rename mode | Ctrl+R confirm rename | Esc cancel",
        UiMode::DeleteConfirm => "Delete mode | Ctrl+D confirm delete | Esc cancel",
    }
}
