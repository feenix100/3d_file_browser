use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

pub fn quad_vertices() -> [Vertex; 6] {
    [
        Vertex {
            position: [-0.5, 0.5, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
        },
    ]
}
