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

use protocol;
use world;
use world::block::{self, BlockSet};
use rand::{self, Rng};
use std::sync::{Arc, RwLock};
use resources;

pub struct Server {
    conn: Option<protocol::Conn>,
    pub world: world::World,

    resources: Arc<RwLock<resources::Manager>>,
    version: usize,
}

impl Server {

    pub fn connect(resources: Arc<RwLock<resources::Manager>>, address: &str) -> Result<Server, protocol::Error> {
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

        let packet = match conn.read_packet().unwrap() {
            protocol::packet::Packet::EncryptionRequest(val) => val,
            protocol::packet::Packet::LoginDisconnect(val) => {
                return Err(protocol::Error::Disconnect(val.reason));
            },
            val => panic!("Wrong packet: {:?}", val),
        };

        unimplemented!();

        let version = resources.read().unwrap().version();
        Ok(Server {
            conn: Some(conn),
            world: world::World::new(),
            resources: resources,
            version: version,
        })
    }

    pub fn dummy_server(resources: Arc<RwLock<resources::Manager>>) -> Server {
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
        }
    }

    pub fn tick(&mut self) {
        let version = self.resources.read().unwrap().version();
        if version != self.version {
            self.version = version;
            self.world.flag_dirty_all();
        }
    }
}
