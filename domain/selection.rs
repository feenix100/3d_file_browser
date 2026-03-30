#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub selected_index: usize,
    pub hovered: Option<usize>,
}

impl SelectionState {
    pub fn move_left(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn move_right(&mut self, len: usize) {
        if len == 0 {
            self.selected_index = 0;
        } else {
            self.selected_index = (self.selected_index + 1).min(len - 1);
        }
    }

    pub fn reset(&mut self) {
        self.selected_index = 0;
        self.hovered = None;
    }

    pub fn clamp(&mut self, len: usize) {
        if len == 0 {
            self.selected_index = 0;
        } else {
            self.selected_index = self.selected_index.min(len - 1);
        }
    }
}
