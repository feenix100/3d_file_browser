use std::sync::Arc;
use std::time::Instant;

use anyhow::{anyhow, Context};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::app::state::AppState;
use crate::render::geometry::{quad_vertices, Vertex};
use crate::render::materials::hologram_material;
use crate::render::text::{build_text_vertices, TextVertex};
use crate::scene::camera::Camera;

const SHADER: &str = r#"
struct Global {
  view_proj: mat4x4<f32>,
  time_pack: vec4<f32>,
  outline_color: vec4<f32>,
  bg_box_color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> global: Global;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) model_0: vec4<f32>,
  @location(2) model_1: vec4<f32>,
  @location(3) model_2: vec4<f32>,
  @location(4) model_3: vec4<f32>,
  @location(5) tint_alpha: vec4<f32>,
  @location(6) style: vec4<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) local_pos: vec2<f32>,
  @location(1) tint_alpha: vec4<f32>,
  @location(2) edge_strength: f32,
  @location(3) shimmer_rate: f32,
  @location(4) world_z: f32,
  @location(5) shape_kind: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  let model = mat4x4<f32>(in.model_0, in.model_1, in.model_2, in.model_3);
  let world = model * vec4<f32>(in.position, 1.0);

  var out: VertexOutput;
  out.position = global.view_proj * world;
  out.local_pos = in.position.xy;
  out.tint_alpha = in.tint_alpha;
  out.edge_strength = in.style.x;
  out.shimmer_rate = in.style.y;
  out.world_z = world.z;
  out.shape_kind = in.style.z;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let uv = in.local_pos;
  let radius = length(uv);
  let is_folder = in.shape_kind > 0.5 && in.shape_kind < 1.5;
  let is_app = in.shape_kind >= 1.5;
  let edge_margin = select(0.445, 0.465, is_folder);
  let border_width = select(0.060, 0.045, is_folder);
  let box_dist = max(abs(uv.x), abs(uv.y));
  let edge = smoothstep(edge_margin - border_width, edge_margin, box_dist);
  let edge_outer = smoothstep(edge_margin + 0.020, edge_margin + 0.055, box_dist);
  let interior_mask = 1.0 - smoothstep(edge_margin, edge_margin + 0.02, box_dist);
  let inner_ring = 1.0 - smoothstep(0.34, 0.37, box_dist);
  let border_line = smoothstep(edge_margin - 0.010, edge_margin - 0.002, box_dist)
      - smoothstep(edge_margin - 0.002, edge_margin + 0.004, box_dist);
  let border_line_outer = smoothstep(edge_margin + 0.004, edge_margin + 0.012, box_dist)
      - smoothstep(edge_margin + 0.012, edge_margin + 0.022, box_dist);
  let corner_mask = smoothstep(0.30, 0.55, abs(uv.x) * abs(uv.y) * 4.2) * edge;
  let corner_spark = smoothstep(0.80, 1.0, sin((uv.x - uv.y) * 38.0 + global.time_pack.x * 3.4) * 0.5 + 0.5);

  // Tiny top notch for executable/app cards.
  let notch = select(0.0, (1.0 - smoothstep(0.0, 0.12, abs(uv.x))) * smoothstep(0.40, 0.50, uv.y), is_app);

  let t = global.time_pack.x;
  let stripe = sin((uv.y + t * 0.55 * in.shimmer_rate) * 48.0) * 0.5 + 0.5;
  let scanline = 0.5 + 0.5 * sin((uv.y * 190.0) - t * 22.0 * in.shimmer_rate);
  let jitter = sin((uv.y * 71.0) + t * 14.0 * in.shimmer_rate + in.world_z * 0.1);
  let sweep = smoothstep(0.25, 1.0, sin(t * in.shimmer_rate + uv.x * 5.4 + jitter * 0.22) * 0.5 + 0.5);
  let shimmer = 0.012 + 0.042 * stripe * sweep + 0.018 * scanline;
  let glitch = smoothstep(0.92, 1.0, sin(t * 7.0 + uv.y * 58.0 + in.world_z * 0.03) * 0.5 + 0.5) * 0.04;
  let portal_swirl = smoothstep(0.44, 0.04, radius) * (0.5 + 0.5 * sin((radius * 46.0) - (t * 9.0 * in.shimmer_rate) + uv.x * 9.0));
  let portal_rings = (1.0 - smoothstep(0.10, 0.46, radius)) * (0.5 + 0.5 * sin(radius * 72.0 - t * 12.0));
  let core = smoothstep(0.15, 0.0, radius);
  let portal_energy = portal_swirl * 0.26 + portal_rings * 0.18 + core * 0.33;

  let interior = interior_mask * in.tint_alpha.a * 1.10 + inner_ring * in.tint_alpha.a * 0.24;
  let edge_glow = edge * (0.30 + 0.44 * in.edge_strength)
    + border_line * 0.50
    + border_line_outer * 0.36
    + edge_outer * 0.20;
  let corner_glow = corner_mask * (0.16 + 0.24 * corner_spark);
  let alpha = clamp(interior + edge_glow + notch * 0.30 + shimmer + glitch + portal_energy * 0.6, 0.24, 0.98);

  let vignette = 1.0 - clamp(dot(uv, uv) * 0.85, 0.0, 0.30);
  let pulse = 0.90 + 0.14 * sin(t * 2.8 + in.world_z * 0.05);
  let outline_band = edge * 1.25 + border_line * 1.55 + border_line_outer * 1.60 + edge_outer * 0.95 + corner_glow * 1.60;
  let portal_tint = global.outline_color.rgb * (portal_energy * 0.44 + core * 0.16);
  let outline_tint = global.outline_color.rgb * outline_band;
  let base_fill = in.tint_alpha.rgb * (0.16 + interior_mask * 0.50 + shimmer * 0.25 + notch * 0.22);
  let color = (base_fill + portal_tint + outline_tint) * vignette * pulse;
  return vec4<f32>(color, alpha);
}
"#;

const BACKGROUND_SHADER: &str = r#"
struct Global {
  view_proj: mat4x4<f32>,
  time_pack: vec4<f32>,
  outline_color: vec4<f32>,
  bg_box_color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> global: Global;

struct VsOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

fn rand1(x: f32) -> f32 {
  return fract(sin(x) * 43758.5453);
}

fn box_center(i: i32, t: f32) -> vec2<f32> {
  let id = f32(i);
  let speed = 0.24 + rand1(id * 13.17 + 1.2) * 0.62;
  let phase_x = rand1(id * 5.11 + 7.0) * 6.2831;
  let phase_y = rand1(id * 9.73 + 3.4) * 6.2831;
  let amp_x = 0.16 + rand1(id * 3.91 + 2.8) * 0.24;
  let amp_y = 0.14 + rand1(id * 6.43 + 4.6) * 0.22;
  return vec2<f32>(
    0.5 + sin(t * speed + phase_x) * amp_x,
    0.5 + cos(t * speed * 0.86 + phase_y) * amp_y
  );
}

fn box_base_size(i: i32) -> f32 {
  let id = f32(i);
  return 0.055 + rand1(id * 17.27 + 9.1) * 0.070;
}

fn collision_intensity(i: i32, t: f32) -> f32 {
  let c = box_center(i, t);
  let s = box_base_size(i);
  var k = 0.0;
  for (var j: i32 = 0; j < 8; j = j + 1) {
    if (j == i) {
      continue;
    }
    let cj = box_center(j, t);
    let sj = box_base_size(j);
    let d = distance(c, cj);
    let overlap = (s + sj) * 1.28 - d;
    k = k + smoothstep(0.0, 0.12, overlap);
  }
  return clamp(k / 3.0, 0.0, 1.0);
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0),
  );
  let p = pos[vid];
  var out: VsOut;
  out.position = vec4<f32>(p, 0.0, 1.0);
  out.uv = p * 0.5 + vec2<f32>(0.5, 0.5);
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let uv = in.uv;
  let t = global.time_pack.x;
  let p = uv - vec2<f32>(0.5, 0.5);

  var outline = 0.0;
  var collision_glow = 0.0;

  for (var i: i32 = 0; i < 8; i = i + 1) {
    let center = box_center(i, t);
    let collision = collision_intensity(i, t);
    let base_size = box_base_size(i);
    let pulse = 0.5 + 0.5 * sin(t * (0.8 + f32(i) * 0.11) + f32(i) * 1.7);
    let half = base_size * (0.88 + pulse * 0.22 + collision * 0.85);

    let thickness = 0.0016 + rand1(f32(i) * 8.31 + 2.0) * 0.0012 + collision * 0.0017;
    let d = max(abs(uv.x - center.x), abs(uv.y - center.y));
    let outer = 1.0 - smoothstep(half, half + thickness, d);
    let inner = 1.0 - smoothstep(half - thickness, half, d);
    let edge = clamp(outer - inner, 0.0, 1.0);

    let near_box = 1.0 - smoothstep(half + 0.05, half + 0.20, d);
    outline = outline + edge * (0.28 + collision * 0.40);
    collision_glow = collision_glow + near_box * collision * 0.08;
  }

  let vignette = 1.0 - smoothstep(0.30, 0.92, length(p));
  let grain = rand1(dot(floor(uv * 900.0), vec2<f32>(1.0, 57.0)) + t * 45.0) * 0.008;
  let scan = (0.5 + 0.5 * sin(uv.y * 1400.0 - t * 12.0)) * 0.008;

  let base = vec3<f32>(0.0, 0.0, 0.0);
  let edge_tint = global.bg_box_color.rgb * outline;
  let collide_tint = global.bg_box_color.rgb * (0.55 * collision_glow);
  let color = base + (edge_tint + collide_tint) * vignette + vec3<f32>(grain + scan);
  return vec4<f32>(color, 1.0);
}
"#;

const TEXT_SHADER: &str = r#"
struct VsIn {
  @location(0) position: vec2<f32>,
  @location(1) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(v: VsIn) -> VsOut {
  var out: VsOut;
  out.position = vec4<f32>(v.position, 0.0, 1.0);
  out.color = v.color;
  return out;
}

@fragment
fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
  return v.color;
}
"#;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct GlobalUniform {
    view_proj: [[f32; 4]; 4],
    time_pack: [f32; 4],
    outline_color: [f32; 4],
    bg_box_color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    tint_alpha: [f32; 4],
    style: [f32; 4], // edge_strength, shimmer_rate, shape_kind, reserved
}

impl InstanceRaw {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 80,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    bg_pipeline: wgpu::RenderPipeline,
    pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    text_vertex_buffer: wgpu::Buffer,
    instance_capacity: usize,
    text_vertex_capacity: usize,
    camera: Camera,
    global_buffer: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
    started: Instant,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, size: PhysicalSize<u32>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .context("failed to create WGPU surface")?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("no suitable graphics adapter found"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .context("failed to create WGPU device")?;

        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera = Camera::new(config.width, config.height);
        let global_uniform = GlobalUniform {
            view_proj: camera.view_proj().to_cols_array_2d(),
            time_pack: [0.0, 0.0, 0.0, 0.0],
            outline_color: [0.08, 0.98, 0.66, 0.0],
            bg_box_color: [0.04, 0.50, 0.20, 0.0],
        };
        let global_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("global-buffer"),
            contents: bytemuck::bytes_of(&global_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("global-bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("global-bg"),
            layout: &global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hologram-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let bg_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("background-shader"),
            source: wgpu::ShaderSource::Wgsl(BACKGROUND_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline-layout"),
            bind_group_layouts: &[&global_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hologram-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout(), InstanceRaw::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let bg_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("background-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &bg_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &bg_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text-shader"),
            source: wgpu::ShaderSource::Wgsl(TEXT_SHADER.into()),
        });
        let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text-pipeline-layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text-pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_shader,
                entry_point: "vs_main",
                buffers: &[TextVertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertices = quad_vertices();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_capacity = 256usize;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance-buffer"),
            size: (instance_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let text_vertex_capacity = 8192usize;
        let text_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text-vertex-buffer"),
            size: (text_vertex_capacity * std::mem::size_of::<TextVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            bg_pipeline,
            pipeline,
            text_pipeline,
            vertex_buffer,
            instance_buffer,
            text_vertex_buffer,
            instance_capacity,
            text_vertex_capacity,
            camera,
            global_buffer,
            global_bind_group,
            started: Instant::now(),
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.camera.aspect = size.width as f32 / size.height as f32;
    }

    pub fn render(&mut self, state: &AppState) -> anyhow::Result<()> {
        let cards = &state.scene.visible_cards;
        self.ensure_instance_capacity(cards.len().max(1));
        self.update_global_uniform(state);

        let instance_data = cards
            .iter()
            .map(|card| {
                let translation = Mat4::from_translation(Vec3::new(
                    card.position.x,
                    card.position.y,
                    -card.position.z,
                ));
                let rotation = Mat4::from_quat(Quat::from_euler(
                    glam::EulerRot::XYZ,
                    card.rotation.x,
                    card.rotation.y,
                    card.rotation.z,
                ));
                let scale = Mat4::from_scale(Vec3::new(
                    card.scale * card.panel_size.x,
                    card.scale * card.panel_size.y,
                    1.0,
                ));
                let model = translation * rotation * scale;
                let mat = hologram_material(
                    card.category,
                    card.focus_weight,
                    card.hover_weight,
                );
                InstanceRaw {
                    model: model.to_cols_array_2d(),
                    tint_alpha: [mat.tint[0], mat.tint[1], mat.tint[2], card.opacity * mat.fill_alpha],
                    style: [mat.edge_strength, mat.shimmer_rate, card.shape_kind, 0.0],
                }
            })
            .collect::<Vec<_>>();

        if !instance_data.is_empty() {
            self.queue
                .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
        }
        let text_vertices = build_text_vertices(state, cards, &self.camera, self.size);
        self.ensure_text_capacity(text_vertices.len().max(1));
        if !text_vertices.is_empty() {
            self.queue
                .write_buffer(&self.text_vertex_buffer, 0, bytemuck::cast_slice(&text_vertices));
        }

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) => {
                self.resize(self.size);
                return Ok(());
            }
            Err(wgpu::SurfaceError::Outdated) => return Ok(()),
            Err(err) => return Err(anyhow!("surface error: {err:?}")),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render-encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.bg_pipeline);
            rpass.set_bind_group(0, &self.global_bind_group, &[]);
            rpass.draw(0..3, 0..1);
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.global_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            rpass.draw(0..6, 0..instance_data.len() as u32);
        }
        if !text_vertices.is_empty() {
            let mut text_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            text_pass.set_pipeline(&self.text_pipeline);
            text_pass.set_vertex_buffer(0, self.text_vertex_buffer.slice(..));
            text_pass.draw(0..text_vertices.len() as u32, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    fn ensure_instance_capacity(&mut self, needed: usize) {
        if needed <= self.instance_capacity {
            return;
        }
        self.instance_capacity = needed.next_power_of_two();
        self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance-buffer-grown"),
            size: (self.instance_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }

    fn ensure_text_capacity(&mut self, needed: usize) {
        if needed <= self.text_vertex_capacity {
            return;
        }
        self.text_vertex_capacity = needed.next_power_of_two();
        self.text_vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text-vertex-buffer-grown"),
            size: (self.text_vertex_capacity * std::mem::size_of::<TextVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }

    fn update_global_uniform(&mut self, state: &AppState) {
        let outline = state.theme.outline_color_rgb();
        let bg_boxes = state.theme.background_box_color_rgb();
        let uniform = GlobalUniform {
            view_proj: self.camera.view_proj().to_cols_array_2d(),
            time_pack: [self.started.elapsed().as_secs_f32(), 0.0, 0.0, 0.0],
            outline_color: [outline[0], outline[1], outline[2], 0.0],
            bg_box_color: [bg_boxes[0], bg_boxes[1], bg_boxes[2], 0.0],
        };
        self.queue
            .write_buffer(&self.global_buffer, 0, bytemuck::bytes_of(&uniform));
    }
}
