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
use std::hash::BuildHasherDefault;
use types::hash::FNVHash;
use resources;
use console;
use render;
use settings::Stevenkey;
use auth;
use ecs;
use entity;
use cgmath;
use collision::Aabb;
use sdl2::keyboard::Keycode;
use types::Gamemode;
use shared::{Axis, Position};
use format;

mod sun;
pub mod plugin_messages;

pub struct Server {
    username: String,
    uuid: protocol::UUID,
    conn: Option<protocol::Conn>,
    read_queue: Option<mpsc::Receiver<Result<packet::Packet, protocol::Error>>>,
    pub disconnect_reason: Option<format::Component>,
    just_disconnected: bool,

    pub world: world::World,
    pub entities: ecs::Manager,
    world_age: i64,
    world_time: f64,
    world_time_target: f64,
    tick_time: bool,

    resources: Arc<RwLock<resources::Manager>>,
    console: Arc<Mutex<console::Console>>,
    version: usize,

    // Entity accessors
    game_info: ecs::Key<entity::GameInfo>,
    player_movement: ecs::Key<entity::player::PlayerMovement>,
    gravity: ecs::Key<entity::Gravity>,
    position: ecs::Key<entity::Position>,
    target_position: ecs::Key<entity::TargetPosition>,
    velocity: ecs::Key<entity::Velocity>,
    gamemode: ecs::Key<Gamemode>,
    pub rotation: ecs::Key<entity::Rotation>,
    target_rotation: ecs::Key<entity::TargetRotation>,
    //

    pub player: Option<ecs::Entity>,
    entity_map: HashMap<i32, ecs::Entity, BuildHasherDefault<FNVHash>>,
    players: HashMap<protocol::UUID, PlayerInfo, BuildHasherDefault<FNVHash>>,

    tick_timer: f64,
    entity_tick_timer: f64,

    sun_model: Option<sun::SunModel>,
}

pub struct PlayerInfo {
    name: String,
    uuid: protocol::UUID,
    skin_url: Option<String>,

    display_name: Option<format::Component>,
    ping: i32,
    gamemode: Gamemode,
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
        use openssl::crypto::pkey;
        use openssl::crypto::rand::rand_bytes;
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

        let packet;
        loop {
            match try!(conn.read_packet()) {
                protocol::packet::Packet::SetInitialCompression(val) => {
                    conn.set_compresssion(val.threshold.0);
                },
                protocol::packet::Packet::EncryptionRequest(val) => {
                    packet = val;
                    break;
                },
                protocol::packet::Packet::LoginSuccess(val) => {
                    warn!("Server is running in offline mode");
                    debug!("Login: {} {}", val.username, val.uuid);
                    let mut read = conn.clone();
                    let mut write = conn.clone();
                    read.state = protocol::State::Play;
                    write.state = protocol::State::Play;
                    let rx = Self::spawn_reader(read);
                    return Ok(Server::new(val.username, protocol::UUID::from_str(&val.uuid), resources, console, Some(write), Some(rx)));
                }
                protocol::packet::Packet::LoginDisconnect(val) => return Err(protocol::Error::Disconnect(val.reason)),
                val => return Err(protocol::Error::Err(format!("Wrong packet: {:?}", val))),
            };
        }

        let mut key = pkey::PKey::new();
        key.load_pub(&packet.public_key.data);
        let shared = rand_bytes(16);

        let shared_e = key.public_encrypt_with_padding(&shared, pkey::EncryptionPadding::PKCS1v15);
        let token_e = key.public_encrypt_with_padding(&packet.verify_token.data, pkey::EncryptionPadding::PKCS1v15);

        try!(profile.join_server(&packet.server_id, &shared, &packet.public_key.data));

        try!(conn.write_packet(protocol::packet::login::serverbound::EncryptionResponse {
            shared_secret: protocol::LenPrefixedBytes::new(shared_e),
            verify_token: protocol::LenPrefixedBytes::new(token_e),
        }));

        let mut read = conn.clone();
        let mut write = conn.clone();

        read.enable_encyption(&shared, true);
        write.enable_encyption(&shared, false);

        let username;
        let uuid;
        loop {
           match try!(read.read_packet()) {
               protocol::packet::Packet::SetInitialCompression(val) => {
                   read.set_compresssion(val.threshold.0);
                   write.set_compresssion(val.threshold.0);
               }
               protocol::packet::Packet::LoginSuccess(val) => {
                   debug!("Login: {} {}", val.username, val.uuid);
                   username = val.username;
                   uuid = val.uuid;
                   read.state = protocol::State::Play;
                   write.state = protocol::State::Play;
                   break;
               }
               protocol::packet::Packet::LoginDisconnect(val) => return Err(protocol::Error::Disconnect(val.reason)),
               val => return Err(protocol::Error::Err(format!("Wrong packet: {:?}", val))),
           }
        }

        let rx = Self::spawn_reader(read);

        Ok(Server::new(username, protocol::UUID::from_str(&uuid), resources, console, Some(write), Some(rx)))
    }

    fn spawn_reader(mut read: protocol::Conn) -> mpsc::Receiver<Result<packet::Packet, protocol::Error>> {
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
        rx
    }

    pub fn dummy_server(resources: Arc<RwLock<resources::Manager>>, console: Arc<Mutex<console::Console>>) -> Server {
        let mut server = Server::new("dummy".to_owned(), protocol::UUID::default(), resources, console, None, None);
        let mut rng = rand::thread_rng();
        for x in -7*16 .. 7*16 {
            for z in -7*16 .. 7*16 {
                let h = 5 + (6.0 * (x as f64 / 16.0).cos() * (z as f64 / 16.0).sin()) as i32;
                for y in 0 .. h {
                    server.world.set_block(Position::new(x, y, z), block::Dirt{ snowy: false, variant: block::DirtVariant::Normal });
                }
                server.world.set_block(Position::new(x, h, z), block::Grass{ snowy: false });

                if x*x + z*z > 16*16 && rng.gen_weighted_bool(80) {
                    for i in 0 .. 5 {
                        server.world.set_block(Position::new(x, h + 1 + i, z), block::Log{ axis: Axis::Y, variant: block::TreeVariant::Oak });
                    }
                    for xx in -2 .. 3 {
                        for zz in -2 .. 3 {
                            if xx == 0 && z == 0 {
                                continue;
                            }
                            server.world.set_block(Position::new(x + xx, h + 3, z + zz), block::Leaves{ variant: block::TreeVariant::Oak, check_decay: false, decayable: false });
                            server.world.set_block(Position::new(x + xx, h + 4, z + zz), block::Leaves{ variant: block::TreeVariant::Oak, check_decay: false, decayable: false });
                            if xx.abs() <= 1 && zz.abs() <= 1 {
                                server.world.set_block(Position::new(x + xx, h + 5, z + zz), block::Leaves{ variant: block::TreeVariant::Oak, check_decay: false, decayable: false });
                            }
                            if xx * xx + zz * zz <= 1 {
                                server.world.set_block(Position::new(x + xx, h + 6, z + zz), block::Leaves{ variant: block::TreeVariant::Oak, check_decay: false, decayable: false });
                            }
                        }
                    }
                }
            }
        }
        server
    }

    fn new(
        username: String, uuid: protocol::UUID,
        resources: Arc<RwLock<resources::Manager>>, console: Arc<Mutex<console::Console>>,
        conn: Option<protocol::Conn>, read_queue: Option<mpsc::Receiver<Result<packet::Packet, protocol::Error>>>
    ) -> Server {
        let mut entities = ecs::Manager::new();
        entity::add_systems(&mut entities);

        let world_entity = entities.get_world();
        let game_info = entities.get_key();
        entities.add_component(world_entity, game_info, entity::GameInfo::new());

        let version = resources.read().unwrap().version();
        Server {
            username: username,
            uuid: uuid,
            conn: conn,
            read_queue: read_queue,
            disconnect_reason: None,
            just_disconnected: false,

            world: world::World::new(),
            world_age: 0,
            world_time: 0.0,
            world_time_target: 0.0,
            tick_time: true,

            version: version,
            resources: resources,
            console: console,

            // Entity accessors
            game_info: game_info,
            player_movement: entities.get_key(),
            gravity: entities.get_key(),
            position: entities.get_key(),
            target_position: entities.get_key(),
            velocity: entities.get_key(),
            gamemode: entities.get_key(),
            rotation: entities.get_key(),
            target_rotation: entities.get_key(),
            //

            entities: entities,
            player: None,
            entity_map: HashMap::with_hasher(BuildHasherDefault::default()),
            players: HashMap::with_hasher(BuildHasherDefault::default()),

            tick_timer: 0.0,
            entity_tick_timer: 0.0,
            sun_model: None,
        }
    }

    pub fn disconnect(&mut self, reason: Option<format::Component>) {
        self.conn = None;
        self.disconnect_reason = reason;
        if let Some(player) = self.player.take() {
            self.entities.remove_entity(player);
        }
        self.just_disconnected = true;
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
        // TODO: Check if the world type actually needs a sun
        if self.sun_model.is_none() {
            self.sun_model = Some(sun::SunModel::new(renderer));
        }

        self.entity_tick(renderer, delta);

        self.tick_timer += delta;
        while self.tick_timer >= 3.0 && self.is_connected() {
            self.minecraft_tick();
            self.tick_timer -= 3.0;
        }

        self.update_time(renderer, delta);

        if let Some(sun_model) = self.sun_model.as_mut() {
            sun_model.tick(renderer, self.world_time, self.world_age);
        }

        self.world.tick(&mut self.entities);

        // Copy to camera
        if let Some(player) = self.player {
            let position = self.entities.get_component(player, self.position).unwrap();
            let rotation = self.entities.get_component(player, self.rotation).unwrap();
            renderer.camera.pos = cgmath::Point::from_vec(position.position + cgmath::Vector3::new(0.0, 1.62, 0.0));
            renderer.camera.yaw = rotation.yaw;
            renderer.camera.pitch = rotation.pitch;
        }
    }

    fn entity_tick(&mut self, renderer: &mut render::Renderer, delta: f64) {
        let world_entity = self.entities.get_world();
        // Update the game's state for entities to read
        self.entities.get_component_mut(world_entity, self.game_info)
            .unwrap().delta = delta;

        // Packets modify entities so need to handled here
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
                            BlockChange => on_block_change,
                            MultiBlockChange => on_multi_block_change,
                            TeleportPlayer => on_teleport,
                            TimeUpdate => on_time_update,
                            ChangeGameState => on_game_state_change,
                            UpdateSign => on_sign_update,
                            PlayerInfo => on_player_info,
                            Disconnect => on_disconnect,
                            // Entities
                            EntityDestroy => on_entity_destroy,
                            SpawnPlayer => on_player_spawn,
                            EntityTeleport => on_entity_teleport,
                            EntityMove => on_entity_move,
                            EntityLook => on_entity_look,
                            EntityLookAndMove => on_entity_look_and_move,
                        }
                    },
                    Err(err) => panic!("Err: {:?}", err),
                }
                // Disconnected
                if self.conn.is_none() {
                    break;
                }
            }

            if self.conn.is_some() {
                self.read_queue = Some(rx);
            }
        }

        if self.is_connected() || self.just_disconnected { // Allow an extra tick when disconnected to clean up
            self.just_disconnected = false;
            self.entity_tick_timer += delta;
            while self.entity_tick_timer >= 3.0 {
                self.entities.tick(&mut self.world, renderer);
                self.entity_tick_timer -= 3.0;
            }

            self.entities.render_tick(&mut self.world, renderer);
        }
    }

    pub fn remove(&mut self, renderer: &mut render::Renderer) {
        self.entities.remove_all_entities(&mut self.world, renderer);
        if let Some(mut sun_model) = self.sun_model.take() {
            sun_model.remove(renderer);
        }
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

    pub fn minecraft_tick(&mut self) {
        use std::f32::consts::PI;
        if let Some(player) = self.player {
            let movement = self.entities.get_component_mut(player, self.player_movement).unwrap();
            let on_ground = self.entities.get_component(player, self.gravity).map_or(false, |v| v.on_ground);
            let position = self.entities.get_component(player, self.target_position).unwrap();
            let rotation = self.entities.get_component(player, self.rotation).unwrap();

            // Force the server to know when touched the ground
            // otherwise if it happens between ticks the server
            // will think we are flying.
            let on_ground = if movement.did_touch_ground {
                movement.did_touch_ground = false;
                true
            } else {
                on_ground
            };

            // Sync our position to the server
            // Use the smaller packets when possible
            let packet = packet::play::serverbound::PlayerPositionLook {
                x: position.position.x,
                y: position.position.y,
                z: position.position.z,
                yaw: -(rotation.yaw as f32) * (180.0 / PI),
                pitch: (-rotation.pitch as f32) * (180.0 / PI) + 180.0,
                on_ground: on_ground,
            };
            self.write_packet(packet);
        }
    }

    pub fn key_press(&mut self, down: bool, key: Stevenkey) {
        if let Some(player) = self.player {
            if let Some(movement) = self.entities.get_component_mut(player, self.player_movement) {
                movement.pressed_keys.insert(key, down);
            }
        }
    }

    pub fn write_packet<T: protocol::PacketType>(&mut self, p: T) {
        let _ = self.conn.as_mut().unwrap().write_packet(p); // TODO handle errors
    }

    fn on_keep_alive(&mut self, keep_alive: packet::play::clientbound::KeepAliveClientbound) {
        self.write_packet(packet::play::serverbound::KeepAliveServerbound {
            id: keep_alive.id,
        });
    }

    fn on_game_join(&mut self, join: packet::play::clientbound::JoinGame) {
        let gamemode = Gamemode::from_int((join.gamemode & 0x7) as i32);
        let player = entity::player::create_local(&mut self.entities);
        if let Some(info) = self.players.get(&self.uuid) {
            let model = self.entities.get_component_mut_direct::<entity::player::PlayerModel>(player).unwrap();
            model.set_skin(info.skin_url.clone());
        }
        *self.entities.get_component_mut(player, self.gamemode).unwrap() = gamemode;
        // TODO: Temp
        self.entities.get_component_mut(player, self.player_movement).unwrap().flying = gamemode.can_fly();

        self.entity_map.insert(join.entity_id, player);
        self.player = Some(player);

        // Let the server know who we are
        self.write_packet(plugin_messages::Brand {
            brand: "Steven".into(),
        }.as_message());
    }

    fn on_respawn(&mut self, respawn: packet::play::clientbound::Respawn) {
        self.world = world::World::new();
        let gamemode = Gamemode::from_int((respawn.gamemode & 0x7) as i32);

        if let Some(player) = self.player {
            *self.entities.get_component_mut(player, self.gamemode).unwrap() = gamemode;
            // TODO: Temp
            self.entities.get_component_mut(player, self.player_movement).unwrap().flying = gamemode.can_fly();
        }
    }

    fn on_disconnect(&mut self, disconnect: packet::play::clientbound::Disconnect) {
        self.disconnect(Some(disconnect.reason));
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
        if game_state.reason == 3 {
            if let Some(player) = self.player {
                let gamemode = Gamemode::from_int(game_state.value as i32);
                *self.entities.get_component_mut(player, self.gamemode).unwrap() = gamemode;
                // TODO: Temp
                self.entities.get_component_mut(player, self.player_movement).unwrap().flying = gamemode.can_fly();
            }
        }
    }

    fn on_entity_destroy(&mut self, entity_destroy: packet::play::clientbound::EntityDestroy) {
        for id in entity_destroy.entity_ids.data {
            if let Some(entity) = self.entity_map.remove(&id.0) {
                self.entities.remove_entity(entity);
            }
        }
    }

    fn on_entity_teleport(&mut self, entity_telport: packet::play::clientbound::EntityTeleport) {
        use std::f64::consts::PI;
        if let Some(entity) = self.entity_map.get(&entity_telport.entity_id.0) {
            let target_position = self.entities.get_component_mut(*entity, self.target_position).unwrap();
            let target_rotation = self.entities.get_component_mut(*entity, self.target_rotation).unwrap();
            target_position.position.x = entity_telport.x;
            target_position.position.y = entity_telport.y;
            target_position.position.z = entity_telport.z;
            target_rotation.yaw = -((entity_telport.yaw as f64) / 256.0) * PI * 2.0;
            target_rotation.pitch = -((entity_telport.pitch as f64) / 256.0) * PI * 2.0;
        }
    }

    fn on_entity_move(&mut self, m: packet::play::clientbound::EntityMove) {
        if let Some(entity) = self.entity_map.get(&m.entity_id.0) {
            let position = self.entities.get_component_mut(*entity, self.target_position).unwrap();
            position.position.x += m.delta_x as f64 / (32.0 * 128.0);
            position.position.y += m.delta_y as f64 / (32.0 * 128.0);
            position.position.z += m.delta_z as f64 / (32.0 * 128.0);
        }
    }

    fn on_entity_look(&mut self, look: packet::play::clientbound::EntityLook) {
        use std::f64::consts::PI;
        if let Some(entity) = self.entity_map.get(&look.entity_id.0) {
            let rotation = self.entities.get_component_mut(*entity, self.target_rotation).unwrap();
            rotation.yaw = -((look.yaw as f64) / 256.0) * PI * 2.0;
            rotation.pitch = -((look.pitch as f64) / 256.0) * PI * 2.0;
        }
    }

    fn on_entity_look_and_move(&mut self, lookmove: packet::play::clientbound::EntityLookAndMove) {
        use std::f64::consts::PI;
        if let Some(entity) = self.entity_map.get(&lookmove.entity_id.0) {
            let position = self.entities.get_component_mut(*entity, self.target_position).unwrap();
            let rotation = self.entities.get_component_mut(*entity, self.target_rotation).unwrap();
            position.position.x += lookmove.delta_x as f64 / (32.0 * 128.0);
            position.position.y += lookmove.delta_y as f64 / (32.0 * 128.0);
            position.position.z += lookmove.delta_z as f64 / (32.0 * 128.0);
            rotation.yaw = -((lookmove.yaw as f64) / 256.0) * PI * 2.0;
            rotation.pitch = -((lookmove.pitch as f64) / 256.0) * PI * 2.0;
        }
    }

    fn on_player_spawn(&mut self, spawn: packet::play::clientbound::SpawnPlayer) {
        use std::f64::consts::PI;
        if let Some(entity) = self.entity_map.remove(&spawn.entity_id.0) {
            self.entities.remove_entity(entity);
        }
        let entity = entity::player::create_remote(&mut self.entities, self.players.get(&spawn.uuid).map_or("MISSING", |v| &v.name));
        let position = self.entities.get_component_mut(entity, self.position).unwrap();
        let target_position = self.entities.get_component_mut(entity, self.target_position).unwrap();
        let rotation = self.entities.get_component_mut(entity, self.rotation).unwrap();
        let target_rotation = self.entities.get_component_mut(entity, self.target_rotation).unwrap();
        position.position.x = spawn.x;
        position.position.y = spawn.y;
        position.position.z = spawn.z;
        target_position.position.x = spawn.x;
        target_position.position.y = spawn.y;
        target_position.position.z = spawn.z;
        rotation.yaw = -((spawn.yaw as f64) / 256.0) * PI * 2.0;
        rotation.pitch = -((spawn.pitch as f64) / 256.0) * PI * 2.0;
        target_rotation.yaw = rotation.yaw;
        target_rotation.pitch = rotation.pitch;
        if let Some(info) = self.players.get(&spawn.uuid) {
            let model = self.entities.get_component_mut_direct::<entity::player::PlayerModel>(entity).unwrap();
            model.set_skin(info.skin_url.clone());
        }
        self.entity_map.insert(spawn.entity_id.0, entity);
    }

    fn on_teleport(&mut self, teleport: packet::play::clientbound::TeleportPlayer) {
        use std::f64::consts::PI;
        if let Some(player) = self.player {
            let position = self.entities.get_component_mut(player, self.target_position).unwrap();
            let rotation = self.entities.get_component_mut(player, self.rotation).unwrap();
            let velocity = self.entities.get_component_mut(player, self.velocity).unwrap();

            position.position.x = calculate_relative_teleport(TeleportFlag::RelX, teleport.flags, position.position.x, teleport.x);
            position.position.y = calculate_relative_teleport(TeleportFlag::RelY, teleport.flags, position.position.y, teleport.y);
            position.position.z = calculate_relative_teleport(TeleportFlag::RelZ, teleport.flags, position.position.z, teleport.z);
            rotation.yaw = calculate_relative_teleport(TeleportFlag::RelYaw, teleport.flags, rotation.yaw, -teleport.yaw as f64 * (PI / 180.0));

            rotation.pitch = -((calculate_relative_teleport(
                TeleportFlag::RelPitch,
                teleport.flags,
                (-rotation.pitch) * (180.0 / PI) + 180.0,
                teleport.pitch as f64
            ) - 180.0) * (PI / 180.0));

            if (teleport.flags & (TeleportFlag::RelX as u8)) == 0 {
                velocity.velocity.x = 0.0;
            }
            if (teleport.flags & (TeleportFlag::RelY as u8)) == 0 {
                velocity.velocity.y = 0.0;
            }
            if (teleport.flags & (TeleportFlag::RelZ as u8)) == 0 {
                velocity.velocity.z = 0.0;
            }

            self.write_packet(packet::play::serverbound::TeleportConfirm {
                teleport_id: teleport.teleport_id,
            });
        }
    }

    fn on_sign_update(&mut self, mut update_sign: packet::play::clientbound::UpdateSign) {
        use format;
        format::convert_legacy(&mut update_sign.line1);
        format::convert_legacy(&mut update_sign.line2);
        format::convert_legacy(&mut update_sign.line3);
        format::convert_legacy(&mut update_sign.line4);
        self.world.add_block_entity_action(world::BlockEntityAction::UpdateSignText(
            update_sign.location,
            update_sign.line1,
            update_sign.line2,
            update_sign.line3,
            update_sign.line4,
        ));
    }

    fn on_player_info(&mut self, player_info: packet::play::clientbound::PlayerInfo) {
        use protocol::packet::PlayerDetail::*;
        use rustc_serialize::base64::FromBase64;
        use serde_json;
        for detail in player_info.inner.players {
            match detail {
                Add { name, uuid, properties, display, gamemode, ping} => {
                    let info = self.players.entry(uuid.clone()).or_insert(PlayerInfo {
                        name: name.clone(),
                        uuid: uuid,
                        skin_url: None,

                        display_name: display.clone(),
                        ping: ping.0,
                        gamemode: Gamemode::from_int(gamemode.0),
                    });
                    // Re-set the props of the player in case of dodgy server implementations
                    info.name = name;
                    info.display_name = display;
                    info.ping = ping.0;
                    info.gamemode = Gamemode::from_int(gamemode.0);
                    for prop in properties {
                        if prop.name != "textures" {
                            continue;
                        }
                        // Ideally we would check the signature of the blob to
                        // verify it was from Mojang and not faked by the server
                        // but this requires the public key which is distributed
                        // authlib. We could download authlib on startup and extract
                        // the key but this seems like overkill compared to just
                        // whitelisting Mojang's texture servers instead.
                        let skin_blob = match prop.value.from_base64() {
                            Ok(val) => val,
                            Err(err) => {
                                error!("Failed to decode skin blob, {:?}", err);
                                continue;
                            },
                        };
                        let skin_blob: serde_json::Value = match serde_json::from_slice(&skin_blob) {
                            Ok(val) => val,
                            Err(err) => {
                                error!("Failed to parse skin blob, {:?}", err);
                                continue;
                            },
                        };
                        if let Some(skin_url) = skin_blob.lookup("textures.SKIN.url").and_then(|v| v.as_string()) {
                            info.skin_url = Some(skin_url.to_owned());
                        }
                    }

                    // Refresh our own skin when the server sends it to us.
                    // The join game packet can come before this packet meaning
                    // we may not have the skin in time for spawning ourselves.
                    // This isn't an issue for other players because this packet
                    // must come before the spawn player packet.
                    if info.uuid == self.uuid {
                        let model = self.entities.get_component_mut_direct::<entity::player::PlayerModel>(self.player.unwrap()).unwrap();
                        model.set_skin(info.skin_url.clone());
                    }
                },
                UpdateGamemode { uuid, gamemode } => {
                    if let Some(info) = self.players.get_mut(&uuid) {
                        info.gamemode = Gamemode::from_int(gamemode.0);
                    }
                },
                UpdateLatency { uuid, ping } => {
                    if let Some(info) = self.players.get_mut(&uuid) {
                        info.ping = ping.0;
                    }
                },
                UpdateDisplayName { uuid, display } => {
                    if let Some(info) = self.players.get_mut(&uuid) {
                        info.display_name = display;
                    }
                },
                Remove { uuid } => {
                    self.players.remove(&uuid);
                },
            }
        }
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
        self.world.unload_chunk(chunk_unload.x, chunk_unload.z, &mut self.entities);
    }

    fn on_block_change(&mut self, block_change: packet::play::clientbound::BlockChange) {
        self.world.set_block(
            block_change.location,
            block::Block::by_vanilla_id(block_change.block_id.0 as usize)
        );
    }

    fn on_multi_block_change(&mut self, block_change: packet::play::clientbound::MultiBlockChange) {
        let ox = block_change.chunk_x << 4;
        let oz = block_change.chunk_z << 4;
        for record in block_change.records.data {
            self.world.set_block(
                Position::new(
                    ox + (record.xz >> 4) as i32,
                    record.y as i32,
                    oz + (record.xz & 0xF) as i32
                ),
                block::Block::by_vanilla_id(record.block_id.0 as usize)
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TeleportFlag {
    RelX = 0b00001,
    RelY = 0b00010,
    RelZ = 0b00100,
    RelYaw = 0b01000,
    RelPitch = 0b10000,
}

fn calculate_relative_teleport(flag: TeleportFlag, flags: u8, base: f64, val: f64) -> f64 {
    if (flags & (flag as u8)) == 0 {
        val
    } else {
        base + val
    }
}
