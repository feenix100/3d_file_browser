use std::collections::VecDeque;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::domain::events::{Notification, NotificationLevel};
use crate::domain::filters::FilterState;
use crate::domain::selection::SelectionState;
use crate::domain::sort::{sort_entries, SortState};
use crate::fs::entries::{DirectorySnapshot, FsEntry};
use crate::fs::navigation::NavigationState;
use crate::scene::card::SceneCard;

#[derive(Debug, Clone)]
pub struct DirectoryState {
    pub snapshot: Option<DirectorySnapshot>,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SceneState {
    pub visible_cards: Vec<SceneCard>,
    pub max_visible_cards: usize,
    pub previous_positions: HashMap<u64, glam::Vec3>,
    pub reduced_motion: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    Normal,
    Search,
    Rename,
    DeleteConfirm,
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub mode: UiMode,
    pub pending_delete: Option<PathBuf>,
    pub breadcrumbs: Vec<String>,
    pub inspector_text: String,
    pub viewport_size: glam::Vec2,
}

#[derive(Debug, Clone)]
pub struct VisualTheme {
    pub style_panel_open: bool,
    pub text_palette: usize,
    pub outline_palette: usize,
    pub background_box_palette: usize,
}

impl Default for VisualTheme {
    fn default() -> Self {
        Self {
            style_panel_open: false,
            text_palette: 0,
            outline_palette: 0,
            background_box_palette: 0,
        }
    }
}

impl VisualTheme {
    pub fn cycle_text_palette(&mut self) {
        self.text_palette = (self.text_palette + 1) % 5;
    }

    pub fn cycle_outline_palette(&mut self) {
        self.outline_palette = (self.outline_palette + 1) % 9;
    }

    pub fn cycle_background_box_palette(&mut self) {
        self.background_box_palette = (self.background_box_palette + 1) % 5;
    }

    pub fn text_color_rgb(&self) -> [f32; 3] {
        match self.text_palette {
            0 => [0.78, 0.98, 0.88], // mint
            1 => [0.86, 0.95, 1.00], // ice
            2 => [0.96, 0.98, 0.80], // chartreuse white
            3 => [0.95, 0.90, 1.00], // soft violet
            _ => [1.00, 0.90, 0.86], // warm
        }
    }

    pub fn outline_color_rgb(&self) -> [f32; 3] {
        match self.outline_palette {
            0 => [0.08, 0.98, 0.66], // neon mint
            1 => [0.20, 0.90, 1.00], // cyan
            2 => [0.40, 1.00, 0.44], // green
            3 => [0.90, 0.88, 1.00], // lavender
            4 => [1.00, 0.65, 0.35], // amber
            5 => [0.96, 0.28, 0.30], // red
            6 => [1.00, 0.48, 0.16], // orange
            7 => [0.66, 0.40, 1.00], // purple
            _ => [0.24, 0.52, 1.00], // blue
        }
    }

    pub fn background_box_color_rgb(&self) -> [f32; 3] {
        match self.background_box_palette {
            0 => [0.04, 0.50, 0.20], // subtle green
            1 => [0.04, 0.40, 0.55], // teal
            2 => [0.10, 0.38, 0.30], // jade
            3 => [0.20, 0.32, 0.50], // indigo
            _ => [0.42, 0.28, 0.14], // copper
        }
    }

    pub fn text_palette_name(&self) -> &'static str {
        match self.text_palette {
            0 => "MINT",
            1 => "ICE",
            2 => "LIME",
            3 => "VIOLET",
            _ => "WARM",
        }
    }

    pub fn outline_palette_name(&self) -> &'static str {
        match self.outline_palette {
            0 => "NEON MINT",
            1 => "CYAN",
            2 => "GREEN",
            3 => "LAVENDER",
            4 => "AMBER",
            5 => "RED",
            6 => "ORANGE",
            7 => "PURPLE",
            _ => "BLUE",
        }
    }

    pub fn background_box_palette_name(&self) -> &'static str {
        match self.background_box_palette {
            0 => "GREEN",
            1 => "TEAL",
            2 => "JADE",
            3 => "INDIGO",
            _ => "COPPER",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub navigation: NavigationState,
    pub directory: DirectoryState,
    pub selection: SelectionState,
    pub sort: SortState,
    pub filter: FilterState,
    pub scene: SceneState,
    pub ui: UiState,
    pub theme: VisualTheme,
    pub pending_ops: Vec<String>,
    pub notifications: VecDeque<Notification>,
}

impl AppState {
    pub fn new(start_path: PathBuf) -> Self {
        Self {
            navigation: NavigationState::new(start_path),
            directory: DirectoryState {
                snapshot: None,
                loading: false,
                error: None,
            },
            selection: SelectionState::default(),
            sort: SortState::default(),
            filter: FilterState::default(),
            scene: SceneState {
                visible_cards: Vec::new(),
                max_visible_cards: 25,
                previous_positions: HashMap::new(),
                reduced_motion: false,
            },
            ui: UiState {
                mode: UiMode::Normal,
                pending_delete: None,
                breadcrumbs: Vec::new(),
                inspector_text: String::new(),
                viewport_size: glam::vec2(1280.0, 720.0),
            },
            theme: VisualTheme::default(),
            pending_ops: Vec::new(),
            notifications: VecDeque::new(),
        }
    }

    pub fn push_notification(&mut self, level: NotificationLevel, message: String) {
        self.notifications.push_back(Notification { level, message });
        while self.notifications.len() > 5 {
            self.notifications.pop_front();
        }
    }

    pub fn filtered_entries(&self) -> Vec<FsEntry> {
        let mut entries = self
            .directory
            .snapshot
            .as_ref()
            .map(|s| s.entries.clone())
            .unwrap_or_default();
        entries = crate::fs::search::filter_entries(entries, self.filter.query.as_str());
        sort_entries(&mut entries, &self.sort);
        entries
    }

    pub fn filtered_len(&self) -> usize {
        self.filtered_entries().len()
    }

    pub fn selected_entry(&self) -> Option<&FsEntry> {
        let sorted = self.filtered_entries();
        let idx = self.selection.selected_index.min(sorted.len().saturating_sub(1));
        sorted.get(idx).map(|_| ()).and_then(|_| {
            self.directory
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.entries.iter().find(|e| e.id == sorted[idx].id))
        })
    }
}
