
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::io::Write;
use byteorder::{WriteBytesExt, NativeEndian};
use world;
use render;

const NUM_WORKERS: usize = 4;

pub struct ChunkBuilder {
    threads: Vec<(mpsc::Sender<BuildReq>, thread::JoinHandle<()>)>,
    free_builders: Vec<usize>,
    built_recv: mpsc::Receiver<(usize, BuildReply)>,

    sections: Vec<(i32, i32, i32)>,
    next_collection: f64,
}

impl ChunkBuilder {
    pub fn new(textures: Arc<RwLock<render::TextureManager>>) -> ChunkBuilder {
        let mut threads = vec![];
        let mut free = vec![];
        let (built_send, built_recv) = mpsc::channel();
        for i in 0 .. NUM_WORKERS {
            let built_send = built_send.clone();
            let (work_send, work_recv) = mpsc::channel();
            let textures = textures.clone();
            let id = i;
            threads.push((work_send, thread::spawn(move || build_func(id, textures, work_recv, built_send))));
            free.push(i);
        }
        ChunkBuilder {
            threads: threads,
            free_builders: free,
            built_recv: built_recv,
            sections: vec![],
            next_collection: 0.0,
        }
    }

    pub fn wait_for_builders(&mut self) {
        while self.free_builders.len() != NUM_WORKERS {
            let (id, _) = self.built_recv.recv().unwrap();
            self.free_builders.push(id);
        }
    }

    pub fn tick(&mut self, world: &mut world::World, renderer: &mut render::Renderer, delta: f64) {
        use std::cmp::Ordering;
        while let Ok((id, val)) = self.built_recv.try_recv() {
            world.reset_building_flag(val.position);

            renderer.update_chunk_solid(val.position, &val.solid_buffer, val.solid_count);

            self.free_builders.push(id);
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
        while let Some((x, y, z)) = self.sections.pop() {
            let t_id = self.free_builders.pop().unwrap();
            world.set_building_flag((x, y, z));
            let (cx, cy, cz) = (x << 4, y << 4, z << 4);
            let mut snapshot = world.capture_snapshot(cx - 2, cy - 2, cz - 2, 20, 20, 20);
            snapshot.make_relative(-2, -2, -2);
            self.threads[t_id].0.send(BuildReq {
                snapshot: snapshot,
                position: (x, y, z),
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
}

struct BuildReply {
    position: (i32, i32, i32),
    solid_buffer: Vec<u8>,
    solid_count: usize,
}

fn build_func(id: usize, textures: Arc<RwLock<render::TextureManager>>, work_recv: mpsc::Receiver<BuildReq>, built_send: mpsc::Sender<(usize, BuildReply)>) {
    use rand::{self, Rng};
    loop {
        let BuildReq {
            snapshot,
            position,
        } = match work_recv.recv() {
            Ok(val) => val,
            Err(_) => return,
        };

        let mut solid_buffer = vec![];
        let mut solid_count = 0;

        let mut rng = rand::thread_rng();

        for y in 0 .. 16 {
            for x in 0 .. 16 {
                for z in 0 .. 16 {
                    let block = snapshot.get_block(x, y, z);
                    if !block.renderable() {
                        continue;
                    }

                    for dir in Direction::all() {

                        let offset = dir.get_offset();
                        let other = snapshot.get_block(x + offset.0, y + offset.1, z + offset.2);
                        if other.renderable() {
                            continue;
                        }

                        let (mut cr, mut cg, mut cb) = (255, 255, 255);
                        if dir == Direction::West || dir == Direction::East {
                            cr = ((cr as f64) * 0.8) as u8;
                            cg = ((cg as f64) * 0.8) as u8;
                            cb = ((cb as f64) * 0.8) as u8;
                        }

                        let stone = render::Renderer::get_texture(&textures, rng.choose(&[
                            "minecraft:blocks/lava_flow",
                            "minecraft:blocks/stone",
                            "minecraft:blocks/melon_side",
                            "minecraft:blocks/sand",
                        ]).unwrap());
                        solid_count += 6;
                        for vert in dir.get_verts() {
                            let mut vert = vert.clone();
                            // TODO
                            vert.r = cr;
                            vert.g = cg;
                            vert.b = cb;

                            vert.x += x as f32;
                            vert.y += y as f32;
                            vert.z += z as f32;

                            vert.toffsetx *= stone.get_width() as i16 * 16;
                            vert.toffsety *= stone.get_height() as i16 * 16;

                            let (bl, sl) = calculate_light(
                                &snapshot,
                                x, y, z,
                                vert.x as f64,
                                vert.y as f64,
                                vert.z as f64,
                                dir,
                                true,
                                false
                            );
                            vert.block_light = bl;
                            vert.sky_light = sl;

                            // TODO
                            vert.tatlas = stone.atlas as i16;
                            vert.tx = stone.get_x() as u16;
                            vert.ty = stone.get_y() as u16;
                            vert.tw = stone.get_width() as u16;
                            vert.th = stone.get_height() as u16;

                            vert.write(&mut solid_buffer);
                        }
                    }
                }
            }
        }

        built_send.send((id, BuildReply {
            position: position,
            solid_buffer: solid_buffer,
            solid_count: solid_count,
        })).unwrap();
    }
}

fn calculate_light(snapshot: &world::Snapshot, orig_x: i32, orig_y: i32, orig_z: i32,
                    x: f64, y: f64, z: f64, face: Direction, smooth: bool, force: bool) -> (u16, u16) {
    use std::cmp::max;
    use world::block;
    let (ox, oy, oz) = face.get_offset();
    // TODO: Cull against check

    let s_block_light = snapshot.get_block_light(orig_x + ox, orig_y + oy, orig_z + oz);
    let s_sky_light = snapshot.get_sky_light(orig_x + ox, orig_y + oy, orig_z + oz);
    if !smooth {
        return ((s_block_light as u16) * 4000, (s_sky_light as u16) * 4000);
    }

    let mut block_light = 0u32;
    let mut sky_light = 0u32;
    let mut count = 0;

    let s_block_light = max(((s_block_light as i8) - 8), 0) as u8;
    let s_sky_light = max(((s_sky_light as i8) - 8), 0) as u8;

    let dx = (ox as f64) * 0.6;
    let dy = (oy as f64) * 0.6;
    let dz = (oz as f64) * 0.6;

    for ox in [-0.6, 0.0].into_iter() {
        for oy in [-0.6, 0.0].into_iter() {
            for oz in [-0.6, 0.0].into_iter() {
                let lx = (x + ox + dx).round() as i32;
                let ly = (y + oy + dy).round() as i32;
                let lz = (z + oz + dz).round() as i32;
                let mut bl = snapshot.get_block_light(lx, ly, lz);
                let mut sl = snapshot.get_sky_light(lx, ly, lz);
                if (force && !snapshot.get_block(lx, ly, lz).in_set(&*block::AIR))
                    || (sl == 0 && bl == 0){
                    bl = s_block_light;
                    sl = s_sky_light;
                }
                block_light += bl as u32;
                sky_light += sl as u32;
                count += 1;
            }
        }
    }

    ((((block_light * 4000) / count) as u16), (((sky_light * 4000) / count) as u16))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
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

    pub fn get_verts(&self) -> &'static [BlockVertex; 4] {
        match *self {
            Direction::Up => PRECOMPUTED_VERTS[0],
            Direction::Down => PRECOMPUTED_VERTS[1],
            Direction::North => PRECOMPUTED_VERTS[2],
            Direction::South => PRECOMPUTED_VERTS[3],
            Direction::West => PRECOMPUTED_VERTS[4],
            Direction::East => PRECOMPUTED_VERTS[5],
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
    x: f32, y: f32, z: f32,
    tx: u16, ty: u16, tw: u16, th: u16,
    toffsetx: i16, toffsety: i16, tatlas: i16,
    r: u8, g: u8, b: u8,
    block_light: u16, sky_light: u16,
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
    fn write<W: Write>(&self, w: &mut W) {
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
