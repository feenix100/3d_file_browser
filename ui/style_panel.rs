#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StylePanelHit {
    None,
    TogglePanel,
    CycleText,
    CycleOutline,
    CycleBackgroundBoxes,
}

pub fn button_rect() -> (f32, f32, f32, f32) {
    let y = panel_rect(720.0).1 - 30.0;
    (14.0, y, 236.0, 34.0)
}

pub fn button_rect_for_height(viewport_height: f32) -> (f32, f32, f32, f32) {
    let y = panel_rect(viewport_height).1 - 42.0;
    (14.0, y.max(10.0), 236.0, 34.0)
}

pub fn panel_rect(viewport_height: f32) -> (f32, f32, f32, f32) {
    let h = 136.0;
    let y = ((viewport_height - h) * 0.5).max(44.0);
    (14.0, y, 372.0, h)
}

pub fn row_rect(row: usize, viewport_height: f32) -> (f32, f32, f32, f32) {
    let (x, y, w, _) = panel_rect(viewport_height);
    (x + 14.0, y + 38.0 + row as f32 * 24.0, w - 28.0, 20.0)
}

pub fn hit_test(x: f32, y: f32, panel_open: bool, viewport_height: f32) -> StylePanelHit {
    if in_rect(x, y, button_rect_for_height(viewport_height)) {
        return StylePanelHit::TogglePanel;
    }
    if !panel_open {
        return StylePanelHit::None;
    }
    if in_rect(x, y, row_rect(0, viewport_height)) {
        return StylePanelHit::CycleText;
    }
    if in_rect(x, y, row_rect(1, viewport_height)) {
        return StylePanelHit::CycleOutline;
    }
    if in_rect(x, y, row_rect(2, viewport_height)) {
        return StylePanelHit::CycleBackgroundBoxes;
    }
    if in_rect(x, y, row_rect(3, viewport_height)) {
        return StylePanelHit::TogglePanel;
    }
    StylePanelHit::None
}

fn in_rect(x: f32, y: f32, rect: (f32, f32, f32, f32)) -> bool {
    let (rx, ry, rw, rh) = rect;
    x >= rx && x <= rx + rw && y >= ry && y <= ry + rh
}
