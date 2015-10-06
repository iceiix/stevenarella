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

extern crate glutin;
extern crate image;
extern crate time;
extern crate byteorder;
extern crate serde_json;
extern crate steven_openssl as openssl;
extern crate hyper;
extern crate flate2;
extern crate rand;
extern crate rustc_serialize;
#[macro_use]
extern crate log;

use std::sync::{Arc, RwLock, Mutex};
use std::marker::PhantomData;

const CL_BRAND: console::CVar<String> = console::CVar {
    ty: PhantomData,
    name: "cl_brand",
    description: "cl_brand has the value of the clients current 'brand'. \
                e.g. \"Steven\" or \"Vanilla\"",
    mutable: false,
    serializable: false,
    default: &|| "steven".to_owned(),
};

pub struct Game {
    renderer: render::Renderer,
    screen_sys: screen::ScreenSystem,
    resource_manager: Arc<RwLock<resources::Manager>>,
    console: Arc<Mutex<console::Console>>,
    should_close: bool,
    mouse_pos: (i32, i32),
}

fn main() {
    let con = Arc::new(Mutex::new(console::Console::new()));
    {
        let mut con = con.lock().unwrap();
        con.register(CL_BRAND);
        con.load_config();
        con.save_config();
    }

    let proxy = console::ConsoleProxy::new(con.clone());

    log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Trace);
        Box::new(proxy)
    }).unwrap();

    info!("Starting steven");

    let resource_manager = Arc::new(RwLock::new(resources::Manager::new()));
    { resource_manager.write().unwrap().tick(); }

    let mut window = glutin::WindowBuilder::new()
        .with_title("Steven".to_string())
        .with_dimensions(854, 480)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
        .with_gl_profile(glutin::GlProfile::Core)
        .with_depth_buffer(24)
        .with_stencil_buffer(0)
        .with_vsync()
        .build().ok().expect("Could not create Glutin window.");

    unsafe {
        window.make_current().ok().expect("Could not set current context.");
    }

    gl::init(&mut window);

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
        should_close: false,
        mouse_pos: (0, 0),
    };

    while !game.should_close {
        { game.resource_manager.write().unwrap().tick(); }

        let now = time::now();
        let diff = now - last_frame;
        last_frame = now;
        let delta = (diff.num_nanoseconds().unwrap() as f64) / frame_time;
        let (width, height) = window.get_inner_size_pixels().unwrap();

        game.screen_sys.tick(delta, &mut game.renderer, &mut ui_container);
        game.console.lock().unwrap().tick(&mut ui_container, &mut game.renderer, delta, width as f64);
        ui_container.tick(&mut game.renderer, delta, width as f64, height as f64);
        game.renderer.tick(delta, width, height);

        let _ = window.swap_buffers();

        for event in window.poll_events() {
            handle_window_event(&window, &mut game, &mut ui_container, event)
        }
    }
}

fn handle_window_event(window: &glutin::Window, game: &mut Game, ui_container: &mut ui::Container, event: glutin::Event) {
    use glutin::{Event, VirtualKeyCode};
    match event {
        Event::Closed => game.should_close = true,

        Event::MouseMoved((x, y)) => {
            game.mouse_pos = (x, y);
            let (width, height) = window.get_inner_size_pixels().unwrap();

            ui_container.hover_at(game, x as f64, y as f64, width as f64, height as f64);
        },

        Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Left) => {
            let (x, y) = game.mouse_pos;
            let (width, height) = window.get_inner_size_pixels().unwrap();

            ui_container.click_at(game, x as f64, y as f64, width as f64, height as f64);
        },

        Event::MouseWheel(delta) => {
            let (x, y) = match delta {
                glutin::MouseScrollDelta::LineDelta(x, y) => (x, y),
                glutin::MouseScrollDelta::PixelDelta(x, y) => (x, y)
            };

            game.screen_sys.on_scroll(x as f64, y as f64);
        },

        Event::KeyboardInput(glutin::ElementState::Pressed, 41 /* ` GRAVE */, _) => {
            game.console.lock().unwrap().toggle();
        },
        Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(VirtualKeyCode::Grave)) => {
            game.console.lock().unwrap().toggle();
        },
        Event::KeyboardInput(glutin::ElementState::Pressed, key, virt) => {
            println!("Key: {:?} {:?}", key, virt);
        },

        _ => ()
    }
}
