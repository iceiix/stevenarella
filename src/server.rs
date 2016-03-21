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

use protocol::{self, mojang};
use world;
use world::block::{self, BlockSet};
use rand::{self, Rng};
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use resources;
use openssl;
use console;
use auth;

pub struct Server {
    conn: Option<protocol::Conn>,
    pub world: world::World,

    resources: Arc<RwLock<resources::Manager>>,
    console: Arc<Mutex<console::Console>>,
    version: usize,
}

impl Server {

    pub fn connect(resources: Arc<RwLock<resources::Manager>>, console: Arc<Mutex<console::Console>>, address: &str) -> Result<Server, protocol::Error> {
        let mut conn = try!(protocol::Conn::new(address));

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
            username: "Thinkofdeath".to_owned()
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

        let profile = {
            let console = console.lock().unwrap();
            mojang::Profile {
                username: console.get(auth::CL_USERNAME).clone(),
                id: console.get(auth::CL_UUID).clone(),
                access_token: console.get(auth::AUTH_TOKEN).clone(),
            }
        };
        try!(profile.join_server(&packet.server_id, &shared, &packet.public_key.data));

        try!(conn.write_packet(protocol::packet::login::serverbound::EncryptionResponse {
            shared_secret: protocol::LenPrefixedBytes::new(shared_e),
            verify_token: protocol::LenPrefixedBytes::new(token_e),
        }));

        let mut read = conn.clone(); // TODO
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

        let version = resources.read().unwrap().version();
        Ok(Server {
            conn: Some(write),
            world: world::World::new(),
            resources: resources,
            console: console,
            version: version,
        })
    }

    pub fn dummy_server(resources: Arc<RwLock<resources::Manager>>, console: Arc<Mutex<console::Console>>) -> Server {
        let mut world = world::World::new();
        let mut rng = rand::thread_rng();
        for x in -7*16 .. 7*16 {
            for z in -7*16 .. 7*16 {
                let h = rng.gen_range(3, 10);
                for y in 0 .. h {
                    world.set_block(x, y, z, block::MISSING.base());
                }
            }
        }
        let version = resources.read().unwrap().version();
        Server {
            conn: None,
            world: world,

            version: version,
            resources: resources,
            console: console,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    pub fn tick(&mut self, delta: f64) {
        let version = self.resources.read().unwrap().version();
        if version != self.version {
            self.version = version;
            self.world.flag_dirty_all();
        }
    }
}
