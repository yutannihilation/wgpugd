mod file;
mod graphics_device;
mod render_pipeline;
mod text;

use crate::file::FilenameTemplate;
use crate::graphics_device::WgpugdCommand;

use std::io::Write;
use std::{fs::File, path::PathBuf};

use extendr_api::{
    graphics::{DeviceDescriptor, DeviceDriver},
    prelude::*,
};

use lyon::lyon_tessellation::VertexBuffers;
use render_pipeline::create_render_pipeline;
use wgpu::util::DeviceExt;

// For general shapes --------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: u32,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// For circles ----------------------------------------------------

// For the sake of performance, we treat circle differently as they can be
// simply represented by a SDF.

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SDFVertex {
    position: [f32; 2],
}

impl SDFVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// TODO: measure and set nicer default values
const VERTEX_SIZE: usize = std::mem::size_of::<Vertex>();
const VERTEX_BUFFER_INITIAL_SIZE: u64 = VERTEX_SIZE as u64 * 10000;
const INDEX_SIZE: usize = std::mem::size_of::<u32>();
const INDEX_BUFFER_INITIAL_SIZE: u64 = INDEX_SIZE as u64 * 10000;

#[rustfmt::skip]
const RECT_VERTICES: &[SDFVertex] = &[
    SDFVertex { position: [ 1.0, -1.0] },
    SDFVertex { position: [-1.0, -1.0] },
    SDFVertex { position: [-1.0,  1.0] },
    SDFVertex { position: [ 1.0,  1.0] },
];
const RECT_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SDFInstance {
    center: [f32; 2],
    radius: f32,
    stroke_width: f32,
    fill_color: u32,
    stroke_color: u32,
}

impl SDFInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Float32,
        3 => Float32,
        4 => Uint32,
        5 => Uint32,
    ];

    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    resolution: [f32; 2],
}

#[allow(dead_code)]
struct WgpuGraphicsDevice {
    device: wgpu::Device,
    queue: wgpu::Queue,

    // For writing out a PNG
    texture: wgpu::Texture,
    texture_extent: wgpu::Extent3d,
    output_buffer: wgpu::Buffer,

    globals_bind_group: wgpu::BindGroup,
    globals_uniform_buffer: wgpu::Buffer,

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    sdf_vertex_buffer: wgpu::Buffer,
    sdf_index_buffer: wgpu::Buffer,
    sdf_render_pipeline: wgpu::RenderPipeline,

    sdf_instances: Vec<SDFInstance>,

    geometry: VertexBuffers<Vertex, u32>,

    // For MSAA
    multisampled_framebuffer: wgpu::TextureView,

    // On clipping or instanced rendering layer, increment this layer id
    current_command: Option<WgpugdCommand>,
    command_queue: Vec<WgpugdCommand>,

    // width and height in point
    width: u32,
    height: u32,

    // The unpadded and padded lengths are both needed because we prepare a
    // buffer in the padded size but do not read the padded part.
    unpadded_bytes_per_row: u32,
    padded_bytes_per_row: u32,

    filename: FilenameTemplate,
    cur_page: u32,
}

impl WgpuGraphicsDevice {
    fn filename(&self) -> PathBuf {
        self.filename.filename(self.cur_page)
    }

    async fn new(filename: &str, width: u32, height: u32) -> Self {
        // Set envvar WGPU_BACKEND to specific backend (e.g., vulkan, dx12, metal, opengl)
        let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);

        // An `Instance` is a "context for all other wgpu objects"
        let instance = wgpu::Instance::new(backend);

        // An `Adapter` is a "handle to a physical graphics"
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None, // Currently no window so no surface
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // A `Device` is a "connection to a graphics device" and a `Queue` is a command queue.
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // This texture is where the RenderPass renders.
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("wgpugd texture descriptor"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // R don't use sRGB, so don't choose Rgba8UnormSrgb here!
            format: wgpu::TextureFormat::Rgba8Unorm,
            // The texture is a rendering target and passed in
            // `color_attachments`, so `RENDER_ATTACHMENT` is needed. Also, it's
            // where the image is copied from so `COPY_SRC` is needed.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        });

        // This code is from the example on the wgpu's repo. Why this is needed?
        // The comment on there says it's WebGPU's requirement that the buffer
        // size is a multiple of `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`
        //
        // ref:
        // https://github.com/gfx-rs/wgpu/blob/312828f12f1a1497bc0387a72a5346ef911acad7/wgpu/examples/capture/main.rs#L170-L191
        let bytes_per_pixel = std::mem::size_of::<u32>() as u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;

        // Output buffer is where the texture is copied to, and then written out
        // as a PNG image.
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wgpugd output buffer"),
            size: (padded_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wgpugd uniform buffer for globals"),
            size: std::mem::size_of::<Globals>() as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("wgpugd globals bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wgpugd globals bind group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_uniform_buffer.as_entire_binding(),
            }],
        });

        let multisampled_framebuffer = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("wgpugd multisampled framebuffer"),
                size: texture_extent,
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            })
            .create_view(&wgpu::TextureViewDescriptor::default());

        let render_pipeline = create_render_pipeline(
            &device,
            "wgpugd render pipeline layout",
            "wgpugd render pipeline",
            &[&globals_bind_group_layout],
            &wgpu::include_wgsl!("shaders/shader.wgsl"),
            &[Vertex::desc()],
            4,
        );

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wgpugd vertex buffer"),
            size: VERTEX_BUFFER_INITIAL_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wgpugd index buffer"),
            size: INDEX_BUFFER_INITIAL_SIZE,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sdf_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("wgpugd vertex buffer"),
            contents: bytemuck::cast_slice(RECT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let sdf_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("wgpugd index buffer"),
            contents: bytemuck::cast_slice(RECT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let sdf_render_pipeline = create_render_pipeline(
            &device,
            "wgpugd render pipeline layout for SDF shapes",
            "wgpugd render pipeline for SDF shapes",
            &[&globals_bind_group_layout],
            &wgpu::include_wgsl!("shaders/sdf_shape.wgsl"),
            &[SDFVertex::desc(), SDFInstance::desc()],
            // Technically, this doesn't need to be multisampled, as the SDF
            // shapes are out of scope of MSAA anyway, but as we share the
            // one renderpipline, the sample count must match the others.
            4,
        );

        let geometry: VertexBuffers<Vertex, u32> = VertexBuffers::new();

        Self {
            device,
            queue,
            texture,
            texture_extent,
            output_buffer,

            globals_bind_group,
            globals_uniform_buffer,

            render_pipeline,

            vertex_buffer,
            index_buffer,

            sdf_vertex_buffer,
            sdf_index_buffer,
            sdf_render_pipeline,

            sdf_instances: Vec::new(),

            geometry,

            multisampled_framebuffer,

            current_command: None,
            command_queue: Vec::new(),

            width,
            height,

            unpadded_bytes_per_row: unpadded_bytes_per_row as _,
            padded_bytes_per_row: padded_bytes_per_row as _,

            filename: FilenameTemplate::new(filename).unwrap(),
            // The page number starts with 0, but newPage() will be immediately
            // called and this gets incremented to 1.
            cur_page: 0,
        }
    }

    fn render(&mut self) -> extendr_api::Result<()> {
        // TODO: do this more nicely...
        if let Some(ref cmd) = self.current_command {
            self.command_queue.push(cmd.clone());
        }

        // TODO: recreate the buffer when the data size is over the current buffer size.
        self.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(self.geometry.vertices.as_slice()),
        );
        self.queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(self.geometry.indices.as_slice()),
        );

        let sdf_instance_buffer =
            &self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("wgpugd instance buffer"),
                    contents: bytemuck::cast_slice(self.sdf_instances.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        self.queue.write_buffer(
            &self.globals_uniform_buffer,
            0,
            bytemuck::cast_slice(&[Globals {
                resolution: [self.width as _, self.height as _],
            }]),
        );

        let texture_view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("wgpugd render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("wgpugd render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &self.multisampled_framebuffer,
                    resolve_target: Some(&texture_view),
                    ops: wgpu::Operations {
                        // TODO: set the proper error from the value of gp->bg
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        // As described in the wgpu's example of MSAA, if the
                        // pre-resolved MSAA data is not used anywhere else, we
                        // should set this to false to save memory.
                        store: false,
                    },
                }],
                depth_stencil_attachment: None,
            });

            let mut begin_id_polygon = 0_u32;
            let mut last_id_polygon;
            let mut begin_id_sdf = 0_u32;
            let mut last_id_sdf;

            for cmd in self.command_queue.iter() {
                match cmd {
                    WgpugdCommand::DrawPolygon(cmd) => {
                        last_id_polygon = begin_id_polygon + cmd.count;

                        render_pass.set_pipeline(&self.render_pipeline);
                        render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
                        render_pass.set_vertex_buffer(
                            0,
                            self.vertex_buffer
                                .slice(0..(VERTEX_SIZE * self.geometry.vertices.len()) as _),
                        );
                        render_pass.set_index_buffer(
                            self.index_buffer
                                .slice(0..(INDEX_SIZE * self.geometry.indices.len()) as _),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(begin_id_polygon..last_id_polygon, 0, 0..1);

                        begin_id_polygon = last_id_polygon;
                    }
                    WgpugdCommand::DrawSDF(cmd) => {
                        last_id_sdf = begin_id_sdf + cmd.count;

                        render_pass.set_pipeline(&self.sdf_render_pipeline);
                        render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, self.sdf_vertex_buffer.slice(..));
                        render_pass.set_vertex_buffer(1, sdf_instance_buffer.slice(..));
                        render_pass.set_index_buffer(
                            self.sdf_index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );
                        render_pass.draw_indexed(
                            0..RECT_INDICES.len() as _,
                            0,
                            begin_id_sdf..last_id_sdf,
                        );

                        begin_id_sdf = last_id_sdf;
                    }
                    WgpugdCommand::SetClipping {
                        x,
                        y,
                        height,
                        width,
                    } => render_pass.set_scissor_rect(*x, *y, *width, *height),
                }
            }

            // reprintln!("{:?}", self.geometry.vertices);
            // reprintln!("{:?}", self.sdf_instances);

            // Return the ownership. Otherwise the next operation on encoder would fail
            drop(render_pass);

            encoder.copy_texture_to_buffer(
                self.texture.as_image_copy(),
                wgpu::ImageCopyBuffer {
                    buffer: &self.output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            std::num::NonZeroU32::new(self.padded_bytes_per_row).unwrap(),
                        ),
                        // This parameter is needed when there are multiple
                        // images, and it's not the case this time.
                        rows_per_image: None,
                    },
                },
                self.texture_extent,
            );
        }

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    // c.f. https://github.com/gfx-rs/wgpu/blob/312828f12f1a1497bc0387a72a5346ef911acad7/wgpu/examples/capture/main.rs#L119
    async fn write_png(&mut self) {
        let file = match File::create(self.filename()) {
            Ok(f) => f,
            Err(e) => {
                reprintln!("Failed to create the output file: {e:?}");
                return;
            }
        };

        let buffer_slice = self.output_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

        // Wait for the future resolves
        self.device.poll(wgpu::Maintain::Wait);

        if let Ok(()) = buffer_future.await {
            let padded_buffer = buffer_slice.get_mapped_range();

            let mut png_encoder = png::Encoder::new(file, self.width, self.height);

            png_encoder.set_depth(png::BitDepth::Eight);
            png_encoder.set_color(png::ColorType::Rgba);

            // TODO: handle results nicely
            let mut png_writer = png_encoder
                .write_header()
                .unwrap()
                .into_stream_writer_with_size(self.unpadded_bytes_per_row as _)
                .unwrap();

            for chunk in padded_buffer.chunks(self.padded_bytes_per_row as _) {
                png_writer
                    // while the buffer is padded, we only need the unpadded part
                    .write_all(&chunk[..self.unpadded_bytes_per_row as _])
                    .unwrap();
            }
            png_writer.finish().unwrap();

            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(padded_buffer);

            self.output_buffer.unmap();
        }
    }
}

/// A WebGPU Graphics Device for R
///
/// @param filename
/// @param width  Device width in inch.
/// @param height Device width in inch.
/// @export
#[extendr]
fn wgpugd(
    #[default = "'Rplot%03d.png'"] filename: &str,
    #[default = "7"] width: i32,
    #[default = "7"] height: i32,
) {
    // Typically, 72 points per inch
    let width_pt = width * 72;
    let height_pt = height * 72;

    let device_driver = pollster::block_on(WgpuGraphicsDevice::new(
        filename,
        width_pt as _,
        height_pt as _,
    ));

    let device_descriptor =
        DeviceDescriptor::new().device_size(0.0, width_pt as _, 0.0, height_pt as _);

    device_driver.create_device::<WgpuGraphicsDevice>(device_descriptor, "wgpugd");
}

extendr_module! {
    mod wgpugd;
    fn wgpugd;
}
