use context::{gl43_core as gl, opengl};

use raw_gl_context::GlContext;
use std::ffi::CString;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const VS_SOURCE: &str = r##"
#version 430
layout(location = 0) in vec3 pos;

out vec4 fragment_color;

void main()
{
    fragment_color = vec4(pos.x, pos.y, pos.x + pos.y, 1.0);
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

fn create_shader() -> gl::types::GLuint {
    let program;
    unsafe {
        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);

        //convert Rust Str to CString
        let vert_source = CString::new(VS_SOURCE).unwrap();
        let frag_source = CString::new(FS_SOURCE).unwrap();

        gl::ShaderSource(vs, 1, [vert_source.as_ptr()].as_ptr(), std::ptr::null());
        gl::ShaderSource(fs, 1, [frag_source.as_ptr()].as_ptr(), std::ptr::null());

        gl::CompileShader(vs);
        gl::CompileShader(fs);

        program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        gl::DetachShader(program, vs);
        gl::DetachShader(program, fs);
        gl::UseProgram(program);

        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
    }

    program
}
fn create_vao(vbo: gl::types::GLuint) -> gl::types::GLuint {
    let mut vao = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribFormat(0, 3, gl::FLOAT, gl::FALSE, 0);
        gl::VertexAttribBinding(0, 0); //attribute 0 uses buffer binding 0

        gl::BindVertexBuffer(0, vbo, 0, std::mem::size_of::<f32>() as i32 * 3);
    }

    vao
}

fn create_vbo<T>(data: &[T]) -> gl::types::GLuint {
    let mut buffer = 0;

    //the size of our blob is the size of a single element(T) * the counts of T in our slice
    let data_size = std::mem::size_of::<T>() * data.len();

    unsafe {
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            data_size.try_into().unwrap(), //we need to cast usize to "isize", panicking is fine in our playground
            data.as_ptr().cast(),          //the pointer to our first element in our slice
            gl::STATIC_DRAW,
        )
    }

    buffer
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

    let mut context = opengl::Context::new(gl_context)?;

    unsafe {
        gl::ClearColor(1.0, 0.0, 0.0, 1.0);
    }

    #[rustfmt::skip]  //skip the default formatting to make it cleaner
    const QUAD_VERTICES: [f32; 3 * 4] = [
        //     X,    Y,   Z    Position
        -0.9, -0.9, 0.0, // bottom left
        -0.9,  0.9, 0.0, // top left
        0.9,  0.9, 0.0, // top right
        0.9, -0.9, 0.0, // bottom right
    ];

    let vbo = create_vbo(&QUAD_VERTICES);
    let vao = create_vao(vbo);
    let shader_program = create_shader();

    unsafe {
        gl::UseProgram(shader_program);
        gl::BindVertexArray(vao);
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
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
                };

                context.update();
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

    let mut context = opengl::Context::new(GLFWContext(window))?;

    unsafe {
        gl::ClearColor(1.0, 0.0, 0.0, 1.0);
    }

    #[rustfmt::skip]  //skip the default formatting to make it cleaner
    const QUAD_VERTICES: [f32; 3 * 4] = [
        //     X,    Y,   Z    Position
        -0.9, -0.9, 0.0, // bottom left
        -0.9,  0.9, 0.0, // top left
        0.9,  0.9, 0.0, // top right
        0.9, -0.9, 0.0, // bottom right
    ];

    let vbo = create_vbo(&QUAD_VERTICES);
    let vao = create_vao(vbo);
    let shader_program = create_shader();

    unsafe {
        gl::UseProgram(shader_program);
        gl::BindVertexArray(vao);
    }

    while !context.raw_context().0.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
        context.update();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                context.raw_context().0.set_should_close(true)
            }
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    glfw_main()
    //winit_main()
}
