#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardCategory {
    Folder,
    File,
    Executable,
    Symlink,
    Other,
}

#[derive(Debug, Clone)]
pub struct SceneCard {
    pub id: u64,
    pub label: String,
    pub category: CardCategory,
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
    pub scale: f32,
    pub panel_size: glam::Vec2,
    pub opacity: f32,
    pub focus_weight: f32,
    pub hover_weight: f32,
    pub shape_kind: f32,
    pub target_position: glam::Vec3,
}
