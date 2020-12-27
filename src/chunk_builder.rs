use crate::model;
use crate::render;
use crate::resources;
use crate::shared::Direction;
use crate::types::bit::Set;
use crate::world;
use crate::world::block;
use rand::{self, Rng, SeedableRng};
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;

const NUM_WORKERS: usize = 8;

pub struct ChunkBuilder {
    threads: Vec<(mpsc::Sender<BuildReq>, thread::JoinHandle<()>)>,
    free_builders: Vec<(usize, Vec<u8>, Vec<u8>)>,
    built_recv: mpsc::Receiver<(usize, BuildReply)>,

    models: Arc<RwLock<model::Factory>>,
    resource_version: usize,
}

impl ChunkBuilder {
    pub fn new(
        resources: Arc<RwLock<resources::Manager>>,
        textures: Arc<RwLock<render::TextureManager>>,
    ) -> ChunkBuilder {
        let models = Arc::new(RwLock::new(model::Factory::new(resources, textures)));

        let mut threads = vec![];
        let mut free = vec![];
        let (built_send, built_recv) = mpsc::channel();
        for i in 0..NUM_WORKERS {
            let built_send = built_send.clone();
            let (work_send, work_recv) = mpsc::channel();
            let models = models.clone();
            let id = i;
            threads.push((
                work_send,
                thread::spawn(move || build_func(id, models, work_recv, built_send)),
            ));
            free.push((i, vec![], vec![]));
        }
        ChunkBuilder {
            threads,
            free_builders: free,
            built_recv,
            models,
            resource_version: 0xFFFF,
        }
    }

    pub fn tick(
        &mut self,
        world: &mut world::World,
        renderer: &mut render::Renderer,
        version: usize,
    ) {
        {
            if version != self.resource_version {
                self.resource_version = version;
                self.models.write().unwrap().version_change();
            }
        }

        while let Ok((id, mut val)) = self.built_recv.try_recv() {
            world.reset_building_flag(val.position);

            if let Some(sec) = world.get_section_mut(val.position.0, val.position.1, val.position.2)
            {
                sec.cull_info = val.cull_info;
                renderer.update_chunk_solid(
                    &mut sec.render_buffer,
                    &val.solid_buffer,
                    val.solid_count,
                );
                renderer.update_chunk_trans(
                    &mut sec.render_buffer,
                    &val.trans_buffer,
                    val.trans_count,
                );
            }

            val.solid_buffer.clear();
            val.trans_buffer.clear();
            self.free_builders
                .push((id, val.solid_buffer, val.trans_buffer));
        }
        if self.free_builders.is_empty() {
            return;
        }
        let dirty_sections = world
            .get_render_list()
            .iter()
            .map(|v| v.0)
            .filter(|v| world.is_section_dirty(*v))
            .collect::<Vec<_>>();
        for (x, y, z) in dirty_sections {
            let t_id = self.free_builders.pop().unwrap();
            world.set_building_flag((x, y, z));
            let (cx, cy, cz) = (x << 4, y << 4, z << 4);
            let mut snapshot = world.capture_snapshot(cx - 2, cy - 2, cz - 2, 20, 20, 20);
            snapshot.make_relative(-2, -2, -2);
            self.threads[t_id.0]
                .0
                .send(BuildReq {
                    snapshot,
                    position: (x, y, z),
                    solid_buffer: t_id.1,
                    trans_buffer: t_id.2,
                })
                .unwrap();
            if self.free_builders.is_empty() {
                return;
            }
        }
    }
}

struct BuildReq {
    snapshot: world::Snapshot,
    position: (i32, i32, i32),
    solid_buffer: Vec<u8>,
    trans_buffer: Vec<u8>,
}

struct BuildReply {
    position: (i32, i32, i32),
    solid_buffer: Vec<u8>,
    solid_count: usize,
    trans_buffer: Vec<u8>,
    trans_count: usize,
    cull_info: CullInfo,
}

fn build_func(
    id: usize,
    models: Arc<RwLock<model::Factory>>,
    work_recv: mpsc::Receiver<BuildReq>,
    built_send: mpsc::Sender<(usize, BuildReply)>,
) {
    loop {
        let BuildReq {
            snapshot,
            position,
            mut solid_buffer,
            mut trans_buffer,
        } = match work_recv.recv() {
            Ok(val) => val,
            Err(_) => return,
        };

        let mut rng = rand_pcg::Pcg32::from_seed([
            ((position.0 as u32) & 0xff) as u8,
            (((position.0 as u32) >> 8) & 0xff) as u8,
            (((position.0 as u32) >> 16) & 0xff) as u8,
            ((position.0 as u32) >> 24) as u8,
            ((position.1 as u32) & 0xff) as u8,
            (((position.1 as u32) >> 8) & 0xff) as u8,
            (((position.1 as u32) >> 16) & 0xff) as u8,
            ((position.1 as u32) >> 24) as u8,
            ((position.2 as u32) & 0xff) as u8,
            (((position.2 as u32) >> 8) & 0xff) as u8,
            (((position.2 as u32) >> 16) & 0xff) as u8,
            ((position.2 as u32) >> 24) as u8,
            (((position.0 as u32 ^ position.2 as u32) | 1) & 0xff) as u8,
            ((((position.0 as u32 ^ position.2 as u32) | 1) >> 8) & 0xff) as u8,
            ((((position.0 as u32 ^ position.2 as u32) | 1) >> 16) & 0xff) as u8,
            (((position.0 as u32 ^ position.2 as u32) | 1) >> 24) as u8,
        ]);

        let mut solid_count = 0;
        let mut trans_count = 0;

        for y in 0..16 {
            for x in 0..16 {
                for z in 0..16 {
                    let block = snapshot.get_block(x, y, z);
                    let mat = block.get_material();
                    if !mat.renderable {
                        // Use one step of the rng so that
                        // if a block is placed in an empty
                        // location is variant doesn't change
                        let _: u32 = rng.gen();
                        continue;
                    }

                    match block {
                        block::Block::Water { .. } | block::Block::FlowingWater { .. } => {
                            let tex = models.read().unwrap().textures.clone();
                            trans_count += model::liquid::render_liquid(
                                tex,
                                false,
                                &snapshot,
                                x,
                                y,
                                z,
                                &mut trans_buffer,
                            );
                            continue;
                        }
                        block::Block::Lava { .. } | block::Block::FlowingLava { .. } => {
                            let tex = models.read().unwrap().textures.clone();
                            solid_count += model::liquid::render_liquid(
                                tex,
                                true,
                                &snapshot,
                                x,
                                y,
                                z,
                                &mut solid_buffer,
                            );
                            continue;
                        }
                        _ => {}
                    }

                    if mat.transparent {
                        trans_count += model::Factory::get_state_model(
                            &models,
                            block,
                            &mut rng,
                            &snapshot,
                            x,
                            y,
                            z,
                            &mut trans_buffer,
                        );
                    } else {
                        solid_count += model::Factory::get_state_model(
                            &models,
                            block,
                            &mut rng,
                            &snapshot,
                            x,
                            y,
                            z,
                            &mut solid_buffer,
                        );
                    }
                }
            }
        }

        let cull_info = build_cull_info(&snapshot);

        built_send
            .send((
                id,
                BuildReply {
                    position,
                    solid_buffer,
                    solid_count,
                    trans_buffer,
                    trans_count,
                    cull_info,
                },
            ))
            .unwrap();
    }
}

fn build_cull_info(snapshot: &world::Snapshot) -> CullInfo {
    let mut visited = Set::new(16 * 16 * 16);
    let mut info = CullInfo::new();

    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                if visited.get(x | (z << 4) | (y << 8)) {
                    continue;
                }

                let touched = flood_fill(snapshot, &mut visited, x as i32, y as i32, z as i32);
                if touched == 0 {
                    continue;
                }

                for d1 in Direction::all() {
                    if (touched & (1 << d1.index())) != 0 {
                        for d2 in Direction::all() {
                            if (touched & (1 << d2.index())) != 0 {
                                info.set_visible(d1, d2);
                            }
                        }
                    }
                }
            }
        }
    }

    info
}

fn flood_fill(snapshot: &world::Snapshot, visited: &mut Set, x: i32, y: i32, z: i32) -> u8 {
    use std::collections::VecDeque;

    let mut next_position = VecDeque::with_capacity(16 * 16);
    next_position.push_back((x, y, z));

    let mut touched = 0;
    while let Some((x, y, z)) = next_position.pop_front() {
        let idx = (x | (z << 4) | (y << 8)) as usize;
        if x < 0 || x > 15 || y < 0 || y > 15 || z < 0 || z > 15 || visited.get(idx) {
            continue;
        }
        visited.set(idx, true);

        if snapshot
            .get_block(x, y, z)
            .get_material()
            .should_cull_against
        {
            continue;
        }

        if x == 0 {
            touched |= 1 << Direction::West.index();
        } else if x == 15 {
            touched |= 1 << Direction::East.index();
        }
        if y == 0 {
            touched |= 1 << Direction::Down.index();
        } else if y == 15 {
            touched |= 1 << Direction::Up.index();
        }
        if z == 0 {
            touched |= 1 << Direction::North.index();
        } else if z == 15 {
            touched |= 1 << Direction::South.index();
        }

        for d in Direction::all() {
            let (ox, oy, oz) = d.get_offset();
            next_position.push_back((x + ox, y + oy, z + oz));
        }
    }
    touched
}

#[derive(Clone, Copy, Default)]
pub struct CullInfo(u64);

impl CullInfo {
    pub fn new() -> CullInfo {
        Default::default()
    }

    pub fn all_vis() -> CullInfo {
        CullInfo(0xFFFFFFFFFFFFFFFF)
    }

    pub fn is_visible(&self, from: Direction, to: Direction) -> bool {
        (self.0 & (1 << (from.index() * 6 + to.index()))) != 0
    }

    pub fn set_visible(&mut self, from: Direction, to: Direction) {
        self.0 |= 1 << (from.index() * 6 + to.index());
    }
}
