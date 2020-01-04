#![feature(trace_macros)]
#![recursion_limit="600"]

extern crate steven_shared as shared;

use crate::shared::{Axis, Direction, Position};
use collision::Aabb3;
use cgmath::Point3;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub mod material;
pub use self::material::Material;

pub use self::Block::*;

pub trait WorldAccess {
    fn get_block(&self, pos: Position) -> Block;
}

#[doc(hidden)]
#[macro_export]
macro_rules! create_ids {
    ($t:ty, ) => ();
    ($t:ty, prev($prev:ident), $name:ident) => (
        #[allow(non_upper_case_globals)]
        pub const $name: $t = $prev + 1;
    );
    ($t:ty, prev($prev:ident), $name:ident, $($n:ident),+) => (
        #[allow(non_upper_case_globals)]
        pub const $name: $t = $prev + 1;
        create_ids!($t, prev($name), $($n),+);
    );
    ($t:ty, $name:ident, $($n:ident),+) => (
        #[allow(non_upper_case_globals)]
        pub const $name: $t = 0;
        create_ids!($t, prev($name), $($n),+);
    );
    ($t:ty, $name:ident) => (
        #[allow(non_upper_case_globals)]
        pub const $name: $t = 0;
    );
}

struct VanillaIDMap {
    flat: Vec<Option<Block>>,
    hier: Vec<Option<Block>>,
    modded: HashMap<String, [Option<Block>; 16]>,
}

macro_rules! define_blocks {
    (
        $(
            $name:ident {
                $(modid $modid:expr,)?
                props {
                    $(
                        $fname:ident : $ftype:ty = [$($val:expr),+],
                    )*
                },
                $(data $datafunc:expr,)?
                $(offset $offsetfunc:expr,)?
                $(material $mat:expr,)?
                model $model:expr,
                $(variant $variant:expr,)?
                $(tint $tint:expr,)?
                $(collision $collision:expr,)?
                $(update_state ($world:ident, $pos:ident) => $update_state:expr,)?
                $(multipart ($mkey:ident, $mval:ident) => $multipart:expr,)?
            }
        )+
    ) => (
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Block {
            $(
                $name {
                    $(
                        $fname : $ftype,
                    )?
                },
            )+
        }
        mod internal_ids {
            create_ids!(usize, $($name),+);
        }

        impl Block {
            #[allow(unused_variables, unreachable_code)]
            pub fn get_internal_id(&self) -> usize {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            internal_ids::$name
                        }
                    )+
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn get_hierarchical_data(&self) -> Option<usize> {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            $(
                                let data: Option<usize> = ($datafunc).map(|v| v);
                                return data;
                            )?
                            Some(0)
                        }
                    )+
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn get_flat_offset(&self) -> Option<usize> {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            $(
                                let offset: Option<usize> = ($offsetfunc).map(|v| v);
                                return offset;
                            )?
                            $(
                                let data: Option<usize> = ($datafunc).map(|v| v);
                                return data;
                            )?
                            Some(0)
                        }
                    )+
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn get_modid(&self) -> Option<&str> {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            $(
                                return Some($modid);
                            )?
                            None
                        }
                    )+
                }
            }

            pub fn by_vanilla_id(id: usize, protocol_version: i32, modded_block_ids: &HashMap<usize, String>) -> Block {
                if protocol_version >= 404 {
                    VANILLA_ID_MAP.flat.get(id).and_then(|v| *v).unwrap_or(Block::Missing{})
                    // TODO: support modded 1.13.2+ blocks after https://github.com/iceiix/stevenarella/pull/145
                } else {
                    if let Some(block) = VANILLA_ID_MAP.hier.get(id).and_then(|v| *v) {
                        block
                    } else {
                        let data = id & 0xf;

                        if let Some(name) = modded_block_ids.get(&(id >> 4)) {
                            if let Some(blocks_by_data) = VANILLA_ID_MAP.modded.get(name) {
                                blocks_by_data[data].unwrap_or(Block::Missing{})
                            } else {
                                //info!("Modded block not supported yet: {}:{} -> {}", id >> 4, data, name);
                                Block::Missing{}
                            }
                        } else {
                            Block::Missing{}
                        }
                    }
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn get_material(&self) -> Material {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            $(return $mat;)?
                            material::SOLID
                        }
                    )+
                }
            }

            #[allow(unused_variables)]
            pub fn get_model(&self) -> (String, String) {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            let parts = $model;
                            (String::from(parts.0), String::from(parts.1))
                        }
                    )+
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn get_model_variant(&self) -> String {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            $(return String::from($variant);)?
                            "normal".to_owned()
                        }
                    )+
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn get_tint(&self) -> TintType {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            $(return $tint;)?
                            TintType::Default
                        }
                    )+
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn get_collision_boxes(&self) -> Vec<Aabb3<f64>> {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            $(return $collision;)?
                            vec![Aabb3::new(
                                Point3::new(0.0, 0.0, 0.0),
                                Point3::new(1.0, 1.0, 1.0)
                            )]
                        }
                    )+
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn update_state<W: WorldAccess>(&self, world: &W, pos: Position) -> Block {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            $(
                                let $world = world;
                                let $pos = pos;
                                return $update_state;
                            )?
                            Block::$name {
                                $($fname: $fname,)?
                            }
                        }
                    )+
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn match_multipart(&self, key: &str, val: &str) -> bool {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)?
                        } => {
                            $(
                                let $mkey = key;
                                let $mval = val;
                                return $multipart;
                            )?
                            false
                        }
                    )+
                }
            }
        }

        lazy_static! {
            static ref VANILLA_ID_MAP: VanillaIDMap = {
                let mut blocks_flat = vec![];
                let mut blocks_hier = vec![];
                let mut blocks_modded: HashMap<String, [Option<Block>; 16]> = HashMap::new();
                let mut flat_id = 0;
                let mut last_internal_id = 0;
                let mut hier_block_id = 0;
                $({
                    #[allow(non_camel_case_types, dead_code)]
                    struct CombinationIter<$($fname),*> {
                        first: bool,
                        finished: bool,
                        state: CombinationIterState<$($fname),*>,
                        orig: CombinationIterOrig<$($fname),*>,
                        current: CombinationIterCurrent,
                    }
                    #[allow(non_camel_case_types)]
                    struct CombinationIterState<$($fname),*> {
                        $($fname: $fname,)?
                    }
                    #[allow(non_camel_case_types)]
                    struct CombinationIterOrig<$($fname),*> {
                        $($fname: $fname,)?
                    }
                    #[allow(non_camel_case_types)]
                    struct CombinationIterCurrent {
                        $($fname: $ftype,)?
                    }

                    #[allow(non_camel_case_types)]
                    impl <$($fname : Iterator<Item=$ftype> + Clone),*> Iterator for CombinationIter<$($fname),*> {
                        type Item = Block;

                        #[allow(unused_mut, unused_variables, unreachable_code, unused_assignments, clippy::never_loop)]
                        fn next(&mut self) -> Option<Self::Item> {
                            if self.finished {
                                return None;
                            }
                            if self.first {
                                self.first = false;
                                return Some(Block::$name {
                                    $(
                                        $fname: self.current.$fname,
                                    )?
                                });
                            }
                            let mut has_value = false;
                            loop {
                                $(
                                    if let Some(val) = self.state.$fname.next() {
                                        self.current.$fname = val;
                                        has_value = true;
                                        break;
                                    }
                                    self.state.$fname = self.orig.$fname.clone();
                                    self.current.$fname = self.state.$fname.next().unwrap();
                                )?
                                self.finished = true;
                                return None;
                            }
                            if has_value {
                                Some(Block::$name {
                                    $(
                                        $fname: self.current.$fname,
                                    )?
                                })
                            } else {
                                None
                            }
                        }
                    }
                    #[allow(non_camel_case_types)]
                    impl <$($fname : Iterator<Item=$ftype> + Clone),*> CombinationIter<$($fname),*> {
                        #[allow(clippy::too_many_arguments)]
                        fn new($(mut $fname:$fname),*) -> CombinationIter<$($fname),*> {
                            CombinationIter {
                                finished: false,
                                first: true,
                                orig: CombinationIterOrig {
                                    $($fname: $fname.clone(),)?
                                },
                                current: CombinationIterCurrent {
                                    $($fname: $fname.next().unwrap(),)?
                                },
                                state: CombinationIterState {
                                    $($fname: $fname,)?
                                }
                            }
                        }
                    }
                    let iter = CombinationIter::new(
                        $({
                            let vals = vec![$($val),+];
                            vals.into_iter()
                        }),*
                    );
                    let mut last_offset: isize = -1;
                    for block in iter {
                        let internal_id = block.get_internal_id();
                        let hier_data: Option<usize> = block.get_hierarchical_data();
                        if let Some(modid) = block.get_modid() {
                            let hier_data = hier_data.unwrap();
                            if !blocks_modded.contains_key(modid) {
                                blocks_modded.insert(modid.to_string(), [None; 16]);
                            }
                            let block_from_data = blocks_modded.get_mut(modid).unwrap();
                            block_from_data[hier_data] = Some(block);
                            continue
                        }

                        let vanilla_id =
                            if let Some(hier_data) = hier_data {
                                if internal_id != last_internal_id {
                                    hier_block_id += 1;
                                }
                                last_internal_id = internal_id;
                                Some((hier_block_id << 4) + hier_data)
                            } else {
                                None
                            };

                        let offset = block.get_flat_offset();
                        if let Some(offset) = offset {
                            let id = flat_id + offset;
                            /*
                            if let Some(vanilla_id) = vanilla_id {
                                debug!("{} block state = {:?} hierarchical {}:{} offset={}", id, block, vanilla_id >> 4, vanilla_id & 0xF, offset);
                            } else {
                                debug!("{} block state = {:?} hierarchical none, offset={}", id, block, offset);
                            }
                            */
                            if offset as isize > last_offset {
                                last_offset = offset as isize;
                            }

                            if blocks_flat.len() <= id {
                                blocks_flat.resize(id + 1, None);
                            }
                            if blocks_flat[id].is_none() {
                                blocks_flat[id] = Some(block);
                            } else {
                                panic!(
                                    "Tried to register {:#?} to {} but {:#?} was already registered",
                                    block,
                                    id,
                                    blocks_flat[id]
                                );
                            }
                        }

                        if let Some(vanilla_id) = vanilla_id {
                            /*
                            if offset.is_none() {
                                debug!("(no flat) block state = {:?} hierarchical {}:{}", block, vanilla_id >> 4, vanilla_id & 0xF);
                            }
                            */

                            if blocks_hier.len() <= vanilla_id {
                                blocks_hier.resize(vanilla_id + 1, None);
                            }
                            if blocks_hier[vanilla_id].is_none() {
                                blocks_hier[vanilla_id] = Some(block);
                            } else {
                                panic!(
                                    "Tried to register {:#?} to {} but {:#?} was already registered",
                                    block,
                                    vanilla_id,
                                    blocks_hier[vanilla_id]
                                );
                            }
                        }
                    }

                    #[allow(unused_assignments)]
                    {
                        flat_id += (last_offset + 1) as usize;
                    }
                })+

                VanillaIDMap { flat: blocks_flat, hier: blocks_hier, modded: blocks_modded }
            };
        }
    );
}

#[derive(Clone, Copy)]
pub enum TintType {
    Default,
    Color{r: u8, g: u8, b: u8},
    Grass,
    Foliage,
}

trace_macros!(true);
define_blocks! {
    Air {
        props {},
        material material::Material {
            collidable: false,
            .. material::INVISIBLE
        },
        model { ("minecraft", "air") },
        collision vec![],
    }
    Missing {
        props {},
        data None::<usize>,
        model { ("steven", "missing_block") },
    }
}
trace_macros!(false);

fn can_burn<W: WorldAccess>(world: &W, pos: Position) -> bool {
    false
}

fn is_snowy<W: WorldAccess>(world: &W, pos: Position) -> bool {
    false
}

fn can_connect_sides<F: Fn(Block) -> bool, W: WorldAccess>(world: &W, pos: Position, f: &F) -> (bool, bool, bool, bool) {
    (can_connect(world, pos.shift(Direction::North), f),
     can_connect(world, pos.shift(Direction::South), f),
     can_connect(world, pos.shift(Direction::West), f),
     can_connect(world, pos.shift(Direction::East), f))
}

fn can_connect<F: Fn(Block) -> bool, W: WorldAccess>(world: &W, pos: Position, f: &F) -> bool {
    let block = world.get_block(pos);
    f(block) || (block.get_material().renderable && block.get_material().should_cull_against)
}

fn can_connect_fence(block: Block) -> bool {
    false
}

fn can_connect_glasspane(block: Block) -> bool {
    false
}

fn can_connect_redstone<W: WorldAccess>(world: &W, pos: Position, dir: Direction) -> RedstoneSide {
    RedstoneSide::None
}

fn fence_gate_data(facing: Direction, in_wall: bool, open: bool, powered: bool) -> Option<usize> {
    if in_wall || powered { return None; }

    Some(facing.horizontal_index() | (if open { 0x4 } else { 0x0 }))
}

fn fence_gate_offset(facing: Direction, in_wall: bool, open: bool, powered: bool) -> Option<usize> {
    Some(if powered { 0 } else { 1<<0 } +
         if open { 0 } else { 1<<1 } +
         if in_wall { 0 } else { 1<<2 } +
         facing.horizontal_offset() * (1<<3))
}

fn fence_gate_collision(facing: Direction, in_wall: bool, open: bool) -> Vec<Aabb3<f64>> {
    if open { return vec![]; }

    let (min_x, min_y, min_z, max_x, max_y, max_z) = if in_wall {
        match facing.axis() {
            Axis::Z => (0.0, 0.0, 3.0/8.0, 1.0, 13.0/16.0, 5.0/8.0),
            Axis::X => (3.0/8.0, 0.0, 0.0, 5.0/8.0, 13.0/16.0, 1.0),
            _ => unreachable!(),
        }
    } else {
        match facing.axis() {
            Axis::Z => (0.0, 0.0, 3.0/8.0, 1.0, 1.0, 5.0/8.0),
            Axis::X => (3.0/8.0, 0.0, 0.0, 5.0/8.0, 1.0, 1.0),
            _ => unreachable!(),
        }
    };

    vec![Aabb3::new(
        Point3::new(min_x, min_y, min_z),
        Point3::new(max_x, max_y, max_z)
    )]
}

fn fence_gate_update_state<W: WorldAccess>(world: &W, pos: Position, facing: Direction) -> bool {
    false
}

fn door_data(facing: Direction, half: DoorHalf, hinge: Side, open: bool, powered: bool) -> Option<usize> {
    match half {
        DoorHalf::Upper => {
            if facing == Direction::North && open {
                Some(0x8
                     | (if hinge == Side::Right { 0x1 } else { 0x0 })
                     | (if powered { 0x2 } else { 0x0 }))
            } else {
                None
            }
        },
        DoorHalf::Lower => {
            if hinge == Side::Left && !powered {
                Some(facing.clockwise().horizontal_index() | (if open { 0x4 } else { 0x0 }))
            } else {
                None
            }
        }
    }
}

fn door_offset(facing: Direction, half: DoorHalf, hinge: Side, open: bool, powered: bool) -> Option<usize> {
    Some(if powered { 0 } else { 1<<0 } +
         if open { 0 } else { 1<<1 } +
         if hinge == Side::Left { 0 } else { 1<<2 } +
         if half == DoorHalf::Upper { 0 } else { 1<<3 } +
         facing.horizontal_offset() * (1<<4))
}


fn update_door_state<W: WorldAccess>(world: &W, pos: Position, ohalf: DoorHalf, ofacing: Direction, ohinge: Side, oopen: bool, opowered: bool) -> (Direction, Side, bool, bool) {
    let oy = if ohalf == DoorHalf::Upper { -1 } else { 1 };

    (ofacing, ohinge, oopen, opowered)
}

fn door_collision(facing: Direction, hinge: Side, open: bool) -> Vec<Aabb3<f64>> {
    use std::f64::consts::PI;
    let mut bounds = Aabb3::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 3.0 / 16.0)
    );
    let mut angle = match facing {
        Direction::South => 0.0,
        Direction::West => PI * 0.5,
        Direction::North => PI,
        Direction::East => PI * 1.5,
        _ => 0.0,
    };
    angle += if open {
        PI * 0.5
    } else {
        0.0
    } * match hinge { Side::Left => 1.0, Side::Right => -1.0 };

    let c = angle.cos();
    let s = angle.sin();

    let x = bounds.min.x - 0.5;
    let z = bounds.min.z - 0.5;
    bounds.min.x = 0.5 + (x*c - z*s);
    bounds.min.z = 0.5 + (z*c + x*s);
    let x = bounds.max.x - 0.5;
    let z = bounds.max.z - 0.5;
    bounds.max.x = 0.5 + (x*c - z*s);
    bounds.max.z = 0.5 + (z*c + x*s);

    vec![bounds]
}

fn update_repeater_state<W: WorldAccess>(world: &W, pos: Position, facing: Direction) -> bool {
    false
}

fn update_double_plant_state<W: WorldAccess>(world: &W, pos: Position, ohalf: BlockHalf, ovariant: DoublePlantVariant) -> (BlockHalf, DoublePlantVariant) {
    if ohalf != BlockHalf::Upper { return (ohalf, ovariant); }

    (ohalf, ovariant)
}

fn piston_collision(extended: bool, facing: Direction) -> Vec<Aabb3<f64>> {
    let (min_x, min_y, min_z, max_x, max_y, max_z) = if extended {
        match facing {
            Direction::Up => (0.0, 0.0, 0.0, 1.0, 0.75, 1.0),
            Direction::Down => (0.0, 0.25, 0.0, 1.0, 1.0, 1.0),
            Direction::North => (0.0, 0.0, 0.25, 1.0, 1.0, 1.0),
            Direction::South => (0.0, 0.0, 0.0, 1.0, 1.0, 0.75),
            Direction::West => (0.25, 0.0, 0.0, 1.0, 1.0, 0.75),
            Direction::East => (0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
            _ => unreachable!(),
        }
    } else {
        (0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    };

    vec![Aabb3::new(
        Point3::new(min_x, min_y, min_z),
        Point3::new(max_x, max_y, max_z)
    )]
}

fn trapdoor_collision(facing: Direction, half: BlockHalf, open: bool) -> Vec<Aabb3<f64>> {
    let (min_x, min_y, min_z, max_x, max_y, max_z) = if open {
        match facing {
            Direction::North => (0.0, 0.0, 3.0/16.0, 1.0, 1.0, 1.0),
            Direction::South => (0.0, 0.0, 0.0, 1.0, 1.0, 3.0/16.0),
            Direction::West => (3.0/16.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            Direction::East => (0.0, 0.0, 0.0, 3.0/16.0, 1.0, 1.0),
            _ => unreachable!(),
        }
    } else {
        match half {
            BlockHalf::Bottom => (0.0, 0.0, 0.0, 1.0, 3.0/16.0, 1.0),
            BlockHalf::Top => (0.0, 3.0/16.0, 0.0, 1.0, 1.0, 1.0),
            _ => unreachable!(),
        }
    };

    vec![Aabb3::new(
        Point3::new(min_x, min_y, min_z),
        Point3::new(max_x, max_y, max_z))
    ]
}

fn fence_collision(north: bool, south: bool, west: bool, east: bool) -> Vec<Aabb3<f64>> {
    let mut collision = vec![Aabb3::new(
        Point3::new(3.0/8.0, 0.0, 3.0/8.0),
        Point3::new(5.0/8.0, 1.5, 5.0/8.0))
    ];

    if north {
        collision.push(Aabb3::new(
            Point3::new(3.0/8.0, 0.0, 0.0),
            Point3::new(5.0/8.0, 1.5, 3.0/8.0))
        );
    }

    if south {
        collision.push(Aabb3::new(
            Point3::new(3.0/8.0, 0.0, 5.0/8.0),
            Point3::new(5.0/8.0, 1.5, 1.0))
        );
    }

    if west {
        collision.push(Aabb3::new(
            Point3::new(0.0, 0.0, 3.0/8.0),
            Point3::new(3.0/8.0, 1.5, 5.0/8.0))
        );
    }

    if east {
        collision.push(Aabb3::new(
            Point3::new(5.0/8.0, 0.0, 3.0/8.0),
            Point3::new(1.0, 1.5, 5.0/8.0))
        );
    }

    collision
}

fn pane_collision(north: bool, south: bool, east: bool, west: bool) -> Vec<Aabb3<f64>> {
    let mut collision = vec![Aabb3::new(
        Point3::new(7.0/16.0, 0.0, 7.0/16.0),
        Point3::new(9.0/16.0, 1.0, 9.0/16.0))
    ];

    if north {
        collision.push(Aabb3::new(
            Point3::new(7.0/16.0, 0.0, 0.0),
            Point3::new(9.0/16.0, 1.0, 9.0/16.0))
        );
    }

    if south {
        collision.push(Aabb3::new(
            Point3::new(7.0/16.0, 0.0, 7.0/16.0),
            Point3::new(9.0/16.0, 1.0, 1.0))
        );
    }

    if west {
        collision.push(Aabb3::new(
            Point3::new(0.0, 0.0, 7.0/16.0),
            Point3::new(9.0/16.0, 1.0, 9.0/16.0))
        );
    }

    if east {
        collision.push(Aabb3::new(
            Point3::new(7.0/16.0, 0.0, 7.0/16.0),
            Point3::new(1.0, 1.0, 9.0/16.0))
        );
    }

    collision
}

fn get_stair_info<W: WorldAccess>(world: &W, pos: Position) -> Option<(Direction, BlockHalf)> {
    None
}

fn update_stair_shape<W: WorldAccess>(world: &W, pos: Position, facing: Direction) -> StairShape {
    if let Some((other_facing, _)) = get_stair_info(world, pos.shift(facing)) {
        if other_facing != facing && other_facing != facing.opposite() {
            if other_facing == facing.clockwise() {
                return StairShape::OuterRight;
            }

            return StairShape::OuterLeft;
        }
    }

    if let Some((other_facing, _)) = get_stair_info(world, pos.shift(facing.opposite())) {
        if other_facing != facing && other_facing != facing.opposite() {
            if other_facing == facing.clockwise() {
                return StairShape::InnerRight;
            }

            return StairShape::InnerLeft;
        }
    }

    StairShape::Straight
}

fn stair_data(facing: Direction, half: BlockHalf, shape: StairShape, waterlogged: bool) -> Option<usize> {
    if shape != StairShape::Straight { return None; }
    if waterlogged { return None; }

    Some((5 - facing.index()) | (if half == BlockHalf::Top { 0x4 } else { 0x0 }))
}

fn stair_offset(facing: Direction, half: BlockHalf, shape: StairShape, waterlogged: bool) -> Option<usize> {
    Some(if waterlogged { 0 } else { 1 } +
         shape.offset() * 2 +
         if half == BlockHalf::Top { 0 } else { 2 * 5 } +
         facing.horizontal_offset() * 2 * 5 * 2)
}

#[allow(clippy::many_single_char_names)]
fn stair_collision(facing: Direction, shape: StairShape, half: BlockHalf) -> Vec<Aabb3<f64>> {
    use std::f64::consts::PI;
    let mut bounds = match shape {
        StairShape::Straight => vec![
            Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.5, 1.0)),
            Aabb3::new(Point3::new(0.0, 0.5, 0.0), Point3::new(1.0, 1.0, 0.5)),
        ],
        StairShape::InnerLeft => vec![
            Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.5, 1.0)),
            Aabb3::new(Point3::new(0.0, 0.5, 0.0), Point3::new(1.0, 1.0, 0.5)),
            Aabb3::new(Point3::new(0.0, 0.5, 0.5), Point3::new(0.5, 1.0, 1.0)),
        ],
        StairShape::InnerRight => vec![
            Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.5, 1.0)),
            Aabb3::new(Point3::new(0.0, 0.5, 0.0), Point3::new(1.0, 1.0, 0.5)),
            Aabb3::new(Point3::new(0.5, 0.5, 0.5), Point3::new(1.0, 1.0, 1.0)),
        ],
        StairShape::OuterLeft => vec![
            Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.5, 1.0)),
            Aabb3::new(Point3::new(0.0, 0.5, 0.0), Point3::new(0.5, 1.0, 0.5)),
        ],
        StairShape::OuterRight => vec![
            Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.5, 1.0)),
            Aabb3::new(Point3::new(0.5, 0.5, 0.0), Point3::new(1.0, 1.0, 0.5)),
        ],
    };
    let mut angle = match facing {
        Direction::North => 0.0,
        Direction::East => PI * 0.5,
        Direction::South => PI,
        Direction::West => PI * 1.5,
        _ => 0.0,
    };

    if half == BlockHalf::Top {
        angle -= PI;
    }

    let c = angle.cos();
    let s = angle.sin();

    for bound in &mut bounds {
        let x = bound.min.x - 0.5;
        let z = bound.min.z - 0.5;
        bound.min.x = 0.5 + (x*c - z*s);
        bound.min.z = 0.5 + (z*c + x*s);
        let x = bound.max.x - 0.5;
        let z = bound.max.z - 0.5;
        bound.max.x = 0.5 + (x*c - z*s);
        bound.max.z = 0.5 + (z*c + x*s);

        if half == BlockHalf::Top {
            let c = PI.cos();
            let s = PI.sin();
            let z = bound.min.z - 0.5;
            let y = bound.min.y - 0.5;
            bound.min.z = 0.5 + (z*c - y*s);
            bound.min.y = 0.5 + (y*c + z*s);
            let z = bound.max.z - 0.5;
            let y = bound.max.y - 0.5;
            bound.max.z = 0.5 + (z*c - y*s);
            bound.max.y = 0.5 + (y*c + z*s);

            bound.min.x = 1.0 - bound.min.x;
            bound.max.x = 1.0 - bound.max.x;
        }
    }

    bounds
}

fn slab_collision(half: BlockHalf) -> Vec<Aabb3<f64>> {
    let (min_x, min_y, min_z, max_x, max_y, max_z) = match half {
        BlockHalf::Top => (0.0, 0.5, 0.0, 1.0, 1.0, 1.0),
        BlockHalf::Bottom => (0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
        BlockHalf::Double => (0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        _ => unreachable!(),
    };

    vec![Aabb3::new(
        Point3::new(min_x, min_y, min_z),
        Point3::new(max_x, max_y, max_z)
    )]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoneVariant {
    Normal,
    Granite,
    SmoothGranite,
    Diorite,
    SmoothDiorite,
    Andesite,
    SmoothAndesite,
}

impl StoneVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            StoneVariant::Normal => "stone",
            StoneVariant::Granite => "granite",
            StoneVariant::SmoothGranite => "smooth_granite",
            StoneVariant::Diorite => "diorite",
            StoneVariant::SmoothDiorite => "smooth_diorite",
            StoneVariant::Andesite => "andesite",
            StoneVariant::SmoothAndesite => "smooth_andesite",
        }
    }
    fn data(self) -> usize {
        match self {
            StoneVariant::Normal => 0,
            StoneVariant::Granite => 1,
            StoneVariant::SmoothGranite => 2,
            StoneVariant::Diorite => 3,
            StoneVariant::SmoothDiorite => 4,
            StoneVariant::Andesite => 5,
            StoneVariant::SmoothAndesite => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DirtVariant {
    Normal,
    Coarse,
    Podzol,
}

impl DirtVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            DirtVariant::Normal => "dirt",
            DirtVariant::Coarse => "coarse_dirt",
            DirtVariant::Podzol => "podzol",
        }
    }

    fn data(self) -> usize {
        match self {
            DirtVariant::Normal => 0,
            DirtVariant::Coarse => 1,
            DirtVariant::Podzol => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BedPart {
    Head,
    Foot
}

impl BedPart {
    pub fn as_string(self) -> &'static str {
        match self {
            BedPart::Head => "head",
            BedPart::Foot => "foot",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SandstoneVariant {
    Normal,
    Chiseled,
    Smooth,
}

impl SandstoneVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            SandstoneVariant::Normal => "sandstone",
            SandstoneVariant::Chiseled => "chiseled_sandstone",
            SandstoneVariant::Smooth => "smooth_sandstone",
        }
    }

    fn data(self) -> usize {
        match self {
            SandstoneVariant::Normal => 0,
            SandstoneVariant::Chiseled => 1,
            SandstoneVariant::Smooth => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoteBlockInstrument {
    Harp,
    BaseDrum,
    Snare,
    Hat,
    Bass,
    Flute,
    Bell,
    Guitar,
    Chime,
    Xylophone,
}

impl NoteBlockInstrument {
    pub fn as_string(self) -> &'static str {
        match self {
            NoteBlockInstrument::Harp => "harp",
            NoteBlockInstrument::BaseDrum => "basedrum",
            NoteBlockInstrument::Snare => "snare",
            NoteBlockInstrument::Hat => "hat",
            NoteBlockInstrument::Bass => "bass",
            NoteBlockInstrument::Flute => "flute",
            NoteBlockInstrument::Bell => "bell",
            NoteBlockInstrument::Guitar => "guitar",
            NoteBlockInstrument::Chime => "chime",
            NoteBlockInstrument::Xylophone => "xylophone",
        }
    }

    fn offset(self) -> usize {
        match self {
            NoteBlockInstrument::Harp => 0,
            NoteBlockInstrument::BaseDrum => 1,
            NoteBlockInstrument::Snare => 2,
            NoteBlockInstrument::Hat => 3,
            NoteBlockInstrument::Bass => 4,
            NoteBlockInstrument::Flute => 5,
            NoteBlockInstrument::Bell => 6,
            NoteBlockInstrument::Guitar => 7,
            NoteBlockInstrument::Chime => 8,
            NoteBlockInstrument::Xylophone => 9,
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RedSandstoneVariant {
    Normal,
    Chiseled,
    Smooth,
}

impl RedSandstoneVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            RedSandstoneVariant::Normal => "red_sandstone",
            RedSandstoneVariant::Chiseled => "chiseled_red_sandstone",
            RedSandstoneVariant::Smooth => "smooth_red_sandstone",
        }
    }

    fn data(self) -> usize {
        match self {
            RedSandstoneVariant::Normal => 0,
            RedSandstoneVariant::Chiseled => 1,
            RedSandstoneVariant::Smooth => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuartzVariant {
    Normal,
    Chiseled,
    PillarVertical,
    PillarNorthSouth,
    PillarEastWest,
}

impl QuartzVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            QuartzVariant::Normal | QuartzVariant::Chiseled => "normal",
            QuartzVariant::PillarVertical => "axis=y",
            QuartzVariant::PillarNorthSouth => "axis=z",
            QuartzVariant::PillarEastWest => "axis=x",
        }
    }

    fn data(self) -> usize {
        match self {
            QuartzVariant::Normal => 0,
            QuartzVariant::Chiseled => 1,
            QuartzVariant::PillarVertical => 2,
            QuartzVariant::PillarNorthSouth => 3,
            QuartzVariant::PillarEastWest => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrismarineVariant {
    Normal,
    Brick,
    Dark,
}

impl PrismarineVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            PrismarineVariant::Normal => "prismarine",
            PrismarineVariant::Brick => "prismarine_bricks",
            PrismarineVariant::Dark => "dark_prismarine",
        }
    }

    fn data(self) -> usize {
        match self {
            PrismarineVariant::Normal => 0,
            PrismarineVariant::Brick => 1,
            PrismarineVariant::Dark => 2,
        }
    }
}

fn mushroom_block_data(is_stem: bool, west: bool, up: bool, south: bool, north: bool, east: bool, down: bool) -> Option<usize> {
    Some(match
        (is_stem, west,  up,    south, north, east,  down) {
        (false, false, false, false, false, false, false) => 0,
        (false, true,  false, false, true,  false, false) => 1,
        (false, false, false, false, true,  false, false) => 2,
        (false, false, false, false, true,  true,  false) => 3,
        (false, true,  false, false, false, false, false) => 4,
        (false, false, true,  false, false, false, false) => 5,
        (false, false, false, false, false, true,  false) => 6,
        (false, true,  false, true,  false, false, false) => 7,
        (false, false, false, true,  false, false, false) => 8,
        (false, false, false, true,  false, true,  false) => 9,
        (false, true,  false, true,  true,  true, false)  => 10,
        (false, true,  true,  true,  true,  true,  true)  => 14,
        (true,  false, false, false, false, false, false) => 15,
        _ => return None,
    })
}

fn mushroom_block_offset(is_stem: bool, west: bool, up: bool, south: bool, north: bool, east: bool, down: bool) -> Option<usize> {
    if is_stem {
        None
    } else {
        Some(if west { 0 } else { 1<<0 } +
             if up { 0 } else { 1<<1 } +
             if south { 0 } else { 1<<2 } +
             if north { 0 } else { 1<<3 } +
             if east { 0 } else { 1<<4 } +
             if down { 0 } else { 1<<5 })
    }
}


fn mushroom_block_variant(is_stem: bool, west: bool, up: bool, south: bool, north: bool, east: bool, down: bool) -> String {
    (if is_stem {
        "all_stem"
    } else {
        match
            (west,  up,    south, north, east,  down) {
            (false, false, false, false, false, false) => "all_inside",
            (true,  false, false, true,  false, false) => "north_west",
            (false, false, false, true,  false, false) => "north",
            (false, false, false, true,  true,  false) => "north_east",
            (true,  false, false, false, false, false) => "west",
            (false, true,  false, false, false, false) => "center",
            (false, false, false, false, true,  false) => "east",
            (true,  false, true,  false, false, false) => "south_west",
            (false, false, true,  false, false, false) => "south",
            (false, false, true,  false, true,  false) => "south_east",
            (true,  false, true,  true,  true,  false)  => "stem",
            (true,  true,  true,  true,  true,  true)  => "all_outside",
            _ => "all_stem",
        }
    }).to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoorHalf {
    Upper,
    Lower
}

impl DoorHalf {
    pub fn as_string(self) -> &'static str {
        match self {
            DoorHalf::Upper => "upper",
            DoorHalf::Lower => "lower",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn as_string(self) -> &'static str {
        match self {
            Side::Left => "left",
            Side::Right => "right",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColoredVariant {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    Silver,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

impl ColoredVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            ColoredVariant::White => "white",
            ColoredVariant::Orange => "orange",
            ColoredVariant::Magenta => "magenta",
            ColoredVariant::LightBlue => "light_blue",
            ColoredVariant::Yellow => "yellow",
            ColoredVariant::Lime => "lime",
            ColoredVariant::Pink => "pink",
            ColoredVariant::Gray => "gray",
            ColoredVariant::Silver => "silver",
            ColoredVariant::Cyan => "cyan",
            ColoredVariant::Purple => "purple",
            ColoredVariant::Blue => "blue",
            ColoredVariant::Brown => "brown",
            ColoredVariant::Green => "green",
            ColoredVariant::Red => "red",
            ColoredVariant::Black => "black",
        }
    }

    fn data(self) -> usize {
        match self {
            ColoredVariant::White => 0,
            ColoredVariant::Orange => 1,
            ColoredVariant::Magenta => 2,
            ColoredVariant::LightBlue => 3,
            ColoredVariant::Yellow => 4,
            ColoredVariant::Lime => 5,
            ColoredVariant::Pink => 6,
            ColoredVariant::Gray => 7,
            ColoredVariant::Silver => 8,
            ColoredVariant::Cyan => 9,
            ColoredVariant::Purple => 10,
            ColoredVariant::Blue => 11,
            ColoredVariant::Brown => 12,
            ColoredVariant::Green => 13,
            ColoredVariant::Red => 14,
            ColoredVariant::Black => 15,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RedFlowerVariant {
    Poppy,
    BlueOrchid,
    Allium,
    AzureBluet,
    RedTulip,
    OrangeTulip,
    WhiteTulip,
    PinkTulip,
    OxeyeDaisy,
}

impl RedFlowerVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            RedFlowerVariant::Poppy => "poppy",
            RedFlowerVariant::BlueOrchid => "blue_orchid",
            RedFlowerVariant::Allium => "allium",
            RedFlowerVariant::AzureBluet => "houstonia",
            RedFlowerVariant::RedTulip => "red_tulip",
            RedFlowerVariant::OrangeTulip => "orange_tulip",
            RedFlowerVariant::WhiteTulip => "white_tulip",
            RedFlowerVariant::PinkTulip => "pink_tulip",
            RedFlowerVariant::OxeyeDaisy => "oxeye_daisy",
        }
    }

    fn data(self) -> usize {
        match self {
            RedFlowerVariant::Poppy => 0,
            RedFlowerVariant::BlueOrchid => 1,
            RedFlowerVariant::Allium => 2,
            RedFlowerVariant::AzureBluet => 3,
            RedFlowerVariant::RedTulip => 4,
            RedFlowerVariant::OrangeTulip => 5,
            RedFlowerVariant::WhiteTulip => 6,
            RedFlowerVariant::PinkTulip => 7,
            RedFlowerVariant::OxeyeDaisy => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MonsterEggVariant {
    Stone,
    Cobblestone,
    StoneBrick,
    MossyBrick,
    CrackedBrick,
    ChiseledBrick,
}

impl MonsterEggVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            MonsterEggVariant::Stone => "stone",
            MonsterEggVariant::Cobblestone => "cobblestone",
            MonsterEggVariant::StoneBrick => "stone_brick",
            MonsterEggVariant::MossyBrick => "mossy_brick",
            MonsterEggVariant::CrackedBrick => "cracked_brick",
            MonsterEggVariant::ChiseledBrick => "chiseled_brick",
        }
    }

    fn data(self) -> usize {
        match self {
            MonsterEggVariant::Stone => 0,
            MonsterEggVariant::Cobblestone => 1,
            MonsterEggVariant::StoneBrick => 2,
            MonsterEggVariant::MossyBrick => 3,
            MonsterEggVariant::CrackedBrick => 4,
            MonsterEggVariant::ChiseledBrick => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoneBrickVariant {
    Normal,
    Mossy,
    Cracked,
    Chiseled,
}

impl StoneBrickVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            StoneBrickVariant::Normal => "stonebrick",
            StoneBrickVariant::Mossy => "mossy_stonebrick",
            StoneBrickVariant::Cracked => "cracked_stonebrick",
            StoneBrickVariant::Chiseled => "chiseled_stonebrick",
        }
    }

    fn data(self) -> usize {
        match self {
            StoneBrickVariant::Normal => 0,
            StoneBrickVariant::Mossy => 1,
            StoneBrickVariant::Cracked => 2,
            StoneBrickVariant::Chiseled => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RailShape {
    NorthSouth,
    EastWest,
    AscendingNorth,
    AscendingSouth,
    AscendingEast,
    AscendingWest,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl RailShape {
    pub fn as_string(self) -> &'static str {
        match self {
            RailShape::NorthSouth => "north_south",
            RailShape::EastWest => "east_west",
            RailShape::AscendingNorth => "ascending_north",
            RailShape::AscendingSouth => "ascending_south",
            RailShape::AscendingEast => "ascending_east",
            RailShape::AscendingWest => "ascending_west",
            RailShape::NorthEast => "north_east",
            RailShape::NorthWest => "north_west",
            RailShape::SouthEast => "south_east",
            RailShape::SouthWest => "south_west",
        }
    }

    pub fn data(self) -> usize {
        match self {
            RailShape::NorthSouth => 0,
            RailShape::EastWest => 1,
            RailShape::AscendingEast => 2,
            RailShape::AscendingWest => 3,
            RailShape::AscendingNorth => 4,
            RailShape::AscendingSouth => 5,
            RailShape::SouthEast => 6,
            RailShape::SouthWest => 7,
            RailShape::NorthWest => 8,
            RailShape::NorthEast => 9,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ComparatorMode {
    Compare,
    Subtract,
}

impl ComparatorMode {
    pub fn as_string(self) -> &'static str {
        match self {
            ComparatorMode::Compare => "compare",
            ComparatorMode::Subtract => "subtract",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RedstoneSide {
    None,
    Side,
    Up,
}

impl RedstoneSide {
    pub fn as_string(self) -> &'static str {
        match self {
            RedstoneSide::None => "none",
            RedstoneSide::Side => "side",
            RedstoneSide::Up => "up",
        }
    }

    pub fn offset(self) -> usize {
        match self {
            RedstoneSide::Up => 0,
            RedstoneSide::Side => 1,
            RedstoneSide::None => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PistonType {
    Normal,
    Sticky,
}

impl PistonType {
    pub fn as_string(self) -> &'static str {
        match self {
            PistonType::Normal => "normal",
            PistonType::Sticky => "sticky",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoneSlabVariant {
    Stone,
    Sandstone,
    PetrifiedWood,
    Cobblestone,
    Brick,
    StoneBrick,
    NetherBrick,
    Quartz,
    RedSandstone,
    Purpur,
}

impl StoneSlabVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            StoneSlabVariant::Stone => "stone",
            StoneSlabVariant::Sandstone => "sandstone",
            StoneSlabVariant::PetrifiedWood => "wood_old",
            StoneSlabVariant::Cobblestone => "cobblestone",
            StoneSlabVariant::Brick => "brick",
            StoneSlabVariant::StoneBrick => "stone_brick",
            StoneSlabVariant::NetherBrick => "nether_brick",
            StoneSlabVariant::Quartz => "quartz",
            StoneSlabVariant::RedSandstone => "red_sandstone",
            StoneSlabVariant::Purpur => "purpur",
        }
    }

    fn data(self) -> usize {
        match self {
            StoneSlabVariant::Stone |
            StoneSlabVariant::RedSandstone |
            StoneSlabVariant::Purpur => 0,
            StoneSlabVariant::Sandstone => 1,
            StoneSlabVariant::PetrifiedWood => 2,
            StoneSlabVariant::Cobblestone => 3,
            StoneSlabVariant::Brick => 4,
            StoneSlabVariant::StoneBrick => 5,
            StoneSlabVariant::NetherBrick => 6,
            StoneSlabVariant::Quartz => 7,
        }
    }

    fn offset(self) -> usize {
        match self {
            StoneSlabVariant::Stone => 0,
            StoneSlabVariant::Sandstone => 1,
            StoneSlabVariant::PetrifiedWood => 2,
            StoneSlabVariant::Cobblestone => 3,
            StoneSlabVariant::Brick => 4,
            StoneSlabVariant::StoneBrick => 5,
            StoneSlabVariant::NetherBrick => 6,
            StoneSlabVariant::Quartz => 7,
            StoneSlabVariant::RedSandstone => 8,
            StoneSlabVariant::Purpur => 9,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WoodSlabVariant {
    Oak,
    Spruce,
    Birch,
    Jungle,
    Acacia,
    DarkOak,
}

impl WoodSlabVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            WoodSlabVariant::Oak => "oak",
            WoodSlabVariant::Spruce => "spruce",
            WoodSlabVariant::Birch => "birch",
            WoodSlabVariant::Jungle => "jungle",
            WoodSlabVariant::Acacia => "acacia",
            WoodSlabVariant::DarkOak => "dark_oak",
        }
    }

    fn data(self) -> usize {
        match self {
            WoodSlabVariant::Oak => 0,
            WoodSlabVariant::Spruce => 1,
            WoodSlabVariant::Birch => 2,
            WoodSlabVariant::Jungle => 3,
            WoodSlabVariant::Acacia => 4,
            WoodSlabVariant::DarkOak => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockHalf {
    Top,
    Bottom,
    Upper,
    Lower,
    Double,
}

impl BlockHalf {
    pub fn as_string(self) -> &'static str {
        match self {
            BlockHalf::Top => "top",
            BlockHalf::Bottom => "bottom",
            BlockHalf::Upper => "upper",
            BlockHalf::Lower => "lower",
            BlockHalf::Double => "double",
        }
    }

    pub fn offset(self) -> usize {
        match self {
            BlockHalf::Top | BlockHalf::Upper => 0,
            BlockHalf::Bottom | BlockHalf::Lower => 1,
            BlockHalf::Double => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CobblestoneWallVariant {
    Normal,
    Mossy,
}

impl CobblestoneWallVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            CobblestoneWallVariant::Normal => "cobblestone",
            CobblestoneWallVariant::Mossy => "mossy_cobblestone",
        }
    }

    pub fn data(self) -> usize {
        match self {
            CobblestoneWallVariant::Normal => 0,
            CobblestoneWallVariant::Mossy => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    South,
    SouthSouthWest,
    SouthWest,
    WestSouthWest,
    West,
    WestNorthWest,
    NorthWest,
    NorthNorthWest,
    North,
    NorthNorthEast,
    NorthEast,
    EastNorthEast,
    East,
    EastSouthEast,
    SouthEast,
    SouthSouthEast,
}

impl Rotation {
    pub fn as_string(self) -> &'static str {
        match self {
            Rotation::South => "south",
            Rotation::SouthSouthWest => "south-southwest",
            Rotation::SouthWest => "southwest",
            Rotation::WestSouthWest => "west-southwest",
            Rotation::West => "west",
            Rotation::WestNorthWest => "west-northwest",
            Rotation::NorthWest => "northwest",
            Rotation::NorthNorthWest => "north-northwest",
            Rotation::North => "north",
            Rotation::NorthNorthEast => "north-northeast",
            Rotation::NorthEast => "northeast",
            Rotation::EastNorthEast => "east-northeast",
            Rotation::East => "east",
            Rotation::EastSouthEast => "east-southeast",
            Rotation::SouthEast => "southseast",
            Rotation::SouthSouthEast => "south-southeast",
        }
    }

    pub fn data(self) -> usize {
        match self {
            Rotation::South => 0,
            Rotation::SouthSouthWest => 1,
            Rotation::SouthWest => 2,
            Rotation::WestSouthWest => 3,
            Rotation::West => 4,
            Rotation::WestNorthWest => 5,
            Rotation::NorthWest => 6,
            Rotation::NorthNorthWest => 7,
            Rotation::North => 8,
            Rotation::NorthNorthEast => 9,
            Rotation::NorthEast => 10,
            Rotation::EastNorthEast => 11,
            Rotation::East => 12,
            Rotation::EastSouthEast => 13,
            Rotation::SouthEast => 14,
            Rotation::SouthSouthEast => 15,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StairShape {
    Straight,
    InnerLeft,
    InnerRight,
    OuterLeft,
    OuterRight,
}

impl StairShape {
    pub fn as_string(self) -> &'static str {
        match self {
            StairShape::Straight => "straight",
            StairShape::InnerLeft => "inner_left",
            StairShape::InnerRight => "inner_right",
            StairShape::OuterLeft => "outer_left",
            StairShape::OuterRight => "outer_right",
        }
    }

    pub fn offset(self) -> usize {
        match self {
            StairShape::Straight => 0,
            StairShape::InnerLeft => 1,
            StairShape::InnerRight => 2,
            StairShape::OuterLeft => 3,
            StairShape::OuterRight => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttachedFace {
    Floor,
    Wall,
    Ceiling,
}

impl AttachedFace {
    pub fn as_string(self) -> &'static str {
        match self {
            AttachedFace::Floor => "floor",
            AttachedFace::Wall => "wall",
            AttachedFace::Ceiling => "ceiling",
        }
    }

    pub fn offset(self) -> usize {
        match self {
            AttachedFace::Floor => 0,
            AttachedFace::Wall => 1,
            AttachedFace::Ceiling => 2,
        }
    }

    pub fn data_with_facing(self, facing: Direction) -> Option<usize> {
        Some(match (self, facing) {
            (AttachedFace::Ceiling, Direction::East) => 0,
            (AttachedFace::Wall, Direction::East) => 1,
            (AttachedFace::Wall, Direction::West) => 2,
            (AttachedFace::Wall, Direction::South) => 3,
            (AttachedFace::Wall, Direction::North) => 4,
            (AttachedFace::Floor, Direction::South) => 5,
            (AttachedFace::Floor, Direction::East) => 6,
            (AttachedFace::Ceiling, Direction::South) => 7,
            _ => return None,
        })
    }

    pub fn data_with_facing_and_powered(self, facing: Direction, powered: bool) -> Option<usize> {
        if let Some(facing_data) = self.data_with_facing(facing) {
            Some(facing_data | (if powered { 0x8 } else { 0x0 }))
        } else {
            None
        }
    }

    pub fn variant_with_facing(self, facing: Direction) -> String {
        match (self, facing) {
            (AttachedFace::Ceiling, Direction::East) => "down_x",
            (AttachedFace::Wall, Direction::East) => "east",
            (AttachedFace::Wall, Direction::West) => "west",
            (AttachedFace::Wall, Direction::South) => "south",
            (AttachedFace::Wall, Direction::North) => "north",
            (AttachedFace::Floor, Direction::South) => "up_z",
            (AttachedFace::Floor, Direction::East) => "up_x",
            (AttachedFace::Ceiling, Direction::South) => "down_z",
            _ => "north", // TODO: support 1.13.2+ new directions
        }.to_owned()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChestType {
    Single,
    Left,
    Right,
}

impl ChestType {
    pub fn as_string(self) -> &'static str {
        match self {
            ChestType::Single => "single",
            ChestType::Left => "left",
            ChestType::Right => "right",
        }
    }

    pub fn offset(self) -> usize {
        match self {
            ChestType::Single => 0,
            ChestType::Left => 1,
            ChestType::Right => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructureBlockMode {
    Save,
    Load,
    Corner,
    Data,
}

impl StructureBlockMode {
    pub fn data(self) -> usize {
        match self {
            StructureBlockMode::Save => 0,
            StructureBlockMode::Load => 1,
            StructureBlockMode::Corner => 2,
            StructureBlockMode::Data => 3,
        }
    }

    pub fn as_string(self) -> &'static str {
        match self {
            StructureBlockMode::Save => "save",
            StructureBlockMode::Load => "load",
            StructureBlockMode::Corner => "corner",
            StructureBlockMode::Data => "data",
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreeVariant {
    Oak,
    Spruce,
    Birch,
    Jungle,
    Acacia,
    DarkOak,
    StrippedSpruce,
    StrippedBirch,
    StrippedJungle,
    StrippedAcacia,
    StrippedDarkOak,
    StrippedOak,
}

impl TreeVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            TreeVariant::Oak => "oak",
            TreeVariant::Spruce => "spruce",
            TreeVariant::Birch => "birch",
            TreeVariant::Jungle => "jungle",
            TreeVariant::Acacia => "acacia",
            TreeVariant::DarkOak => "dark_oak",
            TreeVariant::StrippedSpruce => "stripped_spruce_log",
            TreeVariant::StrippedBirch => "stripped_birch_log",
            TreeVariant::StrippedJungle => "stripped_jungle_log",
            TreeVariant::StrippedAcacia => "stripped_acacia_log",
            TreeVariant::StrippedDarkOak => "stripped_dark_oak_log",
            TreeVariant::StrippedOak => "stripped_oak_log"
        }
    }

    pub fn data(self) -> usize {
        match self {
            TreeVariant::Oak | TreeVariant::Acacia => 0,
            TreeVariant::Spruce | TreeVariant::DarkOak => 1,
            TreeVariant::Birch => 2,
            TreeVariant::Jungle => 3,
            _ => panic!("TreeVariant {:?} has no data (1.13+ only)"),
        }
    }

    pub fn offset(self) -> usize {
        match self {
            TreeVariant::Oak => 0,
            TreeVariant::Spruce => 1,
            TreeVariant::Birch => 2,
            TreeVariant::Jungle => 3,
            TreeVariant::Acacia => 4,
            TreeVariant::DarkOak => 5,
            TreeVariant::StrippedSpruce => 6,
            TreeVariant::StrippedBirch => 7,
            TreeVariant::StrippedJungle => 8,
            TreeVariant::StrippedAcacia => 9,
            TreeVariant::StrippedDarkOak => 10,
            TreeVariant::StrippedOak => 11,
        }
    }

    pub fn plank_data(self) -> usize {
        match self {
            TreeVariant::Oak => 0,
            TreeVariant::Spruce => 1,
            TreeVariant::Birch => 2,
            TreeVariant::Jungle => 3,
            TreeVariant::Acacia => 4,
            TreeVariant::DarkOak => 5,
            _ => panic!("TreeVariant {:?} has no plank data (1.13+ only)"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TallGrassVariant {
    DeadBush,
    TallGrass,
    Fern,
}

impl TallGrassVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            TallGrassVariant::DeadBush => "dead_bush",
            TallGrassVariant::TallGrass => "tall_grass",
            TallGrassVariant::Fern => "fern",
        }
    }

    fn data(self) -> usize {
        match self {
            TallGrassVariant::DeadBush => 0,
            TallGrassVariant::TallGrass => 1,
            TallGrassVariant::Fern => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TallSeagrassHalf {
    Upper,
    Lower,
}

impl TallSeagrassHalf {
    pub fn as_string(self) -> &'static str {
        match self {
            TallSeagrassHalf::Upper => "upper",
            TallSeagrassHalf::Lower => "lower",
        }
    }

    fn offset(self) -> usize {
        match self {
            TallSeagrassHalf::Upper => 0,
            TallSeagrassHalf::Lower => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoublePlantVariant {
    Sunflower,
    Lilac,
    DoubleTallgrass,
    LargeFern,
    RoseBush,
    Peony,
}

impl DoublePlantVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            DoublePlantVariant::Sunflower => "sunflower",
            DoublePlantVariant::Lilac => "syringa",
            DoublePlantVariant::DoubleTallgrass => "double_grass",
            DoublePlantVariant::LargeFern => "double_fern",
            DoublePlantVariant::RoseBush => "double_rose",
            DoublePlantVariant::Peony => "paeonia",
        }
    }

    pub fn data(self) -> usize {
        match self {
            DoublePlantVariant::Sunflower => 0,
            DoublePlantVariant::Lilac => 1,
            DoublePlantVariant::DoubleTallgrass => 2,
            DoublePlantVariant::LargeFern => 3,
            DoublePlantVariant::RoseBush => 4,
            DoublePlantVariant::Peony => 5,
        }
    }

    pub fn offset(self) -> usize {
        match self {
            DoublePlantVariant::Sunflower => 0,
            DoublePlantVariant::Lilac => 1,
            DoublePlantVariant::RoseBush => 2,
            DoublePlantVariant::Peony => 3,
            DoublePlantVariant::DoubleTallgrass => 4,
            DoublePlantVariant::LargeFern => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlowerPotVariant {
    Empty,
    Poppy,
    Dandelion,
    OakSapling,
    SpruceSapling,
    BirchSapling,
    JungleSapling,
    RedMushroom,
    BrownMushroom,
    Cactus,
    DeadBush,
    Fern,
    AcaciaSapling,
    DarkOakSapling,
    BlueOrchid,
    Allium,
    AzureBluet,
    RedTulip,
    OrangeTulip,
    WhiteTulip,
    PinkTulip,
    Oxeye,
}

impl FlowerPotVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            FlowerPotVariant::Empty => "empty",
            FlowerPotVariant::Poppy => "rose",
            FlowerPotVariant::Dandelion => "dandelion",
            FlowerPotVariant::OakSapling => "oak_sapling",
            FlowerPotVariant::SpruceSapling => "spruce_sapling",
            FlowerPotVariant::BirchSapling => "birch_sapling",
            FlowerPotVariant::JungleSapling => "jungle_sapling",
            FlowerPotVariant::RedMushroom => "mushroom_red",
            FlowerPotVariant::BrownMushroom => "mushroom_brown",
            FlowerPotVariant::Cactus => "cactus",
            FlowerPotVariant::DeadBush => "dead_bush",
            FlowerPotVariant::Fern => "fern",
            FlowerPotVariant::AcaciaSapling => "acacia_sapling",
            FlowerPotVariant::DarkOakSapling => "dark_oak_sapling",
            FlowerPotVariant::BlueOrchid => "blue_orchid",
            FlowerPotVariant::Allium => "allium",
            FlowerPotVariant::AzureBluet => "houstonia",
            FlowerPotVariant::RedTulip => "red_tulip",
            FlowerPotVariant::OrangeTulip => "orange_tulip",
            FlowerPotVariant::WhiteTulip => "white_tulip",
            FlowerPotVariant::PinkTulip => "pink_tulip",
            FlowerPotVariant::Oxeye => "oxeye_daisy",
        }
    }

    pub fn offset(self) -> usize {
        match self {
            FlowerPotVariant::Empty => 0,
            FlowerPotVariant::OakSapling => 1,
            FlowerPotVariant::SpruceSapling => 2,
            FlowerPotVariant::BirchSapling => 3,
            FlowerPotVariant::JungleSapling => 4,
            FlowerPotVariant::AcaciaSapling => 5,
            FlowerPotVariant::DarkOakSapling => 6,
            FlowerPotVariant::Fern => 7,
            FlowerPotVariant::Dandelion => 8,
            FlowerPotVariant::Poppy => 9,
            FlowerPotVariant::BlueOrchid => 10,
            FlowerPotVariant::Allium => 11,
            FlowerPotVariant::AzureBluet => 12,
            FlowerPotVariant::RedTulip => 13,
            FlowerPotVariant::OrangeTulip => 14,
            FlowerPotVariant::WhiteTulip => 15,
            FlowerPotVariant::PinkTulip => 16,
            FlowerPotVariant::Oxeye => 17,
            FlowerPotVariant::RedMushroom => 18,
            FlowerPotVariant::BrownMushroom => 19,
            FlowerPotVariant::DeadBush => 20,
            FlowerPotVariant::Cactus => 21,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CoralVariant {
    DeadTube,
    DeadBrain,
    DeadBubble,
    DeadFire,
    DeadHorn,
    Tube,
    Brain,
    Bubble,
    Fire,
    Horn,
}

impl CoralVariant {
    pub fn as_string(self) -> &'static str {
        match self {
            CoralVariant::DeadTube => "dead_tube",
            CoralVariant::DeadBrain => "dead_brain",
            CoralVariant::DeadBubble => "dead_bubble",
            CoralVariant::DeadFire => "dead_fire",
            CoralVariant::DeadHorn => "dead_horn",
            CoralVariant::Tube => "dead_tube",
            CoralVariant::Brain => "brain",
            CoralVariant::Bubble => "bubble",
            CoralVariant::Fire => "fire",
            CoralVariant::Horn => "horn",
        }
    }

    pub fn offset(self) -> usize {
        match self {
            CoralVariant::DeadTube => 0,
            CoralVariant::DeadBrain => 1,
            CoralVariant::DeadBubble => 2,
            CoralVariant::DeadFire => 3,
            CoralVariant::DeadHorn => 4,
            CoralVariant::Tube => 5,
            CoralVariant::Brain => 6,
            CoralVariant::Bubble => 7,
            CoralVariant::Fire => 8,
            CoralVariant::Horn => 9,
        }
    }
}

