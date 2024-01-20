use cgmath::{Matrix4, SquareMatrix};
use wgpu::util::DeviceExt;

use super::{texture, ImageView, Vertex};
use crate::{vec2::Vec2, WgpuState};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Uniform {
    pub matrix: Matrix4<f32>,
    pub size: Vec2<f32>,
    pub padding: Vec2<f32>, // Padding because glsl is adding dumb padding
    pub flip_horizontal: u32,
    pub flip_vertical: u32,
    pub hue: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub grayscale: u32,
    pub invert: u32,
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            matrix: Matrix4::identity(),
            flip_horizontal: Default::default(),
            flip_vertical: Default::default(),
            size: Default::default(),
            padding: Default::default(),
            hue: Default::default(),
            contrast: Default::default(),
            brightness: Default::default(),
            saturation: Default::default(),
            grayscale: Default::default(),
            invert: Default::default(),
        }
    }
}

unsafe impl bytemuck::Pod for Uniform {}
unsafe impl bytemuck::Zeroable for Uniform {}

impl Uniform {
    fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("image_fragment_bind_group_layout"),
        })
    }
}

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub fn new(wgpu: &WgpuState) -> Self {
        let render_pipeline_layout =
            wgpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &Uniform::get_bind_group_layout(&wgpu.device),
                        &texture::Texture::get_bind_group_layout(&wgpu.device),
                    ],
                    push_constant_ranges: &[],
                });

        let vertex = wgpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Image vertex"),
                source: wgpu::ShaderSource::Glsl {
                    shader: include_str!("../../shader/image.vert").into(),
                    stage: wgpu::naga::ShaderStage::Vertex,
                    defines: Default::default(),
                },
            });

        let fragment = wgpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Image fragment"),
                source: wgpu::ShaderSource::Glsl {
                    shader: include_str!("../../shader/image.frag").into(),
                    stage: wgpu::naga::ShaderStage::Fragment,
                    defines: Default::default(),
                },
            });

        let pipeline = wgpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex,
                    entry_point: "main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment,
                    entry_point: "main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu.config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let uniform_buffer = wgpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Uniform Buffer"),
                contents: bytemuck::cast_slice(&[Uniform::default()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let uniform_bind_group_layout = Uniform::get_bind_group_layout(&wgpu.device);
        let uniform_bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Vertex Uniform Bind Group"),
        });

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn prepare(&mut self, wgpu: &WgpuState, uniform: Uniform) {
        wgpu.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        rpass: &mut wgpu::RenderPass<'rpass>,
        image_view: &'rpass ImageView,
    ) {
        let (vertices, indices) = image_view.get_buffers();
        let texture = image_view.get_texture();
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_bind_group(1, &texture.diffuse_bind_group, &[]);
        rpass.set_vertex_buffer(0, vertices.slice(..));
        rpass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..6, 0, 0..1);
    }
}
