extern crate glfw;
extern crate byteorder;

pub mod bit;
pub mod protocol;

mod gl;
use glfw::{Action, Context, Key};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::DepthBits(32));
    glfw.window_hint(glfw::WindowHint::StencilBits(0));

    let (mut window, events) = glfw.create_window(854, 480, "Steven", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    gl::init(&mut window);

    window.set_key_polling(true);
    window.make_current();
    glfw.set_swap_interval(1);

    while !window.should_close() {
        gl::clear_color(1.0, 0.0, 0.0, 1.0);
        gl::clear(gl::ClearFlags::Color | gl::ClearFlags::Depth);

        window.swap_buffers();
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        }
        _ => {}
    }
}
