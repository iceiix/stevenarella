
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
        }
    }

    pub fn wait_for_builders(&mut self) {
        while self.free_builders.len() != NUM_WORKERS {
            let (id, _) = self.built_recv.recv().unwrap();
            self.free_builders.push(id);
        }
    }

    pub fn tick(&mut self, world: &mut world::World, renderer: &mut render::Renderer) {
        while let Ok((id, val)) = self.built_recv.try_recv() {
            world.reset_building_flag(val.position);

            renderer.update_chunk_solid(val.position, &val.solid_buffer, val.solid_count);

            self.free_builders.push(id);
        }
        if self.free_builders.is_empty() {
            return;
        }
        while let Some((x, y, z)) = world.next_dirty_chunk_section() {
            let t_id = self.free_builders.pop().unwrap();
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

                    for verts in &PRECOMPUTED_VERTS {
                        let stone = render::Renderer::get_texture(&textures, rng.choose(&[
                            "minecraft:blocks/lava_flow",
                            "minecraft:blocks/stone",
                            "minecraft:blocks/melon_side",
                            "minecraft:blocks/sand",
                        ]).unwrap());
                        solid_count += 6;
                        for vert in verts {
                            let mut vert = vert.clone();
                            // TODO
                            vert.r = 255;
                            vert.g = 255;
                            vert.b = 255;

                            vert.x += x as f32;
                            vert.y += y as f32;
                            vert.z += z as f32;

                            vert.toffsetx *= stone.get_width() as i16 * 16;
                            vert.toffsety *= stone.get_height() as i16 * 16;

                            // TODO
                            vert.block_light = 15 * 4000;
                            vert.sky_light = 15 * 4000;

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

const PRECOMPUTED_VERTS: [[BlockVertex; 4]; 6] = [
    [ // Up
        BlockVertex::base(0.0, 1.0, 0.0, 0, 0),
        BlockVertex::base(1.0, 1.0, 0.0, 1, 0),
        BlockVertex::base(0.0, 1.0, 1.0, 0, 1),
        BlockVertex::base(1.0, 1.0, 1.0, 1, 1),
    ],
    [ // Down
        BlockVertex::base(0.0, 0.0, 0.0, 0, 1),
        BlockVertex::base(0.0, 0.0, 1.0, 0, 0),
        BlockVertex::base(1.0, 0.0, 0.0, 1, 1),
        BlockVertex::base(1.0, 0.0, 1.0, 1, 0),
    ],
    [ // North
        BlockVertex::base(0.0, 0.0, 0.0, 1, 1),
        BlockVertex::base(1.0, 0.0, 0.0, 0, 1),
        BlockVertex::base(0.0, 1.0, 0.0, 1, 0),
        BlockVertex::base(1.0, 1.0, 0.0, 0, 0),
    ],
    [ // South
        BlockVertex::base(0.0, 0.0, 1.0, 0, 1),
        BlockVertex::base(0.0, 1.0, 1.0, 0, 0),
        BlockVertex::base(1.0, 0.0, 1.0, 1, 1),
        BlockVertex::base(1.0, 1.0, 1.0, 1, 0),
    ],
    [ // West
        BlockVertex::base(0.0, 0.0, 0.0, 0, 1),
        BlockVertex::base(0.0, 1.0, 0.0, 0, 0),
        BlockVertex::base(0.0, 0.0, 1.0, 1, 1),
        BlockVertex::base(0.0, 1.0, 1.0, 1, 0),
    ],
    [ // East
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
