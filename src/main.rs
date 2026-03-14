mod camera;
use std::{f32::consts::PI, sync::Arc};
use glam::Vec3;
use winit::keyboard::KeyCode;
use wgpu::{BufferUsages, util::DeviceExt};
use winit::{
    event::*, event_loop::{ControlFlow, EventLoop}, keyboard::PhysicalKey, window::WindowBuilder
};

use crate::camera::{Camera, CameraController, CameraUniform};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

struct Mesh {
    vertex_buffer: wgpu::Buffer, // For contiguous access to data in gpu memory
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

struct Gfx {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    triangle_render_pipeline: wgpu::RenderPipeline,
    line_list_render_pipeline: wgpu::RenderPipeline,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    meshes: Vec<Mesh>,
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
    fn generate_sphere() -> (Vec<Vertex>, Vec<u16>) {
        let mut indices: Vec<u16> = vec![];
        let mut vertices: Vec<Vertex> = vec![];
        let d_center: f32 = 0.5;
        
        for ring in 0..=12 { // theta = top to bottom
            let theta = ring as f32 * (PI / 12.0);
            for point in 0..12 { // phi = around the ring
                let phi = point as f32 * (2.0 * PI / 12.0);
                let x = d_center * theta.sin() * phi.cos();
                let y = d_center * theta.sin() * phi.sin();
                let z = d_center * theta.cos();
                vertices.push(Vertex {
                    position: [x, y, z],
                    color: [1.0, 1.0, 1.0],
                });
            }
        }
        // Horizontal slice LEFT AND RIGHT
        for ring in 0..12 {
            // Vertical slice UP AND DOWN
            for segment in 0..12 {
                let bottom_l = ring * 12 + segment;
                let bottom_r = if segment == 11 { ring * 12 } else { ring * 12 + segment + 1 };
                let top_l = (ring + 1) * 12 + segment;
                let top_r = if segment == 11 { (ring + 1) * 12 } else { (ring + 1) * 12 + segment + 1 };

                indices.push(bottom_l);
                indices.push(bottom_r);
                indices.push(top_l);
                indices.push(top_l);
                indices.push(bottom_r);
                indices.push(top_r);
            }
        }

        (vertices, indices)
    }
    fn generate_grid() -> (Vec<Vertex>, Vec<u16>) {
        let mut indices: Vec<u16> = vec![]; // Drawing order list 3 values in row will be connected
        let mut vertices: Vec<Vertex> = vec![];
        let mut x = -1.0;
        let mut y = -1.0;
        let z = 0.0;
        for row in 0..=10 {
            x = -1.0;
            if row >= 1 {
                y = y + 0.2;
            }
            for col in 0..=10 {
            vertices.push(Vertex {
                position: [x,y,z],
                color: [1.0,1.0,1.0]
            });
            x = x + 0.2;
            }
        }
        for  row in 0..10 {
            for col in 0..10 {
                if row < 9 {
                indices.push((row + 1) * 11 + col);
                indices.push((row + 1) * 11 + col + 1);
                }
                if col < 9 {
                indices.push(row * 11 + col + 1);
                indices.push((row + 1) * 11 + col + 1);
                }

            }
        }
        (vertices, indices)
    }
}

impl Gfx {
    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }
    fn new(window: &Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        // We create Camera type variable
        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0,0.0,0.0).into(),
            up: Vec3::Y,
            aspect: size.width as f32 / size.height as f32,
            fovy: 35.0,
            znear: 0.1,
            zfar: 100.0,
        };

        // We create conversion of camera struct to more GPU friendly format (flat matrix)
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::POLYGON_MODE_LINE,
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let (sphere_vertecies, sphere_indices) = Vertex::generate_sphere();
        let (grid_vertices, grid_indices) = Vertex::generate_grid();

        let meshes: Vec<Mesh> = vec![
            Gfx::create_mesh(&device, &sphere_vertecies, &sphere_indices),
            Gfx::create_mesh(&device, &grid_vertices, &grid_indices),
        ];
 
        let (camera_buffer, camera_bind_group_layout, camera_bind_group) = Gfx::init_camera_gpu(&device, &camera_uniform);
        // This is all the layout what will ever be used
        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout
                ],
                push_constant_ranges: &[],
            }
        );
        let line_list_render_pipeline = Gfx::create_pipeline(
            &device, &render_pipeline_layout, &shader, format, wgpu::PrimitiveTopology::LineList, wgpu::PolygonMode::Line);
        let triangle_render_pipeline = Gfx::create_pipeline(
            &device, &render_pipeline_layout, &shader, format, wgpu::PrimitiveTopology::TriangleList, wgpu::PolygonMode::Fill);


        let camera_controller = CameraController::new(0.2);

        let gfx = Gfx {
            surface,
            device,
            queue,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            line_list_render_pipeline,
            triangle_render_pipeline,
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
            meshes,
        };

        gfx.surface.configure(&gfx.device, &gfx.config);
        gfx
    }

    fn create_mesh(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u16],
    ) -> Mesh {
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
        let num_indices = indices.len() as u32;

        Mesh {
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }

    // camera buffer/slot creation and filling
    fn init_camera_gpu(
        device: &wgpu::Device,
        camera_uniform: &CameraUniform) -> (wgpu::Buffer, wgpu::BindGroupLayout,wgpu::BindGroup) {
         // camera bugger what is like camera position multiplier
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[*camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // blueprint telling wgpu to expect a uniform buffer at binding slot 0, visible only to vertex shader
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ], 
        label: Some("camera_bind_group_layout"),
        });
        // we bassiclly asign something to our just created slot
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }
        ],
        label: Some("camera_bind_group"),
        });

        (camera_buffer,camera_bind_group_layout,camera_bind_group)
    }
    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        topology: wgpu::PrimitiveTopology,
        polygon_mode: wgpu::PolygonMode,
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
                topology: topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front),
                polygon_mode: polygon_mode, //How pollygons are rendered Fill,Line
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
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            //draw Sphere
            pass.set_pipeline(&self.triangle_render_pipeline);
            // We bind out vertexes to our first buffer slot, Secondly we let our vertex buffer use whole buffer
            pass.set_vertex_buffer(0, self.meshes[0].vertex_buffer.slice(..));
            pass.set_index_buffer(self.meshes[0].index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // We loop though our indeci buffer draw all the indicies in order (0,1,2 -> traingle 1) (1,3,2 -> traingle 2)
            pass.draw_indexed(0..self.meshes[0].num_indices, 0, 0..1);
            // Draw Wireframe
            pass.set_pipeline(&self.line_list_render_pipeline);
            pass.set_vertex_buffer(0, self.meshes[1].vertex_buffer.slice(..));
            pass.set_index_buffer(self.meshes[1].index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..self.meshes[1].num_indices, 0, 0..1);
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
                    WindowEvent::KeyboardInput { 
                        event: KeyEvent {
                            physical_key: PhysicalKey::Code(keycode),
                            state,
                            ..
                        },
                    ..
                } => {
                            let is_pressed = state == ElementState::Pressed;
                            if keycode == KeyCode::Escape && is_pressed {
                                elwt.exit();
                            } else {
                                gfx.camera_controller.handle_key(keycode, is_pressed);
                            }
                        }
                    WindowEvent::RedrawRequested => match gfx.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => gfx.resize(gfx.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(_) => {}
                    },
                    _ => {}
                },
                Event::AboutToWait => {
                    gfx.update();
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
