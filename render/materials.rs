use crate::scene::card::CardCategory;

#[derive(Debug, Clone, Copy)]
pub struct HologramMaterial {
    pub tint: [f32; 3],
    pub fill_alpha: f32,
    pub edge_strength: f32,
    pub shimmer_rate: f32,
}

pub fn hologram_material(
    category: CardCategory,
    focus_weight: f32,
    hover_weight: f32,
) -> HologramMaterial {
    let (base_tint, base_alpha, edge, shimmer) = match category {
        CardCategory::Folder => ([0.04, 1.00, 0.78], 0.36, 1.40, 1.55),
        CardCategory::File => ([0.10, 0.95, 0.36], 0.32, 1.22, 1.40),
        CardCategory::Executable => ([0.20, 1.00, 0.62], 0.40, 1.52, 1.70),
        CardCategory::Symlink => ([0.10, 0.78, 0.95], 0.28, 1.08, 1.12),
        CardCategory::Other => ([0.12, 0.70, 0.82], 0.24, 0.98, 1.00),
    };

    let selection_boost = 1.0 + focus_weight * 1.05 + hover_weight * 0.48;
    HologramMaterial {
        tint: [
            (base_tint[0] * selection_boost).min(1.3),
            (base_tint[1] * selection_boost).min(1.3),
            (base_tint[2] * selection_boost).min(1.3),
        ],
        fill_alpha: (base_alpha + focus_weight * 0.24 + hover_weight * 0.11).min(0.78),
        edge_strength: edge * selection_boost,
        shimmer_rate: shimmer,
    }
}
