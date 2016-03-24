// Copyright 2016 Matthew Collins
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

#![recursion_limit="200"]
#![feature(const_fn)]
#![feature(arc_counts)]

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
extern crate cgmath;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate collision;

#[macro_use]
pub mod macros;

pub mod ecs;
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
pub mod server;
pub mod world;
pub mod chunk_builder;
pub mod auth;
pub mod model;

use std::sync::{Arc, RwLock, Mutex};
use std::marker::PhantomData;
use std::thread;
use std::sync::mpsc;

const CL_BRAND: console::CVar<String> = console::CVar {
    ty: PhantomData,
    name: "cl_brand",
    description: "cl_brand has the value of the clients current 'brand'. e.g. \"Steven\" or \
                  \"Vanilla\"",
    mutable: false,
    serializable: false,
    default: &|| "Steven".to_owned(),
};

pub struct Game {
    renderer: render::Renderer,
    screen_sys: screen::ScreenSystem,
    resource_manager: Arc<RwLock<resources::Manager>>,
    console: Arc<Mutex<console::Console>>,
    should_close: bool,
    mouse_pos: (i32, i32),

    server: server::Server,
    focused: bool,
    chunk_builder: chunk_builder::ChunkBuilder,

    connect_reply: Option<mpsc::Receiver<Result<server::Server, protocol::Error>>>,
}

impl Game {
    pub fn connect_to(&mut self, address: &str) {
        let (tx, rx) = mpsc::channel();
        self.connect_reply = Some(rx);
        let address = address.to_owned();
        let resources = self.resource_manager.clone();
        let console = self.console.clone();
        thread::spawn(move || {
            tx.send(server::Server::connect(resources, console, &address)).unwrap();
        });
    }

    pub fn tick(&mut self, delta: f64) {
        if !self.server.is_connected() {
            self.server.yaw += 0.005 * delta;
            if self.server.yaw > ::std::f64::consts::PI * 2.0 {
                self.server.yaw = 0.0;
            }
        }
        let mut clear_reply = false;
        if let Some(ref recv) = self.connect_reply {
            if let Ok(server) = recv.try_recv() {
                clear_reply = true;
                match server {
                    Ok(val) => {
                        self.screen_sys.pop_screen();
                        self.focused = true;
                        self.server = val;
                    },
                    Err(err) => {
                        let msg = match err {
                            protocol::Error::Disconnect(val) => val,
                            err => {
                                let mut msg = format::TextComponent::new(&format!("{}", err));
                                msg.modifier.color = Some(format::Color::Red);
                                format::Component::Text(msg)
                            },
                        };
                        self.screen_sys.replace_screen(Box::new(screen::ServerList::new(
                            Some(msg)
                        )));
                    }
                }
            }
        }
        if clear_reply {
            self.connect_reply = None;
        }
    }
}

fn main() {
    let con = Arc::new(Mutex::new(console::Console::new()));
    {
        let mut con = con.lock().unwrap();
        con.register(CL_BRAND);
        auth::register_vars(&mut con);
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
    {
        resource_manager.write().unwrap().tick();
    }

    let mut window = glutin::WindowBuilder::new()
                         .with_title("Steven".to_string())
                         .with_dimensions(854, 480)
                         .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
                         .with_gl_profile(glutin::GlProfile::Core)
                         .with_depth_buffer(24)
                         .with_stencil_buffer(0)
                         .with_vsync()
                         .build()
                         .expect("Could not create Glutin window.");

    unsafe {
        window.make_current().expect("Could not set current context.");
    }

    gl::init(&mut window);

    let renderer = render::Renderer::new(resource_manager.clone());
    let mut ui_container = ui::Container::new();

    let mut last_frame = time::now();
    let frame_time = (time::Duration::seconds(1).num_nanoseconds().unwrap() as f64) / 60.0;

    let mut screen_sys = screen::ScreenSystem::new();
    screen_sys.add_screen(Box::new(screen::Login::new(con.clone())));

    let textures = renderer.get_textures();
    let mut game = Game {
        server: server::Server::dummy_server(resource_manager.clone(), con.clone()),
        focused: false,
        renderer: renderer,
        screen_sys: screen_sys,
        resource_manager: resource_manager.clone(),
        console: con,
        should_close: false,
        mouse_pos: (0, 0),
        chunk_builder: chunk_builder::ChunkBuilder::new(resource_manager, textures),
        connect_reply: None,
    };

    while !game.should_close {
        {
            game.resource_manager.write().unwrap().tick();
        }

        let now = time::now();
        let diff = now - last_frame;
        last_frame = now;
        let delta = (diff.num_nanoseconds().unwrap() as f64) / frame_time;
        let (width, height) = window.get_inner_size_pixels().unwrap();

        game.tick(delta);
        game.server.tick(&mut game.renderer, delta);

        game.chunk_builder.tick(&mut game.server.world, &mut game.renderer, delta);

        game.screen_sys.tick(delta, &mut game.renderer, &mut ui_container);
        game.console
            .lock()
            .unwrap()
            .tick(&mut ui_container, &mut game.renderer, delta, width as f64);
        ui_container.tick(&mut game.renderer, delta, width as f64, height as f64);
        game.renderer.tick(delta, width, height);

        let _ = window.swap_buffers();

        for event in window.poll_events() {
            handle_window_event(&window, &mut game, &mut ui_container, event)
        }
    }
}

fn handle_window_event(window: &glutin::Window,
                       game: &mut Game,
                       ui_container: &mut ui::Container,
                       event: glutin::Event) {
    use glutin::{Event, VirtualKeyCode};
    use std::f64::consts::PI;
    match event {
        Event::Closed => game.should_close = true,

        Event::MouseMoved((x, y)) => {
            game.mouse_pos = (x, y);
            let (width, height) = window.get_inner_size_pixels().unwrap();
            if game.focused {
                window.set_cursor_state(glutin::CursorState::Hide).unwrap();
                window.set_cursor_position((width/2) as i32, (height/2) as i32).unwrap();
                let s = 2000.0 + 0.01;
                let (rx, ry) = ((x-(width/2) as i32) as f64 / s, (y-(height/2) as i32) as f64 / s);
                game.server.yaw -= rx;
                game.server.pitch -= ry;
                if game.server.pitch < (PI/2.0) + 0.01 {
                    game.server.pitch = (PI/2.0) + 0.01;
                }
                if game.server.pitch > (PI/2.0)*3.0 - 0.01 {
                    game.server.pitch = (PI/2.0)*3.0 - 0.01;
                }
            } else {
                ui_container.hover_at(game, x as f64, y as f64, width as f64, height as f64);
            }
        }
        Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Left) => {
            let (x, y) = game.mouse_pos;
            let (width, height) = window.get_inner_size_pixels().unwrap();

            if game.server.is_connected() && !game.focused {
                game.focused = true;
                window.set_cursor_state(glutin::CursorState::Hide).unwrap();
                window.set_cursor_position((width/2) as i32, (height/2) as i32).unwrap();
                return;
            }
            if !game.focused {
                ui_container.click_at(game, x as f64, y as f64, width as f64, height as f64);
            }
        }
        Event::MouseWheel(delta, _) => {
            let (x, y) = match delta {
                glutin::MouseScrollDelta::LineDelta(x, y) => (x, y),
                glutin::MouseScrollDelta::PixelDelta(x, y) => (x, y),
            };

            game.screen_sys.on_scroll(x as f64, y as f64);
        }
        Event::KeyboardInput(glutin::ElementState::Released, _, Some(VirtualKeyCode::Escape)) => {
            if game.focused {
                window.set_cursor_state(glutin::CursorState::Normal).unwrap();
                game.focused = false;
            }
        }
        Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(VirtualKeyCode::Grave)) => {
            game.console.lock().unwrap().toggle();
        }
        Event::KeyboardInput(state, key, virt) => {
            if game.focused {
                if let Some(virt) = virt {
                    game.server.key_press(state == glutin::ElementState::Pressed, virt);
                }
            } else {
                ui_container.key_press(game, virt, key, state == glutin::ElementState::Pressed);
            }
        }
        Event::ReceivedCharacter(c) => {
            if !game.focused {
                ui_container.key_type(game, c);
            }
        }
        _ => (),
    }
}
