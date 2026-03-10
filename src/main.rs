use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    event::*, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

struct Gfx {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    num_vertices: u32,
    num_indices: u32,
    vertex_buffer: wgpu::Buffer, // For contiguous access to data in gpu memory
    index_buffer: wgpu::Buffer,
    size: winit::dpi::PhysicalSize<u32>,
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
    fn generate_grid() -> (Vec<Vertex>, Vec<u16>) {
        let mut indices: Vec<u16> = vec![]; // Drawing order list 3 values in row will be connected
        let mut vertices: Vec<Vertex> = vec![];
        let mut x = -1.0;
        let mut y = -1.0;
        let z = 0.0;
        for row in 0..2 {
            x = -1.0;
            if row == 1 {
                y = 1.0;
            }
            for col in 0..2 {
                if col == 1 {
                    x = 1.0
                }
            vertices.push(Vertex {
                position: [x,y,z],
                color: [1.0,1.0,1.0]
            });
            }
        }
        indices.push(0);
        indices.push(1);
        indices.push(2);
        indices.push(1);
        indices.push(3);
        indices.push(2);
        (vertices, indices)
    }
}

impl Gfx {
    fn new(window: &Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let render_pipeline =
            Gfx::create_pipeline(&device, &render_pipeline_layout, &shader, format);

//        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//            label: Some("Vertex Buffer"),
//            contents: bytemuck::cast_slice(VERTICES),
//            usage: wgpu::BufferUsages::VERTEX,
//        });
        let (vertices, indices) = Vertex::generate_grid();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        //  We use our Device (like a remove to our specific GPU) buffer pushes the data into GPU
        let index_buffer= device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Indedx Buffer"),
            contents: bytemuck::cast_slice(&indices), // Converts Vec into raw bytes so GPU understands
            usage: wgpu::BufferUsages::INDEX, // It tells how to use this specific buffer
        });
        // We collect the sizes of our vertecies for future loop
        let num_vertices = vertices.len() as u32;
        let num_indices = indices.len() as u32;

        let gfx = Gfx {
            surface,
            device,
            queue,
            render_pipeline,
            config: wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
            size,
            vertex_buffer,
            index_buffer,
            num_vertices,
            num_indices,
        };

        gfx.surface.configure(&gfx.device, &gfx.config);
        gfx
    }

    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
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
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.09,
                            b: 0.10,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.render_pipeline);
            // We bind out vertexes to our first buffer slot, Secondly we let our vertex buffer use whole buffer
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // We loop though our indeci buffer draw all the indicies in order (0,1,2 -> traingle 1) (1,3,2 -> traingle 2)
            pass.draw_indexed(0..self.num_indices, 0, 0..1);
            //pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("wgpu basic")
            .build(&event_loop)
            .unwrap(),
    );

    let mut gfx = Gfx::new(&window);

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(new_size) => gfx.resize(new_size),
                    WindowEvent::RedrawRequested => match gfx.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => gfx.resize(gfx.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(_) => {}
                    },
                    _ => {}
                },
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
