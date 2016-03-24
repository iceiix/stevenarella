
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::io::Write;
use byteorder::{WriteBytesExt, NativeEndian};
use world;
use render;
use resources;
use model;

const NUM_WORKERS: usize = 8;

pub struct ChunkBuilder {
    threads: Vec<(mpsc::Sender<BuildReq>, thread::JoinHandle<()>)>,
    free_builders: Vec<(usize, Vec<u8>)>,
    built_recv: mpsc::Receiver<(usize, BuildReply)>,

    sections: Vec<(i32, i32, i32, Arc<world::SectionKey>)>,
    next_collection: f64,

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
            free.push((i, vec![]));
        }
        ChunkBuilder {
            threads: threads,
            free_builders: free,
            built_recv: built_recv,
            sections: vec![],
            next_collection: 0.0,
            models: models,
            resources: resources.clone(),
            resource_version: 0xFFFF,
        }
    }

    pub fn tick(&mut self, world: &mut world::World, renderer: &mut render::Renderer, delta: f64) {
        use std::cmp::Ordering;

        {
            let rm = self.resources.read().unwrap();
            if rm.version() != self.resource_version {
                self.resource_version = rm.version();
                self.models.write().unwrap().version_change();
            }
        }

        while let Ok((id, mut val)) = self.built_recv.try_recv() {
            world.reset_building_flag(val.position);

            renderer.update_chunk_solid(val.position, val.key, &val.solid_buffer, val.solid_count);

            val.solid_buffer.clear();
            self.free_builders.push((id, val.solid_buffer));
        }
        if self.free_builders.is_empty() {
            return;
        }
        self.next_collection -= delta;
        if self.next_collection <= 0.0 {
            let mut sections = world.get_dirty_chunk_sections();
            sections.sort_by(|a, b| {
                let xx = ((a.0<<4)+8) as f64 - renderer.camera.pos.x;
                let yy = ((a.1<<4)+8) as f64 - renderer.camera.pos.y;
                let zz = ((a.2<<4)+8) as f64 - renderer.camera.pos.z;
                let a_dist = xx*xx + yy*yy + zz*zz;
                let xx = ((b.0<<4)+8) as f64 - renderer.camera.pos.x;
                let yy = ((b.1<<4)+8) as f64 - renderer.camera.pos.y;
                let zz = ((b.2<<4)+8) as f64 - renderer.camera.pos.z;
                let b_dist = xx*xx + yy*yy + zz*zz;
                if a_dist == b_dist {
                    Ordering::Equal
                } else if a_dist > b_dist {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            });
            self.sections = sections;
            self.next_collection = 60.0;
        }
        while let Some((x, y, z, key)) = self.sections.pop() {
            let t_id = self.free_builders.pop().unwrap();
            world.set_building_flag((x, y, z));
            let (cx, cy, cz) = (x << 4, y << 4, z << 4);
            let mut snapshot = world.capture_snapshot(cx - 2, cy - 2, cz - 2, 20, 20, 20);
            snapshot.make_relative(-2, -2, -2);
            self.threads[t_id.0].0.send(BuildReq {
                snapshot: snapshot,
                position: (x, y, z),
                key: key,
                buffer: t_id.1,
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
    key: Arc<world::SectionKey>,
    buffer: Vec<u8>,
}

struct BuildReply {
    position: (i32, i32, i32),
    solid_buffer: Vec<u8>,
    solid_count: usize,
    key: Arc<world::SectionKey>,
}

fn build_func(id: usize, models: Arc<RwLock<model::Factory>>, work_recv: mpsc::Receiver<BuildReq>, built_send: mpsc::Sender<(usize, BuildReply)>) {
    use rand::{self, Rng, SeedableRng};
    loop {
        let BuildReq {
            snapshot,
            position,
            key,
            buffer,
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

        let mut solid_buffer = buffer;
        let mut solid_count = 0;

        for y in 0 .. 16 {
            for x in 0 .. 16 {
                for z in 0 .. 16 {
                    let block = snapshot.get_block(x, y, z);
                    if !block.get_material().renderable {
						// Use one step of the rng so that
						// if a block is placed in an empty
						// location is variant doesn't change
                        rng.next_u32();
                        continue;
                    }

                    // TODO Liquids need a special case
                    let model_name = block.get_model();
                    let variant = block.get_model_variant();
                    solid_count += model::Factory::get_state_model(
                        &models, &model_name.0, &model_name.1, &variant, &mut rng,
                        &snapshot, x, y, z, &mut solid_buffer
                    );
                }
            }
        }

        built_send.send((id, BuildReply {
            position: position,
            solid_buffer: solid_buffer,
            solid_count: solid_count,
            key: key,
        })).unwrap();
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
