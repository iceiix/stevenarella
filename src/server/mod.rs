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

use crate::ecs;
use crate::entity;
use crate::format;
use crate::protocol::{self, forge, mojang, packet};
use crate::render;
use crate::resources;
use crate::settings::Stevenkey;
use crate::shared::{Axis, Position};
use crate::types::hash::FNVHash;
use crate::types::Gamemode;
use crate::world;
use crate::world::block;
use cgmath::prelude::*;
use log::{debug, error, warn};
use rand::{self, Rng};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;

pub mod plugin_messages;
mod sun;
pub mod target;

pub struct Server {
    uuid: protocol::UUID,
    conn: Option<protocol::Conn>,
    protocol_version: i32,
    forge_mods: Vec<forge::ForgeMod>,
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
    target_info: target::Info,
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
    pub fn connect(
        resources: Arc<RwLock<resources::Manager>>,
        profile: mojang::Profile,
        address: &str,
        protocol_version: i32,
        forge_mods: Vec<forge::ForgeMod>,
    ) -> Result<Server, protocol::Error> {
        let mut conn = protocol::Conn::new(address, protocol_version)?;

        let tag = if !forge_mods.is_empty() {
            "\0FML\0"
        } else {
            ""
        };
        let host = conn.host.clone() + tag;
        let port = conn.port;
        conn.write_packet(protocol::packet::handshake::serverbound::Handshake {
            protocol_version: protocol::VarInt(protocol_version),
            host,
            port,
            next: protocol::VarInt(2),
        })?;
        conn.state = protocol::State::Login;
        conn.write_packet(protocol::packet::login::serverbound::LoginStart {
            username: profile.username.clone(),
        })?;

        use std::rc::Rc;
        let (server_id, public_key, verify_token);
        loop {
            match conn.read_packet()? {
                protocol::packet::Packet::SetInitialCompression(val) => {
                    conn.set_compresssion(val.threshold.0);
                }
                protocol::packet::Packet::EncryptionRequest(val) => {
                    server_id = Rc::new(val.server_id);
                    public_key = Rc::new(val.public_key.data);
                    verify_token = Rc::new(val.verify_token.data);
                    break;
                }
                protocol::packet::Packet::EncryptionRequest_i16(val) => {
                    server_id = Rc::new(val.server_id);
                    public_key = Rc::new(val.public_key.data);
                    verify_token = Rc::new(val.verify_token.data);
                    break;
                }
                protocol::packet::Packet::LoginSuccess_String(val) => {
                    warn!("Server is running in offline mode");
                    debug!("Login: {} {}", val.username, val.uuid);
                    let mut read = conn.clone();
                    let mut write = conn;
                    read.state = protocol::State::Play;
                    write.state = protocol::State::Play;
                    let rx = Self::spawn_reader(read);
                    return Ok(Server::new(
                        protocol_version,
                        forge_mods,
                        protocol::UUID::from_str(&val.uuid).unwrap(),
                        resources,
                        Some(write),
                        Some(rx),
                    ));
                }
                // TODO: avoid duplication
                protocol::packet::Packet::LoginSuccess_UUID(val) => {
                    warn!("Server is running in offline mode");
                    debug!("Login: {} {:?}", val.username, val.uuid);
                    let mut read = conn.clone();
                    let mut write = conn;
                    read.state = protocol::State::Play;
                    write.state = protocol::State::Play;
                    let rx = Self::spawn_reader(read);
                    return Ok(Server::new(
                        protocol_version,
                        forge_mods,
                        val.uuid,
                        resources,
                        Some(write),
                        Some(rx),
                    ));
                }
                protocol::packet::Packet::LoginDisconnect(val) => {
                    return Err(protocol::Error::Disconnect(val.reason))
                }
                val => return Err(protocol::Error::Err(format!("Wrong packet: {:?}", val))),
            };
        }

        let mut shared = [0; 16];
        // TODO: is this cryptographically secure enough?
        rand::thread_rng().fill(&mut shared);

        let shared_e = rsa_public_encrypt_pkcs1::encrypt(&public_key, &shared).unwrap();
        let token_e = rsa_public_encrypt_pkcs1::encrypt(&public_key, &verify_token).unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        {
            profile.join_server(&server_id, &shared, &public_key)?;
        }

        if protocol_version >= 47 {
            conn.write_packet(protocol::packet::login::serverbound::EncryptionResponse {
                shared_secret: protocol::LenPrefixedBytes::new(shared_e),
                verify_token: protocol::LenPrefixedBytes::new(token_e),
            })?;
        } else {
            conn.write_packet(
                protocol::packet::login::serverbound::EncryptionResponse_i16 {
                    shared_secret: protocol::LenPrefixedBytes::new(shared_e),
                    verify_token: protocol::LenPrefixedBytes::new(token_e),
                },
            )?;
        }

        let mut read = conn.clone();
        let mut write = conn;

        read.enable_encyption(&shared, true);
        write.enable_encyption(&shared, false);

        let uuid;
        loop {
            match read.read_packet()? {
                protocol::packet::Packet::SetInitialCompression(val) => {
                    read.set_compresssion(val.threshold.0);
                    write.set_compresssion(val.threshold.0);
                }
                protocol::packet::Packet::LoginSuccess_String(val) => {
                    debug!("Login: {} {}", val.username, val.uuid);
                    uuid = protocol::UUID::from_str(&val.uuid).unwrap();
                    read.state = protocol::State::Play;
                    write.state = protocol::State::Play;
                    break;
                }
                protocol::packet::Packet::LoginSuccess_UUID(val) => {
                    debug!("Login: {} {:?}", val.username, val.uuid);
                    uuid = val.uuid;
                    read.state = protocol::State::Play;
                    write.state = protocol::State::Play;
                    break;
                }
                protocol::packet::Packet::LoginDisconnect(val) => {
                    return Err(protocol::Error::Disconnect(val.reason))
                }
                val => return Err(protocol::Error::Err(format!("Wrong packet: {:?}", val))),
            }
        }

        let rx = Self::spawn_reader(read);

        Ok(Server::new(
            protocol_version,
            forge_mods,
            uuid,
            resources,
            Some(write),
            Some(rx),
        ))
    }

    fn spawn_reader(
        mut read: protocol::Conn,
    ) -> mpsc::Receiver<Result<packet::Packet, protocol::Error>> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            let pck = read.read_packet();
            let was_error = pck.is_err();
            if tx.send(pck).is_err() {
                return;
            }
            if was_error {
                return;
            }
        });
        rx
    }

    pub fn dummy_server(resources: Arc<RwLock<resources::Manager>>) -> Server {
        let mut server = Server::new(
            protocol::SUPPORTED_PROTOCOLS[0],
            vec![],
            protocol::UUID::default(),
            resources,
            None,
            None,
        );
        let mut rng = rand::thread_rng();
        for x in -7 * 16..7 * 16 {
            for z in -7 * 16..7 * 16 {
                let h = 5 + (6.0 * (x as f64 / 16.0).cos() * (z as f64 / 16.0).sin()) as i32;
                for y in 0..h {
                    server.world.set_block(
                        Position::new(x, y, z),
                        block::Dirt {
                            snowy: false,
                            variant: block::DirtVariant::Normal,
                        },
                    );
                }
                server
                    .world
                    .set_block(Position::new(x, h, z), block::Grass { snowy: false });

                if x * x + z * z > 16 * 16 && rng.gen_bool(1.0 / 80.0) {
                    for i in 0..5 {
                        server.world.set_block(
                            Position::new(x, h + 1 + i, z),
                            block::Log {
                                axis: Axis::Y,
                                variant: block::TreeVariant::Oak,
                            },
                        );
                    }
                    for xx in -2..3 {
                        for zz in -2..3 {
                            if xx == 0 && z == 0 {
                                continue;
                            }
                            server.world.set_block(
                                Position::new(x + xx, h + 3, z + zz),
                                block::Leaves {
                                    variant: block::TreeVariant::Oak,
                                    check_decay: false,
                                    decayable: false,
                                    distance: 1,
                                },
                            );
                            server.world.set_block(
                                Position::new(x + xx, h + 4, z + zz),
                                block::Leaves {
                                    variant: block::TreeVariant::Oak,
                                    check_decay: false,
                                    decayable: false,
                                    distance: 1,
                                },
                            );
                            if xx.abs() <= 1 && zz.abs() <= 1 {
                                server.world.set_block(
                                    Position::new(x + xx, h + 5, z + zz),
                                    block::Leaves {
                                        variant: block::TreeVariant::Oak,
                                        check_decay: false,
                                        decayable: false,
                                        distance: 1,
                                    },
                                );
                            }
                            if xx * xx + zz * zz <= 1 {
                                server.world.set_block(
                                    Position::new(x + xx, h + 6, z + zz),
                                    block::Leaves {
                                        variant: block::TreeVariant::Oak,
                                        check_decay: false,
                                        decayable: false,
                                        distance: 1,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
        server
    }

    fn new(
        protocol_version: i32,
        forge_mods: Vec<forge::ForgeMod>,
        uuid: protocol::UUID,
        resources: Arc<RwLock<resources::Manager>>,
        conn: Option<protocol::Conn>,
        read_queue: Option<mpsc::Receiver<Result<packet::Packet, protocol::Error>>>,
    ) -> Server {
        let mut entities = ecs::Manager::new();
        entity::add_systems(&mut entities);

        let world_entity = entities.get_world();
        let game_info = entities.get_key();
        entities.add_component(world_entity, game_info, entity::GameInfo::new());

        let version = resources.read().unwrap().version();
        Server {
            uuid,
            conn,
            protocol_version,
            forge_mods,
            read_queue,
            disconnect_reason: None,
            just_disconnected: false,

            world: world::World::new(protocol_version),
            world_age: 0,
            world_time: 0.0,
            world_time_target: 0.0,
            tick_time: true,

            version,
            resources,

            // Entity accessors
            game_info,
            player_movement: entities.get_key(),
            gravity: entities.get_key(),
            position: entities.get_key(),
            target_position: entities.get_key(),
            velocity: entities.get_key(),
            gamemode: entities.get_key(),
            rotation: entities.get_key(),
            target_rotation: entities.get_key(),
            //
            entities,
            player: None,
            entity_map: HashMap::with_hasher(BuildHasherDefault::default()),
            players: HashMap::with_hasher(BuildHasherDefault::default()),

            tick_timer: 0.0,
            entity_tick_timer: 0.0,
            sun_model: None,

            target_info: target::Info::new(),
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

        // Copy to camera
        if let Some(player) = self.player {
            let position = self.entities.get_component(player, self.position).unwrap();
            let rotation = self.entities.get_component(player, self.rotation).unwrap();
            renderer.camera.pos =
                cgmath::Point3::from_vec(position.position + cgmath::Vector3::new(0.0, 1.62, 0.0));
            renderer.camera.yaw = rotation.yaw;
            renderer.camera.pitch = rotation.pitch;
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

        if self.player.is_some() {
            if let Some((pos, bl, _, _)) = target::trace_ray(
                &self.world,
                4.0,
                renderer.camera.pos.to_vec(),
                renderer.view_vector.cast().unwrap(),
                target::test_block,
            ) {
                self.target_info.update(renderer, pos, bl);
            } else {
                self.target_info.clear(renderer);
            }
        } else {
            self.target_info.clear(renderer);
        }
    }

    fn entity_tick(&mut self, renderer: &mut render::Renderer, delta: f64) {
        let world_entity = self.entities.get_world();
        // Update the game's state for entities to read
        self.entities
            .get_component_mut(world_entity, self.game_info)
            .unwrap()
            .delta = delta;

        // Packets modify entities so need to handled here
        if let Some(rx) = self.read_queue.take() {
            while let Ok(pck) = rx.try_recv() {
                match pck {
                    Ok(pck) => handle_packet! {
                        self pck {
                            PluginMessageClientbound_i16 => on_plugin_message_clientbound_i16,
                            PluginMessageClientbound => on_plugin_message_clientbound_1,
                            JoinGame_WorldNames => on_game_join_worldnames,
                            JoinGame_HashedSeed_Respawn => on_game_join_hashedseed_respawn,
                            JoinGame_i32_ViewDistance => on_game_join_i32_viewdistance,
                            JoinGame_i32 => on_game_join_i32,
                            JoinGame_i8 => on_game_join_i8,
                            JoinGame_i8_NoDebug => on_game_join_i8_nodebug,
                            Respawn_Gamemode => on_respawn_gamemode,
                            Respawn_HashedSeed => on_respawn_hashedseed,
                            Respawn_WorldName => on_respawn_worldname,
                            KeepAliveClientbound_i64 => on_keep_alive_i64,
                            KeepAliveClientbound_VarInt => on_keep_alive_varint,
                            KeepAliveClientbound_i32 => on_keep_alive_i32,
                            ChunkData_Biomes3D_bool => on_chunk_data_biomes3d_bool,
                            ChunkData => on_chunk_data,
                            ChunkData_Biomes3D => on_chunk_data_biomes3d,
                            ChunkData_HeightMap => on_chunk_data_heightmap,
                            ChunkData_NoEntities => on_chunk_data_no_entities,
                            ChunkData_NoEntities_u16 => on_chunk_data_no_entities_u16,
                            ChunkData_17 => on_chunk_data_17,
                            ChunkDataBulk => on_chunk_data_bulk,
                            ChunkDataBulk_17 => on_chunk_data_bulk_17,
                            ChunkUnload => on_chunk_unload,
                            BlockChange_VarInt => on_block_change_varint,
                            BlockChange_u8 => on_block_change_u8,
                            MultiBlockChange_VarInt => on_multi_block_change_varint,
                            MultiBlockChange_u16 => on_multi_block_change_u16,
                            TeleportPlayer_WithConfirm => on_teleport_player_withconfirm,
                            TeleportPlayer_NoConfirm => on_teleport_player_noconfirm,
                            TeleportPlayer_OnGround => on_teleport_player_onground,
                            TimeUpdate => on_time_update,
                            ChangeGameState => on_game_state_change,
                            UpdateBlockEntity => on_block_entity_update,
                            UpdateBlockEntity_Data => on_block_entity_update_data,
                            UpdateSign => on_sign_update,
                            UpdateSign_u16 => on_sign_update_u16,
                            PlayerInfo => on_player_info,
                            PlayerInfo_String => on_player_info_string,
                            Disconnect => on_disconnect,
                            // Entities
                            EntityDestroy => on_entity_destroy,
                            EntityDestroy_u8 => on_entity_destroy_u8,
                            SpawnPlayer_f64_NoMeta => on_player_spawn_f64_nometa,
                            SpawnPlayer_f64 => on_player_spawn_f64,
                            SpawnPlayer_i32 => on_player_spawn_i32,
                            SpawnPlayer_i32_HeldItem => on_player_spawn_i32_helditem,
                            SpawnPlayer_i32_HeldItem_String => on_player_spawn_i32_helditem_string,
                            EntityTeleport_f64 => on_entity_teleport_f64,
                            EntityTeleport_i32 => on_entity_teleport_i32,
                            EntityTeleport_i32_i32_NoGround => on_entity_teleport_i32_i32_noground,
                            EntityMove_i16 => on_entity_move_i16,
                            EntityMove_i8 => on_entity_move_i8,
                            EntityMove_i8_i32_NoGround => on_entity_move_i8_i32_noground,
                            EntityLook_VarInt => on_entity_look_varint,
                            EntityLook_i32_NoGround => on_entity_look_i32_noground,
                            EntityLookAndMove_i16 => on_entity_look_and_move_i16,
                            EntityLookAndMove_i8 => on_entity_look_and_move_i8,
                            EntityLookAndMove_i8_i32_NoGround => on_entity_look_and_move_i8_i32_noground,
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

        if self.is_connected() || self.just_disconnected {
            // Allow an extra tick when disconnected to clean up
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
        self.target_info.clear(renderer);
    }

    fn update_time(&mut self, renderer: &mut render::Renderer, delta: f64) {
        if self.tick_time {
            self.world_time_target += delta / 3.0;
            self.world_time_target = (24000.0 + self.world_time_target) % 24000.0;
            let mut diff = self.world_time_target - self.world_time;
            if diff < -12000.0 {
                diff += 24000.0
            } else if diff > 12000.0 {
                diff -= 24000.0
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
            let movement = self
                .entities
                .get_component_mut(player, self.player_movement)
                .unwrap();
            let on_ground = self
                .entities
                .get_component(player, self.gravity)
                .map_or(false, |v| v.on_ground);
            let position = self
                .entities
                .get_component(player, self.target_position)
                .unwrap();
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
            if self.protocol_version >= 47 {
                let packet = packet::play::serverbound::PlayerPositionLook {
                    x: position.position.x,
                    y: position.position.y,
                    z: position.position.z,
                    yaw: -(rotation.yaw as f32) * (180.0 / PI),
                    pitch: (-rotation.pitch as f32) * (180.0 / PI) + 180.0,
                    on_ground,
                };
                self.write_packet(packet);
            } else {
                let packet = packet::play::serverbound::PlayerPositionLook_HeadY {
                    x: position.position.x,
                    feet_y: position.position.y,
                    head_y: position.position.y + 1.62,
                    z: position.position.z,
                    yaw: -(rotation.yaw as f32) * (180.0 / PI),
                    pitch: (-rotation.pitch as f32) * (180.0 / PI) + 180.0,
                    on_ground,
                };
                self.write_packet(packet);
            }
        }
    }

    pub fn key_press(&mut self, down: bool, key: Stevenkey) {
        if let Some(player) = self.player {
            if let Some(movement) = self
                .entities
                .get_component_mut(player, self.player_movement)
            {
                movement.pressed_keys.insert(key, down);
            }
        }
    }

    pub fn on_right_click(&mut self, renderer: &mut render::Renderer) {
        use crate::shared::Direction;
        if self.player.is_some() {
            if let Some((pos, _, face, at)) = target::trace_ray(
                &self.world,
                4.0,
                renderer.camera.pos.to_vec(),
                renderer.view_vector.cast().unwrap(),
                target::test_block,
            ) {
                if self.protocol_version >= 315 {
                    self.write_packet(packet::play::serverbound::PlayerBlockPlacement_f32 {
                        location: pos,
                        face: protocol::VarInt(match face {
                            Direction::Down => 0,
                            Direction::Up => 1,
                            Direction::North => 2,
                            Direction::South => 3,
                            Direction::West => 4,
                            Direction::East => 5,
                            _ => unreachable!(),
                        }),
                        hand: protocol::VarInt(0),
                        cursor_x: at.x as f32,
                        cursor_y: at.y as f32,
                        cursor_z: at.z as f32,
                    });
                } else if self.protocol_version >= 49 {
                    self.write_packet(packet::play::serverbound::PlayerBlockPlacement_u8 {
                        location: pos,
                        face: protocol::VarInt(match face {
                            Direction::Down => 0,
                            Direction::Up => 1,
                            Direction::North => 2,
                            Direction::South => 3,
                            Direction::West => 4,
                            Direction::East => 5,
                            _ => unreachable!(),
                        }),
                        hand: protocol::VarInt(0),
                        cursor_x: (at.x * 16.0) as u8,
                        cursor_y: (at.y * 16.0) as u8,
                        cursor_z: (at.z * 16.0) as u8,
                    });
                } else if self.protocol_version >= 47 {
                    self.write_packet(packet::play::serverbound::PlayerBlockPlacement_u8_Item {
                        location: pos,
                        face: match face {
                            Direction::Down => 0,
                            Direction::Up => 1,
                            Direction::North => 2,
                            Direction::South => 3,
                            Direction::West => 4,
                            Direction::East => 5,
                            _ => unreachable!(),
                        },
                        hand: None,
                        cursor_x: (at.x * 16.0) as u8,
                        cursor_y: (at.y * 16.0) as u8,
                        cursor_z: (at.z * 16.0) as u8,
                    });
                } else {
                    self.write_packet(
                        packet::play::serverbound::PlayerBlockPlacement_u8_Item_u8y {
                            x: pos.x,
                            y: pos.y as u8,
                            z: pos.x,
                            face: match face {
                                Direction::Down => 0,
                                Direction::Up => 1,
                                Direction::North => 2,
                                Direction::South => 3,
                                Direction::West => 4,
                                Direction::East => 5,
                                _ => unreachable!(),
                            },
                            hand: None,
                            cursor_x: (at.x * 16.0) as u8,
                            cursor_y: (at.y * 16.0) as u8,
                            cursor_z: (at.z * 16.0) as u8,
                        },
                    );
                }
            }
        }
    }

    pub fn write_packet<T: protocol::PacketType>(&mut self, p: T) {
        let _ = self.conn.as_mut().unwrap().write_packet(p); // TODO handle errors
    }

    fn on_keep_alive_i64(
        &mut self,
        keep_alive: packet::play::clientbound::KeepAliveClientbound_i64,
    ) {
        self.write_packet(packet::play::serverbound::KeepAliveServerbound_i64 {
            id: keep_alive.id,
        });
    }

    fn on_keep_alive_varint(
        &mut self,
        keep_alive: packet::play::clientbound::KeepAliveClientbound_VarInt,
    ) {
        self.write_packet(packet::play::serverbound::KeepAliveServerbound_VarInt {
            id: keep_alive.id,
        });
    }

    fn on_keep_alive_i32(
        &mut self,
        keep_alive: packet::play::clientbound::KeepAliveClientbound_i32,
    ) {
        self.write_packet(packet::play::serverbound::KeepAliveServerbound_i32 {
            id: keep_alive.id,
        });
    }

    fn on_plugin_message_clientbound_i16(
        &mut self,
        msg: packet::play::clientbound::PluginMessageClientbound_i16,
    ) {
        self.on_plugin_message_clientbound(&msg.channel, msg.data.data.as_slice())
    }

    fn on_plugin_message_clientbound_1(
        &mut self,
        msg: packet::play::clientbound::PluginMessageClientbound,
    ) {
        self.on_plugin_message_clientbound(&msg.channel, &msg.data)
    }

    fn on_plugin_message_clientbound(&mut self, channel: &str, data: &[u8]) {
        if protocol::is_network_debug() {
            debug!(
                "Received plugin message: channel={}, data={:?}",
                channel, data
            );
        }

        match channel {
            "REGISTER" => {}   // TODO
            "UNREGISTER" => {} // TODO
            "FML|HS" => {
                let msg = crate::protocol::Serializable::read_from(&mut std::io::Cursor::new(data))
                    .unwrap();
                //debug!("FML|HS msg={:?}", msg);

                use forge::FmlHs::*;
                use forge::Phase::*;
                match msg {
                    ServerHello {
                        fml_protocol_version,
                        override_dimension,
                    } => {
                        debug!(
                            "Received FML|HS ServerHello {} {:?}",
                            fml_protocol_version, override_dimension
                        );

                        self.write_plugin_message("REGISTER", b"FML|HS\0FML\0FML|MP\0FML\0FORGE");
                        self.write_fmlhs_plugin_message(&ClientHello {
                            fml_protocol_version,
                        });
                        // Send stashed mods list received from ping packet, client matching server
                        let mods = crate::protocol::LenPrefixed::<
                            crate::protocol::VarInt,
                            forge::ForgeMod,
                        >::new(self.forge_mods.clone());
                        self.write_fmlhs_plugin_message(&ModList { mods });
                    }
                    ModList { mods } => {
                        debug!("Received FML|HS ModList: {:?}", mods);

                        self.write_fmlhs_plugin_message(&HandshakeAck {
                            phase: WaitingServerData,
                        });
                    }
                    ModIdData {
                        mappings,
                        block_substitutions: _,
                        item_substitutions: _,
                    } => {
                        debug!("Received FML|HS ModIdData");
                        for m in mappings.data {
                            let (namespace, name) = m.name.split_at(1);
                            if namespace == protocol::forge::BLOCK_NAMESPACE {
                                self.world
                                    .modded_block_ids
                                    .insert(m.id.0 as usize, name.to_string());
                            }
                        }
                        self.write_fmlhs_plugin_message(&HandshakeAck {
                            phase: WaitingServerComplete,
                        });
                    }
                    RegistryData {
                        has_more,
                        name,
                        ids,
                        substitutions: _,
                        dummies: _,
                    } => {
                        debug!("Received FML|HS RegistryData for {}", name);
                        if name == "minecraft:blocks" {
                            for m in ids.data {
                                self.world.modded_block_ids.insert(m.id.0 as usize, m.name);
                            }
                        }
                        if !has_more {
                            self.write_fmlhs_plugin_message(&HandshakeAck {
                                phase: WaitingServerComplete,
                            });
                        }
                    }
                    HandshakeAck { phase } => match phase {
                        WaitingCAck => {
                            self.write_fmlhs_plugin_message(&HandshakeAck {
                                phase: PendingComplete,
                            });
                        }
                        Complete => {
                            debug!("FML|HS handshake complete!");
                        }
                        _ => unimplemented!(),
                    },
                    _ => (),
                }
            }
            _ => (),
        }
    }

    fn write_fmlhs_plugin_message(&mut self, msg: &forge::FmlHs) {
        use crate::protocol::Serializable;

        let mut buf: Vec<u8> = vec![];
        msg.write_to(&mut buf).unwrap();

        self.write_plugin_message("FML|HS", &buf);
    }

    fn write_plugin_message(&mut self, channel: &str, data: &[u8]) {
        if protocol::is_network_debug() {
            debug!(
                "Sending plugin message: channel={}, data={:?}",
                channel, data
            );
        }
        if self.protocol_version >= 47 {
            self.write_packet(packet::play::serverbound::PluginMessageServerbound {
                channel: channel.to_string(),
                data: data.to_vec(),
            });
        } else {
            self.write_packet(packet::play::serverbound::PluginMessageServerbound_i16 {
                channel: channel.to_string(),
                data: crate::protocol::LenPrefixedBytes::<protocol::VarShort>::new(data.to_vec()),
            });
        }
    }

    fn on_game_join_worldnames(&mut self, join: packet::play::clientbound::JoinGame_WorldNames) {
        self.on_game_join(join.gamemode, join.entity_id)
    }

    fn on_game_join_hashedseed_respawn(
        &mut self,
        join: packet::play::clientbound::JoinGame_HashedSeed_Respawn,
    ) {
        self.on_game_join(join.gamemode, join.entity_id)
    }

    fn on_game_join_i32_viewdistance(
        &mut self,
        join: packet::play::clientbound::JoinGame_i32_ViewDistance,
    ) {
        self.on_game_join(join.gamemode, join.entity_id)
    }

    fn on_game_join_i32(&mut self, join: packet::play::clientbound::JoinGame_i32) {
        self.on_game_join(join.gamemode, join.entity_id)
    }

    fn on_game_join_i8(&mut self, join: packet::play::clientbound::JoinGame_i8) {
        self.on_game_join(join.gamemode, join.entity_id)
    }

    fn on_game_join_i8_nodebug(&mut self, join: packet::play::clientbound::JoinGame_i8_NoDebug) {
        self.on_game_join(join.gamemode, join.entity_id)
    }

    fn on_game_join(&mut self, gamemode: u8, entity_id: i32) {
        let gamemode = Gamemode::from_int((gamemode & 0x7) as i32);
        let player = entity::player::create_local(&mut self.entities);
        if let Some(info) = self.players.get(&self.uuid) {
            let model = self
                .entities
                .get_component_mut_direct::<entity::player::PlayerModel>(player)
                .unwrap();
            model.set_skin(info.skin_url.clone());
        }
        *self
            .entities
            .get_component_mut(player, self.gamemode)
            .unwrap() = gamemode;
        // TODO: Temp
        self.entities
            .get_component_mut(player, self.player_movement)
            .unwrap()
            .flying = gamemode.can_fly();

        self.entity_map.insert(entity_id, player);
        self.player = Some(player);

        // Let the server know who we are
        let brand = plugin_messages::Brand {
            brand: "Steven".into(),
        };
        // TODO: refactor with write_plugin_message
        if self.protocol_version >= 47 {
            self.write_packet(brand.into_message());
        } else {
            self.write_packet(brand.into_message17());
        }
    }

    fn on_respawn_hashedseed(&mut self, respawn: packet::play::clientbound::Respawn_HashedSeed) {
        self.respawn(respawn.gamemode)
    }

    fn on_respawn_gamemode(&mut self, respawn: packet::play::clientbound::Respawn_Gamemode) {
        self.respawn(respawn.gamemode)
    }

    fn on_respawn_worldname(&mut self, respawn: packet::play::clientbound::Respawn_WorldName) {
        self.respawn(respawn.gamemode)
    }

    fn respawn(&mut self, gamemode_u8: u8) {
        self.world = world::World::new(self.protocol_version);
        let gamemode = Gamemode::from_int((gamemode_u8 & 0x7) as i32);

        if let Some(player) = self.player {
            *self
                .entities
                .get_component_mut(player, self.gamemode)
                .unwrap() = gamemode;
            // TODO: Temp
            self.entities
                .get_component_mut(player, self.player_movement)
                .unwrap()
                .flying = gamemode.can_fly();
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
                *self
                    .entities
                    .get_component_mut(player, self.gamemode)
                    .unwrap() = gamemode;
                // TODO: Temp
                self.entities
                    .get_component_mut(player, self.player_movement)
                    .unwrap()
                    .flying = gamemode.can_fly();
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

    fn on_entity_destroy_u8(
        &mut self,
        entity_destroy: packet::play::clientbound::EntityDestroy_u8,
    ) {
        for id in entity_destroy.entity_ids.data {
            if let Some(entity) = self.entity_map.remove(&id) {
                self.entities.remove_entity(entity);
            }
        }
    }

    fn on_entity_teleport_f64(
        &mut self,
        entity_telport: packet::play::clientbound::EntityTeleport_f64,
    ) {
        self.on_entity_teleport(
            entity_telport.entity_id.0,
            entity_telport.x,
            entity_telport.y,
            entity_telport.z,
            entity_telport.yaw as f64,
            entity_telport.pitch as f64,
            entity_telport.on_ground,
        )
    }

    fn on_entity_teleport_i32(
        &mut self,
        entity_telport: packet::play::clientbound::EntityTeleport_i32,
    ) {
        self.on_entity_teleport(
            entity_telport.entity_id.0,
            f64::from(entity_telport.x),
            f64::from(entity_telport.y),
            f64::from(entity_telport.z),
            entity_telport.yaw as f64,
            entity_telport.pitch as f64,
            entity_telport.on_ground,
        )
    }

    fn on_entity_teleport_i32_i32_noground(
        &mut self,
        entity_telport: packet::play::clientbound::EntityTeleport_i32_i32_NoGround,
    ) {
        let on_ground = true; // TODO: how is this supposed to be set? (for 1.7)
        self.on_entity_teleport(
            entity_telport.entity_id,
            f64::from(entity_telport.x),
            f64::from(entity_telport.y),
            f64::from(entity_telport.z),
            entity_telport.yaw as f64,
            entity_telport.pitch as f64,
            on_ground,
        )
    }

    fn on_entity_teleport(
        &mut self,
        entity_id: i32,
        x: f64,
        y: f64,
        z: f64,
        yaw: f64,
        pitch: f64,
        _on_ground: bool,
    ) {
        use std::f64::consts::PI;
        if let Some(entity) = self.entity_map.get(&entity_id) {
            let target_position = self
                .entities
                .get_component_mut(*entity, self.target_position)
                .unwrap();
            let target_rotation = self
                .entities
                .get_component_mut(*entity, self.target_rotation)
                .unwrap();
            target_position.position.x = x;
            target_position.position.y = y;
            target_position.position.z = z;
            target_rotation.yaw = -(yaw / 256.0) * PI * 2.0;
            target_rotation.pitch = -(pitch / 256.0) * PI * 2.0;
        }
    }

    fn on_entity_move_i16(&mut self, m: packet::play::clientbound::EntityMove_i16) {
        self.on_entity_move(
            m.entity_id.0,
            f64::from(m.delta_x),
            f64::from(m.delta_y),
            f64::from(m.delta_z),
        )
    }

    fn on_entity_move_i8(&mut self, m: packet::play::clientbound::EntityMove_i8) {
        self.on_entity_move(
            m.entity_id.0,
            f64::from(m.delta_x),
            f64::from(m.delta_y),
            f64::from(m.delta_z),
        )
    }

    fn on_entity_move_i8_i32_noground(
        &mut self,
        m: packet::play::clientbound::EntityMove_i8_i32_NoGround,
    ) {
        self.on_entity_move(
            m.entity_id,
            f64::from(m.delta_x),
            f64::from(m.delta_y),
            f64::from(m.delta_z),
        )
    }

    fn on_entity_move(&mut self, entity_id: i32, delta_x: f64, delta_y: f64, delta_z: f64) {
        if let Some(entity) = self.entity_map.get(&entity_id) {
            let position = self
                .entities
                .get_component_mut(*entity, self.target_position)
                .unwrap();
            position.position.x += delta_x;
            position.position.y += delta_y;
            position.position.z += delta_z;
        }
    }

    fn on_entity_look(&mut self, entity_id: i32, yaw: f64, pitch: f64) {
        use std::f64::consts::PI;
        if let Some(entity) = self.entity_map.get(&entity_id) {
            let rotation = self
                .entities
                .get_component_mut(*entity, self.target_rotation)
                .unwrap();
            rotation.yaw = -(yaw / 256.0) * PI * 2.0;
            rotation.pitch = -(pitch / 256.0) * PI * 2.0;
        }
    }

    fn on_entity_look_varint(&mut self, look: packet::play::clientbound::EntityLook_VarInt) {
        self.on_entity_look(look.entity_id.0, look.yaw as f64, look.pitch as f64)
    }

    fn on_entity_look_i32_noground(
        &mut self,
        look: packet::play::clientbound::EntityLook_i32_NoGround,
    ) {
        self.on_entity_look(look.entity_id, look.yaw as f64, look.pitch as f64)
    }

    fn on_entity_look_and_move_i16(
        &mut self,
        lookmove: packet::play::clientbound::EntityLookAndMove_i16,
    ) {
        self.on_entity_look_and_move(
            lookmove.entity_id.0,
            f64::from(lookmove.delta_x),
            f64::from(lookmove.delta_y),
            f64::from(lookmove.delta_z),
            lookmove.yaw as f64,
            lookmove.pitch as f64,
        )
    }

    fn on_entity_look_and_move_i8(
        &mut self,
        lookmove: packet::play::clientbound::EntityLookAndMove_i8,
    ) {
        self.on_entity_look_and_move(
            lookmove.entity_id.0,
            f64::from(lookmove.delta_x),
            f64::from(lookmove.delta_y),
            f64::from(lookmove.delta_z),
            lookmove.yaw as f64,
            lookmove.pitch as f64,
        )
    }

    fn on_entity_look_and_move_i8_i32_noground(
        &mut self,
        lookmove: packet::play::clientbound::EntityLookAndMove_i8_i32_NoGround,
    ) {
        self.on_entity_look_and_move(
            lookmove.entity_id,
            f64::from(lookmove.delta_x),
            f64::from(lookmove.delta_y),
            f64::from(lookmove.delta_z),
            lookmove.yaw as f64,
            lookmove.pitch as f64,
        )
    }

    fn on_entity_look_and_move(
        &mut self,
        entity_id: i32,
        delta_x: f64,
        delta_y: f64,
        delta_z: f64,
        yaw: f64,
        pitch: f64,
    ) {
        use std::f64::consts::PI;
        if let Some(entity) = self.entity_map.get(&entity_id) {
            let position = self
                .entities
                .get_component_mut(*entity, self.target_position)
                .unwrap();
            let rotation = self
                .entities
                .get_component_mut(*entity, self.target_rotation)
                .unwrap();
            position.position.x += delta_x;
            position.position.y += delta_y;
            position.position.z += delta_z;
            rotation.yaw = -(yaw / 256.0) * PI * 2.0;
            rotation.pitch = -(pitch / 256.0) * PI * 2.0;
        }
    }

    fn on_player_spawn_f64_nometa(
        &mut self,
        spawn: packet::play::clientbound::SpawnPlayer_f64_NoMeta,
    ) {
        self.on_player_spawn(
            spawn.entity_id.0,
            spawn.uuid,
            spawn.x,
            spawn.y,
            spawn.z,
            spawn.yaw as f64,
            spawn.pitch as f64,
        )
    }

    fn on_player_spawn_f64(&mut self, spawn: packet::play::clientbound::SpawnPlayer_f64) {
        self.on_player_spawn(
            spawn.entity_id.0,
            spawn.uuid,
            spawn.x,
            spawn.y,
            spawn.z,
            spawn.yaw as f64,
            spawn.pitch as f64,
        )
    }

    fn on_player_spawn_i32(&mut self, spawn: packet::play::clientbound::SpawnPlayer_i32) {
        self.on_player_spawn(
            spawn.entity_id.0,
            spawn.uuid,
            f64::from(spawn.x),
            f64::from(spawn.y),
            f64::from(spawn.z),
            spawn.yaw as f64,
            spawn.pitch as f64,
        )
    }

    fn on_player_spawn_i32_helditem(
        &mut self,
        spawn: packet::play::clientbound::SpawnPlayer_i32_HeldItem,
    ) {
        self.on_player_spawn(
            spawn.entity_id.0,
            spawn.uuid,
            f64::from(spawn.x),
            f64::from(spawn.y),
            f64::from(spawn.z),
            spawn.yaw as f64,
            spawn.pitch as f64,
        )
    }

    fn on_player_spawn_i32_helditem_string(
        &mut self,
        spawn: packet::play::clientbound::SpawnPlayer_i32_HeldItem_String,
    ) {
        self.on_player_spawn(
            spawn.entity_id.0,
            protocol::UUID::from_str(&spawn.uuid).unwrap(),
            f64::from(spawn.x),
            f64::from(spawn.y),
            f64::from(spawn.z),
            spawn.yaw as f64,
            spawn.pitch as f64,
        )
    }

    fn on_player_spawn(
        &mut self,
        entity_id: i32,
        uuid: protocol::UUID,
        x: f64,
        y: f64,
        z: f64,
        pitch: f64,
        yaw: f64,
    ) {
        use std::f64::consts::PI;
        if let Some(entity) = self.entity_map.remove(&entity_id) {
            self.entities.remove_entity(entity);
        }
        let entity = entity::player::create_remote(
            &mut self.entities,
            self.players.get(&uuid).map_or("MISSING", |v| &v.name),
        );
        let position = self
            .entities
            .get_component_mut(entity, self.position)
            .unwrap();
        let target_position = self
            .entities
            .get_component_mut(entity, self.target_position)
            .unwrap();
        let rotation = self
            .entities
            .get_component_mut(entity, self.rotation)
            .unwrap();
        let target_rotation = self
            .entities
            .get_component_mut(entity, self.target_rotation)
            .unwrap();
        position.position.x = x;
        position.position.y = y;
        position.position.z = z;
        target_position.position.x = x;
        target_position.position.y = y;
        target_position.position.z = z;
        rotation.yaw = -(yaw / 256.0) * PI * 2.0;
        rotation.pitch = -(pitch / 256.0) * PI * 2.0;
        target_rotation.yaw = rotation.yaw;
        target_rotation.pitch = rotation.pitch;
        if let Some(info) = self.players.get(&uuid) {
            let model = self
                .entities
                .get_component_mut_direct::<entity::player::PlayerModel>(entity)
                .unwrap();
            model.set_skin(info.skin_url.clone());
        }
        self.entity_map.insert(entity_id, entity);
    }

    fn on_teleport_player_withconfirm(
        &mut self,
        teleport: packet::play::clientbound::TeleportPlayer_WithConfirm,
    ) {
        self.on_teleport_player(
            teleport.x,
            teleport.y,
            teleport.z,
            teleport.yaw as f64,
            teleport.pitch as f64,
            teleport.flags,
            Some(teleport.teleport_id),
        )
    }

    fn on_teleport_player_noconfirm(
        &mut self,
        teleport: packet::play::clientbound::TeleportPlayer_NoConfirm,
    ) {
        self.on_teleport_player(
            teleport.x,
            teleport.y,
            teleport.z,
            teleport.yaw as f64,
            teleport.pitch as f64,
            teleport.flags,
            None,
        )
    }

    fn on_teleport_player_onground(
        &mut self,
        teleport: packet::play::clientbound::TeleportPlayer_OnGround,
    ) {
        let flags: u8 = 0; // always absolute
        self.on_teleport_player(
            teleport.x,
            teleport.eyes_y - 1.62,
            teleport.z,
            teleport.yaw as f64,
            teleport.pitch as f64,
            flags,
            None,
        )
    }

    fn on_teleport_player(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        yaw: f64,
        pitch: f64,
        flags: u8,
        teleport_id: Option<protocol::VarInt>,
    ) {
        use std::f64::consts::PI;
        if let Some(player) = self.player {
            let position = self
                .entities
                .get_component_mut(player, self.target_position)
                .unwrap();
            let rotation = self
                .entities
                .get_component_mut(player, self.rotation)
                .unwrap();
            let velocity = self
                .entities
                .get_component_mut(player, self.velocity)
                .unwrap();

            position.position.x =
                calculate_relative_teleport(TeleportFlag::RelX, flags, position.position.x, x);
            position.position.y =
                calculate_relative_teleport(TeleportFlag::RelY, flags, position.position.y, y);
            position.position.z =
                calculate_relative_teleport(TeleportFlag::RelZ, flags, position.position.z, z);
            rotation.yaw = calculate_relative_teleport(
                TeleportFlag::RelYaw,
                flags,
                rotation.yaw,
                -yaw as f64 * (PI / 180.0),
            );

            rotation.pitch = -((calculate_relative_teleport(
                TeleportFlag::RelPitch,
                flags,
                (-rotation.pitch) * (180.0 / PI) + 180.0,
                pitch,
            ) - 180.0)
                * (PI / 180.0));

            if (flags & (TeleportFlag::RelX as u8)) == 0 {
                velocity.velocity.x = 0.0;
            }
            if (flags & (TeleportFlag::RelY as u8)) == 0 {
                velocity.velocity.y = 0.0;
            }
            if (flags & (TeleportFlag::RelZ as u8)) == 0 {
                velocity.velocity.z = 0.0;
            }

            if let Some(teleport_id) = teleport_id {
                self.write_packet(packet::play::serverbound::TeleportConfirm { teleport_id });
            }
        }
    }

    fn on_block_entity_update(
        &mut self,
        block_update: packet::play::clientbound::UpdateBlockEntity,
    ) {
        match block_update.nbt {
            None => {
                // NBT is null, so we need to remove the block entity
                self.world
                    .add_block_entity_action(world::BlockEntityAction::Remove(
                        block_update.location,
                    ));
            }
            Some(nbt) => {
                match block_update.action {
                    // TODO: support more block update actions
                    //1 => // Mob spawner
                    //2 => // Command block text
                    //3 => // Beacon
                    //4 => // Mob head
                    //5 => // Conduit
                    //6 => // Banner
                    //7 => // Structure
                    //8 => // Gateway
                    9 => {
                        // Sign
                        let line1 = format::Component::from_string(
                            nbt.1.get("Text1").unwrap().as_str().unwrap(),
                        );
                        let line2 = format::Component::from_string(
                            nbt.1.get("Text2").unwrap().as_str().unwrap(),
                        );
                        let line3 = format::Component::from_string(
                            nbt.1.get("Text3").unwrap().as_str().unwrap(),
                        );
                        let line4 = format::Component::from_string(
                            nbt.1.get("Text4").unwrap().as_str().unwrap(),
                        );
                        self.world.add_block_entity_action(
                            world::BlockEntityAction::UpdateSignText(Box::new((
                                block_update.location,
                                line1,
                                line2,
                                line3,
                                line4,
                            ))),
                        );
                    }
                    //10 => // Unused
                    //11 => // Jigsaw
                    //12 => // Campfire
                    //14 => // Beehive
                    _ => {
                        debug!("Unsupported block entity action: {}", block_update.action);
                    }
                }
            }
        }
    }

    fn on_block_entity_update_data(
        &mut self,
        _block_update: packet::play::clientbound::UpdateBlockEntity_Data,
    ) {
        // TODO: handle UpdateBlockEntity_Data for 1.7, decompress gzipped_nbt
    }

    fn on_sign_update(&mut self, mut update_sign: packet::play::clientbound::UpdateSign) {
        format::convert_legacy(&mut update_sign.line1);
        format::convert_legacy(&mut update_sign.line2);
        format::convert_legacy(&mut update_sign.line3);
        format::convert_legacy(&mut update_sign.line4);
        self.world
            .add_block_entity_action(world::BlockEntityAction::UpdateSignText(Box::new((
                update_sign.location,
                update_sign.line1,
                update_sign.line2,
                update_sign.line3,
                update_sign.line4,
            ))));
    }

    fn on_sign_update_u16(&mut self, mut update_sign: packet::play::clientbound::UpdateSign_u16) {
        format::convert_legacy(&mut update_sign.line1);
        format::convert_legacy(&mut update_sign.line2);
        format::convert_legacy(&mut update_sign.line3);
        format::convert_legacy(&mut update_sign.line4);
        self.world
            .add_block_entity_action(world::BlockEntityAction::UpdateSignText(Box::new((
                Position::new(update_sign.x, update_sign.y as i32, update_sign.z),
                update_sign.line1,
                update_sign.line2,
                update_sign.line3,
                update_sign.line4,
            ))));
    }

    fn on_player_info_string(
        &mut self,
        _player_info: packet::play::clientbound::PlayerInfo_String,
    ) {
        // TODO: support PlayerInfo_String for 1.7
    }

    fn on_player_info(&mut self, player_info: packet::play::clientbound::PlayerInfo) {
        use crate::protocol::packet::PlayerDetail::*;
        for detail in player_info.inner.players {
            match detail {
                Add {
                    name,
                    uuid,
                    properties,
                    display,
                    gamemode,
                    ping,
                } => {
                    let info = self.players.entry(uuid.clone()).or_insert(PlayerInfo {
                        name: name.clone(),
                        uuid,
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
                        let skin_blob_result = &base64::decode(&prop.value);
                        let skin_blob = match skin_blob_result {
                            Ok(val) => val,
                            Err(err) => {
                                error!("Failed to decode skin blob, {:?}", err);
                                continue;
                            }
                        };
                        let skin_blob: serde_json::Value = match serde_json::from_slice(&skin_blob)
                        {
                            Ok(val) => val,
                            Err(err) => {
                                error!("Failed to parse skin blob, {:?}", err);
                                continue;
                            }
                        };
                        if let Some(skin_url) = skin_blob
                            .pointer("/textures/SKIN/url")
                            .and_then(|v| v.as_str())
                        {
                            info.skin_url = Some(skin_url.to_owned());
                        }
                    }

                    // Refresh our own skin when the server sends it to us.
                    // The join game packet can come before this packet meaning
                    // we may not have the skin in time for spawning ourselves.
                    // This isn't an issue for other players because this packet
                    // must come before the spawn player packet.
                    if info.uuid == self.uuid {
                        let model = self
                            .entities
                            .get_component_mut_direct::<entity::player::PlayerModel>(
                                self.player.unwrap(),
                            )
                            .unwrap();
                        model.set_skin(info.skin_url.clone());
                    }
                }
                UpdateGamemode { uuid, gamemode } => {
                    if let Some(info) = self.players.get_mut(&uuid) {
                        info.gamemode = Gamemode::from_int(gamemode.0);
                    }
                }
                UpdateLatency { uuid, ping } => {
                    if let Some(info) = self.players.get_mut(&uuid) {
                        info.ping = ping.0;
                    }
                }
                UpdateDisplayName { uuid, display } => {
                    if let Some(info) = self.players.get_mut(&uuid) {
                        info.display_name = display;
                    }
                }
                Remove { uuid } => {
                    self.players.remove(&uuid);
                }
            }
        }
    }

    fn load_block_entities(&mut self, block_entities: Vec<Option<crate::nbt::NamedTag>>) {
        for optional_block_entity in block_entities {
            if let Some(block_entity) = optional_block_entity {
                let x = block_entity.1.get("x").unwrap().as_int().unwrap();
                let y = block_entity.1.get("y").unwrap().as_int().unwrap();
                let z = block_entity.1.get("z").unwrap().as_int().unwrap();
                let tile_id = block_entity.1.get("id").unwrap().as_str().unwrap();
                let action;
                match tile_id {
                    // Fake a sign update
                    "Sign" => action = 9,
                    // Not something we care about, so break the loop
                    _ => continue,
                }
                self.on_block_entity_update(packet::play::clientbound::UpdateBlockEntity {
                    location: Position::new(x, y, z),
                    action,
                    nbt: Some(block_entity.clone()),
                });
            }
        }
    }

    fn on_chunk_data_biomes3d_bool(
        &mut self,
        chunk_data: packet::play::clientbound::ChunkData_Biomes3D_bool,
    ) {
        self.world
            .load_chunk115(
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                chunk_data.data.data,
            )
            .unwrap();
        self.load_block_entities(chunk_data.block_entities.data);
    }

    fn on_chunk_data_biomes3d(
        &mut self,
        chunk_data: packet::play::clientbound::ChunkData_Biomes3D,
    ) {
        self.world
            .load_chunk115(
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                chunk_data.data.data,
            )
            .unwrap();
        self.load_block_entities(chunk_data.block_entities.data);
    }

    fn on_chunk_data(&mut self, chunk_data: packet::play::clientbound::ChunkData) {
        self.world
            .load_chunk19(
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                chunk_data.data.data,
            )
            .unwrap();
        self.load_block_entities(chunk_data.block_entities.data);
    }

    fn on_chunk_data_heightmap(
        &mut self,
        chunk_data: packet::play::clientbound::ChunkData_HeightMap,
    ) {
        self.world
            .load_chunk19(
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                chunk_data.data.data,
            )
            .unwrap();
        self.load_block_entities(chunk_data.block_entities.data);
    }

    fn on_chunk_data_no_entities(
        &mut self,
        chunk_data: packet::play::clientbound::ChunkData_NoEntities,
    ) {
        self.world
            .load_chunk19(
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                chunk_data.data.data,
            )
            .unwrap();
    }

    fn on_chunk_data_no_entities_u16(
        &mut self,
        chunk_data: packet::play::clientbound::ChunkData_NoEntities_u16,
    ) {
        let chunk_meta = vec![crate::protocol::packet::ChunkMeta {
            x: chunk_data.chunk_x,
            z: chunk_data.chunk_z,
            bitmask: chunk_data.bitmask,
        }];
        let skylight = false;
        self.world
            .load_chunks18(chunk_data.new, skylight, &chunk_meta, chunk_data.data.data)
            .unwrap();
    }

    fn on_chunk_data_17(&mut self, chunk_data: packet::play::clientbound::ChunkData_17) {
        self.world
            .load_chunk17(
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask,
                chunk_data.add_bitmask,
                chunk_data.compressed_data.data,
            )
            .unwrap();
    }

    fn on_chunk_data_bulk(&mut self, bulk: packet::play::clientbound::ChunkDataBulk) {
        let new = true;
        self.world
            .load_chunks18(
                new,
                bulk.skylight,
                &bulk.chunk_meta.data,
                bulk.chunk_data.to_vec(),
            )
            .unwrap();
    }

    fn on_chunk_data_bulk_17(&mut self, bulk: packet::play::clientbound::ChunkDataBulk_17) {
        self.world
            .load_chunks17(
                bulk.chunk_column_count,
                bulk.data_length,
                bulk.skylight,
                &bulk.chunk_data_and_meta,
            )
            .unwrap();
    }

    fn on_chunk_unload(&mut self, chunk_unload: packet::play::clientbound::ChunkUnload) {
        self.world
            .unload_chunk(chunk_unload.x, chunk_unload.z, &mut self.entities);
    }

    fn on_block_change(&mut self, location: Position, id: i32) {
        self.world.set_block(
            location,
            block::Block::by_vanilla_id(
                id as usize,
                self.protocol_version,
                &self.world.modded_block_ids,
            ),
        )
    }

    fn on_block_change_varint(
        &mut self,
        block_change: packet::play::clientbound::BlockChange_VarInt,
    ) {
        self.on_block_change(block_change.location, block_change.block_id.0)
    }

    fn on_block_change_u8(&mut self, block_change: packet::play::clientbound::BlockChange_u8) {
        self.on_block_change(
            crate::shared::Position::new(block_change.x, block_change.y as i32, block_change.z),
            (block_change.block_id.0 << 4) | (block_change.block_metadata as i32),
        );
    }

    fn on_multi_block_change_varint(
        &mut self,
        block_change: packet::play::clientbound::MultiBlockChange_VarInt,
    ) {
        let ox = block_change.chunk_x << 4;
        let oz = block_change.chunk_z << 4;
        for record in block_change.records.data {
            self.world.set_block(
                Position::new(
                    ox + (record.xz >> 4) as i32,
                    record.y as i32,
                    oz + (record.xz & 0xF) as i32,
                ),
                block::Block::by_vanilla_id(
                    record.block_id.0 as usize,
                    self.protocol_version,
                    &self.world.modded_block_ids,
                ),
            );
        }
    }

    fn on_multi_block_change_u16(
        &mut self,
        block_change: packet::play::clientbound::MultiBlockChange_u16,
    ) {
        let ox = block_change.chunk_x << 4;
        let oz = block_change.chunk_z << 4;

        let mut data = std::io::Cursor::new(block_change.data);

        for _ in 0..block_change.record_count {
            use byteorder::{BigEndian, ReadBytesExt};

            let record = data.read_u32::<BigEndian>().unwrap();

            let id = record & 0x0000_ffff;
            let y = ((record & 0x00ff_0000) >> 16) as i32;
            let z = oz + ((record & 0x0f00_0000) >> 24) as i32;
            let x = ox + ((record & 0xf000_0000) >> 28) as i32;

            self.world.set_block(
                Position::new(x, y, z),
                block::Block::by_vanilla_id(
                    id as usize,
                    self.protocol_version,
                    &self.world.modded_block_ids,
                ),
            );
        }
    }
}

#[allow(clippy::enum_variant_names)]
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
