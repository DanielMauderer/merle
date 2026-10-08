use merle_photo::MerleImage;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.9, -0.9, 0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [-0.9, 0.9, 0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [0.9, -0.9, 0.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [0.9, 0.9, 0.0], tex_coords: [1.0, 0.0] },
];

const INDICES: &[u16] = &[0, 1, 2, 1, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ImageUniform {
    scale: [f32; 2],
    _pad: [f32; 2],
}

impl ImageUniform {
    fn new(target: (u32, u32), image: (u32, u32)) -> Self {
        Self { scale: aspect_scale(target, image), _pad: [0.0; 2] }
    }
}

fn aspect_scale((win_w, win_h): (u32, u32), (img_w, img_h): (u32, u32)) -> [f32; 2] {
    if win_w == 0 || win_h == 0 || img_w == 0 || img_h == 0 {
        return [1.0, 1.0];
    }
    #[allow(clippy::cast_precision_loss)]
    let window_aspect = win_w as f32 / win_h as f32;
    #[allow(clippy::cast_precision_loss)]
    let image_aspect = img_w as f32 / img_h as f32;
    if window_aspect > image_aspect {
        [image_aspect / window_aspect, 1.0]
    } else {
        [1.0, window_aspect / image_aspect]
    }
}

#[derive(Debug)]
pub struct ImageView {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group: Option<wgpu::BindGroup>,
    image_dimensions: (u32, u32),
    target_size: (u32, u32),
}

impl ImageView {
    #[must_use]
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let texture_bind_group_layout = texture_bind_group_layout(device);
        let uniform_bind_group_layout = uniform_bind_group_layout(device);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Image Uniform Buffer"),
            contents: bytemuck::cast_slice(&[ImageUniform::new((0, 0), (0, 0))]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("image_uniform_bind_group"),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let render_pipeline = render_pipeline(
            device,
            format,
            &[&texture_bind_group_layout, &uniform_bind_group_layout],
        );

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            texture_bind_group_layout,
            sampler,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group: None,
            image_dimensions: (0, 0),
            target_size: (0, 0),
        }
    }

    pub fn set_image(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, image: &MerleImage) {
        let max = device.limits().max_texture_dimension_2d;
        let (img_w, img_h) = image.dimensions();
        let rgba = if img_w > max || img_h > max {
            image.preview_image.thumbnail(max, max).to_rgba8()
        } else {
            image.preview_image.to_rgba8()
        };
        let (width, height) = rgba.dimensions();

        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("image_texture"),
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.texture_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("image_texture_bind_group"),
        }));
        self.image_dimensions = (width, height);
        self.write_uniform(queue);
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.target_size = (width, height);
        self.write_uniform(queue);
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        let Some(texture_bind_group) = &self.texture_bind_group else { return };
        let n_index = u32::try_from(INDICES.len()).expect("to large");

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..n_index, 0, 0..1);
    }

    fn write_uniform(&self, queue: &wgpu::Queue) {
        let uniform = ImageUniform::new(self.target_size, self.image_dimensions);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}

fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    })
}

fn uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("image_uniform_bind_group_layout"),
    })
}

fn render_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    let bind_group_layouts: Vec<_> = bind_group_layouts.iter().copied().map(Some).collect();
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &bind_group_layouts,
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(Vertex::desc())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
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
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    })
}
