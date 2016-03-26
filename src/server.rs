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
use cgmath::{self, Vector, Point3};
use collision::{Aabb, Aabb3};
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
    last_position: cgmath::Vector3<f64>,
    pub yaw: f64,
    pub pitch: f64,
    bounds: Aabb3<f64>,
    gamemode: Gamemode,
    flying: bool,
    on_ground: bool,
    did_touch_ground: bool,
    v_speed: f64,

    pressed_keys: HashMap<Keycode, bool>,

    tick_timer: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum Gamemode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

impl Gamemode {
    pub fn from_int(val: i32) -> Gamemode {
        match val {
            3 => Gamemode::Spectator,
            2 => Gamemode::Adventure,
            1 => Gamemode::Creative,
            0 | _ => Gamemode::Survival,
        }
    }

    pub fn can_fly(&self) -> bool {
        match *self {
            Gamemode::Creative | Gamemode::Spectator => true,
            _ => false,
        }
    }

    pub fn always_fly(&self) -> bool {
        match *self {
            Gamemode::Spectator => true,
            _ => false,
        }
    }

    pub fn noclip(&self) -> bool {
        match *self {
            Gamemode::Spectator => true,
            _ => false,
        }
    }
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

        Ok(Server::new(resources, console, Some(write), Some(rx)))
    }

    pub fn dummy_server(resources: Arc<RwLock<resources::Manager>>, console: Arc<Mutex<console::Console>>) -> Server {
        let mut server = Server::new(resources, console, None, None);
        let mut rng = rand::thread_rng();
        for x in -7*16 .. 7*16 {
            for z in -7*16 .. 7*16 {
                let h = rng.gen_range(3, 10);
                for y in 0 .. h {
                    server.world.set_block(x, y, z, block::Dirt{});
                }
            }
        }
        server.gamemode = Gamemode::Spectator;
        server.flying = false;
        server
    }

    fn new(
        resources: Arc<RwLock<resources::Manager>>, console: Arc<Mutex<console::Console>>,
        conn: Option<protocol::Conn>, read_queue: Option<mpsc::Receiver<Result<packet::Packet, protocol::Error>>>
    ) -> Server {
        let version = resources.read().unwrap().version();
        Server {
            conn: conn,
            read_queue: read_queue,

            world: world::World::new(),
            world_age: 0,
            world_time: 0.0,
            world_time_target: 0.0,
            tick_time: true,

            version: version,
            resources: resources,
            console: console,

            pressed_keys: HashMap::new(),

            position: cgmath::Vector3::new(0.5, 13.2, 0.5),
            last_position: cgmath::Vector3::zero(),
            yaw: 0.0,
            pitch: 0.0,
            bounds: Aabb3::new(
                Point3::new(-0.3, 0.0, -0.3),
                Point3::new(0.3, 1.8, 0.3)
            ),
            gamemode: Gamemode::Survival,
            flying: false,
            on_ground: false,
            did_touch_ground: false,
            v_speed: 0.0,

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
                            JoinGame => on_game_join,
                            Respawn => on_respawn,
                            KeepAliveClientbound => on_keep_alive,
                            ChunkData => on_chunk_data,
                            ChunkUnload => on_chunk_unload,
                            TeleportPlayer => on_teleport,
                            TimeUpdate => on_time_update,
                            ChangeGameState => on_game_state_change,
                        }
                    },
                    Err(err) => panic!("Err: {:?}", err),
                }
            }
            self.read_queue = Some(rx);
        }

        self.flying |= self.gamemode.always_fly();
        self.last_position = self.position;
        if self.world.is_chunk_loaded((self.position.x as i32) >> 4, (self.position.z as i32) >> 4) {
            let (forward, yaw) = self.calculate_movement();
            let mut speed = 4.317 / 60.0;
            if self.is_key_pressed(Keycode::LShift) {
                speed = 5.612 / 60.0;
            }
            if self.flying {
                speed *= 2.5;

                if self.is_key_pressed(Keycode::Space) {
                    self.position.y += speed * delta;
                }
                if self.is_key_pressed(Keycode::LCtrl) {
                    self.position.y -= speed * delta;
                }
            } else if self.on_ground {
                if self.is_key_pressed(Keycode::Space) {
                    self.v_speed = 0.15;
                } else {
                    self.v_speed = 0.0;
                }
            } else {
                self.v_speed -= 0.01 * delta;
                if self.v_speed < -0.3 {
                    self.v_speed = -0.3;
                }
            }
            self.position.x += forward * yaw.cos() * delta * speed;
            self.position.z -= forward * yaw.sin() * delta * speed;
            self.position.y += self.v_speed * delta;
        }

        if !self.gamemode.noclip() {
            let mut target = self.position;
            self.position.y = self.last_position.y;
            self.position.z = self.last_position.z;

    		// We handle each axis separately to allow for a sliding
    		// effect when pushing up against walls.

            let (bounds, xhit) = self.check_collisions(self.bounds);
            self.position.x = bounds.min.x + 0.3;
            self.last_position.x = self.position.x;

            self.position.z = target.z;
            let (bounds, zhit) = self.check_collisions(self.bounds);
            self.position.z = bounds.min.z + 0.3;
            self.last_position.z = self.position.z;

    		// Half block jumps
    		// Minecraft lets you 'jump' up 0.5 blocks
    		// for slabs and stairs (or smaller blocks).
    		// Currently we implement this as a teleport to the
    		// top of the block if we could move there
    		// but this isn't smooth.
            if (xhit || zhit) && self.on_ground {
                let mut ox = self.position.x;
                let mut oz = self.position.z;
                self.position.x = target.x;
                self.position.z = target.z;
                for offset in 1 .. 9 {
                    let mini = self.bounds.add_v(cgmath::Vector3::new(0.0, offset as f64 / 16.0, 0.0));
                    let (_, hit) = self.check_collisions(mini);
                    if !hit {
                        target.y += offset as f64 / 16.0;
                        ox = target.x;
                        oz = target.z;
                        break;
                    }
                }
                self.position.x = ox;
                self.position.z = oz;
            }

            self.position.y = target.y;
            let (bounds, yhit) = self.check_collisions(self.bounds);
            self.position.y = bounds.min.y;
            self.last_position.y = self.position.y;
            if yhit {
                self.v_speed = 0.0;
            }

            let ground = Aabb3::new(
                Point3::new(-0.3, -0.05, -0.3),
                Point3::new(0.3, 0.0, 0.3)
            );
            let prev = self.on_ground;
            let (_, hit) = self.check_collisions(ground);
            self.on_ground = hit;
            if !prev && self.on_ground {
                self.did_touch_ground = true;
            }
        }

        self.tick_timer += delta;
        while self.tick_timer >= 3.0 && self.is_connected() {
            self.minecraft_tick();
            self.tick_timer -= 3.0;
        }

        self.update_time(renderer, delta);

        // Copy to camera
        renderer.camera.pos = cgmath::Point::from_vec(self.position + cgmath::Vector3::new(0.0, 1.62, 0.0));
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

    fn check_collisions(&self, bounds: Aabb3<f64>) -> (Aabb3<f64>, bool) {
        let mut bounds = bounds.add_v(self.position);

        let dir = self.position - self.last_position;

        let min_x = (bounds.min.x - 1.0) as i32;
        let min_y = (bounds.min.y - 1.0) as i32;
        let min_z = (bounds.min.z - 1.0) as i32;
        let max_x = (bounds.max.x + 1.0) as i32;
        let max_y = (bounds.max.y + 1.0) as i32;
        let max_z = (bounds.max.z + 1.0) as i32;

        let mut hit = false;
        for y in min_y .. max_y {
            for z in min_z .. max_z {
                for x in min_x .. max_x {
                    let block = self.world.get_block(x, y, z);
                    for bb in block.get_collision_boxes() {
                        let bb = bb.add_v(cgmath::Vector3::new(x as f64, y as f64, z as f64));
                        if bb.collides(&bounds) {
                            bounds = bounds.move_out_of(bb, dir);
                            hit = true;
                        }
                    }
                }
            }
        }

        (bounds, hit)
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
    	// Force the server to know when touched the ground
    	// otherwise if it happens between ticks the server
    	// will think we are flying.
        let on_ground = if self.did_touch_ground {
            self.did_touch_ground = false;
            true
        } else {
            self.on_ground
        };

        // Sync our position to the server
        // Use the smaller packets when possible
        let packet = packet::play::serverbound::PlayerPositionLook {
            x: self.position.x,
            y: self.position.y,
            z: self.position.z,
            yaw: self.yaw as f32,
            pitch: self.pitch as f32,
            on_ground: on_ground,
        };
        self.write_packet(packet);
    }

    pub fn key_press(&mut self, down: bool, key: Keycode) {
        self.pressed_keys.insert(key, down);
    }

    fn is_key_pressed(&self, key: Keycode) -> bool {
        self.pressed_keys.get(&key).map_or(false, |v| *v)
    }

    pub fn write_packet<T: protocol::PacketType>(&mut self, p: T) {
        self.conn.as_mut().unwrap().write_packet(p).unwrap(); // TODO handle errors
    }

    fn on_keep_alive(&mut self, keep_alive: packet::play::clientbound::KeepAliveClientbound) {
        self.write_packet(packet::play::serverbound::KeepAliveServerbound {
            id: keep_alive.id,
        });
    }

    fn on_game_join(&mut self, join: packet::play::clientbound::JoinGame) {
        self.gamemode = Gamemode::from_int((join.gamemode & 0x7) as i32);
        // TODO: Temp
        self.flying = self.gamemode.can_fly();
    }

    fn on_respawn(&mut self, respawn: packet::play::clientbound::Respawn) {
        self.world = world::World::new();
        self.gamemode = Gamemode::from_int((respawn.gamemode & 0x7) as i32);
        // TODO: Temp
        self.flying = self.gamemode.can_fly();
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

    fn on_game_state_change(&mut self, game_state: packet::play::clientbound::ChangeGameState) {
        match game_state.reason {
            3 => {
                self.gamemode = Gamemode::from_int(game_state.value as i32);
                // TODO: Temp
                self.flying = self.gamemode.can_fly();
            },
            _ => {},
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

trait Collidable<T> {
    fn collides(&self, t: &T) -> bool;
    fn move_out_of(self, other: Self, dir: cgmath::Vector3<f64>) -> Self;
}

impl Collidable<Aabb3<f64>> for Aabb3<f64> {
    fn collides(&self, t: &Aabb3<f64>) -> bool {
        !(
            t.min.x >= self.max.x ||
            t.max.x <= self.min.x ||
            t.min.y >= self.max.y ||
            t.max.y <= self.min.y ||
            t.min.z >= self.max.z ||
            t.max.z <= self.min.z
        )
    }

    fn move_out_of(mut self, other: Self, dir: cgmath::Vector3<f64>) -> Self {
        if dir.x != 0.0 {
            if dir.x > 0.0 {
                let ox = self.max.x;
                self.max.x = other.min.x - 0.0001;
                self.min.x += self.max.x - ox;
            } else {
                let ox = self.min.x;
                self.min.x = other.max.x + 0.0001;
                self.max.x += self.min.x - ox;
            }
        }
        if dir.y != 0.0 {
            if dir.y > 0.0 {
                let oy = self.max.y;
                self.max.y = other.min.y - 0.0001;
                self.min.y += self.max.y - oy;
            } else {
                let oy = self.min.y;
                self.min.y = other.max.y + 0.0001;
                self.max.y += self.min.y - oy;
            }
        }
        if dir.z != 0.0 {
            if dir.z > 0.0 {
                let oz = self.max.z;
                self.max.z = other.min.z - 0.0001;
                self.min.z += self.max.z - oz;
            } else {
                let oz = self.min.z;
                self.min.z = other.max.z + 0.0001;
                self.max.z += self.min.z - oz;
            }
        }
        self
    }
}
