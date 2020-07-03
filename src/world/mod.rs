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

use crate::chunk_builder;
use crate::ecs;
use crate::entity::block_entity;
use crate::format;
use crate::protocol;
use crate::render;
use crate::shared::{Direction, Position};
use crate::types::hash::FNVHash;
use crate::types::{bit, nibble};
use cgmath::prelude::*;
use flate2::read::ZlibDecoder;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::hash::BuildHasherDefault;
use std::io::Read;

pub mod biome;
mod storage;

#[derive(Default)]
pub struct World {
    chunks: HashMap<CPos, Chunk, BuildHasherDefault<FNVHash>>,

    render_list: Vec<(i32, i32, i32)>,

    light_updates: VecDeque<LightUpdate>,

    block_entity_actions: VecDeque<BlockEntityAction>,

    protocol_version: i32,
    pub modded_block_ids: HashMap<usize, String>,
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
enum LightType {
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
        World {
            protocol_version,
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
        use std::time::Instant;
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
                let sec = chunk.sections[v.1 as usize].as_ref().unwrap();
                (*v, &sec.render_buffer)
            })
            .collect()
    }

    pub fn get_section_mut(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Section> {
        if let Some(chunk) = self.chunks.get_mut(&CPos(x, z)) {
            if let Some(sec) = chunk.sections[y as usize].as_mut() {
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
        for chunk in self.chunks.values_mut() {
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
        for chunk in self.chunks.values_mut() {
            for sec in &mut chunk.sections {
                if let Some(sec) = sec.as_mut() {
                    sec.dirty = true;
                }
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
                    if cy < 0 || cy > 15 {
                        continue;
                    }
                    let section = &chunk.sections[cy as usize];
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

    fn dirty_chunks_by_bitmask(&mut self, x: i32, z: i32, mask: u16) {
        for i in 0..16 {
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
        use byteorder::ReadBytesExt;

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

            for i in 0..16 {
                if chunk.sections[i].is_none() {
                    let mut fill_sky = chunk.sections.iter().skip(i).all(|v| v.is_none());
                    fill_sky &= (mask & !((1 << i) | ((1 << i) - 1))) == 0;
                    if !fill_sky || mask & (1 << i) != 0 {
                        chunk.sections[i] = Some(Section::new(i as u8, fill_sky));
                    }
                }
                if mask & (1 << i) == 0 {
                    continue;
                }
                let section = chunk.sections[i as usize].as_mut().unwrap();
                section.dirty = true;

                for bi in 0..4096 {
                    let id = data.read_u16::<byteorder::LittleEndian>()?;
                    section.blocks.set(
                        bi,
                        block::Block::by_vanilla_id(
                            id as usize,
                            self.protocol_version,
                            &self.modded_block_ids,
                        ),
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
                let section = chunk.sections[i as usize].as_mut().unwrap();

                data.read_exact(&mut section.block_light.data)?;
            }

            for i in 0..16 {
                if mask & (1 << i) == 0 {
                    continue;
                }
                let section = chunk.sections[i as usize].as_mut().unwrap();

                data.read_exact(&mut section.sky_light.data)?;
            }

            if new {
                data.read_exact(&mut chunk.biomes)?;
            }

            chunk.calculate_heightmap();
        }

        self.dirty_chunks_by_bitmask(x, z, mask);
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
            use byteorder::ReadBytesExt;

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
            let chunk = if new {
                self.chunks.insert(cpos, Chunk::new(cpos));
                self.chunks.get_mut(&cpos).unwrap()
            } else {
                if !self.chunks.contains_key(&cpos) {
                    return Ok(());
                }
                self.chunks.get_mut(&cpos).unwrap()
            };

            // Block type array - whole byte per block
            let mut block_types = [[0u8; 4096]; 16];
            for i in 0..16 {
                if chunk.sections[i].is_none() {
                    let mut fill_sky = chunk.sections.iter().skip(i).all(|v| v.is_none());
                    fill_sky &= (mask & !((1 << i) | ((1 << i) - 1))) == 0;
                    if !fill_sky || mask & (1 << i) != 0 {
                        chunk.sections[i] = Some(Section::new(i as u8, fill_sky));
                    }
                }
                if mask & (1 << i) == 0 {
                    continue;
                }
                let section = chunk.sections[i as usize].as_mut().unwrap();
                section.dirty = true;

                data.read_exact(&mut block_types[i])?;
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
                let section = chunk.sections[i as usize].as_mut().unwrap();

                data.read_exact(&mut section.block_light.data)?;
            }

            // Sky light array - half byte per block - only if 'skylight' is true
            if skylight {
                for i in 0..16 {
                    if mask & (1 << i) == 0 {
                        continue;
                    }
                    let section = chunk.sections[i as usize].as_mut().unwrap();

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

                let section = chunk.sections[i as usize].as_mut().unwrap();

                for bi in 0..4096 {
                    let id = ((block_add[i].get(bi) as u16) << 12)
                        | ((block_types[i][bi] as u16) << 4)
                        | (block_meta[i].get(bi) as u16);
                    section.blocks.set(
                        bi,
                        block::Block::by_vanilla_id(
                            id as usize,
                            self.protocol_version,
                            &self.modded_block_ids,
                        ),
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

        self.dirty_chunks_by_bitmask(x, z, mask);
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
        self.load_chunk19_or_115(true, x, z, new, mask, data)
    }

    pub fn load_chunk115(
        &mut self,
        x: i32,
        z: i32,
        new: bool,
        mask: u16,
        data: Vec<u8>,
    ) -> Result<(), protocol::Error> {
        self.load_chunk19_or_115(false, x, z, new, mask, data)
    }

    #[allow(clippy::or_fun_call)]
    fn load_chunk19_or_115(
        &mut self,
        read_biomes: bool,
        x: i32,
        z: i32,
        new: bool,
        mask: u16,
        data: Vec<u8>,
    ) -> Result<(), protocol::Error> {
        use crate::protocol::{LenPrefixed, Serializable, VarInt};
        use byteorder::ReadBytesExt;
        use std::io::Cursor;

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

            for i in 0..16 {
                if chunk.sections[i].is_none() {
                    let mut fill_sky = chunk.sections.iter().skip(i).all(|v| v.is_none());
                    fill_sky &= (mask & !((1 << i) | ((1 << i) - 1))) == 0;
                    if !fill_sky || mask & (1 << i) != 0 {
                        chunk.sections[i] = Some(Section::new(i as u8, fill_sky));
                    }
                }
                if mask & (1 << i) == 0 {
                    continue;
                }
                let section = chunk.sections[i as usize].as_mut().unwrap();
                section.dirty = true;

                if self.protocol_version >= 451 {
                    let _block_count = data.read_u16::<byteorder::LittleEndian>()?;
                    // TODO: use block_count
                }

                let mut bit_size = data.read_u8()?;
                let mut mappings: HashMap<usize, block::Block, BuildHasherDefault<FNVHash>> =
                    HashMap::with_hasher(BuildHasherDefault::default());
                if bit_size == 0 {
                    bit_size = 13;
                } else {
                    let count = VarInt::read_from(&mut data)?.0;
                    for i in 0..count {
                        let id = VarInt::read_from(&mut data)?.0;
                        let bl = block::Block::by_vanilla_id(
                            id as usize,
                            self.protocol_version,
                            &self.modded_block_ids,
                        );
                        mappings.insert(i as usize, bl);
                    }
                }

                let bits = LenPrefixed::<VarInt, u64>::read_from(&mut data)?.data;
                let m = bit::Map::from_raw(bits, bit_size as usize);

                for bi in 0..4096 {
                    let id = m.get(bi);
                    section.blocks.set(
                        bi,
                        mappings
                            .get(&id)
                            .cloned()
                            // TODO: fix or_fun_call, but do not re-borrow self
                            .unwrap_or(block::Block::by_vanilla_id(
                                id,
                                self.protocol_version,
                                &self.modded_block_ids,
                            )),
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

                if self.protocol_version >= 451 {
                    // Skylight in update skylight packet for 1.14+
                } else {
                    data.read_exact(&mut section.block_light.data)?;
                    data.read_exact(&mut section.sky_light.data)?;
                }
            }

            if read_biomes && new {
                data.read_exact(&mut chunk.biomes)?;
            }

            chunk.calculate_heightmap();
        }

        self.dirty_chunks_by_bitmask(x, z, mask);
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

    sections: [Option<Section>; 16],
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
            sections: [
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
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
        if s_idx < 0 || s_idx > 15 {
            return false;
        }
        let s_idx = s_idx as usize;
        if self.sections[s_idx].is_none() {
            if let block::Air {} = b {
                return false;
            }
            let fill_sky = self.sections.iter().skip(s_idx).all(|v| v.is_none());
            self.sections[s_idx] = Some(Section::new(s_idx as u8, fill_sky));
        }
        {
            let section = self.sections[s_idx as usize].as_mut().unwrap();
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
        if s_idx < 0 || s_idx > 15 {
            return block::Missing {};
        }
        match self.sections[s_idx as usize].as_ref() {
            Some(sec) => sec.get_block(x, y & 0xF, z),
            None => block::Air {},
        }
    }

    fn get_block_light(&self, x: i32, y: i32, z: i32) -> u8 {
        let s_idx = y >> 4;
        if s_idx < 0 || s_idx > 15 {
            return 0;
        }
        match self.sections[s_idx as usize].as_ref() {
            Some(sec) => sec.get_block_light(x, y & 0xF, z),
            None => 0,
        }
    }

    fn set_block_light(&mut self, x: i32, y: i32, z: i32, light: u8) {
        let s_idx = y >> 4;
        if s_idx < 0 || s_idx > 15 {
            return;
        }
        let s_idx = s_idx as usize;
        if self.sections[s_idx].is_none() {
            if light == 0 {
                return;
            }
            let fill_sky = self.sections.iter().skip(s_idx).all(|v| v.is_none());
            self.sections[s_idx] = Some(Section::new(s_idx as u8, fill_sky));
        }
        if let Some(sec) = self.sections[s_idx].as_mut() {
            sec.set_block_light(x, y & 0xF, z, light)
        }
    }

    fn get_sky_light(&self, x: i32, y: i32, z: i32) -> u8 {
        let s_idx = y >> 4;
        if s_idx < 0 || s_idx > 15 {
            return 15;
        }
        match self.sections[s_idx as usize].as_ref() {
            Some(sec) => sec.get_sky_light(x, y & 0xF, z),
            None => 15,
        }
    }

    fn set_sky_light(&mut self, x: i32, y: i32, z: i32, light: u8) {
        let s_idx = y >> 4;
        if s_idx < 0 || s_idx > 15 {
            return;
        }
        let s_idx = s_idx as usize;
        if self.sections[s_idx].is_none() {
            if light == 15 {
                return;
            }
            let fill_sky = self.sections.iter().skip(s_idx).all(|v| v.is_none());
            self.sections[s_idx] = Some(Section::new(s_idx as u8, fill_sky));
        }
        if let Some(sec) = self.sections[s_idx as usize].as_mut() {
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

    y: u8,

    blocks: storage::BlockStorage,

    block_light: nibble::Array,
    sky_light: nibble::Array,

    dirty: bool,
    building: bool,
}

impl Section {
    fn new(y: u8, fill_sky: bool) -> Section {
        let mut section = Section {
            cull_info: chunk_builder::CullInfo::all_vis(),
            render_buffer: render::ChunkBuffer::new(),
            y,

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
