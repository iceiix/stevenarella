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

pub use steven_blocks as block;
use steven_protocol::protocol::LenPrefixed;
use steven_protocol::protocol::Serializable;
use steven_protocol::protocol::VarInt;
use steven_protocol::types::bit;

use crate::chunk_builder;
use crate::ecs;
use crate::entity::block_entity;
use crate::format;
use crate::protocol;
use crate::render;
use crate::shared::{Direction, Position};
use crate::types::hash::FNVHash;
use crate::types::nibble;
use byteorder::ReadBytesExt;
use cgmath::prelude::*;
use flate2::read::ZlibDecoder;
use log::info;
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::hash::BuildHasherDefault;
use std::io::Cursor;
use std::io::Read;

pub mod biome;
mod storage;

#[derive(Default)]
pub struct World {
    chunks: HashMap<CPos, Chunk, BuildHasherDefault<FNVHash>>,
    min_y: i32,
    height: i32,

    render_list: Vec<(i32, i32, i32)>,

    light_updates: VecDeque<LightUpdate>,

    block_entity_actions: VecDeque<BlockEntityAction>,

    protocol_version: i32,
    pub modded_block_ids: HashMap<usize, String>,
    pub id_map: block::VanillaIDMap,
}

#[derive(Clone, Debug)]
pub enum BlockEntityAction {
    Create(Position),
    Remove(Position),
    UpdateSignText(
        Box<(
            Position,
            format::Component,
            format::Component,
            format::Component,
            format::Component,
        )>,
    ),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Block,
    Sky,
}

impl LightType {
    fn get_light(self, world: &World, pos: Position) -> u8 {
        match self {
            LightType::Block => world.get_block_light(pos),
            LightType::Sky => world.get_sky_light(pos),
        }
    }
    fn set_light(self, world: &mut World, pos: Position, light: u8) {
        match self {
            LightType::Block => world.set_block_light(pos, light),
            LightType::Sky => world.set_sky_light(pos, light),
        }
    }
}

struct LightUpdate {
    ty: LightType,
    pos: Position,
}

impl World {
    pub fn new(protocol_version: i32) -> World {
        let id_map = block::VanillaIDMap::new(protocol_version);
        World {
            protocol_version,
            id_map,
            height: 256,
            ..Default::default()
        }
    }

    pub fn is_chunk_loaded(&self, x: i32, z: i32) -> bool {
        self.chunks.contains_key(&CPos(x, z))
    }

    pub fn set_block(&mut self, pos: Position, b: block::Block) {
        if self.set_block_raw(pos, b) {
            self.update_block(pos);
        }
    }

    fn set_block_raw(&mut self, pos: Position, b: block::Block) -> bool {
        let cpos = CPos(pos.x >> 4, pos.z >> 4);
        let chunk = self.chunks.entry(cpos).or_insert_with(|| Chunk::new(cpos));
        if chunk.set_block(pos.x & 0xF, pos.y, pos.z & 0xF, b) {
            if chunk.block_entities.contains_key(&pos) {
                self.block_entity_actions
                    .push_back(BlockEntityAction::Remove(pos));
            }
            if block_entity::BlockEntityType::get_block_entity(b).is_some() {
                self.block_entity_actions
                    .push_back(BlockEntityAction::Create(pos));
            }
            true
        } else {
            false
        }
    }

    pub fn update_block(&mut self, pos: Position) {
        for yy in -1..2 {
            for zz in -1..2 {
                for xx in -1..2 {
                    let bp = pos + (xx, yy, zz);
                    let current = self.get_block(bp);
                    let new = current.update_state(self, bp);
                    if current != new {
                        self.set_block_raw(bp, new);
                    }
                    self.set_dirty(bp.x >> 4, bp.y >> 4, bp.z >> 4);
                    self.update_light(bp, LightType::Block);
                    self.update_light(bp, LightType::Sky);
                }
            }
        }
    }

    fn update_range(&mut self, x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) {
        for by in y1..y2 {
            for bz in z1..z2 {
                for bx in x1..x2 {
                    let bp = Position::new(bx, by, bz);
                    let current = self.get_block(bp);
                    let new = current.update_state(self, bp);
                    let sky_light = self.get_sky_light(bp);
                    let block_light = self.get_block_light(bp);
                    if current != new {
                        self.set_block_raw(bp, new);
                        // Restore old lighting
                        self.set_sky_light(bp, sky_light);
                        self.set_block_light(bp, block_light);
                    }
                }
            }
        }
    }

    pub fn get_block(&self, pos: Position) -> block::Block {
        match self.chunks.get(&CPos(pos.x >> 4, pos.z >> 4)) {
            Some(chunk) => chunk.get_block(pos.x & 0xF, pos.y, pos.z & 0xF),
            None => block::Missing {},
        }
    }

    fn set_block_light(&mut self, pos: Position, light: u8) {
        let cpos = CPos(pos.x >> 4, pos.z >> 4);
        let chunk = self.chunks.entry(cpos).or_insert_with(|| Chunk::new(cpos));
        chunk.set_block_light(pos.x & 0xF, pos.y, pos.z & 0xF, light);
    }

    pub fn get_block_light(&self, pos: Position) -> u8 {
        match self.chunks.get(&CPos(pos.x >> 4, pos.z >> 4)) {
            Some(chunk) => chunk.get_block_light(pos.x & 0xF, pos.y, pos.z & 0xF),
            None => 0,
        }
    }

    fn set_sky_light(&mut self, pos: Position, light: u8) {
        let cpos = CPos(pos.x >> 4, pos.z >> 4);
        let chunk = self.chunks.entry(cpos).or_insert_with(|| Chunk::new(cpos));
        chunk.set_sky_light(pos.x & 0xF, pos.y, pos.z & 0xF, light);
    }

    pub fn get_sky_light(&self, pos: Position) -> u8 {
        match self.chunks.get(&CPos(pos.x >> 4, pos.z >> 4)) {
            Some(chunk) => chunk.get_sky_light(pos.x & 0xF, pos.y, pos.z & 0xF),
            None => 15,
        }
    }

    fn update_light(&mut self, pos: Position, ty: LightType) {
        self.light_updates.push_back(LightUpdate { ty, pos });
    }

    pub fn add_block_entity_action(&mut self, action: BlockEntityAction) {
        self.block_entity_actions.push_back(action);
    }

    #[allow(clippy::verbose_bit_mask)] // "llvm generates better code" for updates_performed & 0xFFF "on x86"
    pub fn tick(&mut self, m: &mut ecs::Manager) {
        use instant::Instant;
        let start = Instant::now();
        let mut updates_performed = 0;
        while !self.light_updates.is_empty() {
            updates_performed += 1;
            self.do_light_update();
            if (updates_performed & 0xFFF == 0) && start.elapsed().subsec_nanos() >= 5000000 {
                // 5 ms for light updates
                break;
            }
        }

        let sign_info: ecs::Key<block_entity::sign::SignInfo> = m.get_key();

        while let Some(action) = self.block_entity_actions.pop_front() {
            match action {
                BlockEntityAction::Remove(pos) => {
                    if let Some(chunk) = self.chunks.get_mut(&CPos(pos.x >> 4, pos.z >> 4)) {
                        if let Some(entity) = chunk.block_entities.remove(&pos) {
                            m.remove_entity(entity);
                        }
                    }
                }
                BlockEntityAction::Create(pos) => {
                    if let Some(chunk) = self.chunks.get_mut(&CPos(pos.x >> 4, pos.z >> 4)) {
                        // Remove existing entity
                        if let Some(entity) = chunk.block_entities.remove(&pos) {
                            m.remove_entity(entity);
                        }
                        let block = chunk.get_block(pos.x & 0xF, pos.y, pos.z & 0xF);
                        if let Some(entity_type) =
                            block_entity::BlockEntityType::get_block_entity(block)
                        {
                            let entity = entity_type.create_entity(m, pos);
                            chunk.block_entities.insert(pos, entity);
                        }
                    }
                }
                BlockEntityAction::UpdateSignText(bx) => {
                    let (pos, line1, line2, line3, line4) = *bx;
                    if let Some(chunk) = self.chunks.get(&CPos(pos.x >> 4, pos.z >> 4)) {
                        if let Some(entity) = chunk.block_entities.get(&pos) {
                            if let Some(sign) = m.get_component_mut(*entity, sign_info) {
                                sign.lines = [line1, line2, line3, line4];
                                sign.dirty = true;
                            }
                        }
                    }
                }
            }
        }
    }

    fn do_light_update(&mut self) {
        use std::cmp;
        if let Some(update) = self.light_updates.pop_front() {
            if update.pos.y < 0
                || update.pos.y > 255
                || !self.is_chunk_loaded(update.pos.x >> 4, update.pos.z >> 4)
            {
                return;
            }

            let block = self.get_block(update.pos).get_material();
            // Find the brightest source of light nearby
            let mut best = update.ty.get_light(self, update.pos);
            let old = best;
            for dir in Direction::all() {
                let light = update.ty.get_light(self, update.pos.shift(dir));
                if light > best {
                    best = light;
                }
            }
            best = best.saturating_sub(cmp::max(1, block.absorbed_light));
            // If the light from the block itself is brighter than the light passing through
            // it use that.
            if update.ty == LightType::Block && block.emitted_light != 0 {
                best = cmp::max(best, block.emitted_light);
            }
            // Sky light doesn't decrease when going down at full brightness
            if update.ty == LightType::Sky
                && block.absorbed_light == 0
                && update.ty.get_light(self, update.pos.shift(Direction::Up)) == 15
            {
                best = 15;
            }

            // Nothing to do, we are already at the right value
            if best == old {
                return;
            }
            // Use our new light value
            update.ty.set_light(self, update.pos, best);
            // Flag surrounding chunks as dirty
            for yy in -1..2 {
                for zz in -1..2 {
                    for xx in -1..2 {
                        let bp = update.pos + (xx, yy, zz);
                        self.set_dirty(bp.x >> 4, bp.y >> 4, bp.z >> 4);
                    }
                }
            }

            // Update surrounding blocks
            for dir in Direction::all() {
                self.update_light(update.pos.shift(dir), update.ty);
            }
        }
    }

    pub fn copy_cloud_heightmap(&mut self, data: &mut [u8]) -> bool {
        let mut dirty = false;
        for c in self.chunks.values_mut() {
            if c.heightmap_dirty {
                dirty = true;
                c.heightmap_dirty = false;
                for xx in 0..16 {
                    for zz in 0..16 {
                        data[(((c.position.0 << 4) as usize + xx) & 0x1FF)
                            + ((((c.position.1 << 4) as usize + zz) & 0x1FF) << 9)] =
                            c.heightmap[(zz << 4) | xx];
                    }
                }
            }
        }
        dirty
    }

    pub fn compute_render_list(&mut self, renderer: &mut render::Renderer) {
        self.render_list.clear();

        let mut valid_dirs = [false; 6];
        for dir in Direction::all() {
            let (ox, oy, oz) = dir.get_offset();
            let dir_vec = cgmath::Vector3::new(ox as f32, oy as f32, oz as f32);
            valid_dirs[dir.index()] = renderer.view_vector.dot(dir_vec) > -0.9;
        }

        let start = (
            ((renderer.camera.pos.x as i32) >> 4),
            ((renderer.camera.pos.y as i32) >> 4),
            ((renderer.camera.pos.z as i32) >> 4),
        );

        let mut process_queue = VecDeque::with_capacity(self.chunks.len() * 16);
        process_queue.push_front((Direction::Invalid, start));

        while let Some((from, pos)) = process_queue.pop_front() {
            let (exists, cull) = if let Some((sec, rendered_on)) =
                self.get_render_section_mut(pos.0, pos.1, pos.2)
            {
                if *rendered_on == renderer.frame_id {
                    continue;
                }
                *rendered_on = renderer.frame_id;

                let min = cgmath::Point3::new(
                    pos.0 as f32 * 16.0,
                    -pos.1 as f32 * 16.0,
                    pos.2 as f32 * 16.0,
                );
                let bounds =
                    collision::Aabb3::new(min, min + cgmath::Vector3::new(16.0, -16.0, 16.0));
                if renderer.frustum.contains(&bounds) == collision::Relation::Out
                    && from != Direction::Invalid
                {
                    continue;
                }
                (
                    sec.is_some(),
                    sec.map_or(chunk_builder::CullInfo::all_vis(), |v| v.cull_info),
                )
            } else {
                continue;
            };

            if exists {
                self.render_list.push(pos);
            }

            for dir in Direction::all() {
                let (ox, oy, oz) = dir.get_offset();
                let opos = (pos.0 + ox, pos.1 + oy, pos.2 + oz);
                if let Some((_, rendered_on)) = self.get_render_section_mut(opos.0, opos.1, opos.2)
                {
                    if *rendered_on == renderer.frame_id {
                        continue;
                    }
                    if from == Direction::Invalid
                        || (valid_dirs[dir.index()] && cull.is_visible(from, dir))
                    {
                        process_queue.push_back((dir.opposite(), opos));
                    }
                }
            }
        }
    }

    pub fn get_render_list(&self) -> Vec<((i32, i32, i32), &render::ChunkBuffer)> {
        self.render_list
            .iter()
            .map(|v| {
                let chunk = self.chunks.get(&CPos(v.0, v.2)).unwrap();
                let sec = chunk.sections.get(&v.1).unwrap();
                (*v, &sec.render_buffer)
            })
            .collect()
    }

    pub fn get_section_mut(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Section> {
        if let Some(chunk) = self.chunks.get_mut(&CPos(x, z)) {
            if let Some(sec) = chunk.sections.get_mut(&y) {
                return Some(sec);
            }
        }
        None
    }

    fn get_render_section_mut(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<(Option<&mut Section>, &mut u32)> {
        if !(0..=15).contains(&y) {
            return None;
        }
        if let Some(chunk) = self.chunks.get_mut(&CPos(x, z)) {
            let rendered = &mut chunk.sections_rendered_on[y as usize];
            if let Some(sec) = chunk.sections.get_mut(&y) {
                return Some((Some(sec), rendered));
            }
            return Some((None, rendered));
        }
        None
    }

    pub fn get_dirty_chunk_sections(&mut self) -> Vec<(i32, i32, i32)> {
        let mut out = vec![];
        for chunk in self.chunks.values_mut() {
            for (y, sec) in &mut chunk.sections {
                if !sec.building && sec.dirty {
                    out.push((chunk.position.0, *y, chunk.position.1));
                }
            }
        }
        out
    }

    fn set_dirty(&mut self, x: i32, y: i32, z: i32) {
        if let Some(chunk) = self.chunks.get_mut(&CPos(x, z)) {
            if let Some(sec) = chunk.sections.get_mut(&y) {
                sec.dirty = true;
            }
        }
    }

    pub fn is_section_dirty(&self, pos: (i32, i32, i32)) -> bool {
        if let Some(chunk) = self.chunks.get(&CPos(pos.0, pos.2)) {
            if let Some(sec) = chunk.sections.get(&pos.1) {
                return sec.dirty && !sec.building;
            }
        }
        false
    }

    pub fn set_building_flag(&mut self, pos: (i32, i32, i32)) {
        if let Some(chunk) = self.chunks.get_mut(&CPos(pos.0, pos.2)) {
            if let Some(sec) = chunk.sections.get_mut(&pos.1) {
                sec.building = true;
                sec.dirty = false;
            }
        }
    }

    pub fn reset_building_flag(&mut self, pos: (i32, i32, i32)) {
        if let Some(chunk) = self.chunks.get_mut(&CPos(pos.0, pos.2)) {
            if let Some(section) = chunk.sections.get_mut(&pos.1) {
                section.building = false;
            }
        }
    }

    pub fn flag_dirty_all(&mut self) {
        for chunk in self.chunks.values_mut() {
            for sec in chunk.sections.values_mut() {
                sec.dirty = true;
            }
        }
    }

    pub fn capture_snapshot(&self, x: i32, y: i32, z: i32, w: i32, h: i32, d: i32) -> Snapshot {
        use std::cmp::{max, min};
        let mut snapshot = Snapshot {
            blocks: storage::BlockStorage::new_default((w * h * d) as usize, block::Missing {}),
            block_light: nibble::Array::new((w * h * d) as usize),
            sky_light: nibble::Array::new((w * h * d) as usize),
            biomes: vec![0; (w * d) as usize],

            x,
            y,
            z,
            w,
            _h: h,
            d,
        };
        for i in 0..(w * h * d) as usize {
            snapshot.sky_light.set(i, 0xF);
        }

        let cx1 = x >> 4;
        let cy1 = y >> 4;
        let cz1 = z >> 4;
        let cx2 = (x + w + 15) >> 4;
        let cy2 = (y + h + 15) >> 4;
        let cz2 = (z + d + 15) >> 4;

        for cx in cx1..cx2 {
            for cz in cz1..cz2 {
                let chunk = match self.chunks.get(&CPos(cx, cz)) {
                    Some(val) => val,
                    None => continue,
                };

                let x1 = min(16, max(0, x - (cx << 4)));
                let x2 = min(16, max(0, x + w - (cx << 4)));
                let z1 = min(16, max(0, z - (cz << 4)));
                let z2 = min(16, max(0, z + d - (cz << 4)));

                for cy in cy1..cy2 {
                    if !(0..=15).contains(&cy) {
                        continue;
                    }
                    let section = chunk.sections.get(&cy);
                    let y1 = min(16, max(0, y - (cy << 4)));
                    let y2 = min(16, max(0, y + h - (cy << 4)));

                    for yy in y1..y2 {
                        for zz in z1..z2 {
                            for xx in x1..x2 {
                                let ox = xx + (cx << 4);
                                let oy = yy + (cy << 4);
                                let oz = zz + (cz << 4);
                                match section.as_ref() {
                                    Some(sec) => {
                                        snapshot.set_block(ox, oy, oz, sec.get_block(xx, yy, zz));
                                        snapshot.set_block_light(
                                            ox,
                                            oy,
                                            oz,
                                            sec.get_block_light(xx, yy, zz),
                                        );
                                        snapshot.set_sky_light(
                                            ox,
                                            oy,
                                            oz,
                                            sec.get_sky_light(xx, yy, zz),
                                        );
                                    }
                                    None => {
                                        snapshot.set_block(ox, oy, oz, block::Air {});
                                    }
                                }
                            }
                        }
                    }
                }
                for zz in z1..z2 {
                    for xx in x1..x2 {
                        let ox = xx + (cx << 4);
                        let oz = zz + (cz << 4);
                        snapshot.set_biome(ox, oz, chunk.get_biome(xx, zz));
                    }
                }
            }
        }

        snapshot
    }

    pub fn unload_chunk(&mut self, x: i32, z: i32, m: &mut ecs::Manager) {
        if let Some(chunk) = self.chunks.remove(&CPos(x, z)) {
            for entity in chunk.block_entities.values() {
                m.remove_entity(*entity);
            }
        }
    }

    pub fn load_chunks18(
        &mut self,
        new: bool,
        skylight: bool,
        chunk_metas: &[crate::protocol::packet::ChunkMeta],
        data: Vec<u8>,
    ) -> Result<(), protocol::Error> {
        let mut data = std::io::Cursor::new(data);

        for chunk_meta in chunk_metas {
            let x = chunk_meta.x;
            let z = chunk_meta.z;
            let mask = chunk_meta.bitmask;

            self.load_chunk18(x, z, new, skylight, mask, &mut data)?;
        }
        Ok(())
    }

    fn dirty_chunks_by_bitmask(&mut self, x: i32, z: i32, mask: u64, num_sections: usize) {
        for i in 0..num_sections {
            if mask & (1 << i) == 0 {
                continue;
            }
            for pos in [
                (-1, 0, 0),
                (1, 0, 0),
                (0, -1, 0),
                (0, 1, 0),
                (0, 0, -1),
                (0, 0, 1),
            ]
            .iter()
            {
                self.flag_section_dirty(x + pos.0, i as i32 + pos.1, z + pos.2);
            }
            let i: i32 = i.try_into().unwrap();
            self.update_range(
                (x << 4) - 1,
                (i << 4) - 1,
                (z << 4) - 1,
                (x << 4) + 17,
                (i << 4) + 17,
                (z << 4) + 17,
            );
        }
    }

    pub fn load_chunk18(
        &mut self,
        x: i32,
        z: i32,
        new: bool,
        _skylight: bool,
        mask: u16,
        data: &mut std::io::Cursor<Vec<u8>>,
    ) -> Result<(), protocol::Error> {
        let cpos = CPos(x, z);
        {
            if new {
                self.chunks.insert(cpos, Chunk::new(cpos));
            } else if !self.chunks.contains_key(&cpos) {
                return Ok(());
            }
            let chunk = self.chunks.get_mut(&cpos).unwrap();

            for i in 0..16 {
                let fill_sky = Self::should_fill_sky(i, chunk, mask as u64);
                if let Entry::Vacant(e) = chunk.sections.entry(i) {
                    if !fill_sky || mask & (1 << i) != 0 {
                        e.insert(Section::new(fill_sky));
                    }
                }
                if mask & (1 << i) == 0 {
                    continue;
                }
                let section = chunk.sections.get_mut(&i).unwrap();
                section.dirty = true;

                for bi in 0..4096 {
                    let id = data.read_u16::<byteorder::LittleEndian>()?;
                    section.blocks.set(
                        bi,
                        self.id_map
                            .by_vanilla_id(id as usize, &self.modded_block_ids),
                    );

                    // Spawn block entities
                    let b = section.blocks.get(bi);
                    if block_entity::BlockEntityType::get_block_entity(b).is_some() {
                        let pos = Position::new(
                            (bi & 0xF) as i32,
                            (bi >> 8) as i32,
                            ((bi >> 4) & 0xF) as i32,
                        ) + (
                            chunk.position.0 << 4,
                            (i << 4) as i32,
                            chunk.position.1 << 4,
                        );
                        if chunk.block_entities.contains_key(&pos) {
                            self.block_entity_actions
                                .push_back(BlockEntityAction::Remove(pos))
                        }
                        self.block_entity_actions
                            .push_back(BlockEntityAction::Create(pos))
                    }
                }
            }

            for i in 0..16 {
                if mask & (1 << i) == 0 {
                    continue;
                }
                let section = chunk.sections.get_mut(&i).unwrap();

                data.read_exact(&mut section.block_light.data)?;
            }

            for i in 0..16 {
                if mask & (1 << i) == 0 {
                    continue;
                }
                let section = chunk.sections.get_mut(&i).unwrap();

                data.read_exact(&mut section.sky_light.data)?;
            }

            if new {
                data.read_exact(&mut chunk.biomes)?;
            }

            chunk.calculate_heightmap();
        }

        self.dirty_chunks_by_bitmask(x, z, mask.into(), 16);
        Ok(())
    }

    pub fn load_chunks17(
        &mut self,
        chunk_column_count: u16,
        data_length: i32,
        skylight: bool,
        data: &[u8],
    ) -> Result<(), protocol::Error> {
        let compressed_chunk_data = &data[0..data_length as usize];
        let metadata = &data[data_length as usize..];

        let mut zlib = ZlibDecoder::new(std::io::Cursor::new(compressed_chunk_data.to_vec()));
        let mut chunk_data = Vec::new();
        zlib.read_to_end(&mut chunk_data)?;

        let mut chunk_data = std::io::Cursor::new(chunk_data);

        // Chunk metadata
        let mut metadata = std::io::Cursor::new(metadata);
        for _i in 0..chunk_column_count {
            let x = metadata.read_i32::<byteorder::BigEndian>()?;
            let z = metadata.read_i32::<byteorder::BigEndian>()?;
            let mask = metadata.read_u16::<byteorder::BigEndian>()?;
            let mask_add = metadata.read_u16::<byteorder::BigEndian>()?;

            let new = true;

            self.load_uncompressed_chunk17(x, z, new, skylight, mask, mask_add, &mut chunk_data)?;
        }

        Ok(())
    }

    pub fn load_chunk17(
        &mut self,
        x: i32,
        z: i32,
        new: bool,
        mask: u16,
        mask_add: u16,
        compressed_data: Vec<u8>,
    ) -> Result<(), protocol::Error> {
        let mut zlib = ZlibDecoder::new(std::io::Cursor::new(compressed_data.to_vec()));
        let mut data = Vec::new();
        zlib.read_to_end(&mut data)?;

        let skylight = true;
        self.load_uncompressed_chunk17(
            x,
            z,
            new,
            skylight,
            mask,
            mask_add,
            &mut std::io::Cursor::new(data),
        )
    }

    #[allow(clippy::needless_range_loop)]
    fn load_uncompressed_chunk17(
        &mut self,
        x: i32,
        z: i32,
        new: bool,
        skylight: bool,
        mask: u16,
        mask_add: u16,
        data: &mut std::io::Cursor<Vec<u8>>,
    ) -> Result<(), protocol::Error> {
        let cpos = CPos(x, z);
        {
            if new {
                self.chunks.insert(cpos, Chunk::new(cpos));
            } else if !self.chunks.contains_key(&cpos) {
                return Ok(());
            }
            let chunk = self.chunks.get_mut(&cpos).unwrap();

            // Block type array - whole byte per block
            let mut block_types = [[0u8; 4096]; 16];
            for i in 0..16 {
                let fill_sky = Self::should_fill_sky(i, chunk, mask as u64);
                if let Entry::Vacant(e) = chunk.sections.entry(i) {
                    if !fill_sky || mask & (1 << i) != 0 {
                        e.insert(Section::new(fill_sky));
                    }
                }
                if mask & (1 << i) == 0 {
                    continue;
                }
                let section = chunk.sections.get_mut(&i).unwrap();
                section.dirty = true;

                data.read_exact(&mut block_types[i as usize])?;
            }

            // Block metadata array - half byte per block
            let mut block_meta: [nibble::Array; 16] = [
                // TODO: cleanup this initialization
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
            ];

            for i in 0..16 {
                if mask & (1 << i) == 0 {
                    continue;
                }

                data.read_exact(&mut block_meta[i].data)?;
            }

            // Block light array - half byte per block
            for i in 0..16 {
                if mask & (1 << i) == 0 {
                    continue;
                }
                let section = chunk.sections.get_mut(&i).unwrap();

                data.read_exact(&mut section.block_light.data)?;
            }

            // Sky light array - half byte per block - only if 'skylight' is true
            if skylight {
                for i in 0..16 {
                    if mask & (1 << i) == 0 {
                        continue;
                    }
                    let section = chunk.sections.get_mut(&i).unwrap();

                    data.read_exact(&mut section.sky_light.data)?;
                }
            }

            // Add array - half byte per block - uses secondary bitmask
            let mut block_add: [nibble::Array; 16] = [
                // TODO: cleanup this initialization
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
                nibble::Array::new(16 * 16 * 16),
            ];

            for i in 0..16 {
                if mask_add & (1 << i) == 0 {
                    continue;
                }
                data.read_exact(&mut block_add[i].data)?;
            }

            // Now that we have the block types, metadata, and add, combine to initialize the blocks
            for i in 0..16 {
                if mask & (1 << i) == 0 {
                    continue;
                }

                let section = chunk.sections.get_mut(&(i as i32)).unwrap();

                for bi in 0..4096 {
                    let id = ((block_add[i].get(bi) as u16) << 12)
                        | ((block_types[i][bi] as u16) << 4)
                        | (block_meta[i].get(bi) as u16);
                    section.blocks.set(
                        bi,
                        self.id_map
                            .by_vanilla_id(id as usize, &self.modded_block_ids),
                    );

                    // Spawn block entities
                    let b = section.blocks.get(bi);
                    if block_entity::BlockEntityType::get_block_entity(b).is_some() {
                        let pos = Position::new(
                            (bi & 0xF) as i32,
                            (bi >> 8) as i32,
                            ((bi >> 4) & 0xF) as i32,
                        ) + (
                            chunk.position.0 << 4,
                            (i << 4) as i32,
                            chunk.position.1 << 4,
                        );
                        if chunk.block_entities.contains_key(&pos) {
                            self.block_entity_actions
                                .push_back(BlockEntityAction::Remove(pos))
                        }
                        self.block_entity_actions
                            .push_back(BlockEntityAction::Create(pos))
                    }
                }
            }

            if new {
                data.read_exact(&mut chunk.biomes)?;
            }

            chunk.calculate_heightmap();
        }

        self.dirty_chunks_by_bitmask(x, z, mask.into(), 16);
        Ok(())
    }

    pub fn load_chunk19(
        &mut self,
        x: i32,
        z: i32,
        new: bool,
        mask: u16,
        data: Vec<u8>,
    ) -> Result<(), protocol::Error> {
        self.load_chunk19_to_118(true, x, z, new, mask.into(), 16, data)
    }

    pub fn load_chunk115(
        &mut self,
        x: i32,
        z: i32,
        new: bool,
        mask: u16,
        data: Vec<u8>,
    ) -> Result<(), protocol::Error> {
        self.load_chunk19_to_118(false, x, z, new, mask.into(), 16, data)
    }

    pub fn load_chunk117(
        &mut self,
        x: i32,
        z: i32,
        new: bool,
        mask: u64,
        data: Vec<u8>,
    ) -> Result<(), protocol::Error> {
        let num_sections = (self.height >> 4) as usize;
        self.load_chunk19_to_118(false, x, z, new, mask, num_sections, data)
    }

    pub fn load_chunk118(
        &mut self,
        x: i32,
        z: i32,
        new: bool,
        data: Vec<u8>,
    ) -> Result<(), protocol::Error> {
        let num_sections = (self.height >> 4) as usize;
        let mut mask = 0;
        for _ in 0..num_sections {
            mask = (mask << 1) | 1;
        }
        self.load_chunk19_to_118(false, x, z, new, mask, num_sections, data)
    }

    pub fn load_dimension_type(&mut self, dimension_tags: Option<crate::nbt::NamedTag>) {
        if let Some(crate::nbt::NamedTag(_, crate::nbt::Tag::Compound(tags))) = dimension_tags {
            self.load_dimension_type_tags(&tags);
        }
    }

    pub fn load_dimension_type_tags(&mut self, tags: &HashMap<String, crate::nbt::Tag>) {
        info!("Dimension type: {:?}", tags);

        if let Some(crate::nbt::Tag::Int(min_y)) = tags.get("min_y") {
            self.min_y = *min_y;
        }

        if let Some(crate::nbt::Tag::Int(height)) = tags.get("height") {
            self.height = *height;
        }
        // TODO: More tags https://wiki.vg/Protocol#Login_.28play.29
    }

    #[allow(clippy::or_fun_call)]
    fn load_chunk19_to_118(
        &mut self,
        read_biomes: bool,
        x: i32,
        z: i32,
        new: bool,
        mask: u64,
        num_sections: usize,
        data: Vec<u8>,
    ) -> Result<(), protocol::Error> {
        let mut data = Cursor::new(data);

        let cpos = CPos(x, z);
        {
            if new {
                self.chunks.insert(cpos, Chunk::new(cpos));
            } else if !self.chunks.contains_key(&cpos) {
                return Ok(());
            }
            let chunk = self.chunks.get_mut(&cpos).unwrap();

            for i1 in 0..num_sections as i32 {
                // Convert the section index to the chunk section's position,
                // including negative values.
                let i: i32 = (i1 as i32) + (self.min_y >> 4);

                // Skip this chunk section if not in the mask bitmap.
                if mask & (1 << i1) == 0 {
                    continue;
                }

                // Populate the section in this chunk if not already present.
                let fill_sky = Self::should_fill_sky(i1, chunk, mask as u64);
                let section = chunk
                    .sections
                    .entry(i)
                    .or_insert_with(|| Section::new(fill_sky));
                section.dirty = true;

                if self.protocol_version >= 451 {
                    let _block_count = data.read_u16::<byteorder::LittleEndian>()?;
                    // TODO: use block_count, "The client will keep count of the blocks as they are
                    // broken and placed, and, if the block count reaches 0, the whole chunk
                    // section is not rendered, even if it still has blocks." per https://wiki.vg/Chunk_Format#Data_structure
                }

                let palette =
                    PaletteParser::new(self.protocol_version, PaletteKind::BlockStates, &mut data)
                        .parse()?;
                SectionParser::new(
                    self.protocol_version,
                    palette,
                    &self.id_map,
                    &self.modded_block_ids,
                    &mut data,
                    section,
                )
                .parse()?;

                for bi in 0..4096 {
                    // Spawn block entities
                    let b = section.blocks.get(bi);
                    if block_entity::BlockEntityType::get_block_entity(b).is_some() {
                        let pos = Position::new(
                            (bi & 0xF) as i32,
                            (bi >> 8) as i32,
                            ((bi >> 4) & 0xF) as i32,
                        ) + (
                            chunk.position.0 << 4,
                            (i << 4) as i32,
                            chunk.position.1 << 4,
                        );
                        if chunk.block_entities.contains_key(&pos) {
                            self.block_entity_actions
                                .push_back(BlockEntityAction::Remove(pos))
                        }
                        self.block_entity_actions
                            .push_back(BlockEntityAction::Create(pos))
                    }
                }

                // Version 1.18+
                if self.protocol_version >= 757 {
                    let _palette =
                        PaletteParser::new(self.protocol_version, PaletteKind::Biomes, &mut data)
                            .parse()?;
                    let _bits = LenPrefixed::<VarInt, u64>::read_from(&mut data)?.data;
                    // TODO: Use biome data
                    // Version 1.14 - 1.17
                } else if self.protocol_version >= 451 {
                    // Skylight in update skylight packet for 1.14+
                } else {
                    data.read_exact(&mut section.block_light.data)?;
                    data.read_exact(&mut section.sky_light.data)?;
                }
            }

            if read_biomes && new {
                if self.protocol_version > 340 {
                    for i in 0..256 {
                        chunk.biomes[i] = data.read_i32::<byteorder::BigEndian>()? as u8;
                    }
                } else {
                    data.read_exact(&mut chunk.biomes)?;
                }
            }

            chunk.calculate_heightmap();
        }

        self.dirty_chunks_by_bitmask(x, z, mask, num_sections);
        let mut remaining = Vec::new();
        data.read_to_end(&mut remaining)?;

        // The rest of the chunk data might be padded out with zeros.
        // See https://bugs.mojang.com/browse/MC-131684.
        assert!(
            remaining.iter().all(|b| *b == 0),
            "Failed to read all chunk data, had {} bytes left",
            remaining.len()
        );
        Ok(())
    }

    fn flag_section_dirty(&mut self, x: i32, y: i32, z: i32) {
        if !(0..=15).contains(&y) {
            return;
        }
        let cpos = CPos(x, z);
        if let Some(chunk) = self.chunks.get_mut(&cpos) {
            if let Some(sec) = chunk.sections.get_mut(&y) {
                sec.dirty = true;
            }
        }
    }

    /// Determine if we should fill the sky in this section based on already
    /// existing chunks, and if we are expecting more chunks in the current
    /// chunk data.
    fn should_fill_sky(s_idx: i32, chunk: &Chunk, mask: u64) -> bool {
        // Check if the chunk already contains previously loaded sections
        // above this one.
        if chunk.sections.keys().any(|s_idx2| *s_idx2 > s_idx) {
            return false;
        }

        // Check if the chunk data we are currently reading is going to load
        // another section above this one.
        let above_mask = !((1 << s_idx) | ((1 << s_idx) - 1));
        (mask & above_mask) == 0
    }

    pub fn set_light_data(
        &mut self,
        x: i32,
        z: i32,
        light_type: LightType,
        masks: Vec<i64>,
        data: Vec<LenPrefixed<VarInt, u8>>,
    ) {
        let cpos = CPos(x, z);
        let chunk = self.chunks.entry(cpos).or_insert_with(|| Chunk::new(cpos));
        let mut data = data.iter();

        for (i, mask) in masks.iter().enumerate() {
            for j in 0..64 {
                if mask & (1 << j) != 0 {
                    let new_light = &data.next().unwrap().data;
                    let s_idx = i as i32 * 64 + j + self.min_y - 1;
                    let section = chunk
                        .sections
                        .entry(s_idx)
                        .or_insert_with(|| Section::new(false));
                    let current_light = match light_type {
                        LightType::Block => &mut section.block_light,
                        LightType::Sky => &mut section.sky_light,
                    };
                    current_light.data.copy_from_slice(new_light);
                }
            }
        }
    }

    pub fn clear_light_data(&mut self, x: i32, z: i32, light_type: LightType, masks: Vec<i64>) {
        let chunk = match self.chunks.get_mut(&CPos(x, z)) {
            Some(c) => c,
            None => return,
        };

        for (i, mask) in masks.iter().enumerate() {
            for j in 0..64 {
                if mask & (1 << j) != 0 {
                    let s_idx = i as i32 * 64 + j + self.min_y - 1;
                    let section = match chunk.sections.get_mut(&s_idx) {
                        Some(s) => s,
                        None => return,
                    };
                    let current_light = match light_type {
                        LightType::Block => &mut section.block_light,
                        LightType::Sky => &mut section.sky_light,
                    };
                    current_light.data.copy_from_slice(&[0u8; 2048]);
                }
            }
        }
    }
}

impl block::WorldAccess for World {
    fn get_block(&self, pos: Position) -> block::Block {
        World::get_block(self, pos)
    }
}

pub struct Snapshot {
    blocks: storage::BlockStorage,
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
        self.blocks.get(self.index(x, y, z))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, b: block::Block) {
        let idx = self.index(x, y, z);
        self.blocks.set(idx, b);
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
        biome::Biome::by_id(self.biomes[((x - self.x) + ((z - self.z) * self.w)) as usize] as usize)
    }

    pub fn set_biome(&mut self, x: i32, z: i32, b: biome::Biome) {
        self.biomes[((x - self.x) + ((z - self.z) * self.w)) as usize] = b.id as u8;
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

    sections: HashMap<i32, Section>,
    sections_rendered_on: [u32; 16],
    biomes: [u8; 16 * 16],

    heightmap: [u8; 16 * 16],
    heightmap_dirty: bool,

    block_entities: HashMap<Position, ecs::Entity, BuildHasherDefault<FNVHash>>,
}

impl Chunk {
    fn new(pos: CPos) -> Chunk {
        Chunk {
            position: pos,
            sections: HashMap::new(),
            sections_rendered_on: [0; 16],
            biomes: [0; 16 * 16],
            heightmap: [0; 16 * 16],
            heightmap_dirty: true,
            block_entities: HashMap::with_hasher(BuildHasherDefault::default()),
        }
    }

    fn calculate_heightmap(&mut self) {
        for x in 0..16 {
            for z in 0..16 {
                let idx = ((z << 4) | x) as usize;
                for yy in 0..256 {
                    let sy = 255 - yy;
                    if let block::Air { .. } = self.get_block(x, sy, z) {
                        continue;
                    }
                    self.heightmap[idx] = sy as u8;
                    break;
                }
            }
        }
        self.heightmap_dirty = true;
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, b: block::Block) -> bool {
        let s_idx = y >> 4;
        if !(0..=15).contains(&s_idx) {
            return false;
        }
        if !self.sections.contains_key(&s_idx) {
            if let block::Air {} = b {
                return false;
            }
            let fill_sky = self.sections.keys().any(|s_idx2| *s_idx2 > s_idx);
            self.sections.insert(s_idx, Section::new(fill_sky));
        }
        {
            let section = self.sections.get_mut(&s_idx).unwrap();
            if !section.set_block(x, y & 0xF, z, b) {
                return false;
            }
        }
        let idx = ((z << 4) | x) as usize;
        match self.heightmap[idx].cmp(&(y as u8)) {
            Ordering::Less => {
                self.heightmap[idx] = y as u8;
                self.heightmap_dirty = true;
            }
            Ordering::Equal => {
                // Find a new lowest
                for yy in 0..y {
                    let sy = y - yy - 1;
                    if let block::Air { .. } = self.get_block(x, sy, z) {
                        continue;
                    }
                    self.heightmap[idx] = sy as u8;
                    break;
                }
                self.heightmap_dirty = true;
            }
            Ordering::Greater => (),
        }
        true
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> block::Block {
        let s_idx = y >> 4;
        if !(0..=15).contains(&s_idx) {
            return block::Missing {};
        }
        match self.sections.get(&s_idx) {
            Some(sec) => sec.get_block(x, y & 0xF, z),
            None => block::Air {},
        }
    }

    fn get_block_light(&self, x: i32, y: i32, z: i32) -> u8 {
        let s_idx = y >> 4;
        if !(0..=15).contains(&s_idx) {
            return 0;
        }
        match self.sections.get(&s_idx) {
            Some(sec) => sec.get_block_light(x, y & 0xF, z),
            None => 0,
        }
    }

    fn set_block_light(&mut self, x: i32, y: i32, z: i32, light: u8) {
        let s_idx = y >> 4;
        if !(0..=15).contains(&s_idx) {
            return;
        }
        if !self.sections.contains_key(&s_idx) {
            if light == 0 {
                return;
            }
            let fill_sky = self.sections.keys().any(|s_idx2| *s_idx2 > s_idx);
            self.sections.insert(s_idx, Section::new(fill_sky));
        }
        if let Some(sec) = self.sections.get_mut(&s_idx) {
            sec.set_block_light(x, y & 0xF, z, light)
        }
    }

    fn get_sky_light(&self, x: i32, y: i32, z: i32) -> u8 {
        let s_idx = y >> 4;
        if !(0..=15).contains(&s_idx) {
            return 15;
        }
        match self.sections.get(&s_idx) {
            Some(sec) => sec.get_sky_light(x, y & 0xF, z),
            None => 15,
        }
    }

    fn set_sky_light(&mut self, x: i32, y: i32, z: i32, light: u8) {
        let s_idx = y >> 4;
        if !(0..=15).contains(&s_idx) {
            return;
        }
        if !self.sections.contains_key(&s_idx) {
            if light == 15 {
                return;
            }
            let fill_sky = self.sections.keys().any(|s_idx2| *s_idx2 > s_idx);
            self.sections.insert(s_idx, Section::new(fill_sky));
        }
        if let Some(sec) = self.sections.get_mut(&s_idx) {
            sec.set_sky_light(x, y & 0xF, z, light)
        }
    }

    fn get_biome(&self, x: i32, z: i32) -> biome::Biome {
        biome::Biome::by_id(self.biomes[((z << 4) | x) as usize] as usize)
    }
}

pub struct Section {
    pub cull_info: chunk_builder::CullInfo,
    pub render_buffer: render::ChunkBuffer,

    blocks: storage::BlockStorage,

    block_light: nibble::Array,
    sky_light: nibble::Array,

    dirty: bool,
    building: bool,
}

impl Section {
    fn new(fill_sky: bool) -> Section {
        let mut section = Section {
            cull_info: chunk_builder::CullInfo::all_vis(),
            render_buffer: render::ChunkBuffer::new(),

            blocks: storage::BlockStorage::new(4096),

            block_light: nibble::Array::new(16 * 16 * 16),
            sky_light: nibble::Array::new(16 * 16 * 16),

            dirty: false,
            building: false,
        };
        if fill_sky {
            for i in 0..16 * 16 * 16 {
                section.sky_light.set(i, 0xF);
            }
        }
        section
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> block::Block {
        self.blocks.get(((y << 8) | (z << 4) | x) as usize)
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, b: block::Block) -> bool {
        if self.blocks.set(((y << 8) | (z << 4) | x) as usize, b) {
            self.dirty = true;
            self.set_sky_light(x, y, z, 0);
            self.set_block_light(x, y, z, 0);
            true
        } else {
            false
        }
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

/// The kind of palette we are reading. This can affect how we interpret bits
/// per entry which is different between block states and biomes.
#[derive(PartialEq)]
enum PaletteKind {
    BlockStates,
    Biomes,
}

/// These are the different formats the palette data can be interpreted as.
/// See https://wiki.vg/Chunk_Format#Palettes for more info.
enum PaletteFormat {
    SingleValued(usize),
    Indirect(Vec<usize>, u8),
    Direct(u8),
}

/// A struct to manage building a palette that can be used to resolve block
/// states or biomes inside a chunk section.
struct PaletteParser<'a> {
    protocol_version: i32,
    kind: PaletteKind,
    data: &'a mut Cursor<Vec<u8>>,
}

impl PaletteParser<'_> {
    pub fn new(
        protocol_version: i32,
        kind: PaletteKind,
        data: &mut Cursor<Vec<u8>>,
    ) -> PaletteParser<'_> {
        if protocol_version < 757 && kind == PaletteKind::Biomes {
            panic!(
                "Protocol {} doesn't support biome palettes",
                protocol_version
            );
        }

        PaletteParser {
            protocol_version,
            kind,
            data,
        }
    }

    fn parse_single_valued_palette(&mut self) -> Result<PaletteFormat, protocol::Error> {
        Ok(PaletteFormat::SingleValued(
            VarInt::read_from(self.data)?.0 as usize,
        ))
    }

    fn parse_indirect_palette(
        &mut self,
        bits_per_entry: u8,
    ) -> Result<PaletteFormat, protocol::Error> {
        let count = VarInt::read_from(self.data)?.0 as usize;
        let mut mapping = Vec::with_capacity(count);

        for _i in 0..count {
            mapping.push(VarInt::read_from(self.data)?.0 as usize);
        }

        Ok(PaletteFormat::Indirect(mapping, bits_per_entry))
    }

    pub fn parse(mut self) -> Result<PaletteFormat, protocol::Error> {
        let mut bits_per_entry = self.data.read_u8()?;

        // Pre 1.18, when bits_per_entry == 0, it indicates we should use
        // an indirect palette rather than the new single valued one. We are
        // setting this to 4 since it's the minimum value for indirect
        // palettes.
        if self.protocol_version < 757 && bits_per_entry == 0 {
            bits_per_entry = 4;
        }

        // Figure out how we should interpret the palette based on bits_per_entry.
        Ok(match self.kind {
            PaletteKind::BlockStates => match bits_per_entry {
                0 => self.parse_single_valued_palette()?,
                n if (1..9).contains(&n) => self.parse_indirect_palette(n.max(4))?,
                n if (9..17).contains(&n) => PaletteFormat::Direct(n),
                // https://wiki.vg/Chunk_Format#Data_structure "This increase can go up to 16 bits per block"...
                n => panic!(
                    "PaletteParser::parse: block state bits_per_entry={} > 16",
                    n
                ),
            },
            PaletteKind::Biomes => match bits_per_entry {
                0 => self.parse_single_valued_palette()?,
                n if (1..4).contains(&n) => self.parse_indirect_palette(n)?,
                n if (4..17).contains(&n) => PaletteFormat::Direct(n),
                n => panic!("PaletteParser::parse: biome bits_per_entry={} > 16", n),
            },
        })
    }
}

/// A struct to manage building a chunk section struct from a palette and the
/// data received from the server.
struct SectionParser<'a> {
    protocol_version: i32,
    palette: PaletteFormat,
    id_map: &'a block::VanillaIDMap,
    modded_block_ids: &'a HashMap<usize, String>,
    data: &'a mut Cursor<Vec<u8>>,
    section: &'a mut Section,
}

impl SectionParser<'_> {
    pub fn new<'a>(
        protocol_version: i32,
        palette: PaletteFormat,
        id_map: &'a block::VanillaIDMap,
        modded_block_ids: &'a HashMap<usize, String>,
        data: &'a mut Cursor<Vec<u8>>,
        section: &'a mut Section,
    ) -> SectionParser<'a> {
        SectionParser {
            protocol_version,
            palette,
            id_map,
            modded_block_ids,
            data,
            section,
        }
    }

    pub fn parse(self) -> Result<(), protocol::Error> {
        let bits = LenPrefixed::<VarInt, u64>::read_from(self.data)?.data;
        let padded = self.protocol_version >= 735;

        match self.palette {
            PaletteFormat::SingleValued(id) => {
                let block = self.id_map.by_vanilla_id(id, self.modded_block_ids);
                for i in 0..4096 {
                    self.section.blocks.set(i, block);
                }
            }
            PaletteFormat::Indirect(mapping, bits_per_entry) => {
                let entries = bit::Map::from_raw(bits, bits_per_entry as usize, padded);
                for i in 0..4096 {
                    let index = entries.get(i);
                    let id = *mapping.get(index).unwrap();
                    let block = self.id_map.by_vanilla_id(id, self.modded_block_ids);
                    self.section.blocks.set(i, block);
                }
            }
            PaletteFormat::Direct(bits_per_entry) => {
                let entries = bit::Map::from_raw(bits, bits_per_entry as usize, padded);
                for i in 0..4096 {
                    let id = entries.get(i);
                    let block = self.id_map.by_vanilla_id(id, self.modded_block_ids);
                    self.section.blocks.set(i, block);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chunk_1_12_2() {
        let mut world = World::new(340);
        let chunk_data = std::fs::read("test/chunk_1.12.2.bin").unwrap();
        world
            .load_chunk19_to_118(true, 7, 8, true, 63, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_13_2() {
        let mut world = World::new(404);
        let chunk_data = std::fs::read("test/chunk_1.13.2.bin").unwrap();
        world
            .load_chunk19_to_118(true, -20, -7, true, 31, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_18w50a() {
        let mut world = World::new(451);
        let chunk_data = std::fs::read("test/chunk_18w50a.bin").unwrap();
        world
            .load_chunk19_to_118(true, -25, -18, true, 31, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_19w02a() {
        let mut world = World::new(452);
        let chunk_data = std::fs::read("test/chunk_19w02a.bin").unwrap();
        world
            .load_chunk19_to_118(true, -10, -26, true, 15, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_14() {
        let mut world = World::new(477);
        let chunk_data = std::fs::read("test/chunk_1.14.bin").unwrap();
        world
            .load_chunk19_to_118(true, -14, 0, true, 63, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_14_1() {
        let mut world = World::new(480);
        let chunk_data = std::fs::read("test/chunk_1.14.1.bin").unwrap();
        world
            .load_chunk19_to_118(true, 2, -25, true, 31, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_14_2() {
        let mut world = World::new(485);
        let chunk_data = std::fs::read("test/chunk_1.14.2.bin").unwrap();
        world
            .load_chunk19_to_118(true, 1, 5, true, 15, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_14_3() {
        let mut world = World::new(490);
        let chunk_data = std::fs::read("test/chunk_1.14.3.bin").unwrap();
        world
            .load_chunk19_to_118(true, -9, -25, true, 31, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_14_4() {
        let mut world = World::new(498);
        let chunk_data = std::fs::read("test/chunk_1.14.4.bin").unwrap();
        world
            .load_chunk19_to_118(true, 2, -14, true, 31, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_15_1() {
        let mut world = World::new(575);
        let chunk_data = std::fs::read("test/chunk_1.15.1.bin").unwrap();
        world
            .load_chunk19_to_118(false, -10, -10, true, 63, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_15_2() {
        let mut world = World::new(578);
        let chunk_data = std::fs::read("test/chunk_1.15.2.bin").unwrap();
        world
            .load_chunk19_to_118(false, -19, -18, true, 31, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_16() {
        let mut world = World::new(735);
        let chunk_data = std::fs::read("test/chunk_1.16.bin").unwrap();
        world
            .load_chunk19_to_118(false, 2, -26, true, 63, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_16_1() {
        let mut world = World::new(736);
        let chunk_data = std::fs::read("test/chunk_1.16.1.bin").unwrap();
        world
            .load_chunk19_to_118(false, -6, -5, true, 31, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_16_2() {
        let mut world = World::new(751);
        let chunk_data = std::fs::read("test/chunk_1.16.2.bin").unwrap();
        world
            .load_chunk19_to_118(false, -22, -20, true, 15, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_16_3() {
        let mut world = World::new(753);
        let chunk_data = std::fs::read("test/chunk_1.16.3.bin").unwrap();
        world
            .load_chunk19_to_118(false, 4, 2, true, 63, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_16_4() {
        let mut world = World::new(754);
        let chunk_data = std::fs::read("test/chunk_1.16.4.bin").unwrap();
        world
            .load_chunk19_to_118(false, -10, -8, true, 15, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_17_1() {
        let mut world = World::new(756);
        let chunk_data = std::fs::read("test/chunk_1.17.1.bin").unwrap();
        world
            .load_chunk19_to_118(false, -3, -25, true, 31, 16, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_18_1() {
        let mut world = World::new(757);
        let chunk_data = std::fs::read("test/chunk_1.18.1.bin").unwrap();
        world
            .load_chunk19_to_118(false, -14, -5, true, 0xffffff, 24, chunk_data)
            .unwrap();
    }

    #[test]
    fn parse_chunk_1_18_2() {
        let mut world = World::new(758);
        let chunk_data = std::fs::read("test/chunk_1.18.2.bin").unwrap();
        world
            .load_chunk19_to_118(false, -10, -8, true, 0xffffff, 24, chunk_data)
            .unwrap();
    }
}
