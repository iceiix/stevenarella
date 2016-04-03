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
use std::hash::BuildHasherDefault;
use types::{bit, nibble, Direction};
use types::hash::FNVHash;
use protocol;
use render;
use collision;
use cgmath;
use chunk_builder;

pub mod biome;

pub struct World {
    chunks: HashMap<CPos, Chunk, BuildHasherDefault<FNVHash>>,

    render_list: Vec<(i32, i32, i32)>,
}

impl World {
    pub fn new() -> World {
        World {
            chunks: HashMap::with_hasher(BuildHasherDefault::default()),
            render_list: vec![],
        }
    }

    pub fn is_chunk_loaded(&self, x: i32, z: i32) -> bool {
        self.chunks.contains_key(&CPos(x, z))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, b: block::Block) {
        self.set_block_raw(x, y, z, b);
        self.update_block(x, y, z);
    }

    fn set_block_raw(&mut self, x: i32, y: i32, z: i32, b: block::Block) {
        let cpos = CPos(x >> 4, z >> 4);
        let chunk = self.chunks.entry(cpos).or_insert_with(|| Chunk::new(cpos));
        chunk.set_block(x & 0xF, y, z & 0xF, b);
    }

    pub fn update_block(&mut self, x: i32, y: i32, z: i32) {
        for yy in -1 .. 2 {
            for zz in -1 .. 2 {
                for xx in -1 .. 2 {
                    let (bx, by, bz) = (x+xx, y+yy, z+zz);
                    let current = self.get_block(bx, by, bz);
                    let new = current.update_state(self, bx, by, bz);
                    if current != new {
                        self.set_block_raw(bx, by, bz, new);
                    }
                    self.set_dirty(bx >> 4, by >> 4, bz >> 4);
                }
            }
        }
    }

    fn update_range(&mut self, x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) {
        for by in y1 .. y2 {
            for bz in z1 .. z2 {
                for bx in x1 .. x2 {
                    let current = self.get_block(bx, by, bz);
                    let new = current.update_state(self, bx, by, bz);
                    if current != new {
                        self.set_block_raw(bx, by, bz, new);
                    }
                }
            }
        }
    }

    pub fn get_block_offset(&self, x: i32, y: i32, z: i32, dir: Direction) -> block::Block {
        let (ox, oy, oz) = dir.get_offset();
        self.get_block(x + ox, y + oy, z + oz)
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> block::Block {
        match self.chunks.get(&CPos(x >> 4, z >> 4)) {
            Some(ref chunk) => chunk.get_block(x & 0xF, y, z & 0xF),
            None => block::Missing{},
        }
    }

    pub fn copy_cloud_heightmap(&mut self, data: &mut [u8]) -> bool {
        let mut dirty = false;
        for (_, c) in &mut self.chunks {
            if c.heightmap_dirty {
                dirty = true;
                c.heightmap_dirty = false;
                for xx in 0 .. 16 {
                    for zz in 0 .. 16 {
                        data[
                            (((c.position.0 << 4) as usize + xx) & 0x1FF) +
                            ((((c.position.1 << 4) as usize + zz) & 0x1FF) << 9)
                        ] = c.heightmap[(zz << 4) | xx];
                    }
                }
            }
        }
        dirty
    }

    pub fn compute_render_list(&mut self, renderer: &mut render::Renderer) {
        use chunk_builder;
        use types::Direction;
        use cgmath::Vector;
        use std::collections::VecDeque;
        self.render_list.clear();

        let mut valid_dirs = [false; 6];
        for dir in Direction::all() {
            let (ox, oy, oz) = dir.get_offset();
            let dir_vec = cgmath::Vector3::new(ox as f32, oy as f32, oz as f32);
            valid_dirs[dir.index()] = renderer.view_vector.dot(dir_vec) > -0.8;
        }

        let start = (
            ((renderer.camera.pos.x as i32) >> 4),
            ((renderer.camera.pos.y as i32) >> 4),
            ((renderer.camera.pos.z as i32) >> 4)
        );

        let mut process_queue = VecDeque::with_capacity(self.chunks.len() * 16);
        process_queue.push_front((Direction::Invalid, start));

        while let Some((from, pos)) = process_queue.pop_front() {
            let (exists, cull) = if let Some((sec, rendered_on)) = self.get_render_section_mut(pos.0, pos.1, pos.2) {
                if *rendered_on == renderer.frame_id {
                    continue;
                }
                *rendered_on = renderer.frame_id;

                let min = cgmath::Point3::new(pos.0 as f32 * 16.0, -pos.1 as f32 * 16.0, pos.2 as f32 * 16.0);
                let bounds = collision::Aabb3::new(min, min + cgmath::Vector3::new(16.0, -16.0, 16.0));
                if renderer.frustum.contains(bounds) == collision::Relation::Out && from != Direction::Invalid {
                    continue;
                }
                (sec.is_some(), sec.map_or(chunk_builder::CullInfo::all_vis(), |v| v.cull_info))
            } else {
                continue;
            };

            if exists {
                self.render_list.push(pos);
            }

            for dir in Direction::all() {
                let (ox, oy, oz) = dir.get_offset();
                let opos = (pos.0 + ox, pos.1 + oy, pos.2 + oz);
                if let Some((_, rendered_on)) = self.get_render_section_mut(opos.0, opos.1, opos.2) {
                    if *rendered_on == renderer.frame_id {
                        continue;
                    }
                    if from == Direction::Invalid || (valid_dirs[dir.index()] && cull.is_visible(from, dir)) {
                        process_queue.push_back((dir.opposite(), opos));
                    }
                }
            }
        }
    }

    pub fn get_render_list(&self) -> Vec<((i32, i32, i32), &render::ChunkBuffer)> {
        self.render_list.iter().map(|v| {
            let chunk = self.chunks.get(&CPos(v.0, v.2)).unwrap();
            let sec = chunk.sections[v.1 as usize].as_ref().unwrap();
            (*v, &sec.render_buffer)
        }).collect()
    }

    pub fn get_section_mut(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Section> {
        if let Some(chunk) = self.chunks.get_mut(&CPos(x, z)) {
            if let Some(sec) = chunk.sections[y as usize].as_mut() {
                return Some(sec);
            }
        }
        None
    }

    fn get_render_section_mut(&mut self, x: i32, y: i32, z: i32) -> Option<(Option<&mut Section>, &mut u32)> {
        if y < 0 || y > 15 {
            return None;
        }
        if let Some(chunk) = self.chunks.get_mut(&CPos(x, z)) {
            let rendered = &mut chunk.sections_rendered_on[y as usize];
            if let Some(sec) = chunk.sections[y as usize].as_mut() {
                return Some((Some(sec), rendered));
            }
            return Some((None, rendered));
        }
        None
    }

    pub fn get_dirty_chunk_sections(&mut self) -> Vec<(i32, i32, i32)> {
        let mut out = vec![];
        for (_, chunk) in &mut self.chunks {
            for sec in &mut chunk.sections {
                if let Some(sec) = sec.as_mut() {
                    if !sec.building && sec.dirty {
                        out.push((chunk.position.0, sec.y as i32, chunk.position.1));
                    }
                }
            }
        }
        out
    }

    fn set_dirty(&mut self, x: i32, y: i32, z: i32) {
        if let Some(chunk) = self.chunks.get_mut(&CPos(x, z)) {
            if let Some(sec) = chunk.sections.get_mut(y as usize).and_then(|v| v.as_mut()) {
                sec.dirty = true;
            }
        }
    }

    pub fn is_section_dirty(&self, pos: (i32, i32, i32)) -> bool {
        if let Some(chunk) = self.chunks.get(&CPos(pos.0, pos.2)) {
            if let Some(sec) = chunk.sections[pos.1 as usize].as_ref() {
                return sec.dirty && !sec.building;
            }
        }
        false
    }

    pub fn set_building_flag(&mut self, pos: (i32, i32, i32)) {
        if let Some(chunk) = self.chunks.get_mut(&CPos(pos.0, pos.2)) {
            if let Some(sec) = chunk.sections[pos.1 as usize].as_mut() {
                sec.building = true;
                sec.dirty = false;
            }
        }
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
            w: w, _h: h, d: d,
        };
        for i in 0 .. (w * h * d) as usize {
            snapshot.sky_light.set(i, 0xF);
            snapshot.blocks[i] = block::Missing{}.get_steven_id() as u16;
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
                                        snapshot.set_block(ox, oy, oz, block::Air{});
                                    },
                                }
                            }
                        }
                    }
                }
                for zz in z1 .. z2 {
                    for xx in x1 .. x2 {
                        let ox = xx + (cx << 4);
                        let oz = zz + (cz << 4);
                        snapshot.set_biome(ox, oz, chunk.get_biome(xx, zz));
                    }
                }
            }
        }

        snapshot
    }

    pub fn unload_chunk(&mut self, x: i32, z: i32) {
        self.chunks.remove(&CPos(x, z));
    }

    pub fn load_chunk(&mut self, x: i32, z: i32, new: bool, mask: u16, data: Vec<u8>) -> Result<(), protocol::Error> {
        use std::io::{Cursor, Read};
        use byteorder::ReadBytesExt;
        use protocol::{VarInt, Serializable, LenPrefixed};

        let mut data = Cursor::new(data);

        let cpos = CPos(x, z);
        {
            let chunk = if new {
                self.chunks.insert(cpos, Chunk::new(cpos));
                self.chunks.get_mut(&cpos).unwrap()
            } else {
                if !self.chunks.contains_key(&cpos) {
                    return Ok(());
                }
                self.chunks.get_mut(&cpos).unwrap()
            };

            for i in 0 .. 16 {
                if mask & (1 << i) == 0 {
                    continue;
                }
                if chunk.sections[i].is_none() {
                    chunk.sections[i] = Some(Section::new(x, i as u8, z));
                }
                let section = chunk.sections[i as usize].as_mut().unwrap();
                section.dirty = true;

                let mut bit_size = try!(data.read_u8());
                let mut block_map = HashMap::with_hasher(BuildHasherDefault::<FNVHash>::default());
                if bit_size == 0 {
                    bit_size = 13;
                } else {                    
                    let count = try!(VarInt::read_from(&mut data)).0;
                    for i in 0 .. count {
                        let id = try!(VarInt::read_from(&mut data)).0;
                        block_map.insert(i as usize, id);
                    }
                }

                let bits = try!(LenPrefixed::<VarInt, u64>::read_from(&mut data)).data;
                let m = bit::Map::from_raw(bits, bit_size as usize);

                for i in 0 .. 4096 {
                    let val = m.get(i);
                    let block_id = block_map.get(&val).map_or(val, |v| *v as usize);
                    let block = block::Block::by_vanilla_id(block_id);
                    let i = i as i32;
                    section.set_block(
                        i & 0xF,
                        i >> 8,
                        (i >> 4) & 0xF,
                        block
                    );
                }

                try!(data.read_exact(&mut section.block_light.data));
                try!(data.read_exact(&mut section.sky_light.data));
            }

            if new {
                try!(data.read_exact(&mut chunk.biomes));
            }

            chunk.calculate_heightmap();
        }

        for i in 0 .. 16 {
            if mask & (1 << i) == 0 {
                continue;
            }
            for pos in [
                (-1, 0, 0), (1, 0, 0),
                (0, -1, 0), (0, 1, 0),
                (0, 0, -1), (0, 0, 1)].into_iter() {
                self.flag_section_dirty(x + pos.0, i as i32 + pos.1, z + pos.2);
            }
            self.update_range(
                (x<<4) - 1, (i<<4) - 1, (z<<4) - 1,
                (x<<4) + 17, (i<<4) + 17, (z<<4) + 17
            );
        }
        Ok(())
    }

    fn flag_section_dirty(&mut self, x: i32, y: i32, z: i32) {
        if y < 0 || y > 15 {
            return;
        }
        let cpos = CPos(x, z);
        if let Some(chunk) = self.chunks.get_mut(&cpos) {
            if let Some(sec) = chunk.sections[y as usize].as_mut() {
                sec.dirty = true;
            }
        }
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
    _h: i32,
    d: i32,
}

impl Snapshot {

    pub fn make_relative(&mut self, x: i32, y: i32, z: i32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> block::Block {
        block::Block::by_steven_id(self.blocks[self.index(x, y, z)] as usize)
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, b: block::Block) {
        let idx = self.index(x, y, z);
        self.blocks[idx] = b.get_steven_id() as u16;
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

    pub fn get_biome(&self, x: i32, z: i32) -> biome::Biome {
        biome::Biome::by_id(self.biomes[((x - self.x) | ((z - self.z) << 4)) as usize] as usize)
    }

    pub fn set_biome(&mut self, x: i32, z: i32, b: biome::Biome) {
        self.biomes[((x - self.x) | ((z - self.z) << 4)) as usize] = b.id as u8;
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
    sections_rendered_on: [u32; 16],
    biomes: [u8; 16 * 16],

    heightmap: [u8; 16 * 16],
    heightmap_dirty: bool,
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
            sections_rendered_on: [0; 16],
            biomes: [0; 16 * 16],
            heightmap: [0; 16 * 16],
            heightmap_dirty: true,
        }
    }

    fn calculate_heightmap(&mut self) {
        for x in 0 .. 16 {
            for z in 0 .. 16 {
                let idx = ((z<<4)|x) as usize;
                for yy in 0 .. 256 {
                    let sy = 255 - yy;
                    if let block::Block::Air{..} = self.get_block(x, sy, z) {
                        continue
                    }
                    self.heightmap[idx] = sy as u8;
                    break;
                }
            }
        }
        self.heightmap_dirty = true;
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, b: block::Block) {
        let s_idx = y >> 4;
        if s_idx < 0 || s_idx > 15 {
            return;
        }
        if self.sections[s_idx as usize].is_none() {
            if let block::Air {} = b {
                return;
            }
            self.sections[s_idx as usize] = Some(Section::new(self.position.0, s_idx as u8, self.position.1));
        }
        {
            let section = self.sections[s_idx as usize].as_mut().unwrap();
            section.set_block(x, y & 0xF, z, b);
        }
        let idx = ((z<<4)|x) as usize;
        if self.heightmap[idx] < y as u8 {
            self.heightmap[idx] = y as u8;
            self.heightmap_dirty = true;
        } else if self.heightmap[idx] == y as u8 {
            // Find a new lowest
            for yy in 0 .. y {
                let sy = y - yy - 1;
                if let block::Block::Air{..} = self.get_block(x, sy, z) {
                    continue
                }
                self.heightmap[idx] = sy as u8;
                break;
            }
            self.heightmap_dirty = true;
        }
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> block::Block {
        let s_idx = y >> 4;
        if s_idx < 0 || s_idx > 15 {
            return block::Missing{};
        }
        match self.sections[s_idx as usize].as_ref() {
            Some(sec) => sec.get_block(x, y & 0xF, z),
            None => block::Air{},
        }
    }

    fn get_biome(&self, x: i32, z: i32) -> biome::Biome {
        biome::Biome::by_id(self.biomes[((z<<4)|x) as usize] as usize)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct SectionKey {
    pos: (i32, u8, i32),
}

pub struct Section {
    pub cull_info: chunk_builder::CullInfo,
    pub render_buffer: render::ChunkBuffer,

    y: u8,

    blocks: bit::Map,
    block_map: Vec<(block::Block, u32)>,
    rev_block_map: HashMap<block::Block, usize, BuildHasherDefault<FNVHash>>,

    block_light: nibble::Array,
    sky_light: nibble::Array,

    dirty: bool,
    building: bool,
}

impl Section {
    fn new(_x: i32, y: u8, _z: i32) -> Section {
        let mut section = Section {
            cull_info: chunk_builder::CullInfo::all_vis(),
            render_buffer: render::ChunkBuffer::new(),
            y: y,

            blocks: bit::Map::new(4096, 4),
            block_map: vec![
                (block::Air{}, 0xFFFFFFFF)
            ],
            rev_block_map: HashMap::with_hasher(BuildHasherDefault::default()),

            block_light: nibble::Array::new(16 * 16 * 16),
            sky_light: nibble::Array::new(16 * 16 * 16),

            dirty: false,
            building: false,
        };
        for i in 0 .. 16*16*16 {
            section.sky_light.set(i, 0xF);
        }
        section.rev_block_map.insert(block::Air{}, 0);
        section
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> block::Block {
        let idx = self.blocks.get(((y << 8) | (z << 4) | x) as usize);
        self.block_map[idx].0
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, b: block::Block) {
        use std::collections::hash_map::Entry;
        let old = self.get_block(x, y, z);
        if old == b {
            return;
        }
        // Clean up old block
        {
            let idx = self.rev_block_map[&old];
            let info = &mut self.block_map[idx];
            info.1 -= 1;
            if info.1 == 0 { // None left of this type
                self.rev_block_map.remove(&old);
            }
        }

        if let Entry::Vacant(entry) = self.rev_block_map.entry(b) {
            let mut found = false;
            let id = entry.insert(self.block_map.len());
            for (i, ref mut info) in self.block_map.iter_mut().enumerate() {
                if info.1 == 0 {
                    info.0 = b;
                    *id = i;
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
                self.block_map.push((b, 0));
            }
        }

        let idx = self.rev_block_map[&b];
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
