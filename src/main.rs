use std::borrow::Cow;
use pollster;
use wgpu;
use winit::{
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{WindowBuilder, Window},
 };

//inside is a closure with a match function to match all ways we expect to interact with the window
// All possible matches contained within Event::*
fn event_handler(event : Event<()>, elwt : &EventLoopWindowTarget<()>, window : &Window, state : &mut WgpuState){
    match event {
        Event::WindowEvent { event, window_id } if window_id == window.id() => 
            match event {
                WindowEvent::Resized(new_size) => {
                    // Reconfigure the surface with the new size
                    state.config.width = new_size.width.max(1);
                    state.config.height = new_size.height.max(1);
                    state.surface.configure(&state.device, &state.config);
                    // On macos the window needs to be redrawn manually after resizing
                    window.request_redraw();
                }
                //Another nested match
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    let frame = state.surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder =
                        state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: None,
                        });
                    {
                        let mut rpass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });
                        rpass.set_pipeline(&state.pipeline);
                        rpass.draw(0..4, 0..1);
                    }

                    state.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
                _ => (),
            },

        Event::AboutToWait => {
            window.request_redraw();
        }, 

        _ => (),
    }
}

//Object for storing all the variables from wgpu we use to render
struct WgpuState {
    surface : wgpu::Surface, 
    queue : wgpu::Queue,
    device : wgpu::Device,
    pipeline : wgpu::RenderPipeline,
    config : wgpu::SurfaceConfiguration
}

async fn wgpu_state_builder(window : &Window) -> WgpuState {
    let instance = wgpu::Instance::default();
    // error with window lacking trait fixed: https://stackoverflow.com/questions/77373416/the-trait-raw-window-handlehasrawdisplayhandle-is-not-implemented-for-windo
    // Creates a surface which handles all the drawing 
    let surface = unsafe { instance.create_surface(&window).unwrap() };

    //Adapter is an object which finds the right gpu and configuration from our specified preferences
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    //Configure surface to match our window settings, https://docs.rs/wgpu/latest/wgpu/type.SurfaceConfiguration.html
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    surface.configure(&device, &config);
    return WgpuState{
        surface : surface, 
        queue : queue,
        device : device,
        pipeline : render_pipeline,
        config : config
    }
}

 //Winit code taken from: https://github.com/rust-windowing/winit/blob/master/examples/window.rs
 //Wgpu code taken from https://zdgeier.com/wgpuintro.html
 fn main() {
    env_logger::init(); // Necessary for logging within WGPU
    let event_loop = EventLoop::new().unwrap(); // Build Event Loop

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(512.0, 512.0))
        .build(&event_loop)
        .unwrap(); // Build window and specify parameters
    let mut wgpu_state = pollster::block_on(wgpu_state_builder(&window));
    // Run the event loop which handles the window
    event_loop.run(move |event, elwt| {
        event_handler(event, elwt, &window, &mut wgpu_state)
    }).unwrap();
 }