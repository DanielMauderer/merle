use wgpu::Instance;

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

