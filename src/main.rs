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

pub mod protocol;
pub mod format;
pub mod nbt;
pub mod item;
pub mod gl;
pub mod types;
pub mod resources;
pub mod render;
pub mod ui;
pub mod screen;
#[macro_use]
pub mod console;

extern crate glfw;
extern crate image;
extern crate time;
extern crate byteorder;
extern crate serde_json;
extern crate steven_openssl as openssl;
extern crate hyper;
extern crate flate2;
extern crate rand;
extern crate rustc_serialize;

use std::sync::{Arc, RwLock};
use std::marker::PhantomData;
use glfw::{Action, Context, Key};

const CL_BRAND: console::CVar<String> = console::CVar {
    ty: PhantomData,
    name: "cl_brand",
    description: "cl_brand has the value of the clients current 'brand'. e.g. \"Steven\" or \"Vanilla\"",
    mutable: false,
    serializable: false, 
    default: &|| "steven".to_owned(),
};

pub struct Game {
    renderer: render::Renderer,
    screen_sys: screen::ScreenSystem,
    resource_manager: Arc<RwLock<resources::Manager>>,
    console: console::Console,
}

fn main() {
    let mut con = console::Console::new();
    con.register(CL_BRAND);

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
    window.set_scroll_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.make_current();
    glfw.set_swap_interval(1);

    let renderer = render::Renderer::new(resource_manager.clone());
    let mut ui_container = ui::Container::new();

    let mut last_frame = time::now();
    let frame_time = (time::Duration::seconds(1).num_nanoseconds().unwrap() as f64) / 60.0;

    let mut screen_sys = screen::ScreenSystem::new();
    screen_sys.add_screen(Box::new(screen::Login::new()));

    let mut game = Game {
        renderer: renderer,
        screen_sys: screen_sys,
        resource_manager: resource_manager,
        console: con,
    };

    while !window.should_close() {
        { game.resource_manager.write().unwrap().tick(); }
        let now = time::now();
        let diff = now - last_frame;
        last_frame = now;
        let delta = (diff.num_nanoseconds().unwrap() as f64) / frame_time;

        game.screen_sys.tick(delta, &mut game.renderer, &mut ui_container);

        let (width, height) = window.get_framebuffer_size();
        ui_container.tick(&mut game.renderer, delta, width as f64, height as f64);
        game.renderer.tick(delta, width as u32, height as u32);

        window.swap_buffers();
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, &mut game, &mut ui_container, event);
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, game: &mut Game, ui_container: &mut ui::Container, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        },
        glfw::WindowEvent::Scroll(x, y) => {
            game.screen_sys.on_scroll(x, y);
        },
        glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
            let (width, height) = window.get_size();
            let (xpos, ypos) = window.get_cursor_pos();
            let (fw, fh) = window.get_framebuffer_size();
            ui_container.click_at(game, xpos*((fw as f64)/(width as f64)), ypos*((fh as f64)/(height as f64)), fw as f64, fh as f64)
        },
        glfw::WindowEvent::CursorPos(xpos, ypos) => {
            let (width, height) = window.get_size();
            let (fw, fh) = window.get_framebuffer_size();
            ui_container.hover_at(game, xpos*((fw as f64)/(width as f64)), ypos*((fh as f64)/(height as f64)), fw as f64, fh as f64)            
        }
        _ => {}
    }
}
