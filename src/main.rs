use anyhow::{Context, Result};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

const WIDTH: u32 = 960;
const HEIGHT: u32 = 544;

mod render;

#[pollster::main]
async fn main() -> Result<()> {
  let event_loop = EventLoop::new()?;
  let window_size = winit::dpi::LogicalSize::new(WIDTH, HEIGHT);
  let window = WindowBuilder::new()
    .with_inner_size(window_size)
    .with_resizable(false)
    .with_title("GPU Path Tracer")
    .build(&event_loop)?;
  let (device, queue, surface, format) = connect_to_gpu(&window).await?;
  let physical_size = window.inner_size();
  let renderer = render::PathTracer::new(
    device,
    queue,
    physical_size.width,
    physical_size.height,
    format,
  );

  event_loop.run(|event, control_handle| {
    control_handle.set_control_flow(ControlFlow::Poll);
    if let Event::WindowEvent { event, .. } = event {
      match event {
        WindowEvent::CloseRequested => control_handle.exit(),
        WindowEvent::RedrawRequested => {
          let frame: wgpu::SurfaceTexture = surface
            .get_current_texture()
            .expect("Failed to get current texture");
          let render_target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
          renderer.render_frame(&render_target);
          frame.present();
          window.request_redraw()
        }
        _ => (),
      }
    }
  })?;

  Ok(())
}

async fn connect_to_gpu(
  window: &'_ Window,
) -> Result<(
  wgpu::Device,
  wgpu::Queue,
  wgpu::Surface<'_>,
  wgpu::TextureFormat,
)> {
  use wgpu::TextureFormat::{Bgra8Unorm, Rgba8Unorm};

  let instance = wgpu::Instance::default();
  let surface = instance.create_surface(window)?;
  let adapter = instance
    .request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::HighPerformance,
      force_fallback_adapter: false,
      compatible_surface: Some(&surface),
    })
    .await
    .context("Failed to find a compatible adapter")?;

  let (device, queue) = adapter
    .request_device(&wgpu::DeviceDescriptor::default())
    .await
    .context("Failed to connect to the GPU")?;
  let caps = surface.get_capabilities(&adapter);
  let format = caps
    .formats
    .into_iter()
    .find(|it| matches!(it, Rgba8Unorm | Bgra8Unorm))
    .context("Couldn't find preferred texture format (Rgba8Unorm or Bgra8Unorm)")?;
  let size = window.inner_size();
  let config = wgpu::SurfaceConfiguration {
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    format,
    width: size.width,
    height: size.height,
    present_mode: wgpu::PresentMode::AutoVsync,
    desired_maximum_frame_latency: 3,
    alpha_mode: caps.alpha_modes[0],
    view_formats: vec![],
  };
  surface.configure(&device, &config);
  Ok((device, queue, surface, format))
}
