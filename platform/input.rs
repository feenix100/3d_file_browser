use crate::domain::actions::Action;
use crate::domain::sort::SortKey;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

#[derive(Debug, Default)]
pub struct InputMapper {
    modifiers: ModifiersState,
    cursor_pos: Option<(f32, f32)>,
}

impl InputMapper {
    pub fn map_window_event(&mut self, event: &WindowEvent) -> Vec<Action> {
        let mut out = Vec::new();
        match event {
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                out.extend(map_key(event.physical_key, self.modifiers));
            }
            WindowEvent::Resized(size) => {
                out.push(Action::ViewportResized {
                    width: size.width as f32,
                    height: size.height as f32,
                });
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Some((position.x as f32, position.y as f32));
            }
            WindowEvent::MouseInput { state, button, .. }
                if *state == ElementState::Pressed && *button == MouseButton::Left =>
            {
                if let Some((x, y)) = self.cursor_pos {
                    out.push(Action::ClickAt { x, y });
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let value = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                out.push(Action::Scroll(value));
            }
            _ => {}
        }
        out
    }
}

fn map_key(key: PhysicalKey, modifiers: ModifiersState) -> Vec<Action> {
    if let PhysicalKey::Code(code) = key {
        match code {
            KeyCode::ArrowLeft if modifiers.alt_key() => vec![Action::NavigateBack],
            KeyCode::ArrowRight if modifiers.alt_key() => vec![Action::NavigateForward],
            KeyCode::ArrowLeft => vec![Action::MoveSelectionLeft],
            KeyCode::ArrowRight => vec![Action::MoveSelectionRight],
            KeyCode::Enter => vec![Action::OpenSelected],
            KeyCode::Backspace => vec![Action::GoUp],
            KeyCode::Escape => vec![Action::ClearMode],
            KeyCode::Delete => vec![Action::RequestDelete],
            KeyCode::F2 => vec![Action::BeginRename],
            KeyCode::KeyN if modifiers.control_key() => vec![Action::CreateFolder],
            KeyCode::KeyF if modifiers.control_key() => vec![Action::StartSearch],
            KeyCode::KeyR if modifiers.control_key() => vec![Action::ConfirmRename],
            KeyCode::KeyD if modifiers.control_key() => vec![Action::ConfirmDelete],
            KeyCode::Digit1 => vec![Action::SetSort(SortKey::Name)],
            KeyCode::Digit2 => vec![Action::SetSort(SortKey::Modified)],
            KeyCode::Digit3 => vec![Action::SetSort(SortKey::Size)],
            KeyCode::Digit4 => vec![Action::SetSort(SortKey::Type)],
            KeyCode::F8 => vec![Action::ToggleStylePanel],
            KeyCode::F9 => vec![Action::CycleTextColor],
            KeyCode::F10 => vec![Action::CycleOutlineColor],
            KeyCode::F11 => vec![Action::CycleBackgroundBoxColor],
            _ => vec![Action::Noop],
        }
    } else {
        vec![Action::Noop]
    }
}
