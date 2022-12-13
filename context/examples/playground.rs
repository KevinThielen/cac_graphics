use context::gl43_core as gl;

use raw_gl_context::GlContext;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    //create a context from the existing winit window
    let context = GlContext::create(
        &window,
        raw_gl_context::GlConfig {
            alpha_bits: 0,
            version: (4, 3),
            profile: raw_gl_context::Profile::Core,
            ..Default::default()
        },
    )
    .unwrap();

    //actually use the context
    context.make_current();

    //load the OpenGL functions with the context
    gl::load_with(|symbol| context.get_proc_address(symbol));

    unsafe {
        //red and alpha channels as 1.0, rest as 0.0
        gl::ClearColor(1.0, 0.0, 0.0, 1.0);
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                unsafe {
                    // clear the "screen"
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                // "update" the screen
                context.swap_buffers();
            }
            _ => {}
        }
    });
}
