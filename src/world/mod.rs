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

pub mod block;

use std::collections::HashMap;
use types::bit;
use types::nibble;

pub struct World {
    chunks: HashMap<CPos, Chunk>,
}

impl World {
    pub fn new() -> World {
        World {
            chunks: HashMap::new(),
        }
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, b: &'static block::Block) {
        let cpos = CPos(x >> 4, z >> 4);
        if !self.chunks.contains_key(&cpos) {
            self.chunks.insert(cpos, Chunk::new(cpos));
        }
        let chunk = self.chunks.get_mut(&cpos).unwrap();
        chunk.set_block(x & 0xF, y, z & 0xF, b);
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> &'static block::Block {
        match self.chunks.get(&CPos(x >> 4, z >> 4)) {
            Some(ref chunk) => chunk.get_block(x & 0xF, y, z & 0xF),
            None => block::AIR,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct CPos(i32, i32);

pub struct Chunk {
    position: CPos,

    sections: [Option<Section>; 16],
    biomes: [u8; 16 * 16],
}

impl Chunk {
    fn new(pos: CPos) -> Chunk {
        Chunk {
            position: pos,
            sections: [
                None,None,None,None,
                None,None,None,None,
                None,None,None,None,
                None,None,None,None,
            ],
            biomes: [0; 16 * 16],
        }
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, b: &'static block::Block) {
        let s_idx = y >> 4;
        if s_idx < 0 || s_idx > 15 {
            return;
        }
        if self.sections[s_idx as usize].is_none() {
            if b == block::AIR {
                return;
            }
            self.sections[s_idx as usize] = Some(Section::new(s_idx as u8));
        }
        let section = self.sections[s_idx as usize].as_mut().unwrap();
        section.set_block(x, y & 0xF, z, b);
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> &'static block::Block {
        let s_idx = y >> 4;
        if s_idx < 0 || s_idx > 15 {
            return block::AIR;
        }
        match self.sections[s_idx as usize].as_ref() {
            Some(sec) => sec.get_block(x, y & 0xF, z),
            None => block::AIR,
        }
    }
}

struct Section {
    y: u8,

    blocks: bit::Map,
    block_map: Vec<(&'static block::Block, u32)>,
    rev_block_map: HashMap<&'static block::Block, usize>,

    block_light: nibble::Array,
    sky_light: nibble::Array,

    dirty: bool,
}

impl Section {
    fn new(y: u8) -> Section {
        let mut section = Section {
            y: y,

            blocks: bit::Map::new(4096, 4),
            block_map: vec![
                (block::AIR, 0xFFFFFFFF)
            ],
            rev_block_map: HashMap::new(),

            block_light: nibble::Array::new(16 * 16 * 16),
            sky_light: nibble::Array::new(16 * 16 * 16),

            dirty: false,
        };
        for i in 0 .. 16*16*16 {
            section.sky_light.set(i, 0xF);
        }
        section.rev_block_map.insert(block::AIR, 0);
        section
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> &'static block::Block {
        let idx = self.blocks.get(((y << 8) | (z << 4) | x) as usize);
        self.block_map[idx].0
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, b: &'static block::Block) {
        let old = self.get_block(x, y, z);
        if old == b {
            return;
        }
        // Clean up old block
        {
            let idx = self.rev_block_map[old];
            let info = &mut self.block_map[idx];
            info.1 -= 1;
            if info.1 == 0 { // None left of this type
                self.rev_block_map.remove(old);
            }
        }

        if !self.rev_block_map.contains_key(b) {
            let mut found = false;
            for (i, ref mut info) in self.block_map.iter_mut().enumerate() {
                if info.1 == 0 {
                    info.0 = b;
                    self.rev_block_map.insert(b, i);
                    found = true;
                    break;
                }
            }
            if !found {
                if self.block_map.len() >= 1 << self.blocks.bit_size {
                    let new_size = self.blocks.bit_size << 1;
                    let new_blocks = self.blocks.resize(new_size);
                    self.blocks = new_blocks;
                }
                self.rev_block_map.insert(b, self.block_map.len());
                self.block_map.push((b, 0));
            }
        }

        let idx = self.rev_block_map[b];
        let info = &mut self.block_map[idx];
        info.1 += 1;
        self.blocks.set(((y << 8) | (z << 4) | x) as usize, idx);
        self.dirty = true;
    }

    fn get_block_light(&self, x: i32, y: i32, z: i32) -> u8 {
        self.block_light.get(((y << 8) | (z << 4) | x) as usize)
    }

    fn set_block_light(&mut self, x: i32, y: i32, z: i32, l: u8) {
        self.block_light.set(((y << 8) | (z << 4) | x) as usize, l);
    }

    fn get_sky_light(&self, x: i32, y: i32, z: i32) -> u8 {
        self.sky_light.get(((y << 8) | (z << 4) | x) as usize)
    }

    fn set_sky_light(&mut self, x: i32, y: i32, z: i32, l: u8) {
        self.sky_light.set(((y << 8) | (z << 4) | x) as usize, l);
    }
}
