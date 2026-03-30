pub mod commands;
pub mod state;

use std::path::PathBuf;

use crate::domain::actions::Action;
use crate::domain::events::NotificationLevel;
use crate::domain::sort::SortKey;
use crate::fs::navigation::go_up_path;
use crate::fs::{FsRequest, FsResponse, FsService};
use crate::platform::os::normalize_path;
use crate::scene::deck::rebuild_scene_deck;
use crate::ui::breadcrumbs::path_to_breadcrumbs;
use crate::ui::inspector::selected_entry_summary;
use crate::ui::style_panel::{hit_test, StylePanelHit};

use self::commands::AppCommand;
use self::state::{AppState, UiMode};

pub struct App {
    pub state: AppState,
    fs: FsService,
}

impl App {
    pub fn new(start_path: PathBuf) -> Self {
        let normalized = normalize_path(&start_path);
        Self {
            state: AppState::new(normalized),
            fs: FsService::start(),
        }
    }

    pub fn dispatch(&mut self, command: AppCommand) {
        match command {
            AppCommand::LoadDirectory(path) => {
                self.state.directory.loading = true;
                self.fs.send(FsRequest::LoadDirectory(path));
            }
            AppCommand::CreateFolder { parent, name } => {
                self.state.pending_ops.push("Create folder".to_string());
                self.fs.send(FsRequest::CreateFolder { parent, name });
            }
            AppCommand::Rename { from, to_name } => {
                self.state.pending_ops.push("Rename".to_string());
                self.fs.send(FsRequest::Rename { from, to_name });
            }
            AppCommand::Delete { path } => {
                self.state.pending_ops.push("Delete".to_string());
                self.fs.send(FsRequest::Delete { path });
            }
        }
    }

    pub fn tick(&mut self) {
        while let Some(resp) = self.fs.try_recv() {
            self.handle_fs_response(resp);
        }
        self.refresh_scene();
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::MoveSelectionLeft => self.state.selection.move_left(),
            Action::MoveSelectionRight => self.state.selection.move_right(self.state.filtered_len()),
            Action::Scroll(delta) => {
                if delta < 0.0 {
                    self.state.selection.move_right(self.state.filtered_len());
                } else if delta > 0.0 {
                    self.state.selection.move_left();
                }
            }
            Action::OpenSelected => self.open_selected(),
            Action::GoUp => self.go_up(),
            Action::NavigateBack => self.navigate_back(),
            Action::NavigateForward => self.navigate_forward(),
            Action::StartSearch => self.state.ui.mode = UiMode::Search,
            Action::UpdateSearch(query) => {
                self.state.filter.query = query;
                self.state.selection.clamp(self.state.filtered_len());
            }
            Action::ClearMode => {
                self.state.ui.mode = UiMode::Normal;
                self.state.ui.pending_delete = None;
            }
            Action::SetSort(key) => {
                self.state.sort.key = key;
                self.state.sort.toggle_direction();
            }
            Action::CycleSort => {
                self.state.sort.key = match self.state.sort.key {
                    SortKey::Name => SortKey::Modified,
                    SortKey::Modified => SortKey::Size,
                    SortKey::Size => SortKey::Type,
                    SortKey::Type => SortKey::Name,
                };
            }
            Action::CreateFolder => {
                let parent = self.state.navigation.current_path.clone();
                let name = "New Folder".to_string();
                self.dispatch(AppCommand::CreateFolder { parent, name });
            }
            Action::BeginRename => {
                self.state.ui.mode = UiMode::Rename;
            }
            Action::ConfirmRename => {
                if let Some(entry) = self.state.selected_entry() {
                    self.dispatch(AppCommand::Rename {
                        from: entry.path.clone(),
                        to_name: format!("{}_renamed", entry.name),
                    });
                    self.state.ui.mode = UiMode::Normal;
                }
            }
            Action::RequestDelete => {
                if let Some(entry) = self.state.selected_entry() {
                    self.state.ui.pending_delete = Some(entry.path.clone());
                    self.state.ui.mode = UiMode::DeleteConfirm;
                }
            }
            Action::ConfirmDelete => {
                if let Some(path) = self.state.ui.pending_delete.take() {
                    self.dispatch(AppCommand::Delete { path });
                    self.state.ui.mode = UiMode::Normal;
                }
            }
            Action::ClickAt { x, y } => match hit_test(
                x,
                y,
                self.state.theme.style_panel_open,
                self.state.ui.viewport_size.y,
            ) {
                StylePanelHit::None => {}
                StylePanelHit::TogglePanel => {
                    self.state.theme.style_panel_open = !self.state.theme.style_panel_open
                }
                StylePanelHit::CycleText => self.state.theme.cycle_text_palette(),
                StylePanelHit::CycleOutline => self.state.theme.cycle_outline_palette(),
                StylePanelHit::CycleBackgroundBoxes => {
                    self.state.theme.cycle_background_box_palette()
                }
            },
            Action::ViewportResized { width, height } => {
                self.state.ui.viewport_size = glam::vec2(width.max(1.0), height.max(1.0));
            }
            Action::ToggleStylePanel => {
                self.state.theme.style_panel_open = !self.state.theme.style_panel_open;
            }
            Action::CycleTextColor => {
                self.state.theme.cycle_text_palette();
            }
            Action::CycleOutlineColor => {
                self.state.theme.cycle_outline_palette();
            }
            Action::CycleBackgroundBoxColor => {
                self.state.theme.cycle_background_box_palette();
            }
            Action::HoverIndex(idx) => self.state.selection.hovered = Some(idx),
            Action::SelectIndex(idx) => self.state.selection.selected_index = idx,
            Action::ActivateFavorite(path) => self.navigate_to(path),
            Action::NavigateToPath(path) => self.navigate_to(path),
            Action::Noop => {}
        }
    }

    fn open_selected(&mut self) {
        let Some(entry) = self.state.selected_entry().cloned() else {
            return;
        };

        if entry.kind.is_dir() {
            self.navigate_to(entry.path);
            return;
        }

        match crate::platform::shell::open_with_default_app(&entry.path) {
            Ok(_) => self.state.push_notification(
                NotificationLevel::Info,
                format!("Opened {}", entry.name),
            ),
            Err(err) => self
                .state
                .push_notification(NotificationLevel::Error, err.to_string()),
        }
    }

    fn go_up(&mut self) {
        if let Some(up) = go_up_path(&self.state.navigation.current_path) {
            self.navigate_to(up);
        }
    }

    fn navigate_to(&mut self, path: PathBuf) {
        let path = normalize_path(&path);
        if path == self.state.navigation.current_path {
            return;
        }

        self.state
            .navigation
            .visit(path.clone(), self.state.navigation.current_path.clone());
        self.dispatch(AppCommand::LoadDirectory(path));
    }

    fn navigate_back(&mut self) {
        if let Some(path) = self.state.navigation.back() {
            self.dispatch(AppCommand::LoadDirectory(path));
        }
    }

    fn navigate_forward(&mut self) {
        if let Some(path) = self.state.navigation.forward() {
            self.dispatch(AppCommand::LoadDirectory(path));
        }
    }

    fn handle_fs_response(&mut self, response: FsResponse) {
        match response {
            FsResponse::DirectoryLoaded(result) => {
                self.state.directory.loading = false;
                match result {
                    Ok(snapshot) => {
                        self.state.navigation.current_path = snapshot.path.clone();
                        self.state.directory.snapshot = Some(snapshot);
                        self.state.selection.reset();
                        self.state.directory.error = None;
                    }
                    Err(err) => {
                        self.state.directory.error = Some(err.to_string());
                        self.state
                            .push_notification(NotificationLevel::Error, err.to_string());
                    }
                }
            }
            FsResponse::OperationFinished(result) => {
                if !self.state.pending_ops.is_empty() {
                    self.state.pending_ops.remove(0);
                }
                match result {
                    Ok(message) => {
                        self.state
                            .push_notification(NotificationLevel::Info, message);
                        self.dispatch(AppCommand::LoadDirectory(
                            self.state.navigation.current_path.clone(),
                        ));
                    }
                    Err(err) => self
                        .state
                        .push_notification(NotificationLevel::Error, err.to_string()),
                }
            }
        }
    }

    fn refresh_scene(&mut self) {
        let previous_positions = self.state.scene.previous_positions.clone();
        let cards = rebuild_scene_deck(
            self.state.filtered_entries(),
            self.state.selection.selected_index,
            self.state.selection.hovered,
            self.state.scene.max_visible_cards,
            self.state.filter.query.as_str(),
            &previous_positions,
            self.state.scene.reduced_motion,
        );
        self.state.scene.previous_positions = cards
            .iter()
            .map(|c| (c.id, c.position))
            .collect();
        self.state.scene.visible_cards = cards;
        self.state.ui.breadcrumbs = path_to_breadcrumbs(&self.state.navigation.current_path);
        self.state.ui.inspector_text = selected_entry_summary(self.state.selected_entry());
    }
}
