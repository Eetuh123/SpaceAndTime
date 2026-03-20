mod camera;
mod texture;
mod spacetime;
mod body;
mod physics;
use std::{f32::consts::PI, sync::Arc};
use glam::{Mat4, Quat, Vec3};
use winit::{keyboard::KeyCode};
use wgpu::{util::DeviceExt};
use winit::{
    event::*, event_loop::{ControlFlow, EventLoop}, keyboard::PhysicalKey, window::WindowBuilder
};

use crate::{body::Body, spacetime::SpacetimeUniform, texture::Texture};
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
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MeshInstanceRaw {
    model: [[f32; 4]; 4],
}
struct MeshInstance {
    scale: Vec3,
    rotation: Quat,
    translation: Vec3,
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
    space_time_uniform: SpacetimeUniform,
    gravity_buffer: wgpu::Buffer,
    gravity_bind_group: wgpu::BindGroup,
    meshes: Vec<Mesh>,
    size: winit::dpi::PhysicalSize<u32>,
    depth_texture: Texture,
    grid_instances: Vec<MeshInstance>,
    sphere_instances: Vec<MeshInstance>,
    grid_instances_buffer: wgpu::Buffer,
    sphere_instances_buffer: wgpu::Buffer,
    list_of_bodies: Vec<Body>,
    last_frame_time: std::time::Instant,
}

impl MeshInstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshInstanceRaw>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x4,
            },
            
        ]
        }
    }
}

impl MeshInstance {
    fn to_raw(&self) -> MeshInstanceRaw {
        MeshInstanceRaw { 
            model: (Mat4::from_scale_rotation_translation(self.scale,self.rotation,self.translation)).to_cols_array_2d()
        }
    }
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
    fn generate_sphere() -> (Vec<Vertex>, Vec<u32>) {
        let mut indices: Vec<u32> = vec![];
        let mut vertices: Vec<Vertex> = vec![];
        let d_center: f32 = -0.2;
        
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
    fn generate_grid() -> (Vec<Vertex>, Vec<u32>) {
        let size: f32 = 16.0;
        let start = -size;
        let lines: f32 = 200.0;
        let step = (size * 2.0) / lines;
        let mut indices: Vec<u32> = vec![]; // Drawing order list 3 values in row will be connected
        let mut vertices: Vec<Vertex> = vec![];
        for row in 0..=lines as u32 {
            for col in 0..=lines as u32 {
            vertices.push(Vertex {
                position: [(col as f32 * step) + start,0.0,(row as f32 * step) + start],
                color: [1.0,1.0,1.0]
            });
            }
        }
        for  row in 0..lines as u32 {
            for col in 0..lines as u32 {
                if row < (lines as u32 - 1) {
                indices.push((row + 1) * (lines as u32 + 1) + col);
                indices.push((row + 1) * (lines as u32 + 1) + col + 1);
                }
                if col < (lines as u32 - 1) {
                indices.push(row * (lines as u32 + 1) + col + 1);
                indices.push((row + 1) * (lines as u32 + 1) + col + 1);
                }

            }
        }
        (vertices, indices)
    }
}

impl Gfx {
    fn update(&mut self) {

        let now = std::time::Instant::now();
        let delta_time = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        let mut forces = vec![Vec3::ZERO; self.list_of_bodies.len()];

        for i in 0..self.list_of_bodies.len() {
            for j in 0..self.list_of_bodies.len() {
                if i != j {
                    forces[i] += self.list_of_bodies[i].force_from(&self.list_of_bodies[j]);
                }
            }
        }

        for (i, body) in self.list_of_bodies.iter().enumerate() {
            self.sphere_instances[i].translation = body.position;
        }

        for (body, force) in self.list_of_bodies.iter_mut().zip(forces.iter()) {
            body.step(*force, delta_time);
        }

        self.space_time_uniform.update_all(&self.list_of_bodies);
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        let instance_data = self.sphere_instances.iter().map(MeshInstance::to_raw).collect::<Vec<_>>();
        self.queue.write_buffer(&self.sphere_instances_buffer, 0, bytemuck::cast_slice(&instance_data));
        self.queue.write_buffer(&self.gravity_buffer, 0, bytemuck::cast_slice(&[self.space_time_uniform]));
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }
    fn new(window: &Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let list_of_bodies = vec![
            Body {
                position: Vec3::new(0.0, 1.0, 0.0),
                velocity: Vec3::new(0.0, 0.0, 0.0),
                mass: 75.0,
                radius: 1.0,
            },
            Body {
                position: Vec3::new(3.0, 1.0, 3.0),
                velocity: Vec3::new(-0.707, 0.0, 0.707) * 3.8,
                mass: 10.0,
                radius: 1.0,
            }
        ];

        let sphere_instances = vec![
            MeshInstance {
                translation: list_of_bodies[0].position,
                scale: Vec3::ONE,
                rotation: Quat::IDENTITY,
            },
            MeshInstance {
                translation: list_of_bodies[1].position,
                scale: Vec3::ONE,
                rotation: Quat::IDENTITY,
            }
        ];
        let grid_instances = vec![
            MeshInstance {
                translation: Vec3::ZERO,
                scale: Vec3::ONE,
                rotation: Quat::IDENTITY,
            }
        ];

        let space_time_uniform = SpacetimeUniform::new(&list_of_bodies);

        // We create Camera type variable
        let camera = Camera {
            eye: (0.0, 5.0, 10.0).into(),
            target: (0.0,0.0,0.0).into(),
            up: Vec3::Y,
            aspect: size.width as f32 / size.height as f32,
            fovy: 65.0,
            znear: 0.1,
            zfar: 200.0,
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

        let (gravity_buffer, gravity_bind_group_layout, gravity_bind_group)  = Gfx::init_gravity_gpu(&device, &space_time_uniform);

 
        let (camera_buffer, camera_bind_group_layout, camera_bind_group) = Gfx::init_camera_gpu(&device, &camera_uniform);
        // This is all the layout what will ever be used
        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &gravity_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );
        let line_list_render_pipeline = Gfx::create_pipeline(
            &device, &render_pipeline_layout, &shader, format, wgpu::PrimitiveTopology::LineList, wgpu::PolygonMode::Line, "vs_main");
        let triangle_render_pipeline = Gfx::create_pipeline(
            &device, &render_pipeline_layout, &shader, format, wgpu::PrimitiveTopology::TriangleList, wgpu::PolygonMode::Fill, "vs_main_no_gravity");


        let camera_controller = CameraController::new(0.2);

        let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
        };

        let sphere_instances_buffer = Gfx::create_instance_buffer(&device, &sphere_instances);
        let grid_instances_buffer = Gfx::create_instance_buffer(&device, &grid_instances);

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let gfx = Gfx {
            surface,
            device,
            queue,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            space_time_uniform,
            gravity_buffer,
            gravity_bind_group,
            line_list_render_pipeline,
            triangle_render_pipeline,
            config,
            size,
            meshes,
            depth_texture,
            grid_instances,
            sphere_instances,
            grid_instances_buffer,
            sphere_instances_buffer,
            list_of_bodies,
            last_frame_time: std::time::Instant::now(),
        };

        gfx.surface.configure(&gfx.device, &gfx.config);
        gfx
    }

    fn create_instance_buffer(
        device: &wgpu::Device,
        instances: &[MeshInstance],
    ) -> wgpu::Buffer {

        let instance_data = instances.iter().map(MeshInstance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
            
        instance_buffer
    }

    fn create_mesh(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Mesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        //creates A buffer list of indecies which our GPU can read
        let index_buffer= device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
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

    fn init_gravity_gpu(
        device: &wgpu::Device,
        space_time_uniform: &SpacetimeUniform) -> (wgpu::Buffer, wgpu::BindGroupLayout,wgpu::BindGroup) {
        let gravity_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gravity_Buffer"),
            contents: bytemuck::cast_slice(&[*space_time_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let gravity_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
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
        label: Some("gravity_bind_group_layout"),
        });
        let gravity_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gravity_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                binding: 0,
                resource: gravity_buffer.as_entire_binding(),
            }
        ],
        label: Some("gravity_bind_group"),
        });

        (gravity_buffer,gravity_bind_group_layout,gravity_bind_group)
    }

    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        topology: wgpu::PrimitiveTopology,
        polygon_mode: wgpu::PolygonMode,
        vertex_entry: &str,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: vertex_entry,
                buffers: &[
                    Vertex::desc(),
                    MeshInstanceRaw::desc()
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
            depth_stencil: Some(wgpu::DepthStencilState { 
                format: texture::Texture::DEPTH_FORMAT, 
                depth_write_enabled: true, 
                depth_compare: wgpu::CompareFunction::Less, 
                stencil: wgpu::StencilState::default(), 
                bias: wgpu::DepthBiasState::default(),
            }),
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
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture")
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
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
                    view: &self.depth_texture.view, 
                    depth_ops: Some(wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(1.0), 
                        store: wgpu::StoreOp::Store,
                    }), 
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            // We give renderer Camera data so it can use it do calculation based on camera angle
            pass.set_bind_group(0, &self.camera_bind_group , &[]);
            pass.set_bind_group(1, &self.gravity_bind_group , &[]);

            // Draw Sphere
            pass.set_pipeline(&self.triangle_render_pipeline);
            // We bind our vertices to slot 0, instance transforms to slot 1
            pass.set_vertex_buffer(0, self.meshes[0].vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.sphere_instances_buffer.slice(..));
            pass.set_index_buffer(self.meshes[0].index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // Draw all indices, once per instance
            pass.draw_indexed(0..self.meshes[0].num_indices, 0, 0..self.sphere_instances.len() as _);

            // Draw Wireframe grid
            pass.set_pipeline(&self.line_list_render_pipeline);
            pass.set_vertex_buffer(0, self.meshes[1].vertex_buffer.slice(..));
            // We bind our vertices to slot 0, instance transforms to slot 1
            pass.set_vertex_buffer(1, self.grid_instances_buffer.slice(..));
            pass.set_index_buffer(self.meshes[1].index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // Draw all indices, once per instance
            pass.draw_indexed(0..self.meshes[1].num_indices, 0, 0..self.grid_instances.len() as _);
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
