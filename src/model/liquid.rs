
use std::io::Write;
use std::sync::{Arc, RwLock};
use world::{self, block};
use shared::Direction;
use model::BlockVertex;
use render;

pub fn render_liquid<W: Write>(textures: Arc<RwLock<render::TextureManager>>,lava: bool, snapshot: &world::Snapshot, x: i32, y: i32, z: i32, buf: &mut W) -> usize {
    let get_liquid = if lava {
        get_lava_level
    } else {
        get_water_level
    };

    let mut count = 0;

    let (tl, tr, bl, br) = if get_liquid(snapshot, x, y + 1, z).is_some() {
        (8, 8, 8, 8)
    } else {
        (
            average_liquid_level(get_liquid, snapshot, x, y, z),
            average_liquid_level(get_liquid, snapshot, x+1, y, z),
            average_liquid_level(get_liquid, snapshot, x, y, z+1),
            average_liquid_level(get_liquid, snapshot, x+1, y, z+1)
        )
    };

    let tex = match snapshot.get_block(x, y, z) {
        block::Block::Water{..} => render::Renderer::get_texture(&textures, "minecraft:blocks/water_still"),
        block::Block::FlowingWater{..} => render::Renderer::get_texture(&textures, "minecraft:blocks/water_flow"),
        block::Block::Lava{..} => render::Renderer::get_texture(&textures, "minecraft:blocks/lava_still"),
        block::Block::FlowingLava{..} => render::Renderer::get_texture(&textures, "minecraft:blocks/lava_flow"),
        _ => unreachable!(),
    };
    let ux1 = 0i16;
    let ux2 = 16i16 * tex.get_width() as i16;
    let uy1 = 0i16;
    let uy2 = 16i16 * tex.get_height() as i16;

    for dir in Direction::all() {
        let (ox, oy, oz) = dir.get_offset();
        let special = dir == Direction::Up && (tl < 8 || tr < 8 || bl < 8 || br < 8);
        let block = snapshot.get_block(x+ox, y+oy, z+oz);
        if special || (!block.get_material().should_cull_against && get_liquid(snapshot, x+ox, y+oy, z+oz).is_none()) {
            let verts = BlockVertex::face_by_direction(dir);
            for vert in verts {
                let mut vert = vert.clone();
                vert.tx = tex.get_x() as u16;
                vert.ty = tex.get_y() as u16;
                vert.tw = tex.get_width() as u16;
                vert.th = tex.get_height() as u16;
                vert.tatlas = tex.atlas as i16;
                vert.r = 255;
                vert.g = 255;
                vert.b = 255;

                if vert.y == 0.0 {
                    vert.y = y as f32;
                } else {
                    let height = match (vert.x, vert.z) {
                        (0.0, 0.0) => ((16.0 / 8.0) * (tl as f32)) as i32,
                        (_, 0.0) => ((16.0 / 8.0) * (tr as f32)) as i32,
                        (0.0, _) => ((16.0 / 8.0) * (bl as f32)) as i32,
                        (_, _) => ((16.0 / 8.0) * (br as f32)) as i32,
                    };
					vert.y = (height as f32)/16.0 + (y as f32);
                }

                vert.x += x as f32;
                vert.z += z as f32;

                let (bl, sl) = super::calculate_light(
                    &snapshot,
                    x, y, z,
                    vert.x as f64,
                    vert.y as f64,
                    vert.z as f64,
                    dir,
                    !lava,
                    false
                );
                vert.block_light = bl;
                vert.sky_light = sl;

                if vert.toffsetx == 0 {
                    vert.toffsetx = ux1;
                } else {
                    vert.toffsetx = ux2;
                }

                if vert.toffsety == 0 {
                    vert.toffsety = uy1;
                } else {
                    vert.toffsety = uy2;
                }

                vert.write(buf);
            }
            count += 6;
        }
    }

    count
}

fn average_liquid_level(
    get: fn(&world::Snapshot, i32, i32, i32) -> Option<i32>,
    snapshot: &world::Snapshot, x: i32, y: i32, z: i32
) -> i32 {
    let mut level = 0;
    for xx in -1 .. 1 {
        for zz in -1 .. 1 {
            if get(snapshot, x+xx, y+1, z+zz).is_some() {
                return 8;
            }
            if let Some(l) = get(snapshot, x+xx, y, z+zz) {
                let nl = 7 - (l & 0x7);
                if nl > level {
                    level = nl;
                }
            }
        }
    }
    level
}

fn get_water_level(snapshot: &world::Snapshot, x: i32, y: i32, z: i32) -> Option<i32> {
    match snapshot.get_block(x, y, z) {
        block::Block::Water{level} | block::Block::FlowingWater{level} => Some(level as i32),
        _ => None,
    }
}

fn get_lava_level(snapshot: &world::Snapshot, x: i32, y: i32, z: i32) -> Option<i32> {
    match snapshot.get_block(x, y, z) {
        block::Block::Lava{level} | block::Block::FlowingLava{level} => Some(level as i32),
        _ => None,
    }
}
