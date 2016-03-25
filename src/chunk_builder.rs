
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::io::Write;
use byteorder::{WriteBytesExt, NativeEndian};
use world;
use world::block;
use render;
use resources;
use model;
use types::bit::Set;

const NUM_WORKERS: usize = 8;

pub struct ChunkBuilder {
    threads: Vec<(mpsc::Sender<BuildReq>, thread::JoinHandle<()>)>,
    free_builders: Vec<(usize, Vec<u8>, Vec<u8>)>,
    built_recv: mpsc::Receiver<(usize, BuildReply)>,

    models: Arc<RwLock<model::Factory>>,
    resources: Arc<RwLock<resources::Manager>>,
    resource_version: usize,
}

impl ChunkBuilder {
    pub fn new(resources: Arc<RwLock<resources::Manager>>, textures: Arc<RwLock<render::TextureManager>>) -> ChunkBuilder {
        let models = Arc::new(RwLock::new(model::Factory::new(resources.clone(), textures)));

        let mut threads = vec![];
        let mut free = vec![];
        let (built_send, built_recv) = mpsc::channel();
        for i in 0 .. NUM_WORKERS {
            let built_send = built_send.clone();
            let (work_send, work_recv) = mpsc::channel();
            let models = models.clone();
            let id = i;
            threads.push((work_send, thread::spawn(move || build_func(id, models, work_recv, built_send))));
            free.push((i, vec![], vec![]));
        }
        ChunkBuilder {
            threads: threads,
            free_builders: free,
            built_recv: built_recv,
            models: models,
            resources: resources.clone(),
            resource_version: 0xFFFF,
        }
    }

    pub fn tick(&mut self, world: &mut world::World, renderer: &mut render::Renderer, _delta: f64) {
        {
            let rm = self.resources.read().unwrap();
            if rm.version() != self.resource_version {
                self.resource_version = rm.version();
                self.models.write().unwrap().version_change();
            }
        }

        while let Ok((id, mut val)) = self.built_recv.try_recv() {
            world.reset_building_flag(val.position);

            if let Some(sec) = world.get_section_mut(val.position.0, val.position.1, val.position.2) {
                sec.cull_info = val.cull_info;
                renderer.update_chunk_solid(&mut sec.render_buffer, &val.solid_buffer, val.solid_count);
                renderer.update_chunk_trans(&mut sec.render_buffer, &val.trans_buffer, val.trans_count);
            }

            val.solid_buffer.clear();
            val.trans_buffer.clear();
            self.free_builders.push((id, val.solid_buffer, val.trans_buffer));
        }
        if self.free_builders.is_empty() {
            return;
        }
        let dirty_sections = world.get_render_list().iter()
                .map(|v| v.0)
                .filter(|v| world.is_section_dirty(*v))
                .collect::<Vec<_>>();
        for (x,y, z) in dirty_sections {
            let t_id = self.free_builders.pop().unwrap();
            world.set_building_flag((x, y, z));
            let (cx, cy, cz) = (x << 4, y << 4, z << 4);
            let mut snapshot = world.capture_snapshot(cx - 2, cy - 2, cz - 2, 20, 20, 20);
            snapshot.make_relative(-2, -2, -2);
            self.threads[t_id.0].0.send(BuildReq {
                snapshot: snapshot,
                position: (x, y, z),
                solid_buffer: t_id.1,
                trans_buffer: t_id.2,
            }).unwrap();
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

fn build_func(id: usize, models: Arc<RwLock<model::Factory>>, work_recv: mpsc::Receiver<BuildReq>, built_send: mpsc::Sender<(usize, BuildReply)>) {
    use rand::{self, Rng, SeedableRng};
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

        let mut rng = rand::XorShiftRng::from_seed([
            position.0 as u32,
            position.1 as u32,
            position.2 as u32,
            (position.0 as u32 ^ position.2 as u32) | 1,
        ]);

        let mut solid_count = 0;
        let mut trans_count = 0;

        for y in 0 .. 16 {
            for x in 0 .. 16 {
                for z in 0 .. 16 {
                    let block = snapshot.get_block(x, y, z);
                    let mat = block.get_material();
                    if !mat.renderable {
						// Use one step of the rng so that
						// if a block is placed in an empty
						// location is variant doesn't change
                        rng.next_u32();
                        continue;
                    }

                    match block {
                        block::Block::Water{..} | block::Block::FlowingWater{..} => {
                            let tex = models.read().unwrap().textures.clone();
                            trans_count += model::liquid::render_liquid(tex, false, &snapshot, x, y, z, &mut trans_buffer);
                            continue;
                        },
                        block::Block::Lava{..} | block::Block::FlowingLava{..} => {
                            let tex = models.read().unwrap().textures.clone();
                            solid_count += model::liquid::render_liquid(tex, true, &snapshot, x, y, z, &mut solid_buffer);
                            continue;
                        },
                        _ => {},
                    }

                    let model_name = block.get_model();
                    let variant = block.get_model_variant();
                    if !mat.transparent {
                        solid_count += model::Factory::get_state_model(
                            &models, &model_name.0, &model_name.1, &variant, &mut rng,
                            &snapshot, x, y, z, &mut solid_buffer
                        );
                    } else {
                        trans_count += model::Factory::get_state_model(
                            &models, &model_name.0, &model_name.1, &variant, &mut rng,
                            &snapshot, x, y, z, &mut trans_buffer
                        );
                    }
                }
            }
        }

        let cull_info = build_cull_info(&snapshot);

        built_send.send((id, BuildReply {
            position: position,
            solid_buffer: solid_buffer,
            solid_count: solid_count,
            trans_buffer: trans_buffer,
            trans_count: trans_count,
            cull_info: cull_info,
        })).unwrap();
    }
}

fn build_cull_info(snapshot: &world::Snapshot) -> CullInfo {
    let mut visited = Set::new(16 * 16 * 16);
    let mut info = CullInfo::new();

    for y in 0 .. 16 {
        for z in 0 .. 16 {
            for x in 0 .. 16 {
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
    let idx = (x | (z << 4) | (y << 8)) as usize;
    if x < 0 || x > 15 || y < 0 || y > 15 || z < 0 || z > 15 || visited.get(idx) {
        return 0;
    }
    visited.set(idx, true);

    if snapshot.get_block(x, y, z).get_material().should_cull_against {
        return 0;
    }

    let mut touched = 0;

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
        touched |= flood_fill(snapshot, visited, x+ox, y+oy, z+oz);
    }
    touched
}

#[derive(Clone, Copy)]
pub struct CullInfo(u64);

impl CullInfo {
    pub fn new() -> CullInfo {
        CullInfo(0)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Invalid,
    Up,
    Down,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn all() -> Vec<Direction> {
        vec![
            Direction::Up, Direction::Down,
            Direction::North, Direction::South,
            Direction::West, Direction::East,
        ]
    }

    pub fn from_string(val: &str) -> Direction {
        match val {
            "up" => Direction::Up,
            "down" => Direction::Down,
            "north" => Direction::North,
            "south" => Direction::South,
            "west" => Direction::West,
            "east" => Direction::East,
            _ => Direction::Invalid,
        }
    }

    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
            _ => unreachable!(),
        }
    }

    pub fn get_verts(&self) -> &'static [BlockVertex; 4] {
        match *self {
            Direction::Up => PRECOMPUTED_VERTS[0],
            Direction::Down => PRECOMPUTED_VERTS[1],
            Direction::North => PRECOMPUTED_VERTS[2],
            Direction::South => PRECOMPUTED_VERTS[3],
            Direction::West => PRECOMPUTED_VERTS[4],
            Direction::East => PRECOMPUTED_VERTS[5],
            _ => unreachable!(),
        }
    }

    pub fn get_offset(&self) -> (i32, i32, i32) {
        match *self {
            Direction::Up => (0, 1, 0),
            Direction::Down => (0, -1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
            _ => unreachable!(),
        }
    }

    pub fn as_string(&self) -> &'static str {
        match *self {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::North => "north",
            Direction::South => "south",
            Direction::West => "west",
            Direction::East => "east",
            Direction::Invalid => "invalid",
        }
    }

    pub fn index(&self) -> usize {
        match *self {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
            _ => unreachable!(),
        }
    }
}

const PRECOMPUTED_VERTS: [&'static [BlockVertex; 4]; 6] = [
    &[ // Up
        BlockVertex::base(0.0, 1.0, 0.0, 0, 0),
        BlockVertex::base(1.0, 1.0, 0.0, 1, 0),
        BlockVertex::base(0.0, 1.0, 1.0, 0, 1),
        BlockVertex::base(1.0, 1.0, 1.0, 1, 1),
    ],
    &[ // Down
        BlockVertex::base(0.0, 0.0, 0.0, 0, 1),
        BlockVertex::base(0.0, 0.0, 1.0, 0, 0),
        BlockVertex::base(1.0, 0.0, 0.0, 1, 1),
        BlockVertex::base(1.0, 0.0, 1.0, 1, 0),
    ],
    &[ // North
        BlockVertex::base(0.0, 0.0, 0.0, 1, 1),
        BlockVertex::base(1.0, 0.0, 0.0, 0, 1),
        BlockVertex::base(0.0, 1.0, 0.0, 1, 0),
        BlockVertex::base(1.0, 1.0, 0.0, 0, 0),
    ],
    &[ // South
        BlockVertex::base(0.0, 0.0, 1.0, 0, 1),
        BlockVertex::base(0.0, 1.0, 1.0, 0, 0),
        BlockVertex::base(1.0, 0.0, 1.0, 1, 1),
        BlockVertex::base(1.0, 1.0, 1.0, 1, 0),
    ],
    &[ // West
        BlockVertex::base(0.0, 0.0, 0.0, 0, 1),
        BlockVertex::base(0.0, 1.0, 0.0, 0, 0),
        BlockVertex::base(0.0, 0.0, 1.0, 1, 1),
        BlockVertex::base(0.0, 1.0, 1.0, 1, 0),
    ],
    &[ // East
        BlockVertex::base(1.0, 0.0, 0.0, 1, 1),
        BlockVertex::base(1.0, 0.0, 1.0, 0, 1),
        BlockVertex::base(1.0, 1.0, 0.0, 1, 0),
        BlockVertex::base(1.0, 1.0, 1.0, 0, 0),
    ],
];

#[derive(Clone)]
pub struct BlockVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub tx: u16,
    pub ty: u16,
    pub tw: u16,
    pub th: u16,
    pub toffsetx: i16,
    pub toffsety: i16,
    pub tatlas: i16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub block_light: u16,
    pub sky_light: u16,
}

impl BlockVertex {
    const fn base(x: f32, y: f32, z: f32, tx: i16, ty: i16) -> BlockVertex {
        BlockVertex {
            x: x, y: y, z: z,
            tx: 0, ty: 0, tw: 0, th: 0,
            toffsetx: tx, toffsety: ty, tatlas: 0,
            r: 0, g: 0, b: 0,
            block_light: 0, sky_light: 0,
        }
    }
    pub fn write<W: Write>(&self, w: &mut W) {
        let _ = w.write_f32::<NativeEndian>(self.x);
        let _ = w.write_f32::<NativeEndian>(self.y);
        let _ = w.write_f32::<NativeEndian>(self.z);
        let _ = w.write_u16::<NativeEndian>(self.tx);
        let _ = w.write_u16::<NativeEndian>(self.ty);
        let _ = w.write_u16::<NativeEndian>(self.tw);
        let _ = w.write_u16::<NativeEndian>(self.th);
        let _ = w.write_i16::<NativeEndian>(self.toffsetx);
        let _ = w.write_i16::<NativeEndian>(self.toffsety);
        let _ = w.write_i16::<NativeEndian>(self.tatlas);
        let _ = w.write_i16::<NativeEndian>(0);
        let _ = w.write_u8(self.r);
        let _ = w.write_u8(self.g);
        let _ = w.write_u8(self.b);
        let _ = w.write_u8(255);
        let _ = w.write_u16::<NativeEndian>(self.block_light);
        let _ = w.write_u16::<NativeEndian>(self.sky_light);
        let _ = w.write_u16::<NativeEndian>(0);
        let _ = w.write_u16::<NativeEndian>(0);
    }
}
