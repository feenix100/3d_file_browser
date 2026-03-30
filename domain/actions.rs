use std::path::PathBuf;

use super::sort::SortKey;

#[derive(Debug, Clone)]
pub enum Action {
    MoveSelectionLeft,
    MoveSelectionRight,
    Scroll(f32),
    OpenSelected,
    GoUp,
    NavigateBack,
    NavigateForward,
    StartSearch,
    UpdateSearch(String),
    ClearMode,
    SetSort(SortKey),
    CycleSort,
    CreateFolder,
    BeginRename,
    ConfirmRename,
    RequestDelete,
    ConfirmDelete,
    ClickAt { x: f32, y: f32 },
    ViewportResized { width: f32, height: f32 },
    ToggleStylePanel,
    CycleTextColor,
    CycleOutlineColor,
    CycleBackgroundBoxColor,
    HoverIndex(usize),
    SelectIndex(usize),
    ActivateFavorite(PathBuf),
    NavigateToPath(PathBuf),
    Noop,
}
