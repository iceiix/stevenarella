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
use world::block;
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
    pub fn dummy_server(resources: Arc<RwLock<resources::Manager>>) -> Server {
        let mut world = world::World::new();
        let mut rng = rand::thread_rng();
        for x in -7*16 .. 7*16 {
            for z in -7*16 .. 7*16 {
                let h = rng.gen_range(3, 10);
                for y in 0 .. h {
                    world.set_block(x, y, z, block::MISSING);
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
