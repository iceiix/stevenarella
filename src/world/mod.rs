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
use self::block::BlockSet;

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
            None => block::AIR.base(),
        }
    }

    pub fn next_dirty_chunk_section(&mut self) -> Option<(i32, i32, i32)> {
        for (_, chunk) in &mut self.chunks {
            for sec in &mut chunk.sections {
                if let Some(sec) = sec.as_mut() {
                    if !sec.building && sec.dirty {
                        sec.building = true;
                        sec.dirty = false;
                        return Some((chunk.position.0, sec.y as i32, chunk.position.1));
                    }
                }
            }
        }
        None
    }

    pub fn reset_building_flag(&mut self, pos: (i32, i32, i32)) {
        if let Some(chunk) = self.chunks.get_mut(&CPos(pos.0, pos.2)) {
            if let Some(section) = chunk.sections[pos.1 as usize].as_mut() {
                section.building = false;
            }
        }
    }

    pub fn flag_dirty_all(&mut self) {
        for (_, chunk) in &mut self.chunks {
            for sec in &mut chunk.sections {
                if let Some(sec) = sec.as_mut() {
                    sec.dirty = true;
                }
            }
        }
    }

    pub fn capture_snapshot(&self, x: i32, y: i32, z: i32, w: i32, h: i32, d: i32) -> Snapshot {
        use std::cmp::{min, max};
        let mut snapshot = Snapshot {
            blocks: vec![0; (w * h * d) as usize],
            block_light: nibble::Array::new((w * h * d) as usize),
            sky_light: nibble::Array::new((w * h * d) as usize),
            biomes: vec![0; (w * d) as usize],

            x: x, y: y, z: z,
            w: w, h: h, d: d,
        };
        for i in 0 .. (w * h * d) as usize {
            snapshot.sky_light.set(i, 0xF);
            snapshot.blocks[i] = block::MISSING.base().steven_id() as u16;
        }

        let cx1 = x >> 4;
        let cy1 = y >> 4;
        let cz1 = z >> 4;
        let cx2 = (x + w + 15) >> 4;
        let cy2 = (y + h + 15) >> 4;
        let cz2 = (z + d + 15) >> 4;

        for cx in cx1 .. cx2 {
            for cz in cz1 .. cz2 {
                let chunk = match self.chunks.get(&CPos(cx, cz)) {
                    Some(val) => val,
                    None => continue,
                };

                let x1 = min(16, max(0, x - (cx<<4)));
                let x2 = min(16, max(0, x + w - (cx<<4)));
                let z1 = min(16, max(0, z - (cz<<4)));
                let z2 = min(16, max(0, z + d - (cz<<4)));

                for cy in cy1 .. cy2 {
                    if cy < 0 || cy > 15 {
                        continue;
                    }
                    let section = &chunk.sections[cy as usize];
                    let y1 = min(16, max(0, y - (cy<<4)));
                    let y2 = min(16, max(0, y + h - (cy<<4)));

                    for yy in y1 .. y2 {
                        for zz in z1 .. z2 {
                            for xx in x1 .. x2 {
                                let ox = xx + (cx << 4);
                                let oy = yy + (cy << 4);
                                let oz = zz + (cz << 4);
                                match section.as_ref() {
                                    Some(sec) => {
                                        snapshot.set_block(ox, oy, oz, sec.get_block(xx, yy, zz));
                                        snapshot.set_block_light(ox, oy, oz, sec.get_block_light(xx, yy, zz));
                                        snapshot.set_sky_light(ox, oy, oz, sec.get_sky_light(xx, yy, zz));
                                    },
                                    None => {
                                        snapshot.set_block(ox, oy, oz, block::AIR.base());
                                    },
                                }
                            }
                        }
                    }
                }
                // TODO: Biomes
            }
        }

        snapshot
    }
}

pub struct Snapshot {
    blocks: Vec<u16>,
    block_light: nibble::Array,
    sky_light: nibble::Array,
    biomes: Vec<u8>,

    x: i32,
    y: i32,
    z: i32,
    w: i32,
    h: i32,
    d: i32,
}

impl Snapshot {

    pub fn make_relative(&mut self, x: i32, y: i32, z: i32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> &'static block::Block {
        block::get_block_by_steven_id(self.blocks[self.index(x, y, z)] as usize)
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, b: &'static block::Block) {
        let idx = self.index(x, y, z);
        self.blocks[idx] = b.steven_id() as u16;
    }

    pub fn get_block_light(&self, x: i32, y: i32, z: i32) -> u8 {
        self.block_light.get(self.index(x, y, z))
    }

    pub fn set_block_light(&mut self, x: i32, y: i32, z: i32, l: u8) {
        let idx = self.index(x, y, z);
        self.block_light.set(idx, l);
    }

    pub fn get_sky_light(&self, x: i32, y: i32, z: i32) -> u8 {
        self.sky_light.get(self.index(x, y, z))
    }

    pub fn set_sky_light(&mut self, x: i32, y: i32, z: i32, l: u8) {
        let idx = self.index(x, y, z);
        self.sky_light.set(idx, l);
    }

    #[inline]
    fn index(&self, x: i32, y: i32, z: i32) -> usize {
        ((x - self.x) + ((z - self.z) * self.w) + ((y - self.y) * self.w * self.d)) as usize
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
            if b.in_set(&*block::AIR) {
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
            return block::AIR.base();
        }
        match self.sections[s_idx as usize].as_ref() {
            Some(sec) => sec.get_block(x, y & 0xF, z),
            None => block::AIR.base(),
        }
    }
}

struct Section {
    y: u8,

    blocks: bit::Map,
    block_map: Vec<(&'static block::Block, u32)>,
    rev_block_map: HashMap<usize, usize>,

    block_light: nibble::Array,
    sky_light: nibble::Array,

    dirty: bool,
    building: bool,
}

impl Section {
    fn new(y: u8) -> Section {
        let mut section = Section {
            y: y,

            blocks: bit::Map::new(4096, 4),
            block_map: vec![
                (block::AIR.base(), 0xFFFFFFFF)
            ],
            rev_block_map: HashMap::new(),

            block_light: nibble::Array::new(16 * 16 * 16),
            sky_light: nibble::Array::new(16 * 16 * 16),

            dirty: false,
            building: false,
        };
        for i in 0 .. 16*16*16 {
            section.sky_light.set(i, 0xF);
        }
        section.rev_block_map.insert(block::AIR.base().steven_id(), 0);
        section
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> &'static block::Block {
        let idx = self.blocks.get(((y << 8) | (z << 4) | x) as usize);
        self.block_map[idx].0
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, b: &'static block::Block) {
        let old = self.get_block(x, y, z);
        if old.equals(b) {
            return;
        }
        // Clean up old block
        {
            let idx = self.rev_block_map[&old.steven_id()];
            let info = &mut self.block_map[idx];
            info.1 -= 1;
            if info.1 == 0 { // None left of this type
                self.rev_block_map.remove(&old.steven_id());
            }
        }

        if !self.rev_block_map.contains_key(&b.steven_id()) {
            let mut found = false;
            for (i, ref mut info) in self.block_map.iter_mut().enumerate() {
                if info.1 == 0 {
                    info.0 = b;
                    self.rev_block_map.insert(b.steven_id(), i);
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
                self.rev_block_map.insert(b.steven_id(), self.block_map.len());
                self.block_map.push((b, 0));
            }
        }

        let idx = self.rev_block_map[&b.steven_id()];
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
