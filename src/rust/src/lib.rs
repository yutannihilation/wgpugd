mod util;

use std::fs::File;
use std::io::Write;

use extendr_api::{
    graphics::{
        ClippingStrategy, DevDesc, DeviceDescriptor, DeviceDriver, R_GE_gcontext, TextMetric,
    },
    prelude::*,
};
use wgpu::include_wgsl;

#[allow(dead_code)]
struct WgpuGraphicsDevice {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    texture_extent: wgpu::Extent3d,
    output_buffer: wgpu::Buffer,

    render_pipeline: wgpu::RenderPipeline,

    width: u32,
    height: u32,
    unpadded_bytes_per_row: u32,
    padded_bytes_per_row: u32,
}

const POINT: f64 = 12.0;

impl WgpuGraphicsDevice {
    async fn new(width: u32, height: u32) -> Self {
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
            // TODO: change this to 4 when enabling MSAA
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("wgpugd render pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(&include_wgsl!("shaders/shader.wgsl"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wgpugd render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            // TOOO: Use MSAA
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                }],
            }),
            multiview: None,
        });

        Self {
            device,
            queue,
            texture,
            texture_extent,
            output_buffer,

            render_pipeline,

            width,
            height,
            unpadded_bytes_per_row: unpadded_bytes_per_row as _,
            padded_bytes_per_row: padded_bytes_per_row as _,
        }
    }

    fn render(&mut self) -> extendr_api::Result<()> {
        let texture_view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("wgpugd render encoder"),
            });

        {
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("wgpugd render pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            // TODO: set the proper error from the value of gp->bg
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: true,
                        },
                    }],
                    // Since wgpugd is a 2D graphics device, we don't need the depth
                    // buffers. However, stencil buffers might be used for masking
                    // and clipping features. I don't figure out yet...
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.draw(0..3, 0..1);
            }

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
        let buffer_slice = self.output_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

        // Wait for the future resolves
        self.device.poll(wgpu::Maintain::Wait);

        if let Ok(()) = buffer_future.await {
            let padded_buffer = buffer_slice.get_mapped_range();

            let mut png_encoder = png::Encoder::new(
                File::create("tmp_wgpugd.png").unwrap(),
                self.width,
                self.height,
            );

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

impl DeviceDriver for WgpuGraphicsDevice {
    const CLIPPING_STRATEGY: ClippingStrategy = ClippingStrategy::Device;

    fn close(&mut self, _: DevDesc) {
        self.render().unwrap();
        pollster::block_on(self.write_png());
    }
}

/// A graphic device that does nothing
///
/// @param width  Device width in inch.
/// @param height Device width in inch.
/// @export
#[extendr]
fn wgpugd(width: i32, height: i32) {
    // Typically, 72 points per inch
    let width_pt = width * 72;
    let height_pt = height * 72;

    let device_driver = pollster::block_on(WgpuGraphicsDevice::new(width_pt as _, height_pt as _));

    let device_descriptor =
        // In SVG's coordinate y=0 is at top, so, we need to flip it by setting bottom > top.
        DeviceDescriptor::new().device_size(0.0, width_pt as _, height_pt as _, 0.0);

    device_driver.create_device::<WgpuGraphicsDevice>(device_descriptor, "wgpugd");
}

extendr_module! {
    mod wgpugd;
    fn wgpugd;
}
