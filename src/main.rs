use winit::{
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{WindowBuilder, Window, self},
};
use wgpu_tutorial::WgpuState;

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
                    state.render()
                }
                _ => (),
            },

        Event::AboutToWait => {
            window.request_redraw();
        }, 

        _ => (),
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
    let mut wgpu_state = pollster::block_on(WgpuState::wgpu_state_builder(&window));
    // Run the event loop which handles the window
    event_loop.run(move |event, elwt| {
        event_handler(event, elwt, &window, &mut wgpu_state)
    }).unwrap();
 }