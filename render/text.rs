use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use winit::dpi::PhysicalSize;

use crate::app::state::{AppState, UiMode};
use crate::scene::camera::Camera;
use crate::scene::card::SceneCard;
use crate::ui::style_panel::{button_rect_for_height, panel_rect, row_rect};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl TextVertex {
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub fn build_text_vertices(
    state: &AppState,
    cards: &[SceneCard],
    camera: &Camera,
    size: PhysicalSize<u32>,
) -> Vec<TextVertex> {
    let mut out = Vec::new();
    let width = size.width.max(1) as f32;
    let height = size.height.max(1) as f32;

    // Keep overlay intentionally sparse so the 3D deck remains the focal element.
    build_minimal_overlay_text(&mut out, state, width, height);
    build_selected_and_neighbor_labels(&mut out, state, cards, camera, width, height);
    build_style_panel_button(&mut out, state, width, height);
    if state.theme.style_panel_open {
        build_style_panel(&mut out, state, width, height);
    }
    out
}

fn build_minimal_overlay_text(out: &mut Vec<TextVertex>, state: &AppState, width: f32, height: f32) {
    let base = state.theme.text_color_rgb();
    let secondary = [base[0], base[1], base[2], 0.96];
    append_rect_px(out, 12.0, 10.0, width * 0.48, 20.0, width, height, [0.0, 0.0, 0.0, 0.55]);
    append_rect_px(
        out,
        12.0,
        height - 94.0,
        width * 0.90,
        90.0,
        width,
        height,
        [0.0, 0.0, 0.0, 0.52],
    );

    let sel = state
        .selected_entry()
        .map(|e| sanitize(&e.name, 28))
        .unwrap_or_else(|| "NONE".to_string());
    append_text_readable(
        out,
        20.0,
        18.0,
        &format!("SELECTED > {sel}"),
        3.2,
        width,
        height,
        secondary,
    );

    let mode_line = match state.ui.mode {
        UiMode::Normal => "MODE NORMAL",
        UiMode::Search => "MODE SEARCH CTRL+F TO ENTER ESC TO EXIT",
        UiMode::Rename => "MODE RENAME CTRL+R CONFIRM ESC CANCEL",
        UiMode::DeleteConfirm => "MODE DELETE CTRL+D CONFIRM ESC CANCEL",
    };
    append_text_readable(out, 20.0, height - 86.0, mode_line, 2.45, width, height, secondary);
    append_text_readable(
        out,
        20.0,
        height - 66.0,
        "KEYS LEFT/RIGHT MOVE ENTER OPEN BACKSPACE UP ESC CANCEL",
        2.2,
        width,
        height,
        secondary,
    );
    append_text_readable(
        out,
        20.0,
        height - 50.0,
        "ALT+LEFT/RIGHT HISTORY CTRL+F SEARCH CTRL+N NEW FOLDER",
        2.2,
        width,
        height,
        secondary,
    );
    append_text_readable(
        out,
        20.0,
        height - 34.0,
        "F2 RENAME DELETE ASK DELETE CTRL+R DO RENAME CTRL+D DELETE",
        2.2,
        width,
        height,
        secondary,
    );
    append_text_readable(
        out,
        20.0,
        height - 18.0,
        "SORT 1 NAME 2 MODIFIED 3 SIZE 4 TYPE F8/F9/F10/F11 STYLE",
        2.2,
        width,
        height,
        secondary,
    );
}

fn build_selected_and_neighbor_labels(
    out: &mut Vec<TextVertex>,
    state: &AppState,
    cards: &[SceneCard],
    camera: &Camera,
    width: f32,
    height: f32,
) {
    let vp = camera.view_proj();
    for card in cards {
        if card.focus_weight < 0.9 && card.hover_weight < 0.5 && card.opacity < 0.7 {
            continue;
        }
        let world = Vec3::new(card.position.x, card.position.y + 0.94, -card.position.z);
        if let Some((sx, sy)) = project_world_to_screen(vp, world, width, height) {
            let selected = card.focus_weight > 0.9;
            let base = state.theme.text_color_rgb();
            let color = if selected {
                [
                    (base[0] + 0.20).min(1.0),
                    (base[1] + 0.20).min(1.0),
                    (base[2] + 0.20).min(1.0),
                    1.0,
                ]
            } else {
                [base[0], base[1], base[2], 0.96]
            };
            let scale = if selected { 4.1 } else { 3.45 };
            let label = sanitize(&card.label, if selected { 28 } else { 20 });
            append_text_readable(out, sx - 78.0, sy, &label, scale, width, height, color);
        }
    }
}

fn build_style_panel_button(out: &mut Vec<TextVertex>, state: &AppState, width: f32, height: f32) {
    let (x, y, w, h) = button_rect_for_height(height);
    let base = state.theme.text_color_rgb();
    let text = [base[0], base[1], base[2], 0.98];
    let bg = if state.theme.style_panel_open {
        [0.02, 0.08, 0.06, 0.82]
    } else {
        [0.0, 0.0, 0.0, 0.72]
    };
    append_rect_px(out, x, y, w, h, width, height, bg);
    append_rect_px(out, x, y, w, 2.0, width, height, [base[0], base[1], base[2], 0.90]);
    append_rect_px(
        out,
        x,
        y + h - 2.0,
        w,
        2.0,
        width,
        height,
        [base[0], base[1], base[2], 0.85],
    );
    append_text_readable(
        out,
        x + 12.0,
        y + 9.0,
        "STYLE PANEL  CLICK OR F8",
        2.05,
        width,
        height,
        text,
    );
}

fn build_style_panel(out: &mut Vec<TextVertex>, state: &AppState, width: f32, height: f32) {
    let (x, y, mut w, h) = panel_rect(height);
    w = w.min(width - x - 12.0);
    let base = state.theme.text_color_rgb();
    let text_color = [base[0], base[1], base[2], 0.98];

    append_rect_px(out, x, y, w, h, width, height, [0.0, 0.0, 0.0, 0.74]);
    append_rect_px(out, x, y, w, 2.0, width, height, [base[0], base[1], base[2], 0.85]);
    append_rect_px(out, x, y + h - 2.0, w, 2.0, width, height, [base[0], base[1], base[2], 0.80]);
    append_rect_px(out, x, y, 2.0, h, width, height, [base[0], base[1], base[2], 0.80]);
    append_rect_px(out, x + w - 2.0, y, 2.0, h, width, height, [base[0], base[1], base[2], 0.80]);

    append_text_readable(out, x + 14.0, y + 14.0, "STYLE PANEL", 2.0, width, height, text_color);
    let (r0x, r0y, r0w, r0h) = row_rect(0, height);
    append_rect_px(out, r0x, r0y, r0w, r0h, width, height, [0.0, 0.0, 0.0, 0.42]);
    let (r1x, r1y, r1w, r1h) = row_rect(1, height);
    append_rect_px(out, r1x, r1y, r1w, r1h, width, height, [0.0, 0.0, 0.0, 0.42]);
    let (r2x, r2y, r2w, r2h) = row_rect(2, height);
    append_rect_px(out, r2x, r2y, r2w, r2h, width, height, [0.0, 0.0, 0.0, 0.42]);
    let (r3x, r3y, r3w, r3h) = row_rect(3, height);
    append_rect_px(out, r3x, r3y, r3w, r3h, width, height, [0.0, 0.0, 0.0, 0.42]);
    append_text_readable(
        out,
        x + 16.0,
        y + 42.0,
        &format!("CLICK OR F9 TEXT COLOR: {}", state.theme.text_palette_name()),
        1.95,
        width,
        height,
        text_color,
    );
    append_text_readable(
        out,
        x + 16.0,
        y + 66.0,
        &format!("CLICK OR F10 OUTLINE: {}", state.theme.outline_palette_name()),
        1.95,
        width,
        height,
        text_color,
    );
    append_text_readable(
        out,
        x + 16.0,
        y + 90.0,
        &format!("CLICK OR F11 BG BOXES: {}", state.theme.background_box_palette_name()),
        1.95,
        width,
        height,
        text_color,
    );
    append_text_readable(
        out,
        x + 16.0,
        y + 114.0,
        "CLICK HERE OR PRESS F8 TO CLOSE",
        1.85,
        width,
        height,
        text_color,
    );
}

fn sanitize(raw: &str, max: usize) -> String {
    raw.chars()
        .map(|c| {
            let up = c.to_ascii_uppercase();
            if up.is_ascii_alphanumeric() || " >-_.:/\\[]()+=".contains(up) {
                up
            } else {
                ' '
            }
        })
        .collect::<String>()
        .chars()
        .take(max)
        .collect()
}

fn project_world_to_screen(vp: glam::Mat4, world: Vec3, width: f32, height: f32) -> Option<(f32, f32)> {
    let clip = vp * Vec4::new(world.x, world.y, world.z, 1.0);
    if clip.w <= 0.01 {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    if ndc.z < -1.0 || ndc.z > 1.0 {
        return None;
    }
    let sx = (ndc.x * 0.5 + 0.5) * width;
    let sy = (1.0 - (ndc.y * 0.5 + 0.5)) * height;
    Some((sx, sy))
}

fn append_text_px(
    out: &mut Vec<TextVertex>,
    x: f32,
    y: f32,
    text: &str,
    scale: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
) {
    let mut cursor = x.round();
    let y = y.round();
    for ch in text.chars().take(64) {
        let glyph = glyph_3x5(ch);
        for row in 0..5 {
            for col in 0..3 {
                let bit_index = 14 - (row * 3 + col);
                let bit = (glyph >> bit_index) & 1;
                if bit == 1 {
                    append_rect_px(
                        out,
                        (cursor + col as f32 * scale).round(),
                        y + row as f32 * scale,
                        scale * 0.90,
                        scale * 0.90,
                        width,
                        height,
                        color,
                    );
                }
            }
        }
        cursor += scale * 4.0;
    }
}

fn append_text_readable(
    out: &mut Vec<TextVertex>,
    x: f32,
    y: f32,
    text: &str,
    scale: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
) {
    append_text_px(
        out,
        x + scale * 0.55,
        y + scale * 0.55,
        text,
        scale,
        width,
        height,
        [0.0, 0.0, 0.0, (color[3] * 0.65).min(1.0)],
    );
    append_text_px(
        out,
        x + scale * 0.35,
        y + scale * 0.35,
        text,
        scale,
        width,
        height,
        [0.0, 0.0, 0.0, (color[3] * 0.85).min(1.0)],
    );
    append_text_px(out, x, y, text, scale, width, height, color);
}

fn append_rect_px(
    out: &mut Vec<TextVertex>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
) {
    let x = x.round();
    let y = y.round();
    let w = w.round().max(1.0);
    let h = h.round().max(1.0);
    let x0 = (x / width) * 2.0 - 1.0;
    let y0 = 1.0 - (y / height) * 2.0;
    let x1 = ((x + w) / width) * 2.0 - 1.0;
    let y1 = 1.0 - ((y + h) / height) * 2.0;

    out.push(TextVertex { position: [x0, y0], color });
    out.push(TextVertex { position: [x1, y0], color });
    out.push(TextVertex { position: [x0, y1], color });
    out.push(TextVertex { position: [x1, y0], color });
    out.push(TextVertex { position: [x1, y1], color });
    out.push(TextVertex { position: [x0, y1], color });
}

fn glyph_3x5(c: char) -> u16 {
    match c {
        'A' => 0b010101111101101,
        'B' => 0b110101110101110,
        'C' => 0b011100100100011,
        'D' => 0b110101101101110,
        'E' => 0b111100110100111,
        'F' => 0b111100110100100,
        'G' => 0b011100101101011,
        'H' => 0b101101111101101,
        'I' => 0b111010010010111,
        'J' => 0b001001001101010,
        'K' => 0b101101110101101,
        'L' => 0b100100100100111,
        'M' => 0b101111101101101,
        'N' => 0b101111111111101,
        'O' => 0b010101101101010,
        'P' => 0b110101110100100,
        'Q' => 0b010101101111011,
        'R' => 0b110101110110101,
        'S' => 0b011100010001110,
        'T' => 0b111010010010010,
        'U' => 0b101101101101111,
        'V' => 0b101101101101010,
        'W' => 0b101101111111101,
        'X' => 0b101101010101101,
        'Y' => 0b101101010010010,
        'Z' => 0b111001010100111,
        '0' => 0b111101101101111,
        '1' => 0b010110010010111,
        '2' => 0b111001111100111,
        '3' => 0b111001111001111,
        '4' => 0b101101111001001,
        '5' => 0b111100111001111,
        '6' => 0b111100111101111,
        '7' => 0b111001001001001,
        '8' => 0b111101111101111,
        '9' => 0b111101111001111,
        '>' => 0b100010001010100,
        '-' => 0b000000111000000,
        '_' => 0b000000000000111,
        ':' => 0b000010000010000,
        '.' => 0b000000000000010,
        '/' => 0b001001010100100,
        '\\' => 0b100100010001001,
        '[' => 0b011010010010011,
        ']' => 0b110010010010110,
        '+' => 0b000010111010000,
        '=' => 0b000111000111000,
        '(' => 0b001010010010001,
        ')' => 0b100010010010100,
        ' ' => 0,
        _ => 0,
    }
}
