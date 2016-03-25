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

use protocol::{self, mojang, packet};
use world;
use world::block;
use rand::{self, Rng};
use std::sync::{Arc, RwLock, Mutex};
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use resources;
use openssl;
use console;
use render;
use auth;
use cgmath::{self, Vector};
use sdl2::keyboard::Keycode;

pub struct Server {
    conn: Option<protocol::Conn>,
    read_queue: Option<mpsc::Receiver<Result<packet::Packet, protocol::Error>>>,

    pub world: world::World,
    world_age: i64,
    world_time: f64,
    world_time_target: f64,
    tick_time: bool,

    resources: Arc<RwLock<resources::Manager>>,
    console: Arc<Mutex<console::Console>>,
    version: usize,

    pub position: cgmath::Vector3<f64>,
    pub yaw: f64,
    pub pitch: f64,

    pressed_keys: HashMap<Keycode, bool>,

    tick_timer: f64,
}

macro_rules! handle_packet {
    ($s:ident $pck:ident {
        $($packet:ident => $func:ident,)*
    }) => (
        match $pck {
        $(
            protocol::packet::Packet::$packet(val) => $s.$func(val),
        )*
            _ => {},
        }
    )
}

impl Server {

    pub fn connect(resources: Arc<RwLock<resources::Manager>>, console: Arc<Mutex<console::Console>>, address: &str) -> Result<Server, protocol::Error> {
        let mut conn = try!(protocol::Conn::new(address));

        let profile = {
            let console = console.lock().unwrap();
            mojang::Profile {
                username: console.get(auth::CL_USERNAME).clone(),
                id: console.get(auth::CL_UUID).clone(),
                access_token: console.get(auth::AUTH_TOKEN).clone(),
            }
        };

        let host = conn.host.clone();
        let port = conn.port;
        try!(conn.write_packet(protocol::packet::handshake::serverbound::Handshake {
             protocol_version: protocol::VarInt(protocol::SUPPORTED_PROTOCOL),
             host: host,
             port: port,
             next: protocol::VarInt(2),
         }));
        conn.state = protocol::State::Login;
        try!(conn.write_packet(protocol::packet::login::serverbound::LoginStart {
            username: profile.username.clone(),
        }));

        let packet = match try!(conn.read_packet()) {
            protocol::packet::Packet::EncryptionRequest(val) => val,
            protocol::packet::Packet::LoginDisconnect(val) => return Err(protocol::Error::Disconnect(val.reason)),
            val => return Err(protocol::Error::Err(format!("Wrong packet: {:?}", val))),
        };

        let mut key = openssl::PublicKey::new(&packet.public_key.data);
        let shared = openssl::gen_random(16);

        let shared_e = key.encrypt(&shared);
        let token_e = key.encrypt(&packet.verify_token.data);

        try!(profile.join_server(&packet.server_id, &shared, &packet.public_key.data));

        try!(conn.write_packet(protocol::packet::login::serverbound::EncryptionResponse {
            shared_secret: protocol::LenPrefixedBytes::new(shared_e),
            verify_token: protocol::LenPrefixedBytes::new(token_e),
        }));

        let mut read = conn.clone();
        let mut write = conn.clone();

        read.enable_encyption(&shared, true);
        write.enable_encyption(&shared, false);

        loop {
           match try!(read.read_packet()) {
               protocol::packet::Packet::SetInitialCompression(val) => {
                   read.set_compresssion(val.threshold.0, true);
                   write.set_compresssion(val.threshold.0, false);
               }
               protocol::packet::Packet::LoginSuccess(val) => {
                   debug!("Login: {} {}", val.username, val.uuid);
                   read.state = protocol::State::Play;
                   write.state = protocol::State::Play;
                   break;
               }
               protocol::packet::Packet::LoginDisconnect(val) => return Err(protocol::Error::Disconnect(val.reason)),
               val => return Err(protocol::Error::Err(format!("Wrong packet: {:?}", val))),
           }
        }

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            loop {
                let pck = read.read_packet();
                let was_error = pck.is_err();
                if let Err(_) = tx.send(pck) {
                    return;
                }
                if was_error {
                    return;
                }
            }
        });

        let version = resources.read().unwrap().version();
        Ok(Server {
            conn: Some(write),
            read_queue: Some(rx),

            world: world::World::new(),
            world_age: 0,
            world_time: 0.0,
            world_time_target: 0.0,
            tick_time: true,

            resources: resources,
            console: console,
            version: version,

            pressed_keys: HashMap::new(),

            position: cgmath::Vector3::zero(),
            yaw: 0.0,
            pitch: 0.0,

            tick_timer: 0.0,
        })
    }

    pub fn dummy_server(resources: Arc<RwLock<resources::Manager>>, console: Arc<Mutex<console::Console>>) -> Server {
        let mut world = world::World::new();
        let mut rng = rand::thread_rng();
        for x in -7*16 .. 7*16 {
            for z in -7*16 .. 7*16 {
                let h = rng.gen_range(3, 10);
                for y in 0 .. h {
                    world.set_block(x, y, z, block::Dirt{});
                }
            }
        }
        let version = resources.read().unwrap().version();
        Server {
            conn: None,
            read_queue: None,

            world: world,
            world_age: 0,
            world_time: 0.0,
            world_time_target: 0.0,
            tick_time: true,

            version: version,
            resources: resources,
            console: console,

            pressed_keys: HashMap::new(),

            position: cgmath::Vector3::new(0.5, 13.2, 0.5),
            yaw: 0.0,
            pitch: 0.0,

            tick_timer: 0.0,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    pub fn tick(&mut self, renderer: &mut render::Renderer, delta: f64) {
        let version = self.resources.read().unwrap().version();
        if version != self.version {
            self.version = version;
            self.world.flag_dirty_all();
        }

        if let Some(rx) = self.read_queue.take() {
            while let Ok(pck) = rx.try_recv() {
                match pck {
                    Ok(pck) => handle_packet!{
                        self pck {
                            KeepAliveClientbound => on_keep_alive,
                            ChunkData => on_chunk_data,
                            ChunkUnload => on_chunk_unload,
                            TeleportPlayer => on_teleport,
                            TimeUpdate => on_time_update,
                        }
                    },
                    Err(err) => panic!("Err: {:?}", err),
                }
            }
            self.read_queue = Some(rx);
        }

        let (forward, yaw) = self.calculate_movement();

        if self.world.is_chunk_loaded((self.position.x as i32) >> 4, (self.position.z as i32) >> 4) {
                let mut speed = 4.317 / 60.0;
                if self.is_key_pressed(Keycode::LShift) {
                    speed = 5.612 / 60.0;
                }
                // TODO: only do this for flying
                speed *= 2.5;

                if self.is_key_pressed(Keycode::Space) {
                    self.position.y += speed * delta;
                }
                if self.is_key_pressed(Keycode::LCtrl) {
                    self.position.y -= speed * delta;
                }
                self.position.x += forward * yaw.cos() * delta * speed;
                self.position.z -= forward * yaw.sin() * delta * speed;
        }

        self.tick_timer += delta;
        while self.tick_timer >= 3.0 && self.is_connected() {
            self.minecraft_tick();
            self.tick_timer -= 3.0;
        }

        self.update_time(renderer, delta);

        // Copy to camera
        renderer.camera.pos = cgmath::Point::from_vec(self.position + cgmath::Vector3::new(0.0, 1.8, 0.0));
        renderer.camera.yaw = self.yaw;
        renderer.camera.pitch = self.pitch;
    }

    fn update_time(&mut self, renderer: &mut render::Renderer, delta: f64) {
        if self.tick_time {
            self.world_time_target += delta / 3.0;
            self.world_time_target = (24000.0 + self.world_time_target) % 24000.0;
        	let mut diff = self.world_time_target - self.world_time;
        	if diff < -12000.0 {
        		diff = 24000.0 + diff
        	} else if diff > 12000.0 {
        		diff = diff - 24000.0
        	}
            self.world_time += diff * (1.5 / 60.0) * delta;
            self.world_time = (24000.0 + self.world_time) % 24000.0;
        } else {
            self.world_time = self.world_time_target;
        }
        renderer.sky_offset = self.calculate_sky_offset();
    }

    fn calculate_sky_offset(&self) -> f32 {
        use std::f32::consts::PI;
        let mut offset = ((1.0 + self.world_time as f32) / 24000.0) - 0.25;
        if offset < 0.0 {
            offset += 1.0;
        } else if offset > 1.0 {
            offset -= 1.0;
        }

        let prev_offset = offset;
        offset = 1.0 - (((offset * PI).cos() + 1.0) / 2.0);
        offset = prev_offset + (offset - prev_offset) / 3.0;

        offset = 1.0 - ((offset * PI * 2.0).cos() * 2.0 + 0.2);
        if offset > 1.0 {
            offset = 1.0;
        } else if offset < 0.0 {
            offset = 0.0;
        }
        offset = 1.0 - offset;
        offset * 0.8 + 0.2
    }

    fn calculate_movement(&self) -> (f64, f64) {
        use std::f64::consts::PI;
        let mut forward = 0.0f64;
        let mut yaw = self.yaw - (PI/2.0);
        if self.is_key_pressed(Keycode::W) || self.is_key_pressed(Keycode::S) {
            forward = 1.0;
            if self.is_key_pressed(Keycode::S) {
                yaw += PI;
            }
        }
        let mut change = 0.0;
        if self.is_key_pressed(Keycode::A) {
            change = (PI / 2.0) / (forward.abs() + 1.0);
        }
        if self.is_key_pressed(Keycode::D) {
            change = -(PI / 2.0) / (forward.abs() + 1.0);
        }
        if self.is_key_pressed(Keycode::A) || self.is_key_pressed(Keycode::D) {
            forward = 1.0;
        }
        if self.is_key_pressed(Keycode::S) {
            yaw -= change;
        } else {
            yaw += change;
        }

        (forward, yaw)
    }

    pub fn minecraft_tick(&mut self) {

        // Sync our position to the server
        let packet = packet::play::serverbound::PlayerPositionLook {
            x: self.position.x,
            y: self.position.y,
            z: self.position.z,
            yaw: self.yaw as f32,
            pitch: self.pitch as f32,
            on_ground: false,
        };
        self.write_packet(packet);
    }

    pub fn key_press(&mut self, down: bool, key: Keycode) {
        self.pressed_keys.insert(key, down);
    }

    fn is_key_pressed(&self, key: Keycode) -> bool {
        self.pressed_keys.get(&key).map(|v| *v).unwrap_or(false)
    }

    pub fn write_packet<T: protocol::PacketType>(&mut self, p: T) {
        self.conn.as_mut().unwrap().write_packet(p).unwrap(); // TODO handle errors
    }

    fn on_keep_alive(&mut self, keep_alive: packet::play::clientbound::KeepAliveClientbound) {
        self.write_packet(packet::play::serverbound::KeepAliveServerbound {
            id: keep_alive.id,
        });
    }

    fn on_time_update(&mut self, time_update: packet::play::clientbound::TimeUpdate) {
        self.world_age = time_update.time_of_day;
        self.world_time_target = (time_update.time_of_day % 24000) as f64;
        if self.world_time_target < 0.0 {
            self.world_time_target = -self.world_time_target;
            self.tick_time = false;
        } else {
            self.tick_time = true;
        }
    }

    fn on_teleport(&mut self, teleport: packet::play::clientbound::TeleportPlayer) {
        // TODO: relative teleports
        self.position.x = teleport.x;
        self.position.y = teleport.y;
        self.position.z = teleport.z;
        self.yaw = teleport.yaw as f64;
        self.pitch = teleport.pitch as f64;

        self.write_packet(packet::play::serverbound::TeleportConfirm {
            teleport_id: teleport.teleport_id,
        });
    }

    fn on_chunk_data(&mut self, chunk_data: packet::play::clientbound::ChunkData) {
        self.world.load_chunk(
            chunk_data.chunk_x,
            chunk_data.chunk_z,
            chunk_data.new,
            chunk_data.bitmask.0 as u16,
            chunk_data.data.data
        ).unwrap();
    }

    fn on_chunk_unload(&mut self, chunk_unload: packet::play::clientbound::ChunkUnload) {
        self.world.unload_chunk(chunk_unload.x, chunk_unload.z);
    }
}
