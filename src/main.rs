// Copyright 2015 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod bit;
pub mod protocol;
pub mod format;
pub mod nbt;
pub mod item;
pub mod gl;
pub mod types;
pub mod resources;
pub mod render;

extern crate glfw;
extern crate image;
extern crate time;
extern crate byteorder;
extern crate serde_json;
extern crate steven_openssl as openssl;
extern crate hyper;
extern crate flate2;

use std::sync::{Arc, RwLock};
use glfw::{Action, Context, Key};

fn main() {
    let resource_manager = Arc::new(RwLock::new(resources::Manager::new()));
    { resource_manager.write().unwrap().tick(); }

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

    let mut renderer = render::Renderer::new(resource_manager.clone());

    let mut last_frame = time::now();
    let frame_time = (time::Duration::seconds(1).num_nanoseconds().unwrap() as f64) / 60.0;

    while !window.should_close() {
        { resource_manager.write().unwrap().tick(); }
        let now = time::now();
        let diff = now - last_frame;
        last_frame = now;
        let delta = (diff.num_nanoseconds().unwrap() as f64) / frame_time;

        let (width, height) = window.get_framebuffer_size();
        renderer.tick(delta, width as u32, height as u32);

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
