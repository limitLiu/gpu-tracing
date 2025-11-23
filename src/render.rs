use bytemuck::{Pod, Zeroable};
use wgpu::{Device, Queue};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
  width: u32,
  height: u32,
}

pub struct PathTracer {
  device: Device,
  display_bind_group: wgpu::BindGroup,
  display_pipeline: wgpu::RenderPipeline,
  queue: Queue,
  uniform_buffer: wgpu::Buffer,
  uniforms: Uniforms,
}

impl PathTracer {
  pub fn new(
    device: Device,
    queue: Queue,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
  ) -> PathTracer {
    device.on_uncaptured_error(std::sync::Arc::new(|e| {
      panic!("Aborting due to an error: {e}")
    }));
    let shader_module = compile_shader_module(&device);
    let (display_pipeline, display_layout) =
      create_display_pipeline(&device, &shader_module, format);

    let uniforms = Uniforms { width, height };

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("uniforms"),
      size: std::mem::size_of::<Uniforms>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: true,
    });
    uniform_buffer
      .slice(..)
      .get_mapped_range_mut()
      .copy_from_slice(bytemuck::bytes_of(&uniforms));
    uniform_buffer.unmap();

    let display_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &display_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
          buffer: &uniform_buffer,
          offset: 0,
          size: None,
        }),
      }],
    });

    PathTracer {
      device,
      queue,
      uniforms,
      uniform_buffer,
      display_pipeline,
      display_bind_group,
    }
  }

  pub fn render_frame(&self, target: &wgpu::TextureView) {
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("render frame"),
      });
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("display pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: target,
        resolve_target: None,
        depth_slice: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });
    render_pass.set_pipeline(&self.display_pipeline);
    render_pass.set_bind_group(0, &self.display_bind_group, &[]);
    render_pass.draw(0..6, 0..1);
    drop(render_pass);
    let command_buffer = encoder.finish();
    self.queue.submit(Some(command_buffer));
  }
}

fn compile_shader_module(device: &Device) -> wgpu::ShaderModule {
  use std::borrow::Cow;
  let code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders.wgsl"));
  device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: None,
    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
  })
}

fn create_display_pipeline(
  device: &wgpu::Device,
  shader_module: &wgpu::ShaderModule,
  format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
  let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: None,
    entries: &[wgpu::BindGroupLayoutEntry {
      binding: 0,
      visibility: wgpu::ShaderStages::FRAGMENT,
      ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      count: None,
    }],
  });

  let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("display"),
    layout: Some(
      &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
        ..Default::default()
      }),
    ),
    vertex: wgpu::VertexState {
      module: shader_module,
      entry_point: Some("display_vs"),
      buffers: &[],
      compilation_options: wgpu::PipelineCompilationOptions::default(),
    },
    primitive: wgpu::PrimitiveState {
      topology: wgpu::PrimitiveTopology::TriangleList,
      front_face: wgpu::FrontFace::Ccw,
      polygon_mode: wgpu::PolygonMode::Fill,
      ..Default::default()
    },
    depth_stencil: None,
    multisample: wgpu::MultisampleState::default(),
    fragment: Some(wgpu::FragmentState {
      module: shader_module,
      entry_point: Some("display_fs"),
      targets: &[Some(wgpu::ColorTargetState {
        format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
      })],
      compilation_options: wgpu::PipelineCompilationOptions::default(),
    }),
    multiview: None,
    cache: None,
  });
  (pipeline, bind_group_layout)
}
