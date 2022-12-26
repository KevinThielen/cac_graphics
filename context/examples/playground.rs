use context::{
    buffer::Buffer,
    opengl,
    render_target::RenderTarget,
    shader::{Shader, Stage},
    vertex_layout::{Stride, VertexLayout},
    Context as _, Primitive,
};

use core::Color32;
use raw_gl_context::GlContext;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const VS_SOURCE: &str = r##"layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 color;

out vec4 fragment_color;

void main()
{
    fragment_color = color;
    gl_Position = vec4(pos, 1.0);
}"##;

const FS_SOURCE: &str = r##"
#version 430
layout(location=0) out vec4 result;

in vec4 fragment_color;
void main()
{
    result = fragment_color;
}"##;

#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: Color32,
}

impl Vertex {
    const BOTTOM_LEFT: Self = Self::new(-0.9, -0.9, Color32::GAINSBORO);
    const TOP_LEFT: Self = Self::new(-0.9, 0.9, Color32::PERSIAN_INDIGO);
    const TOP_RIGHT: Self = Self::new(0.9, 0.9, Color32::UNITY_YELLOW);
    const BOTTOM_RIGHT: Self = Self::new(0.9, -0.9, Color32::DARK_JUNGLE_GREEN);

    const fn new(x: f32, y: f32, color: Color32) -> Self {
        Self {
            position: [x, y, 0.0],
            color,
        }
    }
}

unsafe impl context::buffer::FlatData for Vertex {}

const QUAD_VERTICES: [Vertex; 4] = [
    Vertex::BOTTOM_LEFT,
    Vertex::TOP_LEFT,
    Vertex::TOP_RIGHT,
    Vertex::BOTTOM_RIGHT,
];

mod attribute {
    use context::vertex_layout::{AttributeKind, Components, VertexAttribute};

    pub const POSITION: VertexAttribute = VertexAttribute {
        location: 0,
        components: Components::Vec3,
        kind: AttributeKind::F32,
        normalized: false,
        local_offset: 0,
    };

    pub const COLOR: VertexAttribute = VertexAttribute {
        location: 1,
        components: Components::Vec4,
        kind: AttributeKind::F32,
        normalized: false,
        local_offset: std::mem::size_of::<f32>() * 3,
    };
}

struct GLContext(GlContext);

impl context::opengl::GLContext for GLContext {
    fn swap_buffers(&mut self) {
        self.0.swap_buffers();
    }

    fn get_proc_address(&mut self, name: &'static str) -> *const std::ffi::c_void {
        self.0.get_proc_address(name)
    }
}

fn winit_main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .format_timestamp(None)
        .init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    //create a context from the existing winit window
    let gl_context = GlContext::create(
        &window,
        raw_gl_context::GlConfig {
            alpha_bits: 0,
            version: (4, 3),
            profile: raw_gl_context::Profile::Core,
            ..Default::default()
        },
    )
    .unwrap();

    gl_context.make_current();
    let gl_context = GLContext(gl_context);

    let mut ctx = opengl::Context::new(gl_context)?;

    let screen = RenderTarget::with_clear_color(Color32::DARK_JUNGLE_GREEN);

    let vertex_buffer = Buffer::with_vertex_data(&mut ctx, &QUAD_VERTICES)?;

    let layout = VertexLayout::new(&mut ctx);
    layout.set_attributes(&mut ctx, 0, &[attribute::POSITION]);

    layout.set_buffer(
        &mut ctx,
        0,
        vertex_buffer,
        0,
        Stride::Interleaved(&[attribute::POSITION]),
    )?;

    let vs = Stage::new_vertex(&mut ctx, &["#version 430\n", VS_SOURCE])?;
    let fs = Stage::new_fragment(&mut ctx, &[FS_SOURCE])?;
    let shader = Shader::new(&mut ctx, &[vs, fs])?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                screen.clear(&mut ctx);

                ctx.draw(&screen, Primitive::TriangleStrip, shader, layout, 0, 4)
                    .unwrap();
                ctx.update();
            }
            _ => {}
        }
    });
}

use glfw::{Action, Context, Key};

struct GLFWContext(glfw::Window);
impl opengl::GLContext for GLFWContext {
    fn swap_buffers(&mut self) {
        self.0.swap_buffers()
    }

    fn get_proc_address(&mut self, name: &'static str) -> *const std::ffi::c_void {
        self.0.get_proc_address(name)
    }
}

fn glfw_main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .format_timestamp(None)
        .init();

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Create a windowed mode window and its OpenGL context
    // window hints have to be set before the window is created
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
    glfw.window_hint(glfw::WindowHint::Resizable(false));

    let (mut window, events) = glfw
        .create_window(300, 300, "Hello this is GLFW", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();

    let mut ctx = opengl::Context::new(GLFWContext(window))?;

    let screen = RenderTarget::with_clear_color(Color32::DARK_JUNGLE_GREEN);

    let vertex_buffer = Buffer::with_vertex_data(&mut ctx, &QUAD_VERTICES)?;

    let layout = VertexLayout::new(&mut ctx);
    layout.set_attributes(&mut ctx, 0, &[attribute::POSITION, attribute::COLOR]);

    layout.set_buffer(
        &mut ctx,
        0,
        vertex_buffer,
        0,
        Stride::Bytes(std::mem::size_of::<Vertex>()),
    )?;

    let vs = Stage::new_vertex(&mut ctx, &["#version 430\n", VS_SOURCE])?;
    let fs = Stage::new_fragment(&mut ctx, &[FS_SOURCE])?;
    let shader = Shader::new(&mut ctx, &[vs, fs])?;

    use context::vertex_layout::NativeContext;
    if let (Some(layout), buffers) = layout.get_mut_with_buffers(&mut ctx, [vertex_buffer]) {
        if let Some(vbo) = buffers[0] {
            layout.set_attribute_buffer(vbo)?;
        }
    }

    while !ctx.raw_context().0.should_close() {
        screen.clear(&mut ctx);
        ctx.draw(&screen, Primitive::TriangleStrip, shader, layout, 0, 4)?;

        ctx.update();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                ctx.raw_context().0.set_should_close(true)
            }
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    glfw_main()
    //winit_main()
}
