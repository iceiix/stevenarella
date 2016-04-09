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

#![recursion_limit="300"]
#![feature(const_fn)]

extern crate sdl2;
extern crate image;
extern crate time;
extern crate byteorder;
extern crate serde_json;
extern crate openssl;
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
pub extern crate steven_blocks;
extern crate steven_shared as shared;

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
pub mod settings;
#[macro_use]
pub mod console;
pub mod server;
pub mod world;
pub mod chunk_builder;
pub mod auth;
pub mod model;
pub mod entity;

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
            self.renderer.camera.yaw += 0.005 * delta;
            if self.renderer.camera.yaw > ::std::f64::consts::PI * 2.0 {
                self.renderer.camera.yaw = 0.0;
            }
        }

        if let Some(disconnect_reason) = self.server.disconnect_reason.take() {
            self.screen_sys.replace_screen(Box::new(screen::ServerList::new(
                Some(disconnect_reason)
            )));
        }
        if !self.server.is_connected() {
            self.focused = false;
        }

        let mut clear_reply = false;
        if let Some(ref recv) = self.connect_reply {
            if let Ok(server) = recv.try_recv() {
                clear_reply = true;
                match server {
                    Ok(val) => {
                        self.screen_sys.pop_screen();
                        self.focused = true;
                        self.server.remove(&mut self.renderer);
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
        settings::register_vars(&mut con);
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

    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();

    sdl_video.gl_set_swap_interval(1);

    let window = sdl2::video::WindowBuilder::new(&sdl_video, "Steven", 854, 480)
                            .opengl()
                            .resizable()
                            .build()
                            .expect("Could not create sdl window.");
    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_stencil_size(0);
    gl_attr.set_depth_size(24);
    gl_attr.set_context_major_version(3);
    gl_attr.set_context_minor_version(2);
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);

    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).expect("Could not set current context.");

    gl::init(&sdl_video);

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
        chunk_builder: chunk_builder::ChunkBuilder::new(resource_manager, textures),
        connect_reply: None,
    };
    game.renderer.camera.pos = cgmath::Point3::new(0.5, 13.2, 0.5);

    let mut events = sdl.event_pump().unwrap();
    while !game.should_close {
        let version = {
            let mut res = game.resource_manager.write().unwrap();
            res.tick();
            res.version()
        };

        let now = time::now();
        let diff = now - last_frame;
        last_frame = now;
        let delta = (diff.num_nanoseconds().unwrap() as f64) / frame_time;
        let (width, height) = window.drawable_size();

        game.tick(delta);
        game.server.tick(&mut game.renderer, delta);

        game.renderer.update_camera(width, height);
        game.server.world.compute_render_list(&mut game.renderer);
        game.chunk_builder.tick(&mut game.server.world, &mut game.renderer, version);

        game.screen_sys.tick(delta, &mut game.renderer, &mut ui_container);
        game.console
            .lock()
            .unwrap()
            .tick(&mut ui_container, &mut game.renderer, delta, width as f64);
        ui_container.tick(&mut game.renderer, delta, width as f64, height as f64);
        game.renderer.tick(&mut game.server.world, delta, width, height);

        window.gl_swap_window();

        for event in events.poll_iter() {
            handle_window_event(&window, &mut game, &mut ui_container, event)
        }
    }
}

fn handle_window_event(window: &sdl2::video::Window,
                       game: &mut Game,
                       ui_container: &mut ui::Container,
                       event: sdl2::event::Event) {
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use sdl2::mouse::Mouse;
    use std::f64::consts::PI;

    let mouse = window.subsystem().sdl().mouse();

    match event {
        Event::Quit{..} => game.should_close = true,

        Event::MouseMotion{x, y, xrel, yrel, ..} => {
            let (width, height) = window.size();
            if game.focused {
                if !mouse.relative_mouse_mode() {
                    mouse.set_relative_mouse_mode(true);
                }
                if let Some(player) = game.server.player {
                    let s = 2000.0 + 0.01;
                    let (rx, ry) = (xrel as f64 / s, yrel as f64 / s);
                    let rotation = game.server.entities.get_component_mut(player, game.server.rotation).unwrap();
                    rotation.yaw -= rx;
                    rotation.pitch -= ry;
                    if rotation.pitch < (PI/2.0) + 0.01 {
                        rotation.pitch = (PI/2.0) + 0.01;
                    }
                    if rotation.pitch > (PI/2.0)*3.0 - 0.01 {
                        rotation.pitch = (PI/2.0)*3.0 - 0.01;
                    }
                }
            } else {
                if mouse.relative_mouse_mode() {
                    mouse.set_relative_mouse_mode(false);
                }
                ui_container.hover_at(game, x as f64, y as f64, width as f64, height as f64);
            }
        }
        Event::MouseButtonUp{mouse_btn: Mouse::Left, x, y, ..} => {
            let (width, height) = window.size();

            if game.server.is_connected() && !game.focused && !game.screen_sys.is_current_closable() {
                game.focused = true;
                if !mouse.relative_mouse_mode() {
                    mouse.set_relative_mouse_mode(true);
                }
                return;
            }
            if !game.focused {
                if mouse.relative_mouse_mode() {
                    mouse.set_relative_mouse_mode(false);
                }
                ui_container.click_at(game, x as f64, y as f64, width as f64, height as f64);
            }
        }
        Event::MouseWheel{x, y, ..} => {
            game.screen_sys.on_scroll(x as f64, y as f64);
        }
        Event::KeyUp{keycode: Some(Keycode::Escape), ..} => {
            if game.focused {
                mouse.set_relative_mouse_mode(false);
                game.focused = false;
                game.screen_sys.replace_screen(Box::new(screen::SettingsMenu::new(game.console.clone(), true)));
            } else if game.screen_sys.is_current_closable() {
                mouse.set_relative_mouse_mode(true);
                game.focused = true;
                game.screen_sys.pop_screen();
            }
        }
        Event::KeyDown{keycode: Some(Keycode::Backquote), ..} => {
            game.console.lock().unwrap().toggle();
        }
        Event::KeyDown{keycode: Some(key), ..} => {
            if game.focused {
                let console = game.console.lock().unwrap();
                if let Some(steven_key) = settings::Stevenkey::get_by_keycode(key, &console) {
                    game.server.key_press(true, steven_key);
                }
            } else {
                ui_container.key_press(game, key, true);
            }
        }
        Event::KeyUp{keycode: Some(key), ..} => {
            if game.focused {
                let console = game.console.lock().unwrap();
                if let Some(steven_key) = settings::Stevenkey::get_by_keycode(key, &console) {
                    game.server.key_press(false, steven_key);
                }
            } else {
                ui_container.key_press(game, key, false);
            }
        }
        Event::TextInput{text, ..} => {
            if !game.focused {
                for c in text.chars() {
                    ui_container.key_type(game, c);
                }
            }
        }
        _ => (),
    }
}
