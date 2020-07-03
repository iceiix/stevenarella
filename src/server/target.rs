use crate::render;
use crate::render::model;
use crate::shared::{Direction, Position};
use crate::world;
use crate::world::block;
use collision::{self, Aabb};

pub struct Info {
    model: Option<model::ModelKey>,
    last_block: block::Block,
    last_pos: Position,
}

impl Default for Info {
    fn default() -> Self {
        Self::new()
    }
}

impl Info {
    pub fn new() -> Info {
        Info {
            model: None,
            last_block: block::Air {},
            last_pos: Position::new(0, 0, 0),
        }
    }

    pub fn clear(&mut self, renderer: &mut render::Renderer) {
        self.last_block = block::Air {};
        if let Some(model) = self.model.take() {
            renderer.model.remove_model(model);
        }
    }

    pub fn update(&mut self, renderer: &mut render::Renderer, pos: Position, bl: block::Block) {
        if self.last_block == bl && self.last_pos == pos {
            return;
        }
        self.last_block = bl;
        self.last_pos = pos;
        if let Some(model) = self.model.take() {
            renderer.model.remove_model(model);
        }
        let mut parts = vec![];

        const LINE_SIZE: f64 = 1.0 / 128.0;
        let tex = render::Renderer::get_texture(renderer.get_textures_ref(), "steven:solid");

        for bound in bl.get_collision_boxes() {
            let bound = bound.add_v(cgmath::Vector3::new(
                pos.x as f64,
                pos.y as f64,
                pos.z as f64,
            ));
            for point in [
                (bound.min.x, bound.min.z),
                (bound.min.x, bound.max.z),
                (bound.max.x, bound.min.z),
                (bound.max.x, bound.max.z),
            ]
            .iter()
            {
                model::append_box(
                    &mut parts,
                    (point.0 - LINE_SIZE) as f32,
                    (bound.min.y - LINE_SIZE) as f32,
                    (point.1 - LINE_SIZE) as f32,
                    (LINE_SIZE * 2.0) as f32,
                    ((bound.max.y - bound.min.y) + LINE_SIZE * 2.0) as f32,
                    (LINE_SIZE * 2.0) as f32,
                    [
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                    ],
                );
            }

            for point in [
                (bound.min.x, bound.min.z, bound.max.x, bound.min.z),
                (bound.min.x, bound.max.z, bound.max.x, bound.max.z),
                (bound.min.x, bound.min.z, bound.min.x, bound.max.z),
                (bound.max.x, bound.min.z, bound.max.x, bound.max.z),
            ]
            .iter()
            {
                model::append_box(
                    &mut parts,
                    (point.0 - LINE_SIZE) as f32,
                    (bound.min.y - LINE_SIZE) as f32,
                    (point.1 - LINE_SIZE) as f32,
                    ((point.2 - point.0) + (LINE_SIZE * 2.0)) as f32,
                    (LINE_SIZE * 2.0) as f32,
                    ((point.3 - point.1) + (LINE_SIZE * 2.0)) as f32,
                    [
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                    ],
                );
                model::append_box(
                    &mut parts,
                    (point.0 - LINE_SIZE) as f32,
                    (bound.max.y - LINE_SIZE) as f32,
                    (point.1 - LINE_SIZE) as f32,
                    ((point.2 - point.0) + (LINE_SIZE * 2.0)) as f32,
                    (LINE_SIZE * 2.0) as f32,
                    ((point.3 - point.1) + (LINE_SIZE * 2.0)) as f32,
                    [
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                        Some(tex.clone()),
                    ],
                );
            }
        }

        for part in &mut parts {
            part.r = 0;
            part.g = 0;
            part.b = 0;
        }

        self.model = Some(renderer.model.create_model(model::DEFAULT, vec![parts]));
    }
}

#[allow(clippy::type_complexity)]
pub fn test_block(
    world: &world::World,
    pos: Position,
    s: cgmath::Vector3<f64>,
    d: cgmath::Vector3<f64>,
) -> (
    bool,
    Option<(Position, block::Block, Direction, cgmath::Vector3<f64>)>,
) {
    let block = world.get_block(pos);
    let posf = cgmath::Vector3::new(pos.x as f64, pos.y as f64, pos.z as f64);
    for bound in block.get_collision_boxes() {
        let bound = bound.add_v(posf);
        if let Some(hit) = intersects_line(bound, s, d) {
            let cursor = hit - posf;
            let face = find_face(bound, hit);
            return (true, Some((pos, block, face, cursor)));
        }
    }
    (false, None)
}

fn find_face(bound: collision::Aabb3<f64>, hit: cgmath::Vector3<f64>) -> Direction {
    if (bound.min.x - hit.x).abs() < 0.01 {
        Direction::West
    } else if (bound.max.x - hit.x).abs() < 0.01 {
        Direction::East
    } else if (bound.min.y - hit.y).abs() < 0.01 {
        Direction::Down
    } else if (bound.max.y - hit.y).abs() < 0.01 {
        Direction::Up
    } else if (bound.min.z - hit.z).abs() < 0.01 {
        Direction::North
    } else if (bound.max.z - hit.z).abs() < 0.01 {
        Direction::South
    } else {
        Direction::Up
    }
}

fn intersects_line(
    bound: collision::Aabb3<f64>,
    origin: cgmath::Vector3<f64>,
    dir: cgmath::Vector3<f64>,
) -> Option<cgmath::Vector3<f64>> {
    const RIGHT: usize = 0;
    const LEFT: usize = 1;
    const MIDDLE: usize = 2;
    let mut quadrant = [0, 0, 0];
    let mut candidate_plane = [0.0, 0.0, 0.0];
    let mut max_t = [0.0, 0.0, 0.0];
    let mut inside = true;
    for i in 0..3 {
        if origin[i] < bound.min[i] {
            quadrant[i] = LEFT;
            candidate_plane[i] = bound.min[i];
            inside = false;
        } else if origin[i] > bound.max[i] {
            quadrant[i] = RIGHT;
            candidate_plane[i] = bound.max[i];
            inside = false;
        } else {
            quadrant[i] = MIDDLE;
        }
    }
    if inside {
        return Some(origin);
    }

    for i in 0..3 {
        if quadrant[i] != MIDDLE && dir[i] != 0.0 {
            max_t[i] = (candidate_plane[i] - origin[i]) / dir[i];
        }
    }
    let mut which_plane = 0;
    for i in 1..3 {
        if max_t[which_plane] < max_t[i] {
            which_plane = i;
        }
    }
    if max_t[which_plane] < 0.0 {
        return None;
    }

    let mut coord = cgmath::Vector3::new(0.0, 0.0, 0.0);
    for i in 0..3 {
        if which_plane != i {
            coord[i] = origin[i] + max_t[which_plane] * dir[i];
            if coord[i] < bound.min[i] || coord[i] > bound.max[i] {
                return None;
            }
        } else {
            coord[i] = candidate_plane[i];
        }
    }
    Some(coord)
}

pub fn trace_ray<F, R>(
    world: &world::World,
    max: f64,
    s: cgmath::Vector3<f64>,
    d: cgmath::Vector3<f64>,
    collide_func: F,
) -> Option<R>
where
    F: Fn(&world::World, Position, cgmath::Vector3<f64>, cgmath::Vector3<f64>) -> (bool, Option<R>),
{
    struct Gen {
        count: i32,
        base: f64,
        d: f64,
    }
    impl Gen {
        fn new(start: f64, mut d: f64) -> Gen {
            let base = if d > 0.0 {
                (start.ceil() - start) / d
            } else if d < 0.0 {
                d = d.abs();
                (start - start.floor()) / d
            } else {
                0.0
            };
            Gen { count: 0, base, d }
        }

        fn next(&mut self) -> f64 {
            self.count += 1;
            if self.d == 0.0 {
                ::std::f64::INFINITY
            } else {
                self.base + ((self.count as f64 - 1.0) / self.d)
            }
        }
    }

    let mut x_gen = Gen::new(s.x, d.x);
    let mut y_gen = Gen::new(s.y, d.y);
    let mut z_gen = Gen::new(s.z, d.z);
    let mut next_nx = x_gen.next();
    let mut next_ny = y_gen.next();
    let mut next_nz = z_gen.next();

    let mut x = s.x.floor() as i32;
    let mut y = s.y.floor() as i32;
    let mut z = s.z.floor() as i32;

    loop {
        let (hit, ret) = collide_func(world, Position::new(x, y, z), s, d);
        if hit {
            return ret;
        }
        let next_n = if next_nx <= next_ny {
            if next_nx <= next_nz {
                let old = next_nx;
                next_nx = x_gen.next();
                x += d.x.signum() as i32;
                old
            } else {
                let old = next_nz;
                next_nz = z_gen.next();
                z += d.z.signum() as i32;
                old
            }
        } else if next_ny <= next_nz {
            let old = next_ny;
            next_ny = y_gen.next();
            y += d.y.signum() as i32;
            old
        } else {
            let old = next_nz;
            next_nz = z_gen.next();
            z += d.z.signum() as i32;
            old
        };
        if next_n > max {
            break;
        }
    }

    None
}
