#![recursion_limit = "600"]
#![allow(clippy::identity_op)]
#![allow(clippy::collapsible_if)]

extern crate steven_shared as shared;

use crate::shared::{Axis, Direction, Position};
use cgmath::Point3;
use collision::Aabb3;
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
                                $($fname,)?
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

        mod block_registration_functions {
            use super::*;
            $(
                #[allow(non_snake_case)]
                pub fn $name(
                    blocks_flat: &mut Vec<Option<Block>>,
                    blocks_hier: &mut Vec<Option<Block>>,
                    blocks_modded: &mut HashMap<String, [Option<Block>; 16]>,
                    flat_id: &mut usize,
                    last_internal_id: &mut usize,
                    hier_block_id: &mut usize,
                    ) {
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
                                    $($fname,)?
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
                            if !(*blocks_modded).contains_key(modid) {
                                (*blocks_modded).insert(modid.to_string(), [None; 16]);
                            }
                            let block_from_data = (*blocks_modded).get_mut(modid).unwrap();
                            block_from_data[hier_data] = Some(block);
                            continue
                        }

                        let vanilla_id =
                            if let Some(hier_data) = hier_data {
                                if internal_id != *last_internal_id {
                                    *hier_block_id += 1;
                                }
                                *last_internal_id = internal_id;
                                Some((*hier_block_id << 4) + hier_data)
                            } else {
                                None
                            };

                        let offset = block.get_flat_offset();
                        if let Some(offset) = offset {
                            let id = *flat_id + offset;
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

                            if (*blocks_flat).len() <= id {
                                (*blocks_flat).resize(id + 1, None);
                            }
                            if (*blocks_flat)[id].is_none() {
                                (*blocks_flat)[id] = Some(block);
                            } else {
                                panic!(
                                    "Tried to register {:#?} to {} but {:#?} was already registered",
                                    block,
                                    id,
                                    (*blocks_flat)[id]
                                );
                            }
                        }

                        if let Some(vanilla_id) = vanilla_id {
                            /*
                            if offset.is_none() {
                                debug!("(no flat) block state = {:?} hierarchical {}:{}", block, vanilla_id >> 4, vanilla_id & 0xF);
                            }
                            */

                            if (*blocks_hier).len() <= vanilla_id {
                                (*blocks_hier).resize(vanilla_id + 1, None);
                            }
                            if (*blocks_hier)[vanilla_id].is_none() {
                                (*blocks_hier)[vanilla_id] = Some(block);
                            } else {
                                panic!(
                                    "Tried to register {:#?} to {} but {:#?} was already registered",
                                    block,
                                    vanilla_id,
                                    (*blocks_hier)[vanilla_id]
                                );
                            }
                        }
                    }

                    #[allow(unused_assignments)]
                    {
                        *flat_id += (last_offset + 1) as usize;
                    }
                }
            )+
        }

        lazy_static! {
            static ref VANILLA_ID_MAP: VanillaIDMap = {
                let mut blocks_flat = vec![];
                let mut blocks_hier = vec![];
                let mut blocks_modded: HashMap<String, [Option<Block>; 16]> = HashMap::new();
                let mut flat_id = 0;
                let mut last_internal_id = 0;
                let mut hier_block_id = 0;

                $(
                    block_registration_functions::$name(&mut blocks_flat,
                                                        &mut blocks_hier,
                                                        &mut blocks_modded,
                                                        &mut flat_id,
                                                        &mut last_internal_id,
                                                        &mut hier_block_id);
                )+

                VanillaIDMap { flat: blocks_flat, hier: blocks_hier, modded: blocks_modded }
            };
        }
    );
}

#[derive(Clone, Copy)]
pub enum TintType {
    Default,
    Color { r: u8, g: u8, b: u8 },
    Grass,
    Foliage,
}

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
    Stone {
        props {
            variant: StoneVariant = [
                StoneVariant::Normal,
                StoneVariant::Granite,
                StoneVariant::SmoothGranite,
                StoneVariant::Diorite,
                StoneVariant::SmoothDiorite,
                StoneVariant::Andesite,
                StoneVariant::SmoothAndesite
            ],
        },
        data Some(variant.data()),
        model { ("minecraft", variant.as_string() ) },
    }
    Grass {
        props {
            snowy: bool = [false, true],
        },
        data { if snowy { None } else { Some(0) } },
        offset { if snowy { Some(0) } else { Some(1) } },
        model { ("minecraft", "grass") },
        variant format!("snowy={}", snowy),
        tint TintType::Grass,
        update_state (world, pos) => Block::Grass{snowy: is_snowy(world, pos)},
    }
    Dirt {
        props {
            snowy: bool = [false, true],
            variant: DirtVariant = [
                DirtVariant::Normal,
                DirtVariant::Coarse,
                DirtVariant::Podzol
            ],
        },
        data if !snowy { Some(variant.data()) } else { None },
        offset {
            if variant == DirtVariant::Podzol {
                Some(variant.data() + if snowy { 0 } else { 1 })
            } else {
                if snowy {
                    None
                } else {
                    Some(variant.data())
                }
            }
        },
        model { ("minecraft", variant.as_string()) },
        variant {
            if variant == DirtVariant::Podzol {
                format!("snowy={}", snowy)
            } else {
                "normal".to_owned()
            }
        },
        update_state (world, pos) => if variant == DirtVariant::Podzol {
            Block::Dirt{snowy: is_snowy(world, pos), variant}
        } else {
            Block::Dirt{snowy, variant}
        },
    }
    Cobblestone {
        props {},
        model { ("minecraft", "cobblestone") },
    }
    Planks {
        props {
            variant: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle,
                TreeVariant::Acacia,
                TreeVariant::DarkOak
            ],
        },
        data Some(variant.plank_data()),
        model { ("minecraft", format!("{}_planks", variant.as_string()) ) },
    }
    Sapling {
        props {
            variant: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle,
                TreeVariant::Acacia,
                TreeVariant::DarkOak
            ],
            stage: u8 = [0, 1],
        },
        data Some(variant.plank_data() | ((stage as usize) << 3)),
        offset Some((variant.plank_data() << 1) | (stage as usize)),
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_sapling", variant.as_string()) ) },
        variant format!("stage={}", stage),
        collision vec![],
    }
    Bedrock {
        props {},
        model { ("minecraft", "bedrock") },
    }
    FlowingWater {
        props {
            level: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(level as usize),
        offset None,
        material Material {
            absorbed_light: 2,
            ..material::TRANSPARENT
        },
        model { ("minecraft", "flowing_water") },
        collision vec![],
    }
    Water {
        props {
            level: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(level as usize),
        material Material {
            absorbed_light: 2,
            ..material::TRANSPARENT
        },
        model { ("minecraft", "water") },
        collision vec![],
    }
    FlowingLava {
        props {
            level: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(level as usize),
        offset None,
        material Material {
            absorbed_light: 15,
            emitted_light: 15,
            ..material::NON_SOLID
        },
        model { ("minecraft", "flowing_lava") },
        collision vec![],
    }
    Lava {
        props {
            level: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(level as usize),
        material Material {
            absorbed_light: 15,
            emitted_light: 15,
            ..material::NON_SOLID
        },
        model { ("minecraft", "lava") },
        collision vec![],
    }
    Sand {
        props {
            red: bool = [false, true],
        },
        data Some(if red { 1 } else { 0 }),
        model { ("minecraft", if red { "red_sand" } else { "sand" } ) },
    }
    Gravel {
        props {},
        model { ("minecraft", "gravel") },
    }
    GoldOre {
        props {},
        model { ("minecraft", "gold_ore") },
    }
    IronOre {
        props {},
        model { ("minecraft", "iron_ore") },
    }
    CoalOre {
        props {},
        model { ("minecraft", "coal_ore") },
    }
    Log {
        props {
            variant: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle,
                TreeVariant::Acacia,
                TreeVariant::DarkOak,
                TreeVariant::StrippedSpruce,
                TreeVariant::StrippedBirch,
                TreeVariant::StrippedJungle,
                TreeVariant::StrippedAcacia,
                TreeVariant::StrippedDarkOak,
                TreeVariant::StrippedOak
            ],
            axis: Axis = [Axis::Y, Axis::Z, Axis::X, Axis::None],
        },
        data match variant {
            TreeVariant::Oak | TreeVariant::Spruce | TreeVariant::Birch | TreeVariant::Jungle =>
                Some(variant.data() | (axis.index() << 2)),
            _ => None,
        },
        offset match axis {
            Axis::None => None,
            Axis::X => Some(variant.offset() * 3 + 0),
            Axis::Y => Some(variant.offset() * 3 + 1),
            Axis::Z => Some(variant.offset() * 3 + 2),
        },
        model { ("minecraft", format!("{}_log", variant.as_string()) ) },
        variant format!("axis={}", axis.as_string()),
    }
    Wood {
        props {
            variant: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle,
                TreeVariant::Acacia,
                TreeVariant::DarkOak,
                TreeVariant::StrippedSpruce,
                TreeVariant::StrippedBirch,
                TreeVariant::StrippedJungle,
                TreeVariant::StrippedAcacia,
                TreeVariant::StrippedDarkOak,
                TreeVariant::StrippedOak
            ],
            axis: Axis = [Axis::X, Axis::Y, Axis::Z],
        },
        data None::<usize>,
        offset Some(variant.offset() * 3 + axis.index()),
        model { ("minecraft", format!("{}_wood", variant.as_string()) ) },
        variant format!("axis={}", axis.as_string()),
    }
    Leaves {
        props {
            variant: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle,
                TreeVariant::Acacia,
                TreeVariant::DarkOak
            ],
            decayable: bool = [false, true],
            check_decay: bool = [false, true],
            distance: u8 = [1, 2, 3, 4, 5, 6, 7],
        },
        data match variant {
            TreeVariant::Oak | TreeVariant::Spruce | TreeVariant::Birch | TreeVariant::Jungle =>
                if distance == 1 {
                    Some(variant.data()
                          | (if decayable { 0x4 } else { 0x0 })
                          | (if check_decay { 0x8 } else { 0x0 }))
                } else {
                    None
                },
            _ => None,
        },
        offset if check_decay {
            None
        } else {
            Some(variant.offset() * (7 * 2) + ((distance as usize - 1) << 1) + (if decayable { 0 } else { 1 }))
        },
        material material::LEAVES,
        model { ("minecraft", format!("{}_leaves", variant.as_string()) ) },
        tint TintType::Foliage,
    }
    Sponge {
        props {
            wet: bool = [false, true],
        },
        data Some(if wet { 1 } else { 0 }),
        model { ("minecraft", "sponge") },
        variant format!("wet={}", wet),
    }
    Glass {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "glass") },
    }
    LapisOre {
        props {},
        model { ("minecraft", "lapis_ore") },
    }
    LapisBlock {
        props {},
        model { ("minecraft", "lapis_block") },
    }
    Dispenser {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            triggered: bool = [false, true],
        },
        data Some(facing.index() | (if triggered { 0x8 } else { 0x0 })),
        offset Some((facing.offset() << 1) | (if triggered { 0 } else { 1 })),
        model { ("minecraft", "dispenser") },
        variant format!("facing={}", facing.as_string()),
    }
    Sandstone {
        props {
            variant: SandstoneVariant = [
                SandstoneVariant::Normal,
                SandstoneVariant::Chiseled,
                SandstoneVariant::Smooth
            ],
        },
        data Some(variant.data()),
        model { ("minecraft", variant.as_string() ) },
    }
    NoteBlock {
        props {
            instrument: NoteBlockInstrument = [
                NoteBlockInstrument::Harp,
                NoteBlockInstrument::BaseDrum,
                NoteBlockInstrument::Snare,
                NoteBlockInstrument::Hat,
                NoteBlockInstrument::Bass,
                NoteBlockInstrument::Flute,
                NoteBlockInstrument::Bell,
                NoteBlockInstrument::Guitar,
                NoteBlockInstrument::Chime,
                NoteBlockInstrument::Xylophone
            ],
            note: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24],
            powered: bool = [true, false],
        },
        data if instrument == NoteBlockInstrument::Harp && note == 0 && powered { Some(0) } else { None },
        offset Some(instrument.offset() * (25 * 2) + ((note as usize) << 1) + if powered { 0 } else { 1 }),
        model { ("minecraft", "noteblock") },
    }
    Bed {
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            occupied: bool = [false, true],
            part: BedPart = [BedPart::Head, BedPart::Foot],
        },
        data if color != ColoredVariant::White { None } else { Some(facing.horizontal_index()
                  | (if occupied { 0x4 } else { 0x0 })
                  | (if part == BedPart::Head { 0x8 } else { 0x0 }))},
        offset Some(color.data() * (2 * 2 * 4)
                  + (facing.horizontal_offset() * (2 * 2))
                  + (if occupied { 0 } else { 2 })
                  + (if part == BedPart::Head { 0 } else { 1 })),
        material material::NON_SOLID,
        model { ("minecraft", "bed") },
        variant format!("facing={},part={}", facing.as_string(), part.as_string()),
        collision vec![Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 9.0/16.0, 1.0))],
    }
    GoldenRail {
        props {
            powered: bool = [false, true],
            shape: RailShape = [
                RailShape::NorthSouth,
                RailShape::EastWest,
                RailShape::AscendingNorth,
                RailShape::AscendingSouth,
                RailShape::AscendingEast,
                RailShape::AscendingWest
            ],
        },
        data Some(shape.data() | (if powered { 0x8 } else { 0x0 })),
        offset Some(shape.data() + (if powered { 0 } else { 6 })),
        material material::NON_SOLID,
        model { ("minecraft", "golden_rail") },
        variant format!("powered={},shape={}", powered, shape.as_string()),
        collision vec![],
    }
    DetectorRail {
        props {
            powered: bool = [false, true],
            shape: RailShape = [
                RailShape::NorthSouth,
                RailShape::EastWest,
                RailShape::AscendingNorth,
                RailShape::AscendingSouth,
                RailShape::AscendingEast,
                RailShape::AscendingWest
            ],
        },
        data Some(shape.data() | (if powered { 0x8 } else { 0x0 })),
        offset Some(shape.data() + (if powered { 0 } else { 6 })),
        material material::NON_SOLID,
        model { ("minecraft", "detector_rail") },
        variant format!("powered={},shape={}", powered, shape.as_string()),
        collision vec![],
    }
    StickyPiston {
        props {
            extended: bool = [false, true],
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index() | (if extended { 0x8 } else { 0x0 })),
        offset Some(facing.offset() + (if extended { 0 } else { 6 })),
        material Material {
            should_cull_against: !extended,
            ..material::NON_SOLID
        },
        model { ("minecraft", "sticky_piston") },
        variant format!("extended={},facing={}", extended, facing.as_string()),
        collision piston_collision(extended, facing),
    }
    Web {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "web") },
        collision vec![],
    }
    TallGrass {
        props {
            variant: TallGrassVariant = [
                TallGrassVariant::DeadBush,
                TallGrassVariant::TallGrass,
                TallGrassVariant::Fern
            ],
        },
        data Some(variant.data()),
        material material::NON_SOLID,
        model { ("minecraft", variant.as_string() ) },
        tint TintType::Grass,
        collision vec![],
    }
    Seagrass {
        props {},
        data None::<usize>,
        offset Some(0),
        material material::NON_SOLID,
        model { ("minecraft", "seagrass") },
        collision vec![],
    }
    TallSeagrass {
        props {
            half: TallSeagrassHalf = [
                TallSeagrassHalf::Upper,
                TallSeagrassHalf::Lower
            ],
        },
        data None::<usize>,
        offset Some(half.offset()),
        material material::NON_SOLID,
        model { ("minecraft", "tall_seagrass") },
        collision vec![],
    }
    DeadBush {
        props {},
        offset None,
        material material::NON_SOLID,
        model { ("minecraft", "dead_bush") },
        collision vec![],
    }
    Piston {
        props {
            extended: bool = [false, true],
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index() | (if extended { 0x8 } else { 0x0 })),
        offset Some(facing.offset() + (if extended { 0 } else { 6 })),
        material Material {
            should_cull_against: !extended,
            ..material::NON_SOLID
        },
        model { ("minecraft", "piston") },
        variant format!("extended={},facing={}", extended, facing.as_string()),
        collision piston_collision(extended, facing),
    }
    PistonHead {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            short: bool = [false, true],
            variant: PistonType = [PistonType::Normal, PistonType::Sticky],
        },
        data if !short { Some(facing.index() | if variant == PistonType::Sticky { 0x8 } else { 0x0 })} else { None },
        offset Some(facing.offset() * 4 +
                    (if short { 0 } else { 2 }) +
                    (if variant == PistonType::Normal { 0 } else { 1 })),
        material material::NON_SOLID,
        model { ("minecraft", "piston_head") },
        variant format!("facing={},short={},type={}", facing.as_string(), short, variant.as_string()),
        collision {
            let (min_x, min_y, min_z, max_x, max_y, max_z) = match facing {
                Direction::Up => (3.0/8.0, -0.25, 3.0/8.0, 5.0/8.0, 0.75, 5.0/8.0),
                Direction::Down => (3.0/8.0, 0.25, 3.0/8.0, 5.0/8.0, 1.25, 0.625),
                Direction::North => (3.0/8.0, 3.0/8.0, 0.25, 5.0/8.0, 5.0/8.0, 1.25),
                Direction::South => (3.0/8.0, 3.0/8.0, -0.25, 5.0/8.0, 5.0/8.0, 0.75),
                Direction::West => (0.25, 3.0/8.0, 3.0/8.0, 1.25, 5.0/8.0, 5.0/8.0),
                Direction::East => (-0.25, 3.0/8.0, 3.0/8.0, 0.75, 5.0/8.0, 5.0/8.0),
                _ => unreachable!(),
            };

            vec![Aabb3::new(
                Point3::new(min_x, min_y, min_z),
                Point3::new(max_x, max_y, max_z)
            )]
        },
    }
    Wool {
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data Some(color.data()),
        model { ("minecraft", format!("{}_wool", color.as_string()) ) },
    }
    ThermalExpansionRockwool {
        modid "ThermalExpansion:Rockwool",
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data Some(color.data()),
        model { ("minecraft", format!("{}_wool", color.as_string()) ) },
    }
    ThermalFoundationRockwool {
        modid "thermalfoundation:rockwool",
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data Some(color.data()),
        model { ("minecraft", format!("{}_wool", color.as_string()) ) },
    }
    PistonExtension {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            variant: PistonType = [PistonType::Normal, PistonType::Sticky],
        },
        data if facing == Direction::Up && variant == PistonType::Normal { Some(0) } else { None },
        offset Some(facing.offset() * 2 + (if variant == PistonType::Normal { 0 } else { 1 })),
        material material::INVISIBLE,
        model { ("minecraft", "piston_extension") },
    }
    YellowFlower {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "dandelion") },
        collision vec![],
    }
    RedFlower {
        props {
            variant: RedFlowerVariant = [
                RedFlowerVariant::Poppy,
                RedFlowerVariant::BlueOrchid,
                RedFlowerVariant::Allium,
                RedFlowerVariant::AzureBluet,
                RedFlowerVariant::RedTulip,
                RedFlowerVariant::OrangeTulip,
                RedFlowerVariant::WhiteTulip,
                RedFlowerVariant::PinkTulip,
                RedFlowerVariant::OxeyeDaisy
            ],
        },
        data Some(variant.data()),
        material material::NON_SOLID,
        model { ("minecraft", variant.as_string()) },
        collision vec![],
    }
    BrownMushroom {
        props {},
        material Material {
            emitted_light: 1,
            ..material::NON_SOLID
        },
        model { ("minecraft", "brown_mushroom") },
        collision vec![],
    }
    RedMushroom {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "red_mushroom") },
        collision vec![],
    }
    GoldBlock {
        props {},
        model { ("minecraft", "gold_block") },
    }
    IronBlock {
        props {},
        model { ("minecraft", "iron_block") },
    }
    DoubleStoneSlab {
        props {
            seamless: bool = [false, true],
            variant: StoneSlabVariant = [
                StoneSlabVariant::Stone,
                StoneSlabVariant::Sandstone,
                StoneSlabVariant::PetrifiedWood,
                StoneSlabVariant::Cobblestone,
                StoneSlabVariant::Brick,
                StoneSlabVariant::StoneBrick,
                StoneSlabVariant::NetherBrick,
                StoneSlabVariant::Quartz
            ],
        },
        data {
            let data = if seamless {
                match variant {
                    StoneSlabVariant::Stone => 8,
                    StoneSlabVariant::Sandstone => 9,
                    StoneSlabVariant::Quartz => 15,
                    _ => return None,
                }
            } else {
                variant.data()
            };

            Some(data)
        },
        offset None,
        model { ("minecraft", format!("{}_double_slab", variant.as_string()) ) },
        variant if seamless { "all" } else { "normal" },
    }
    StoneSlab {
        props {
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            variant: StoneSlabVariant = [
                StoneSlabVariant::Stone,
                StoneSlabVariant::Sandstone,
                StoneSlabVariant::PetrifiedWood,
                StoneSlabVariant::Cobblestone,
                StoneSlabVariant::Brick,
                StoneSlabVariant::StoneBrick,
                StoneSlabVariant::NetherBrick,
                StoneSlabVariant::Quartz
            ],
        },
        data Some(variant.data() | (if half == BlockHalf::Top { 0x8 } else { 0x0 })),
        offset None,
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("half={}", half.as_string()),
        collision slab_collision(half),
    }
    BrickBlock {
        props {},
        model { ("minecraft", "brick_block") },
    }
    TNT {
        props {
            explode: bool = [false, true],
        },
        data Some(if explode { 1 } else { 0 }),
        offset Some(if explode { 0 } else { 1 }),
        model { ("minecraft", "tnt") },
    }
    BookShelf {
        props {},
        model { ("minecraft", "bookshelf") },
    }
    MossyCobblestone {
        props {},
        model { ("minecraft", "mossy_cobblestone") },
    }
    Obsidian {
        props {},
        model { ("minecraft", "obsidian") },
    }
    Torch {
        props {
            facing: Direction = [
                Direction::East,
                Direction::West,
                Direction::South,
                Direction::North,
                Direction::Up
            ],
        },
        data {
            Some(match facing {
                Direction::East => 1,
                Direction::West => 2,
                Direction::South => 3,
                Direction::North => 4,
                Direction::Up => 5,
                _ => unreachable!(),
            })
        },
        offset {
            Some(match facing {
                Direction::Up => 0,
                Direction::North => 1,
                Direction::South => 2,
                Direction::West => 3,
                Direction::East => 4,
                _ => unreachable!(),
            })
        },
        material Material {
            emitted_light: 14,
            ..material::NON_SOLID
        },
        model { ("minecraft", "torch") },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
    }
    Fire {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            up: bool = [false, true],
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
        },
        data if !up && !north && !south && !west && !east { Some(age as usize) } else { None },
        offset Some(
            if west  { 0 } else { 1<<0 } |
            if up    { 0 } else { 1<<1 } |
            if south { 0 } else { 1<<2 } |
            if north { 0 } else { 1<<3 } |
            if east  { 0 } else { 1<<4 } |
            ((age as usize) << 5)),
        material Material {
            emitted_light: 15,
            ..material::NON_SOLID
        },
        model { ("minecraft", "fire") },
        collision vec![],
        update_state (world, pos) => {
            Fire{
                age,
                up: can_burn(world, pos.shift(Direction::Up)),
                north: can_burn(world, pos.shift(Direction::North)),
                south: can_burn(world, pos.shift(Direction::South)),
                west: can_burn(world, pos.shift(Direction::West)),
                east: can_burn(world, pos.shift(Direction::East))
            }
        },
        multipart (key, val) => match key {
            "up" => up == (val == "true"),
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    MobSpawner {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "mob_spawner") },
    }
    OakStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "oak_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::OakStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    Chest {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            type_: ChestType = [
                ChestType::Single,
                ChestType::Left,
                ChestType::Right
            ],
            waterlogged: bool = [true, false],
        },
        data if type_ == ChestType::Single && !waterlogged { Some(facing.index()) } else { None },
        offset Some(if waterlogged { 0 } else { 1 } +
            type_.offset() * 2 +
            facing.horizontal_offset() * (2 * 3)),
        material material::NON_SOLID,
        model { ("minecraft", "chest") },
    }
    RedstoneWire {
        props {
            north: RedstoneSide = [RedstoneSide::None, RedstoneSide::Side, RedstoneSide::Up],
            south: RedstoneSide = [RedstoneSide::None, RedstoneSide::Side, RedstoneSide::Up],
            west: RedstoneSide = [RedstoneSide::None, RedstoneSide::Side, RedstoneSide::Up],
            east: RedstoneSide = [RedstoneSide::None, RedstoneSide::Side, RedstoneSide::Up],
            power: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data {
            if north == RedstoneSide::None && south == RedstoneSide::None
                && west == RedstoneSide::None && east == RedstoneSide::None {
                Some(power as usize)
            } else {
                None
            }
        },
        offset Some(
            west.offset() +
            south.offset() * 3 +
            (power as usize) * (3 * 3) +
            north.offset() * (3 * 3 * 16) +
            east.offset() * (3 * 3 * 16 * 3)),
        material material::NON_SOLID,
        model { ("minecraft", "redstone_wire") },
        tint TintType::Color{r: ((255.0 / 30.0) * (f64::from(power)) + 14.0) as u8, g: 0, b: 0},
        collision vec![],
        update_state (world, pos) => Block::RedstoneWire {
            north: can_connect_redstone(world, pos, Direction::North),
            south: can_connect_redstone(world, pos, Direction::South),
            west: can_connect_redstone(world, pos, Direction::West),
            east: can_connect_redstone(world, pos, Direction::East),
            power
        },
        multipart (key, val) => match key {
            "north" => val.contains(north.as_string()),
            "south" => val.contains(south.as_string()),
            "west" => val.contains(west.as_string()),
            "east" => val.contains(east.as_string()),
            _ => false,
        },
    }
    DiamondOre {
        props {},
        model { ("minecraft", "diamond_ore") },
    }
    DiamondBlock {
        props {},
        model { ("minecraft", "diamond_block") },
    }
    CraftingTable {
        props {},
        model { ("minecraft", "crafting_table") },
    }
    Wheat {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5, 6, 7],
        },
        data Some(age as usize),
        material material::NON_SOLID,
        model { ("minecraft", "wheat") },
        variant format!("age={}", age),
        collision vec![],
    }
    Farmland {
        props {
            moisture: u8 = [0, 1, 2, 3, 4, 5, 6, 7],
        },
        data Some(moisture as usize),
        material material::NON_SOLID,
        model { ("minecraft", "farmland") },
        variant format!("moisture={}", moisture),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 15.0/16.0, 1.0)
        )],
    }
    Furnace {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            lit: bool = [true, false],
        },
        data if !lit { Some(facing.index()) } else { None },
        offset Some(if lit { 0 } else { 1 } + facing.horizontal_offset() * 2),
        model { ("minecraft", "furnace") },
        variant format!("facing={}", facing.as_string()),
    }
    FurnaceLit {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset None,
        material Material {
            emitted_light: 13,
            ..material::SOLID
        },
        model { ("minecraft", "lit_furnace") },
        variant format!("facing={}", facing.as_string()),
    }
    StandingSign {
        props {
            rotation: Rotation = [
                Rotation::South,
                Rotation::SouthSouthWest,
                Rotation::SouthWest,
                Rotation::WestSouthWest,
                Rotation::West,
                Rotation::WestNorthWest,
                Rotation::NorthWest,
                Rotation::NorthNorthWest,
                Rotation::North,
                Rotation::NorthNorthEast,
                Rotation::NorthEast,
                Rotation::EastNorthEast,
                Rotation::East,
                Rotation::EastSouthEast,
                Rotation::SouthEast,
                Rotation::SouthSouthEast
            ],
            waterlogged: bool = [true, false],
        },
        data if !waterlogged { Some(rotation.data()) } else { None },
        offset Some(rotation.data() * 2 + if waterlogged { 0 } else { 1 }),
        material material::INVISIBLE,
        model { ("minecraft", "standing_sign") },
        collision vec![],
    }
    WoodenDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        offset door_offset(facing, half, hinge, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "wooden_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::WoodenDoor{facing, half, hinge, open, powered}
        },
    }
    Ladder {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            waterlogged: bool = [true, false],
        },
        data if !waterlogged { Some(facing.index()) } else { None },
        offset Some(if waterlogged { 0 } else { 1 } + facing.horizontal_offset() * 2),
        material material::NON_SOLID,
        model { ("minecraft", "ladder") },
        variant format!("facing={}", facing.as_string()),
    }
    Rail {
        props {
            shape: RailShape = [
                RailShape::NorthSouth,
                RailShape::EastWest,
                RailShape::NorthEast,
                RailShape::NorthWest,
                RailShape::SouthEast,
                RailShape::SouthWest,
                RailShape::AscendingNorth,
                RailShape::AscendingSouth,
                RailShape::AscendingEast,
                RailShape::AscendingWest
            ],
        },
        data Some(shape.data()),
        material material::NON_SOLID,
        model { ("minecraft", "rail") },
        variant format!("shape={}", shape.as_string()),
        collision vec![],
    }
    StoneStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "stone_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::StoneStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    WallSign {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            waterlogged: bool = [true, false],
        },
        data if !waterlogged { Some(facing.index()) } else { None },
        offset Some(if waterlogged { 0 } else { 1 } + facing.horizontal_offset() * 2),
        material material::INVISIBLE,
        model { ("minecraft", "wall_sign") },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
    }
    Lever {
        props {
            face: AttachedFace = [
                AttachedFace::Floor,
                AttachedFace::Wall,
                AttachedFace::Ceiling
            ],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            powered: bool = [false, true],
        },
        data face.data_with_facing_and_powered(facing, powered),
        offset Some(face.offset() * (4 * 2) + facing.horizontal_offset() * 2 + if powered { 0 } else { 1 }),
        material material::NON_SOLID,
        model { ("minecraft", "lever") },
        variant format!("facing={},powered={}", face.variant_with_facing(facing), powered),
        collision vec![],
    }
    StonePressurePlate {
        props {
            powered: bool = [false, true],
        },
        data Some(if powered { 1 } else { 0 }),
        offset Some(if powered { 0 } else { 1 }),
        material material::NON_SOLID,
        model { ("minecraft", "stone_pressure_plate") },
        variant format!("powered={}", powered),
        collision vec![],
    }
    IronDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        offset door_offset(facing, half, hinge, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "iron_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::IronDoor{facing, half, hinge, open, powered}
        },
    }
    WoodenPressurePlate {
        props {
            wood: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle,
                TreeVariant::Acacia,
                TreeVariant::DarkOak
            ],
            powered: bool = [false, true],
        },
        data if wood == TreeVariant::Oak { Some(if powered { 1 } else { 0 }) } else { None },
        offset Some(wood.offset() * 2 + if powered { 0 } else { 1 }),
        material material::NON_SOLID,
        model { ("minecraft", "wooden_pressure_plate") },
        variant format!("powered={}", powered),
        collision vec![],
    }
    RedstoneOre {
        props {
            lit: bool = [true, false],
        },
        data if !lit { Some(0) } else { None },
        offset Some(if lit { 0 } else { 1 }),
        model { ("minecraft", if lit { "lit_redstone_ore" } else { "redstone_ore" }) },
    }
    RedstoneOreLit {
        props {},
        offset None,
        material Material {
            emitted_light: 9,
            ..material::SOLID
        },
        model { ("minecraft", "lit_redstone_ore") },
    }
    RedstoneTorchUnlit {
        props {
            facing: Direction = [
                Direction::East,
                Direction::West,
                Direction::South,
                Direction::North,
                Direction::Up
            ],
        },
        data {
            Some(match facing {
                Direction::East => 1,
                Direction::West => 2,
                Direction::South => 3,
                Direction::North => 4,
                Direction::Up => 5,
                _ => unreachable!(),
            })
        },
        offset None,
        material material::NON_SOLID,
        model { ("minecraft", "unlit_redstone_torch") },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
    }
    RedstoneTorchLit {
        props {
            facing: Direction = [
                Direction::East,
                Direction::West,
                Direction::South,
                Direction::North,
                Direction::Up
            ],
        },
        data {
            Some(match facing {
                Direction::East => 1,
                Direction::West => 2,
                Direction::South => 3,
                Direction::North => 4,
                Direction::Up => 5,
                _ => unreachable!(),
            })
        },
        offset None,
        material Material {
            emitted_light: 7,
            ..material::NON_SOLID
        },
        model { ("minecraft", "redstone_torch") },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
    }
    RedstoneTorchStanding {
        props {
            lit: bool = [true, false],
        },
        data None::<usize>,
        offset Some(if lit { 0 } else { 1 }),
        material material::NON_SOLID,
        model { ("minecraft", if lit { "redstone_torch" } else { "unlit_redstone_torch" }) },
        variant "facing=up",
        collision vec![],
    }
    RedstoneTorchWall {
        props {
            facing: Direction = [
                Direction::East,
                Direction::West,
                Direction::South,
                Direction::North
            ],
            lit: bool = [true, false],
        },
        data None::<usize>,
        offset Some(if lit { 0 } else { 1 } + facing.horizontal_offset() * 2),
        material Material {
            emitted_light: 7,
            ..material::NON_SOLID
        },
        model { ("minecraft", if lit { "redstone_torch" } else { "unlit_redstone_torch" }) },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
    }
    StoneButton {
        props {
            face: AttachedFace = [
                AttachedFace::Floor,
                AttachedFace::Wall,
                AttachedFace::Ceiling
            ],
            facing: Direction = [
                Direction::East,
                Direction::West,
                Direction::South,
                Direction::North
            ],
            powered: bool = [false, true],
        },
        data face.data_with_facing_and_powered(facing, powered),
        offset Some(face.offset() * (4 * 2) + facing.horizontal_offset() * 2 + if powered { 0 } else { 1 }),
        material material::NON_SOLID,
        model { ("minecraft", "stone_button") },
        variant format!("facing={},powered={}", face.variant_with_facing(facing), powered),
    }
    SnowLayer {
        props {
            layers: u8 = [1, 2, 3, 4, 5, 6, 7, 8],
        },
        data Some(layers as usize - 1),
        material material::NON_SOLID,
        model { ("minecraft", "snow_layer") },
        variant format!("layers={}", layers),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, (f64::from(layers) - 1.0)/8.0, 1.0),
        )],
    }
    Ice {
        props {},
        material Material {
            absorbed_light: 2,
            ..material::TRANSPARENT
        },
        model { ("minecraft", "ice") },
    }
    Snow {
        props {},
        model { ("minecraft", "snow") },
    }
    Cactus {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(age as usize),
        material material::NON_SOLID,
        model { ("minecraft", "cactus") },
        collision vec![Aabb3::new(
            Point3::new(1.0/16.0, 0.0, 1.0/16.0),
            Point3::new(1.0 - (1.0/16.0), 1.0 - (1.0/16.0), 1.0 - (1.0/16.0))
        )],
    }
    Clay {
        props {},
        model { ("minecraft", "clay") },
    }
    Reeds {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(age as usize),
        material material::NON_SOLID,
        model { ("minecraft", "reeds") },
        tint TintType::Foliage,
        collision vec![],
    }
    Jukebox {
        props {
            has_record: bool = [false, true],
        },
        data Some(if has_record { 1 } else { 0 }),
        offset Some(if has_record { 0 } else { 1 }),
        model { ("minecraft", "jukebox") },
    }
    Fence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            waterlogged: bool = [false, true],
        },
        data if !north && !south && !east && !west && !waterlogged { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
            if waterlogged { 0 } else { 1<<1 } +
            if south { 0 } else { 1<<2 } +
            if north { 0 } else { 1<<3 } +
            if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::Fence{north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    PumpkinFace {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            without_face: bool = [false, true],
        },
        data Some(facing.horizontal_index() | (if without_face { 0x4 } else { 0x0 })),
        offset None,
        model { ("minecraft", "pumpkin") },
        variant format!("facing={}", facing.as_string()),
    }
    Pumpkin {
        props {},
        data None::<usize>,
        offset Some(0),
        model { ("minecraft", "pumpkin") },
    }
    Netherrack {
        props {},
        model { ("minecraft", "netherrack") },
    }
    SoulSand {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "soul_sand") },
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 7.0/8.0, 1.0)
        )],
    }
    Glowstone {
        props {},
        material Material {
            emitted_light: 15,
            ..material::SOLID
        },
        model { ("minecraft", "glowstone") },
    }
    Portal {
        props {
            axis: Axis = [Axis::X, Axis::Z],
        },
        data Some(axis.index()),
        offset Some(axis.index() - 1),
        material Material {
            emitted_light: 11,
            ..material::TRANSPARENT
        },
        model { ("minecraft", "portal") },
        variant format!("axis={}", axis.as_string()),
        collision vec![],
    }
    PumpkinCarved {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data None::<usize>,
        offset Some(facing.horizontal_offset()),
        material Material {
            emitted_light: 15,
            ..material::SOLID
        },
        model { ("minecraft", "carved_pumpkin") },
        variant format!("facing={}", facing.as_string()),
    }
    PumpkinLit {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            without_face: bool = [false, true],
        },
        data Some(facing.horizontal_index() | (if without_face { 0x4 } else { 0x0 })),
        offset if without_face { None } else { Some(facing.horizontal_offset()) },
        material Material {
            emitted_light: 15,
            ..material::SOLID
        },
        model { ("minecraft", "lit_pumpkin") },
        variant format!("facing={}", facing.as_string()),
    }
    Cake {
        props {
            bites: u8 = [0, 1, 2, 3, 4, 5, 6],
        },
        data Some(bites as usize),
        material material::NON_SOLID,
        model { ("minecraft", "cake") },
        variant format!("bites={}", bites),
        collision vec![Aabb3::new(
            Point3::new((1.0 + (f64::from(bites) * 2.0)) / 16.0, 0.0, 1.0/16.0),
            Point3::new(1.0 - (1.0/16.0), 0.5, 1.0 - (1.0/16.0))
        )],
    }
    Repeater {
        props {
            delay: u8 = [1, 2, 3, 4],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            locked: bool = [false, true],
            powered: bool = [true, false],
        },
        data if powered { None } else { if !locked { Some(facing.horizontal_index() | (delay as usize - 1) << 2) } else { None } },
        offset Some(if powered { 0 } else { 1<<0 } +
            if locked { 0 } else { 1<<1 } +
            facing.horizontal_offset() * (2 * 2) +
            ((delay - 1) as usize) * (2 * 2 * 4)),
        material material::NON_SOLID,
        model { ("minecraft", if powered { "powered_repeater" } else { "unpowered_repeater" }) },
        variant format!("delay={},facing={},locked={}", delay, facing.as_string(), locked),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0/8.0, 1.0)
        )],
        update_state (world, pos) => Repeater{delay, facing, locked: update_repeater_state(world, pos, facing), powered},
    }
    RepeaterPowered {
        props {
            delay: u8 = [1, 2, 3, 4],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            locked: bool = [false, true],
        },
        data if !locked { Some(facing.horizontal_index() | (delay as usize - 1) << 2) } else { None },
        offset None,
        material material::NON_SOLID,
        model { ("minecraft", "powered_repeater") },
        variant format!("delay={},facing={},locked={}", delay, facing.as_string(), locked),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0/8.0, 1.0)
        )],
        update_state (world, pos) => RepeaterPowered{delay, facing, locked: update_repeater_state(world, pos, facing)},
    }
    StainedGlass {
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data Some(color.data()),
        material material::TRANSPARENT,
        model { ("minecraft", format!("{}_stained_glass", color.as_string()) ) },
    }
    TrapDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            open: bool = [false, true],
            waterlogged: bool = [true, false],
            powered: bool = [true, false],
            wood: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle,
                TreeVariant::Acacia,
                TreeVariant::DarkOak
            ],
        },
        data if waterlogged || powered || wood != TreeVariant::Oak { None } else { Some(match facing {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            _ => unreachable!(),
        } | (if open { 0x4 } else { 0x0 }) | (if half == BlockHalf::Top { 0x8 } else { 0x0 }))},
        offset Some(if waterlogged { 0 } else { 1<<0 } +
            if powered { 0 } else { 1<<1 } +
            if open { 0 } else { 1<<2 } +
            if half == BlockHalf::Top { 0 } else { 1<<3 } +
            facing.horizontal_offset() * (2 * 2 * 2 * 2) +
            wood.offset() * (2 * 2 * 2 * 2 * 4)),
        material material::NON_SOLID,
        model { ("minecraft", "trapdoor") },
        variant format!("facing={},half={},open={}", facing.as_string(), half.as_string(), open),
        collision trapdoor_collision(facing, half, open),
    }
    MonsterEgg {
        props {
            variant: MonsterEggVariant = [
                MonsterEggVariant::Stone,
                MonsterEggVariant::Cobblestone,
                MonsterEggVariant::StoneBrick,
                MonsterEggVariant::MossyBrick,
                MonsterEggVariant::CrackedBrick,
                MonsterEggVariant::ChiseledBrick
            ],
        },
        data Some(variant.data()),
        model { ("minecraft", format!("{}_monster_egg", variant.as_string())) },
    }
    StoneBrick {
        props {
            variant: StoneBrickVariant = [
                StoneBrickVariant::Normal,
                StoneBrickVariant::Mossy,
                StoneBrickVariant::Cracked,
                StoneBrickVariant::Chiseled
            ],
        },
        data Some(variant.data()),
        model { ("minecraft", variant.as_string() ) },
    }
    BrownMushroomBlock {
        props {
            is_stem: bool = [true, false],
            west: bool = [true, false],
            up: bool = [true, false],
            south: bool = [true, false],
            north: bool = [true, false],
            east: bool = [true, false],
            down: bool = [true, false],
        },
        data mushroom_block_data(is_stem, west, up, south, north, east, down),
        offset mushroom_block_offset(is_stem, west, up, south, north, east, down),
        model { ("minecraft", "brown_mushroom_block") },
        variant format!("variant={}", mushroom_block_variant(is_stem, west, up, south, north, east, down)),
    }
    RedMushroomBlock {
        props {
            is_stem: bool = [true, false],
            west: bool = [true, false],
            up: bool = [true, false],
            south: bool = [true, false],
            north: bool = [true, false],
            east: bool = [true, false],
            down: bool = [true, false],
        },
        data mushroom_block_data(is_stem, west, up, south, north, east, down),
        offset mushroom_block_offset(is_stem, west, up, south, north, east, down),
        model { ("minecraft", "red_mushroom_block") },
        variant format!("variant={}", mushroom_block_variant(is_stem, west, up, south, north, east, down)),
    }
    MushroomStem {
        props {
            west: bool = [true, false],
            up: bool = [true, false],
            south: bool = [true, false],
            north: bool = [true, false],
            east: bool = [true, false],
            down: bool = [true, false],
        },
        data None::<usize>,
        offset mushroom_block_offset(false, west, up, south, north, east, down),
        model { ("minecraft", "mushroom_stem") },
        variant "variant=all_stem".to_string(),
    }
    IronBars {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            waterlogged: bool = [true, false],
        },
        data if !waterlogged && !north && !south && !west && !east { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if waterlogged { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "iron_bars") },
        collision pane_collision(north, south, east, west),
        update_state (world, pos) => {
            let f = |block| match block {
                Block::IronBars{..} => true,
                _ => false,
            };

            let (north, south, west, east) = can_connect_sides(world, pos, &f);
            Block::IronBars{north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    GlassPane {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            waterlogged: bool = [true, false],
        },
        data if !waterlogged && !north && !south && !west && !east { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if waterlogged { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "glass_pane") },
        collision pane_collision(north, south, east, west),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_glasspane);
            Block::GlassPane{north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    MelonBlock {
        props {},
        model { ("minecraft", "melon_block") },
    }
    AttachedPumpkinStem {
        props {
             facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data None::<usize>,
        offset Some(facing.horizontal_offset()),
        material material::NON_SOLID,
        model { ("minecraft", "pumpkin_stem") },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
        update_state (world, pos) => {
            let facing = match (world.get_block(pos.shift(Direction::East)), world.get_block(pos.shift(Direction::West)),
                                world.get_block(pos.shift(Direction::North)), world.get_block(pos.shift(Direction::South))) {
                (Block::Pumpkin{ .. }, _, _, _) => Direction::East,
                (_, Block::Pumpkin{ .. }, _, _) => Direction::West,
                (_, _, Block::Pumpkin{ .. }, _) => Direction::North,
                (_, _, _, Block::Pumpkin{ .. }) => Direction::South,
                _ => Direction::Up,
            };

            Block::AttachedPumpkinStem{facing}
        },
    }
    AttachedMelonStem {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data None::<usize>,
        offset Some(facing.horizontal_offset()),
        material material::NON_SOLID,
        model { ("minecraft", "melon_stem") },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
        update_state (world, pos) => {
            let facing = match (world.get_block(pos.shift(Direction::East)), world.get_block(pos.shift(Direction::West)),
                                world.get_block(pos.shift(Direction::North)), world.get_block(pos.shift(Direction::South))) {
                (Block::MelonBlock{ .. }, _, _, _) => Direction::East,
                (_, Block::MelonBlock{ .. }, _, _) => Direction::West,
                (_, _, Block::MelonBlock{ .. }, _) => Direction::North,
                (_, _, _, Block::MelonBlock{ .. }) => Direction::South,
                _ => Direction::Up,
            };

            Block::AttachedMelonStem{facing}
        },
    }
    PumpkinStem {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5, 6, 7],
            facing: Direction = [
                Direction::Up,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data if facing == Direction::Up { Some(age as usize) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "pumpkin_stem") },
        variant {
            if facing == Direction::Up {
                format!("age={},facing={}", age, facing.as_string())
            } else {
                format!("facing={}", facing.as_string())
            }
        },
        tint TintType::Color{r: age as u8 * 32, g: 255 - (age as u8 * 8), b: age as u8 * 4},
        collision vec![],
        update_state (world, pos) => {
            let facing = match (world.get_block(pos.shift(Direction::East)), world.get_block(pos.shift(Direction::West)),
                                world.get_block(pos.shift(Direction::North)), world.get_block(pos.shift(Direction::South))) {
                (Block::Pumpkin{ .. }, _, _, _) => Direction::East,
                (_, Block::Pumpkin{ .. }, _, _) => Direction::West,
                (_, _, Block::Pumpkin{ .. }, _) => Direction::North,
                (_, _, _, Block::Pumpkin{ .. }) => Direction::South,
                _ => Direction::Up,
            };

            Block::PumpkinStem{age, facing}
        },
    }
    MelonStem {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5, 6, 7],
            facing: Direction = [
                Direction::Up,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data if facing == Direction::North { Some(age as usize) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "melon_stem") },
        variant {
            if facing == Direction::Up {
                format!("age={},facing={}", age, facing.as_string())
            } else {
                format!("facing={}", facing.as_string())
            }
        },
        tint TintType::Color{r: age as u8 * 32, g: 255 - (age as u8 * 8), b: age as u8 * 4},
        collision vec![],
        update_state (world, pos) => {
            let facing = match (world.get_block(pos.shift(Direction::East)), world.get_block(pos.shift(Direction::West)),
                                world.get_block(pos.shift(Direction::North)), world.get_block(pos.shift(Direction::South))) {
                (Block::MelonBlock{ .. }, _, _, _) => Direction::East,
                (_, Block::MelonBlock{ .. }, _, _) => Direction::West,
                (_, _, Block::MelonBlock{ .. }, _) => Direction::North,
                (_, _, _, Block::MelonBlock{ .. }) => Direction::South,
                _ => Direction::Up,
            };

            Block::MelonStem{age, facing}
        },
    }
    Vine {
        props {
             up: bool = [false, true],
             south: bool = [false, true],
             west: bool = [false, true],
             north: bool = [false, true],
             east: bool = [false, true],
        },
        data if !up {
            Some((if south { 0x1 } else { 0x0 })
                | (if west { 0x2 } else { 0x0 })
                | (if north { 0x4 } else { 0x0 })
                | (if east { 0x8 } else { 0x0 }))
        } else {
            None
        },
        offset Some(if west { 0 } else { 1<<0 } +
                    if up { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "vine") },
        variant format!("east={},north={},south={},up={},west={}", east, north, south, up, west),
        tint TintType::Foliage,
        collision vec![],
        update_state (world, pos) => {
            let mat = world.get_block(pos.shift(Direction::Up)).get_material();
            let up = mat.renderable && (mat.should_cull_against || mat.never_cull /* Because leaves */);
            Vine{up, south, west, north, east}
        },
    }
    FenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        offset fence_gate_offset(facing, in_wall, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
        update_state (world, pos) => Block::FenceGate{
            facing,
            in_wall: fence_gate_update_state(world, pos, facing),
            open,
            powered
        },
    }
    BrickStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "brick_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::BrickStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    StoneBrickStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "stone_brick_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::StoneBrickStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    Mycelium {
        props {
            snowy: bool = [false, true],
        },
        data if snowy { None } else { Some(0) },
        offset Some(if snowy { 0 } else { 1 }),
        material material::SOLID,
        model { ("minecraft", "mycelium") },
        variant format!("snowy={}", snowy),
        update_state (world, pos) => Block::Mycelium{snowy: is_snowy(world, pos)},
    }
    Waterlily {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "waterlily") },
        tint TintType::Foliage,
        collision vec![Aabb3::new(
            Point3::new(1.0/16.0, 0.0, 1.0/16.0),
            Point3::new(15.0/16.0, 3.0/32.0, 15.0/16.0))
        ],
    }
    NetherBrick {
        props {},
        model { ("minecraft", "nether_brick") },
    }
    NetherBrickFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            waterlogged: bool = [true, false],
        },
        data if !north && !south && !west && !east && !waterlogged { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
            if waterlogged { 0 } else { 1<<1 } +
            if south { 0 } else { 1<<2 } +
            if north { 0 } else { 1<<3 } +
            if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "nether_brick_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let f = |block| match block {
                Block::NetherBrickFence{..} |
                Block::FenceGate{..} |
                Block::SpruceFenceGate{..} |
                Block::BirchFenceGate{..} |
                Block::JungleFenceGate{..} |
                Block::DarkOakFenceGate{..} |
                Block::AcaciaFenceGate{..} => true,
                _ => false,
            };

            let (north, south, west, east) = can_connect_sides(world, pos, &f);
            Block::NetherBrickFence{north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    NetherBrickStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "nether_brick_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::NetherBrickStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    NetherWart {
        props {
            age: u8 = [0, 1, 2, 3],
        },
        data Some(age as usize),
        material material::NON_SOLID,
        model { ("minecraft", "nether_wart") },
        variant format!("age={}", age),
        collision vec![],
    }
    EnchantingTable {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "enchanting_table") },
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.75, 1.0))
        ],
    }
    BrewingStand {
        props {
            has_bottle_0: bool = [false, true],
            has_bottle_1: bool = [false, true],
            has_bottle_2: bool = [false, true],
        },
        data Some((if has_bottle_0 { 0x1 } else { 0x0 })
                  | (if has_bottle_1 { 0x2 } else { 0x0 })
                  | (if has_bottle_2 { 0x4 } else { 0x0 })),
        offset Some(if has_bottle_0 { 0 } else { 1<<0 } +
                    if has_bottle_1 { 0 } else { 1<<1 } +
                    if has_bottle_2 { 0 } else { 1<<2 }),
        material Material {
            emitted_light: 1,
            ..material::NON_SOLID
        },
        model { ("minecraft", "brewing_stand") },
        multipart (key, val) => match key {
            "has_bottle_0" => (val == "true") == has_bottle_0,
            "has_bottle_1" => (val == "true") == has_bottle_1,
            "has_bottle_2" => (val == "true") == has_bottle_2,
            _ => false,
        },
    }
    Cauldron {
        props {
            level: u8 = [0, 1, 2, 3],
        },
        data Some(level as usize),
        material material::NON_SOLID,
        model { ("minecraft", "cauldron") },
        variant format!("level={}", level),
    }
    EndPortal {
        props {},
        material Material {
            emitted_light: 15,
            ..material::NON_SOLID
        },
        model { ("minecraft", "end_portal") },
        collision vec![],
    }
    EndPortalFrame {
        props {
            eye: bool = [false, true],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
        },
        data Some(facing.horizontal_index() | (if eye { 0x4 } else { 0x0 })),
        offset Some(facing.horizontal_offset() + (if eye { 0 } else { 4 })),
        material Material {
            emitted_light: 1,
            ..material::NON_SOLID
        },
        model { ("minecraft", "end_portal_frame") },
        variant format!("eye={},facing={}", eye, facing.as_string()),
        collision {
            let mut collision = vec![Aabb3::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 13.0/16.0, 1.0)
            )];

            if eye {
                collision.push(Aabb3::new(
                    Point3::new(5.0/16.0, 13.0/16.0, 5.0/16.0),
                    Point3::new(11.0/16.0, 1.0, 11.0/16.0)
                ));
            }

            collision
        },
    }
    EndStone {
        props {},
        model { ("minecraft", "end_stone") },
    }
    DragonEgg {
        props {},
        material Material {
            emitted_light: 1,
            ..material::NON_SOLID
        },
        model { ("minecraft", "dragon_egg") },
        collision vec![Aabb3::new(
            Point3::new(1.0/16.0, 0.0, 1.0/16.0),
            Point3::new(15.0/16.0, 1.0, 15.0/16.0)
        )],
    }
    RedstoneLamp {
        props {},
        model { ("minecraft", "redstone_lamp") },
    }
    RedstoneLampLit {
        props {},
        material Material {
            emitted_light: 15,
            ..material::NON_SOLID
        },
        model { ("minecraft", "lit_redstone_lamp") },
    }
    DoubleWoodenSlab {
        props {
            variant: WoodSlabVariant = [
                WoodSlabVariant::Oak,
                WoodSlabVariant::Spruce,
                WoodSlabVariant::Birch,
                WoodSlabVariant::Jungle,
                WoodSlabVariant::Acacia,
                WoodSlabVariant::DarkOak
            ],
        },
        data Some(variant.data()),
        offset None,
        model { ("minecraft", format!("{}_double_slab", variant.as_string()) ) },
    }
    WoodenSlab {
        props {
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            variant: WoodSlabVariant = [
                WoodSlabVariant::Oak,
                WoodSlabVariant::Spruce,
                WoodSlabVariant::Birch,
                WoodSlabVariant::Jungle,
                WoodSlabVariant::Acacia,
                WoodSlabVariant::DarkOak
            ],
        },
        data Some(variant.data() | (if half == BlockHalf::Top { 0x8 } else { 0x0 })),
        offset None,
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("half={}", half.as_string()),
        collision slab_collision(half),
    }
    Cocoa {
        props {
            age: u8 = [0, 1, 2],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index() | ((age as usize) << 2)),
        offset Some(facing.horizontal_offset() + ((age as usize) * 4)),
        material material::NON_SOLID,
        model { ("minecraft", "cocoa") },
        variant format!("age={},facing={}", age, facing.as_string()),
        collision {
            let i = 4.0 + f64::from(age) * 2.0;
            let j = 5.0 + f64::from(age) * 2.0;
            let f = i / 2.0;

            let (min_x, min_y, min_z, max_x, max_y, max_z) = match facing {
                Direction::North => (8.0 - f, 12.0 - j, 1.0, 8.0 + f, 12.0, 8.0 + i),
                Direction::South => (8.0 - f, 12.0 - j, 15.0 - i, 8.0 + f, 12.0, 15.0),
                Direction::West => (1.0, 12.0 - j, 8.0 - f, 1.0 + i, 12.0, 8.0 + f),
                Direction::East => (15.0 - i, 12.0 - j, 8.0 - f, 15.0, 12.0, 8.0 + f),
                _ => unreachable!(),
            };

            vec![Aabb3::new(
                Point3::new(min_x / 16.0, min_y / 16.0, min_z / 16.0),
                Point3::new(max_x / 16.0, max_y / 16.0, max_z / 16.0))
            ]
        },
    }
    SandstoneStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "sandstone_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::SandstoneStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    EmeraldOre {
        props {},
        material material::SOLID,
        model { ("minecraft", "emerald_ore") },
    }
    EnderChest {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            waterlogged: bool = [true, false],
        },
        data if waterlogged { None } else { Some(facing.index()) },
        offset Some(if waterlogged { 0 } else { 1 } + facing.horizontal_offset() * 2),
        material Material {
            emitted_light: 7,
            ..material::NON_SOLID
        },
        model { ("minecraft", "ender_chest") },
        variant format!("facing={}", facing.as_string()),
        collision vec![Aabb3::new(
            Point3::new(1.0/16.0, 0.0, 1.0/16.0),
            Point3::new(15.0/16.0, 7.0/8.0, 15.0/16.0)
        )],
    }
    TripwireHook {
        props {
            attached: bool = [false, true],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            powered: bool = [false, true],
        },
        data Some(facing.horizontal_index()
                  | (if attached { 0x4 } else { 0x0 })
                  | (if powered { 0x8 } else { 0x0 })),
        offset Some(if powered { 0 } else { 1 } +
                    facing.horizontal_offset() * 2 +
                    if attached { 0 } else { 2 * 4 }),
        material material::NON_SOLID,
        model { ("minecraft", "tripwire_hook") },
        variant format!("attached={},facing={},powered={}", attached, facing.as_string(), powered),
        collision vec![],
    }
    Tripwire {
        props {
            powered: bool = [false, true],
            attached: bool = [false, true],
            disarmed: bool = [false, true],
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            mojang_cant_even: bool = [false, true],
        },
        data if !north && !south && !east && !west {
            Some((if powered { 0x1 } else { 0x0 })
                 | (if attached { 0x4 } else { 0x0 })
                 | (if disarmed { 0x8 } else { 0x0 })
                 | (if mojang_cant_even { 0x2 } else { 0x0 }))
        } else {
            None
        },
        offset if mojang_cant_even {
            None
        } else {
            Some(if west { 0 } else { 1<<0 } +
                 if south { 0 } else { 1<<1 } +
                 if powered { 0 } else { 1<<2 } +
                 if north { 0 } else { 1<<3 } +
                 if east { 0 } else { 1<<4 } +
                 if disarmed { 0 } else { 1<<5 } +
                 if attached { 0 } else { 1<<6 })
        },
        material material::TRANSPARENT,
        model { ("minecraft", "tripwire") },
        variant format!("attached={},east={},north={},south={},west={}", attached, east, north, south, west),
        collision vec![],
        update_state (world, pos) => {
            let f = |dir| {
                match world.get_block(pos.shift(dir)) {
                    Block::TripwireHook{facing, ..} => facing.opposite() == dir,
                    Block::Tripwire{..} => true,
                    _ => false,
                }
            };

            Tripwire{
                powered,
                attached,
                disarmed,
                north: f(Direction::North),
                south: f(Direction::South),
                west: f(Direction::West),
                east: f(Direction::East),
                mojang_cant_even
            }
        },
    }
    EmeraldBlock {
        props {},
        model { ("minecraft", "emerald_block") },
    }
    SpruceStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "spruce_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::SpruceStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    BirchStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "birch_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::BirchStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    JungleStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "jungle_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::JungleStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    CommandBlock {
        props {
            conditional: bool = [false, true],
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index() | (if conditional { 0x8 } else { 0x0 })),
        offset Some(facing.offset() + (if conditional { 0 } else { 6 })),
        model { ("minecraft", "command_block") },
        variant format!("conditional={},facing={}", conditional, facing.as_string()),
    }
    Beacon {
        props {},
        material Material {
            emitted_light: 15,
            ..material::NON_SOLID
        },
        model { ("minecraft", "beacon") },
    }
    CobblestoneWall {
        props {
            up: bool = [false, true],
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            variant: CobblestoneWallVariant = [
                CobblestoneWallVariant::Normal,
                CobblestoneWallVariant::Mossy
            ],
            waterlogged: bool = [true, false],
        },
        data if !north && !south && !east && !west && !up && !waterlogged { Some(variant.data()) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if waterlogged { 0 } else { 1<<1 } +
                    if up { 0 } else { 1<<2 } +
                    if south { 0 } else { 1<<3 } +
                    if north { 0 } else { 1<<4 } +
                    if east { 0 } else { 1<<5 } +
                    if variant == CobblestoneWallVariant::Normal { 0 } else { 1<<6 }),
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_wall", variant.as_string())) },
        update_state (world, pos) => {
            let f = |block| match block {
                Block::CobblestoneWall{..} |
                Block::FenceGate{..} |
                Block::SpruceFenceGate{..} |
                Block::BirchFenceGate{..} |
                Block::JungleFenceGate{..} |
                Block::DarkOakFenceGate{..} |
                Block::AcaciaFenceGate{..} => true,
                _ => false,
            };

            let (north, south, west, east) = can_connect_sides(world, pos, &f);
            let up = !(match world.get_block(pos.shift(Direction::Up)) {
                Block::Air{..} => true,
                _ => false,
            }) || !((north && south && !west && !east) || (!north && !south && west && east));
            Block::CobblestoneWall{up, north, south, west, east, variant, waterlogged}
        },
        multipart (key, val) => match key {
            "up" => up == (val == "true"),
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "east" => east == (val == "true"),
            "west" => west == (val == "true"),
            _ => false,
        },
    }
    FlowerPot {
        props {
            contents: FlowerPotVariant = [
                FlowerPotVariant::Empty,
                FlowerPotVariant::Poppy,
                FlowerPotVariant::Dandelion,
                FlowerPotVariant::OakSapling,
                FlowerPotVariant::SpruceSapling,
                FlowerPotVariant::BirchSapling,
                FlowerPotVariant::JungleSapling,
                FlowerPotVariant::RedMushroom,
                FlowerPotVariant::BrownMushroom,
                FlowerPotVariant::Cactus,
                FlowerPotVariant::DeadBush,
                FlowerPotVariant::Fern,
                FlowerPotVariant::AcaciaSapling,
                FlowerPotVariant::DarkOakSapling,
                FlowerPotVariant::BlueOrchid,
                FlowerPotVariant::Allium,
                FlowerPotVariant::AzureBluet,
                FlowerPotVariant::RedTulip,
                FlowerPotVariant::OrangeTulip,
                FlowerPotVariant::WhiteTulip,
                FlowerPotVariant::PinkTulip,
                FlowerPotVariant::Oxeye
            ],
            legacy_data: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data if contents == FlowerPotVariant::Empty { Some(legacy_data as usize) } else { None },
        offset if legacy_data != 0 { None } else { Some(contents.offset()) },
        material material::NON_SOLID,
        model { ("minecraft", "flower_pot") },
    }
    Carrots {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5, 6, 7],
        },
        data Some(age as usize),
        material material::NON_SOLID,
        model { ("minecraft", "carrots") },
        variant format!("age={}", age),
        collision vec![],
    }
    Potatoes {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5, 6, 7],
        },
        data Some(age as usize),
        material material::NON_SOLID,
        model { ("minecraft", "potatoes") },
        variant format!("age={}", age),
        collision vec![],
    }
    WoodenButton {
        props {
            face: AttachedFace = [
                AttachedFace::Floor,
                AttachedFace::Wall,
                AttachedFace::Ceiling
            ],
            facing: Direction = [
                Direction::East,
                Direction::West,
                Direction::South,
                Direction::North
            ],
            powered: bool = [false, true],
            variant: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle,
                TreeVariant::Acacia,
                TreeVariant::DarkOak
            ],
        },
        data if variant == TreeVariant::Oak { face.data_with_facing_and_powered(facing, powered) } else { None },
        offset Some(variant.offset() * (3 * 4 * 2) + face.offset() * (4 * 2) + facing.horizontal_offset() * 2 + if powered { 0 } else { 1 }),
        material material::NON_SOLID,
        model { ("minecraft", "wooden_button") },
        variant format!("facing={},powered={}", face.variant_with_facing(facing), powered),
    }
    SkullSkeletonWall {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            nodrop: bool = [false, true],
        },
        data if !nodrop { Some(facing.index()) } else { None },
        offset if !nodrop && facing != Direction::Up { Some(facing.horizontal_offset()) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "skull") },
        variant format!("facing={},nodrop={}", facing.as_string(), nodrop),
        collision {
            let (min_x, min_y, min_z, max_x, max_y, max_z) = match facing {
                Direction::Up => (0.25, 0.0, 0.25, 0.75, 0.5, 0.75),
                Direction::North => (0.25, 0.25, 0.5, 0.75, 0.75, 1.0),
                Direction::South => (0.25, 0.25, 0.0, 0.75, 0.75, 0.5),
                Direction::West => (0.5, 0.25, 0.25, 1.0, 0.75, 0.75),
                Direction::East => (0.0, 0.25, 0.25, 0.5, 0.75, 0.75),
                _ => unreachable!(),
            };

            vec![Aabb3::new(
                Point3::new(min_x, min_y, min_z),
                Point3::new(max_x, max_y, max_z)
            )]
        },
    }
    SkullSkeleton
    {
        props {
            rotation: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data None::<usize>,
        offset Some(rotation as usize),
        material material::NON_SOLID,
        model { ("minecraft", "skull") },
        collision {
            let (min_x, min_y, min_z, max_x, max_y, max_z) = (0.25, 0.0, 0.25, 0.75, 0.5, 0.75);

            vec![Aabb3::new(
                Point3::new(min_x, min_y, min_z),
                Point3::new(max_x, max_y, max_z)
            )]
        },
    }
    SkullWitherSkeletonWall {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data None::<usize>,
        offset Some(facing.horizontal_offset()),
        material material::NON_SOLID,
        model { ("minecraft", "skull") },
        collision {
            let (min_x, min_y, min_z, max_x, max_y, max_z) = match facing {
                Direction::North => (0.25, 0.25, 0.5, 0.75, 0.75, 1.0),
                Direction::South => (0.25, 0.25, 0.0, 0.75, 0.75, 0.5),
                Direction::West => (0.5, 0.25, 0.25, 1.0, 0.75, 0.75),
                Direction::East => (0.0, 0.25, 0.25, 0.5, 0.75, 0.75),
                _ => unreachable!(),
            };

            vec![Aabb3::new(
                Point3::new(min_x, min_y, min_z),
                Point3::new(max_x, max_y, max_z)
            )]
        },
    }
    SkullWitherSkeleton {
        props {
            rotation: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data None::<usize>,
        offset Some(rotation as usize),
        material material::NON_SOLID,
        model { ("minecraft", "skull") },
        collision {
            let (min_x, min_y, min_z, max_x, max_y, max_z) = (0.25, 0.0, 0.25, 0.75, 0.5, 0.75);

            vec![Aabb3::new(
                Point3::new(min_x, min_y, min_z),
                Point3::new(max_x, max_y, max_z)
            )]
        },
    }
    ZombieWallHead {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data None::<usize>,
        offset Some(facing.horizontal_offset()),
        material material::NON_SOLID,
        model { ("minecraft", "zombie_wall_head") },
    }
    ZombieHead {
        props {
            rotation: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data None::<usize>,
        offset Some(rotation as usize),
        material material::NON_SOLID,
        model { ("minecraft", "zombie_head") },
    }
    PlayerWallHead {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data None::<usize>,
        offset Some(facing.horizontal_offset()),
        material material::NON_SOLID,
        model { ("minecraft", "player_wall_head") },
    }
    PlayerHead {
        props {
            rotation: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data None::<usize>,
        offset Some(rotation as usize),
        material material::NON_SOLID,
        model { ("minecraft", "player_head") },
    }
    CreeperWallHead {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data None::<usize>,
        offset Some(facing.horizontal_offset()),
        material material::NON_SOLID,
        model { ("minecraft", "creeper_wall_head") },
    }
    CreeperHead {
        props {
            rotation: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data None::<usize>,
        offset Some(rotation as usize),
        material material::NON_SOLID,
        model { ("minecraft", "creeper_head") },
    }
    DragonWallHead {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data None::<usize>,
        offset Some(facing.horizontal_offset()),
        material material::NON_SOLID,
        model { ("minecraft", "dragon_wall_head") },
    }
    DragonHead {
        props {
            rotation: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data None::<usize>,
        offset Some(rotation as usize),
        material material::NON_SOLID,
        model { ("minecraft", "dragon_head") },
    }
    Anvil {
        props {
            damage: u8 = [0, 1, 2],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index() | (match damage { 0 => 0x0, 1 => 0x4, 2 => 0x8, _ => unreachable!() })),
        offset Some(facing.horizontal_offset() + (damage as usize) * 4),
        material material::NON_SOLID,
        model { ("minecraft", "anvil") },
        variant format!("damage={},facing={}", damage, facing.as_string()),
        collision match facing.axis() {
            Axis::Z => vec![Aabb3::new(
                Point3::new(1.0/8.0, 0.0, 0.0),
                Point3::new(7.0/8.0, 1.0, 1.0)
            )],
            Axis::X => vec![Aabb3::new(
                Point3::new(0.0, 0.0, 1.0/8.0),
                Point3::new(1.0, 1.0, 7.0/8.0)
            )],
            _ => unreachable!(),
        },
    }
    TrappedChest {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            type_: ChestType = [
                ChestType::Single,
                ChestType::Left,
                ChestType::Right
            ],
            waterlogged: bool = [true, false],
        },
        data if type_ == ChestType::Single && !waterlogged { Some(facing.index()) } else { None },
        offset Some(if waterlogged { 0 } else { 1 } +
            type_.offset() * 2 +
            facing.horizontal_offset() * (2 * 3)),
        material material::NON_SOLID,
        model { ("minecraft", "trapped_chest") },
        variant format!("facing={}", facing.as_string()),
        collision vec![Aabb3::new(
            Point3::new(1.0/16.0, 0.0, 1.0/16.0),
            Point3::new(15.0/16.0, 7.0/8.0, 15.0/16.0)
        )],
    }
    LightWeightedPressurePlate {
        props {
            power: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(power as usize),
        material material::NON_SOLID,
        model { ("minecraft", "light_weighted_pressure_plate") },
        variant format!("power={}", power),
        collision vec![],
    }
    HeavyWeightedPressurePlate {
        props {
            power: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(power as usize),
        material material::NON_SOLID,
        model { ("minecraft", "heavy_weighted_pressure_plate") },
        variant format!("power={}", power),
        collision vec![],
    }
    ComparatorUnpowered {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            mode: ComparatorMode = [ComparatorMode::Compare, ComparatorMode::Subtract],
            powered: bool = [false, true],
        },
        data Some(facing.horizontal_index()
                  | (if mode == ComparatorMode::Subtract { 0x4 } else { 0x0 })
                  | (if powered { 0x8 } else { 0x0 })),
        offset Some(if powered { 0 } else { 1<<0 } +
                    if mode == ComparatorMode::Compare { 0 } else { 1<<1 } +
                    facing.horizontal_offset() * (1<<2)),
        material material::NON_SOLID,
        model { ("minecraft", "unpowered_comparator") },
        variant format!("facing={},mode={},powered={}", facing.as_string(), mode.as_string(), powered),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0/8.0, 1.0)
        )],
    }
    ComparatorPowered {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            mode: ComparatorMode = [ComparatorMode::Compare, ComparatorMode::Subtract],
            powered: bool = [false, true],
        },
        data Some(facing.horizontal_index()
                  | (if mode == ComparatorMode::Subtract { 0x4 } else { 0x0 })
                  | (if powered { 0x8 } else { 0x0 })),
        offset None,
        material material::NON_SOLID,
        model { ("minecraft", "powered_comparator") },
        variant format!("facing={},mode={},powered={}", facing.as_string(), mode.as_string(), powered),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0/8.0, 1.0)
        )],
    }
    DaylightDetector {
        props {
            power: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            inverted: bool = [true, false],
        },
        data if inverted { None } else { Some(power as usize) },
        offset Some((power as usize) + if inverted { 0 } else { 16 }),
        material material::NON_SOLID,
        model { ("minecraft", "daylight_detector") },
        variant format!("power={}", power),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 3.0/8.0, 1.0)
        )],
    }
    RedstoneBlock {
        props {},
        model { ("minecraft", "redstone_block") },
    }
    QuartzOre {
        props {},
        model { ("minecraft", "quartz_ore") },
    }
    Hopper {
        props {
            enabled: bool = [false, true],
            facing: Direction = [
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index() | (if enabled { 0x8 } else { 0x0 })),
        offset Some(match facing {
            Direction::Down => 0,
            Direction::North => 1,
            Direction::South => 2,
            Direction::West => 3,
            Direction::East => 4,
            _ => unreachable!(),
        } + if enabled { 0 } else { 5 }),
        material material::NON_SOLID,
        model { ("minecraft", "hopper") },
        variant format!("facing={}", facing.as_string()),
    }
    QuartzBlock {
        props {
            variant: QuartzVariant = [
                QuartzVariant::Normal,
                QuartzVariant::Chiseled,
                QuartzVariant::PillarVertical,
                QuartzVariant::PillarNorthSouth,
                QuartzVariant::PillarEastWest
            ],
        },
        data Some(variant.data()),
        model { ("minecraft", match variant {
            QuartzVariant::Normal => "quartz_block",
            QuartzVariant::Chiseled => "chiseled_quartz_block",
            QuartzVariant::PillarVertical |
            QuartzVariant::PillarNorthSouth |
            QuartzVariant::PillarEastWest => "quartz_column",
        } ) },
        variant variant.as_string(),
    }
    QuartzStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "quartz_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::QuartzStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    ActivatorRail {
        props {
            shape: RailShape = [
                RailShape::NorthSouth,
                RailShape::EastWest,
                RailShape::AscendingNorth,
                RailShape::AscendingSouth,
                RailShape::AscendingEast,
                RailShape::AscendingWest
            ],
            powered: bool = [false, true],
        },
        data Some(shape.data() | (if powered { 0x8 } else { 0x0 })),
        offset Some(shape.data() + (if powered { 0 } else { 6 })),
        material material::NON_SOLID,
        model { ("minecraft", "activator_rail") },
        variant format!("powered={},shape={}", powered, shape.as_string()),
        collision vec![],
    }
    Dropper {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            triggered: bool = [false, true],
        },
        data Some(facing.index() | (if triggered { 0x8 } else { 0x0 })),
        offset Some(if triggered { 0 } else { 1 } + facing.offset() * 2),
        model { ("minecraft", "dropper") },
        variant format!("facing={}", facing.as_string()),
    }
    StainedHardenedClay {
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data Some(color.data()),
        model { ("minecraft", format!("{}_stained_hardened_clay", color.as_string()) ) },
    }
    StainedGlassPane {
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
            waterlogged: bool = [true, false],
        },
        data if !north && !south && !east && !west && !waterlogged { Some(color.data()) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if waterlogged { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 } +
                    color.data() * (1<<5)),
        material material::TRANSPARENT,
        model { ("minecraft", format!("{}_stained_glass_pane", color.as_string()) ) },
        collision pane_collision(north, south, east, west),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_glasspane);
            Block::StainedGlassPane{color, north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "east" => east == (val == "true"),
            "west" => west == (val == "true"),
            _ => false,
        },
    }
    Leaves2 {
        props {
            check_decay: bool = [false, true],
            decayable: bool = [false, true],
            variant: TreeVariant = [
                TreeVariant::Acacia,
                TreeVariant::DarkOak
            ],
        },
        data Some(variant.data()
                  | (if decayable { 0x4 } else { 0x0 })
                  | (if check_decay { 0x8 } else { 0x0 })),
        offset None,
        material material::LEAVES,
        model { ("minecraft", format!("{}_leaves", variant.as_string()) ) },
        tint TintType::Foliage,
    }
    Log2 {
        props {
            axis: Axis = [Axis::None, Axis::X, Axis::Y, Axis::Z],
            variant: TreeVariant = [
                TreeVariant::Acacia,
                TreeVariant::DarkOak
            ],
        },
        data Some(variant.data() | (axis.index() << 2)),
        offset None,
        model { ("minecraft", format!("{}_log", variant.as_string()) ) },
        variant format!("axis={}", axis.as_string()),
    }
    AcaciaStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "acacia_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::AcaciaStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    DarkOakStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "dark_oak_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::DarkOakStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    Slime {
        props {},
        material material::TRANSPARENT,
        model { ("minecraft", "slime") },
    }
    Barrier {
        props {},
        material material::INVISIBLE,
        model { ("minecraft", "barrier") },
    }
    IronTrapDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            open: bool = [false, true],
            waterlogged: bool = [true, false],
            powered: bool = [true, false],
        },
        data if waterlogged || powered { None } else { Some(match facing {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            _ => unreachable!(),
        } | (if open { 0x4 } else { 0x0 }) | (if half == BlockHalf::Top { 0x8 } else { 0x0 }))},
        offset Some(if waterlogged { 0 } else { 1<<0 } +
            if powered { 0 } else { 1<<1 } +
            if open { 0 } else { 1<<2 } +
            if half == BlockHalf::Top { 0 } else { 1<<3 } +
            facing.horizontal_offset() * (1<<4)),
        material material::NON_SOLID,
        model { ("minecraft", "iron_trapdoor") },
        variant format!("facing={},half={},open={}", facing.as_string(), half.as_string(), open),
        collision trapdoor_collision(facing, half, open),
    }
    Prismarine {
        props {
            variant: PrismarineVariant = [
                PrismarineVariant::Normal,
                PrismarineVariant::Brick,
                PrismarineVariant::Dark
            ],
        },
        data Some(variant.data()),
        model { ("minecraft", variant.as_string() ) },
    }
    PrismarineStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
            variant: PrismarineVariant = [
                    PrismarineVariant::Normal,
                    PrismarineVariant::Brick,
                    PrismarineVariant::Dark
                ],
        },
        data None::<usize>,
        offset Some(stair_offset(facing, half, shape, waterlogged).unwrap() + (2 * 5 * 2 * 4) * variant.data()),
        material material::NON_SOLID,
        model { ("minecraft", match variant {
            PrismarineVariant::Normal => "prismarine_stairs",
            PrismarineVariant::Brick => "prismarine_brick_stairs",
            PrismarineVariant::Dark => "dark_prismarine_stairs",
        }) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::PrismarineStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged, variant},
    }
    PrismarineSlab {
        props {
            type_: BlockHalf = [
                BlockHalf::Top,
                BlockHalf::Bottom,
                BlockHalf::Double
            ],
            waterlogged: bool = [true, false],
            variant: PrismarineVariant = [
                    PrismarineVariant::Normal,
                    PrismarineVariant::Brick,
                    PrismarineVariant::Dark
                ],
        },
        data None::<usize>,
        offset Some(if waterlogged { 0 } else { 1 } + type_.offset() * 2 + variant.data() * (2 * 3)),
        material material::NON_SOLID,
        model { ("minecraft", match variant {
            PrismarineVariant::Normal => "prismarine_slab",
            PrismarineVariant::Brick => "prismarine_brick_slab",
            PrismarineVariant::Dark => "dark_prismarine_slab",
        }) },
        variant format!("type={}", type_.as_string()),
        collision slab_collision(type_),
    }
    SeaLantern {
        props {},
        material Material {
            emitted_light: 15,
            ..material::SOLID
        },
        model { ("minecraft", "sea_lantern") },
    }
    HayBlock {
        props {
            axis: Axis = [Axis::X, Axis::Y, Axis::Z],
        },
        data Some(match axis { Axis::X => 0x4, Axis::Y => 0x0, Axis::Z => 0x8, _ => unreachable!() }),
        offset Some(match axis { Axis::X => 0, Axis::Y => 1, Axis::Z => 2, _ => unreachable!() }),
        model { ("minecraft", "hay_block") },
        variant format!("axis={}", axis.as_string()),
    }
    Carpet {
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data Some(color.data()),
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_carpet", color.as_string()) ) },
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0/16.0, 1.0)
        )],
    }
    HardenedClay {
        props {},
        model { ("minecraft", "hardened_clay") },
    }
    CoalBlock {
        props {},
        model { ("minecraft", "coal_block") },
    }
    PackedIce {
        props {},
        model { ("minecraft", "packed_ice") },
    }
    DoublePlant {
        props {
            half: BlockHalf = [BlockHalf::Lower, BlockHalf::Upper],
            variant: DoublePlantVariant = [
                DoublePlantVariant::Sunflower,
                DoublePlantVariant::Lilac,
                DoublePlantVariant::DoubleTallgrass,
                DoublePlantVariant::LargeFern,
                DoublePlantVariant::RoseBush,
                DoublePlantVariant::Peony
            ],
        },
        data Some(variant.data() | (if half == BlockHalf::Upper { 0x8 } else { 0x0 })),
        offset Some(half.offset() + variant.offset() * 2),
        material material::NON_SOLID,
        model { ("minecraft", variant.as_string()) },
        variant format!("half={}", half.as_string()),
        tint TintType::Foliage,
        collision vec![],
        update_state (world, pos) => {
            let (half, variant) = update_double_plant_state(world, pos, half, variant);
            Block::DoublePlant{half, variant}
        },
    }
    StandingBanner {
        props {
            rotation: Rotation = [
                Rotation::South,
                Rotation::SouthSouthWest,
                Rotation::SouthWest,
                Rotation::WestSouthWest,
                Rotation::West,
                Rotation::WestNorthWest,
                Rotation::NorthWest,
                Rotation::NorthNorthWest,
                Rotation::North,
                Rotation::NorthNorthEast,
                Rotation::NorthEast,
                Rotation::EastNorthEast,
                Rotation::East,
                Rotation::EastSouthEast,
                Rotation::SouthEast,
                Rotation::SouthSouthEast
            ],
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data if color != ColoredVariant::White { None } else { Some(rotation.data()) },
        offset Some(rotation.data() + color.data() * 16),
        material material::NON_SOLID,
        model { ("minecraft", "standing_banner") },
        variant format!("rotation={}", rotation.as_string()),
    }
    WallBanner {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data if color != ColoredVariant::White { None } else { Some(facing.index()) },
        offset Some(facing.horizontal_offset() + color.data() * 4),
        material material::NON_SOLID,
        model { ("minecraft", "wall_banner") },
        variant format!("facing={}", facing.as_string()),
    }
    DaylightDetectorInverted {
        props {
            power: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(power as usize),
        offset None,
        material material::NON_SOLID,
        model { ("minecraft", "daylight_detector_inverted") },
        variant format!("power={}", power),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 3.0/8.0, 1.0)
        )],
    }
    RedSandstone {
        props {
            variant: RedSandstoneVariant = [
                RedSandstoneVariant::Normal,
                RedSandstoneVariant::Chiseled,
                RedSandstoneVariant::Smooth
            ],
        },
        data Some(variant.data()),
        model { ("minecraft", variant.as_string()) },
    }
    RedSandstoneStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "red_sandstone_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::RedSandstoneStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    WoodenSlabFlat {
        props {
            type_: BlockHalf = [
                BlockHalf::Top,
                BlockHalf::Bottom,
                BlockHalf::Double
            ],
            waterlogged: bool = [true, false],
            variant: WoodSlabVariant = [
                WoodSlabVariant::Oak,
                WoodSlabVariant::Spruce,
                WoodSlabVariant::Birch,
                WoodSlabVariant::Jungle,
                WoodSlabVariant::Acacia,
                WoodSlabVariant::DarkOak
            ],
        },
        data None::<usize>,
        offset Some(if waterlogged { 0 } else { 1 } + type_.offset() * 2 + variant.data() * (2 * 3)),
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("type={}", type_.as_string()),
        collision slab_collision(type_),
    }
    StoneSlabFlat {
        props {
            type_: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom, BlockHalf::Double],
            variant: StoneSlabVariant = [
                StoneSlabVariant::Stone,
                StoneSlabVariant::Sandstone,
                StoneSlabVariant::PetrifiedWood,
                StoneSlabVariant::Cobblestone,
                StoneSlabVariant::Brick,
                StoneSlabVariant::StoneBrick,
                StoneSlabVariant::NetherBrick,
                StoneSlabVariant::Quartz,
                StoneSlabVariant::RedSandstone,
                StoneSlabVariant::Purpur
            ],
            waterlogged: bool = [true, false],
        },
        data None::<usize>,
        offset Some(if waterlogged { 0 } else { 1 } + type_.offset() * 2 + variant.offset() * (2 * 3)),
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("type={}", type_.as_string()),
        collision slab_collision(type_),
    }
    DoubleStoneSlab2 {
        props {
            seamless: bool = [false, true],
            variant: StoneSlabVariant = [
                StoneSlabVariant::RedSandstone
            ],
        },
        data Some(variant.data() | (if seamless { 0x8 } else { 0x0 })),
        offset None,
        material material::SOLID,
        model { ("minecraft", format!("{}_double_slab", variant.as_string()) ) },
        variant if seamless { "all" } else { "normal" },
    }
    StoneSlab2 {
        props {
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            variant: StoneSlabVariant = [StoneSlabVariant::RedSandstone],
        },
        data Some(variant.data() | (if half == BlockHalf::Top { 0x8 } else { 0x0 })),
        offset None,
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("half={}", half.as_string()),
        collision slab_collision(half),
    }
    SmoothStone {
        props {
            variant: StoneSlabVariant = [
                StoneSlabVariant::Stone,
                StoneSlabVariant::Sandstone,
                StoneSlabVariant::Quartz,
                StoneSlabVariant::RedSandstone
            ],
        },
        data None::<usize>,
        offset Some(match variant {
            StoneSlabVariant::Stone => 0,
            StoneSlabVariant::Sandstone => 1,
            StoneSlabVariant::Quartz => 2,
            StoneSlabVariant::RedSandstone => 3,
            _ => unreachable!(),
        }),
        model { ("minecraft", format!("smooth_{}", variant.as_string()) ) },
    }
    SpruceFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        offset fence_gate_offset(facing, in_wall, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "spruce_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
        update_state (world, pos) => Block::SpruceFenceGate{
            facing,
            in_wall: fence_gate_update_state(world, pos, facing),
            open,
            powered
        },
    }
    BirchFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        offset fence_gate_offset(facing, in_wall, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "birch_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
        update_state (world, pos) => Block::BirchFenceGate{
            facing,
            in_wall: fence_gate_update_state(world, pos, facing),
            open,
            powered
        },
    }
    JungleFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        offset fence_gate_offset(facing, in_wall, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "jungle_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
        update_state (world, pos) => Block::JungleFenceGate{
            facing,
            in_wall: fence_gate_update_state(world, pos, facing),
            open,
            powered
        },
    }
    DarkOakFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        offset fence_gate_offset(facing, in_wall, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "dark_oak_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
        update_state (world, pos) => Block::DarkOakFenceGate{
            facing,
            in_wall: fence_gate_update_state(world, pos, facing),
            open,
            powered
        },
    }
    AcaciaFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        offset fence_gate_offset(facing, in_wall, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "acacia_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
        update_state (world, pos) => Block::AcaciaFenceGate{
            facing,
            in_wall: fence_gate_update_state(world, pos, facing),
            open,
            powered
        },
    }
    SpruceFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            waterlogged: bool = [true, false],
        },
        data if !north && !south && !west && !east && !waterlogged { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if waterlogged { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "spruce_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::SpruceFence{north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    BirchFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            waterlogged: bool = [true, false],
        },
        data if !north && !south && !west && !east && !waterlogged { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if waterlogged { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "birch_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::BirchFence{north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    JungleFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            waterlogged: bool = [true, false],
        },
        data if !north && !south && !west && !east && !waterlogged { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if waterlogged { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "jungle_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::JungleFence{north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    DarkOakFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            waterlogged: bool = [true, false],
        },
        data if !north && !south && !west && !east && !waterlogged { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if waterlogged { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "dark_oak_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::DarkOakFence{north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    AcaciaFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
            waterlogged: bool = [true, false],
        },
        data if !north && !south && !west && !east && !waterlogged { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if waterlogged { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 }),
        material material::NON_SOLID,
        model { ("minecraft", "acacia_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::AcaciaFence{north, south, west, east, waterlogged}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    SpruceDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        offset door_offset(facing, half, hinge, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "spruce_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::SpruceDoor{facing, half, hinge, open, powered}
        },
    }
    BirchDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        offset door_offset(facing, half, hinge, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "birch_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::BirchDoor{facing, half, hinge, open, powered}
        },
    }
    JungleDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        offset door_offset(facing, half, hinge, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "jungle_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::JungleDoor{facing, half, hinge, open, powered}
        },
    }
    AcaciaDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        offset door_offset(facing, half, hinge, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "acacia_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::AcaciaDoor{facing, half, hinge, open, powered}
        },
    }
    DarkOakDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        offset door_offset(facing, half, hinge, open, powered),
        material material::NON_SOLID,
        model { ("minecraft", "dark_oak_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::DarkOakDoor{facing, half, hinge, open, powered}
        },
    }
    EndRod {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        material Material {
            emitted_light: 14,
            ..material::NON_SOLID
        },
        model { ("minecraft", "end_rod") },
        variant format!("facing={}", facing.as_string()),
        collision {
            match facing.axis() {
                Axis::Y => vec![Aabb3::new(
                    Point3::new(3.0/8.0, 0.0, 3.0/8.0),
                    Point3::new(5.0/8.0, 1.0, 5.0/8.0))
                ],
                Axis::Z => vec![Aabb3::new(
                    Point3::new(3.0/8.0, 3.0/8.0, 0.0),
                    Point3::new(5.0/8.0, 5.0/8.0, 1.0))
                ],
                Axis::X => vec![Aabb3::new(
                    Point3::new(0.0, 3.0/8.0, 3.0/8.0),
                    Point3::new(1.0, 5.0/8.0, 5.0/8.0))
                ],
                _ => unreachable!(),
            }
        },
    }
    ChorusPlant {
        props {
            up: bool = [false, true],
            down: bool = [false, true],
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
        },
        data if !up && !down && !north && !south && !west && !east { Some(0) } else { None },
        offset Some(if west { 0 } else { 1<<0 } +
                    if up { 0 } else { 1<<1 } +
                    if south { 0 } else { 1<<2 } +
                    if north { 0 } else { 1<<3 } +
                    if east { 0 } else { 1<<4 } +
                    if down { 0 } else { 1<<5 }),
        material material::NON_SOLID,
        model { ("minecraft", "chorus_plant") },
        collision {
            let mut collision = vec![Aabb3::new(
                Point3::new(3.0/16.0, 3.0/16.0, 3.0/16.0),
                Point3::new(13.0/16.0, 13.0/16.0, 13.0/16.0))
            ];

            if up {
                collision.push(Aabb3::new(
                    Point3::new(3.0/16.0, 13.0/16.0, 3.0/16.0),
                    Point3::new(13.0/16.0, 1.0, 13.0/16.0))
                );
            }

            if down {
                collision.push(Aabb3::new(
                    Point3::new(3.0/16.0, 0.0, 3.0/16.0),
                    Point3::new(13.0/16.0, 3.0/16.0, 13.0/16.0))
                );
            }

            if north {
                collision.push(Aabb3::new(
                    Point3::new(3.0/16.0, 3.0/16.0, 0.0),
                    Point3::new(13.0/16.0, 13.0/16.0, 3.0/16.0))
                );
            }

            if south {
                collision.push(Aabb3::new(
                    Point3::new(3.0/16.0, 3.0/16.0, 13.0/16.0),
                    Point3::new(13.0/16.0, 13.0/16.0, 1.0))
                );
            }

            if east {
                collision.push(Aabb3::new(
                    Point3::new(13.0/16.0, 3.0/16.0, 3.0/16.0),
                    Point3::new(1.0, 13.0/16.0, 13.0/16.0))
                );
            }

            if west {
                collision.push(Aabb3::new(
                    Point3::new(0.0, 3.0/16.0, 3.0/16.0),
                    Point3::new(3.0/16.0, 13.0/16.0, 13.0/16.0))
                );
            }

            collision
        },
        update_state (world, pos) => Block::ChorusPlant {
            up: match world.get_block(pos.shift(Direction::Up)) { Block::ChorusPlant{..} | Block::ChorusFlower{..} => true, _ => false,},
            down: match world.get_block(pos.shift(Direction::Down)) { Block::ChorusPlant{..} | Block::ChorusFlower{..} | Block::EndStone{..} => true, _ => false,},
            north: match world.get_block(pos.shift(Direction::North)) { Block::ChorusPlant{..} | Block::ChorusFlower{..} => true, _ => false,},
            south: match world.get_block(pos.shift(Direction::South)) { Block::ChorusPlant{..} | Block::ChorusFlower{..} => true, _ => false,},
            west: match world.get_block(pos.shift(Direction::West)) { Block::ChorusPlant{..} | Block::ChorusFlower{..} => true, _ => false,},
            east: match world.get_block(pos.shift(Direction::East)) { Block::ChorusPlant{..} | Block::ChorusFlower{..} => true, _ => false,},
        },
        multipart (key, val) => match key {
            "up" => up == (val == "true"),
            "down" => down == (val == "true"),
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "east" => east == (val == "true"),
            "west" => west == (val == "true"),
            _ => false,
        },
    }
    ChorusFlower {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5],
        },
        data Some(age as usize),
        material material::NON_SOLID,
        model { ("minecraft", "chorus_flower") },
        variant format!("age={}", age),
    }
    PurpurBlock {
        props {},
        model { ("minecraft", "purpur_block") },
    }
    PurpurPillar {
        props {
            axis: Axis = [Axis::X, Axis::Y, Axis::Z],
        },
        data Some(match axis { Axis::X => 0x4, Axis::Y => 0x0, Axis::Z => 0x8, _ => unreachable!() }),
        offset Some(match axis { Axis::X => 0, Axis::Y => 1, Axis::Z => 2, _ => unreachable!() }),
        model { ("minecraft", "purpur_pillar") },
        variant format!("axis={}", axis.as_string()),
    }
    PurpurStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft,
                StairShape::InnerRight,
                StairShape::OuterLeft,
                StairShape::OuterRight
            ],
            waterlogged: bool = [true, false],
        },
        data stair_data(facing, half, shape, waterlogged),
        offset stair_offset(facing, half, shape, waterlogged),
        material material::NON_SOLID,
        model { ("minecraft", "purpur_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        collision stair_collision(facing, shape, half),
        update_state (world, pos) => Block::PurpurStairs{facing, half, shape: update_stair_shape(world, pos, facing), waterlogged},
    }
    PurpurDoubleSlab {
        props {
            variant: StoneSlabVariant = [StoneSlabVariant::Purpur],
        },
        offset None,
        model { ("minecraft", format!("{}_double_slab", variant.as_string()) ) },
    }
    PurpurSlab {
        props {
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            variant: StoneSlabVariant = [StoneSlabVariant::Purpur],
        },
        data if half == BlockHalf::Top { Some(0x8) } else { Some(0) },
        offset None,
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("half={},variant=default", half.as_string()),
        collision slab_collision(half),
    }
    EndBricks {
        props {},
        model { ("minecraft", "end_bricks") },
    }
    Beetroots {
        props {
            age: u8 = [0, 1, 2, 3],
        },
        data Some(age as usize),
        material material::NON_SOLID,
        model { ("minecraft", "beetroots") },
        variant format!("age={}", age),
        collision vec![],
    }
    GrassPath {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "grass_path") },
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 15.0/16.0, 1.0)
        )],
    }
    EndGateway {
        props {},
        material material::NON_SOLID,
        model { ("minecraft", "end_gateway") },
        collision vec![],
    }
    RepeatingCommandBlock {
        props {
            conditional: bool = [false, true],
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index() | (if conditional { 0x8 } else { 0x0 })),
        offset Some(facing.offset() + (if conditional { 0 } else { 6 })),
        model { ("minecraft", "repeating_command_block") },
        variant format!("conditional={},facing={}", conditional, facing.as_string()),
    }
    ChainCommandBlock {
        props {
            conditional: bool = [false, true],
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index() | (if conditional { 0x8 } else { 0x0 })),
        offset Some(facing.offset() + (if conditional { 0 } else { 6 })),
        model { ("minecraft", "chain_command_block") },
        variant format!("conditional={},facing={}", conditional, facing.as_string()),
    }
    FrostedIce {
        props {
            age: u8 = [ 0, 1, 2, 3 ],
        },
        data if age == 0 { Some(0) } else { None },
        offset Some(age as usize),
        model { ("minecraft", "frosted_ice") },
    }
    MagmaBlock {
        props {},
        model { ("minecraft", "magma") },
    }
    NetherWartBlock {
        props {},
        model { ("minecraft", "nether_wart_block") },
    }
    RedNetherBrick {
        props {},
        model { ("minecraft", "red_nether_brick") },
    }
    BoneBlock {
        props {
            axis: Axis = [Axis::Y, Axis::Z, Axis::X],
        },
        data Some(axis.index() << 2),
        offset Some(match axis { Axis::X => 0, Axis::Y => 1, Axis::Z => 2, _ => unreachable!() }),
        model { ("minecraft", "bone_block") },
        variant format!("axis={}", axis.as_string()),
    }
    StructureVoid {
        props {},
        material material::Material {
            collidable: false,
            .. material::INVISIBLE
        },
        model { ("minecraft", "structure_void") },
        // TODO: a small hit box but no collision
        collision vec![],
    }
    Observer {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            powered: bool = [false, true],
        },
        data Some(facing.index() | (if powered { 0x8 } else { 0x0 })),
        offset Some(if powered { 0 } else { 1 } + facing.offset() * 2),
        model { ("minecraft", "observer") },
        variant format!("facing={},powered={}", facing.as_string(), powered),
    }
    // TODO: Shulker box textures (1.11+), since there is no model, we use wool for now
    // The textures should be built from textures/blocks/shulker_top_<color>.png
    // and textures/entity/shulker/shulker_<color>.png
    ShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data None::<usize>,
        offset Some(facing.offset()),
        model { ("minecraft", "sponge") },
    }
    WhiteShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "white_wool") },
    }
    OrangeShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "orange_wool") },
    }
    MagentaShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "magenta_wool") },
    }
    LightBlueShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "light_blue_wool") },
    }
    YellowShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "yellow_wool") },
    }
    LimeShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "lime_wool") },
    }
    PinkShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "pink_wool") },
    }
    GrayShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "gray_wool") },
    }
    LightGrayShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "light_gray_wool") },
    }
    CyanShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "cyan_wool") },
    }
    PurpleShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "purple_wool") },
    }
    BlueShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "blue_wool") },
    }
    BrownShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "brown_wool") },
    }
    GreenShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "green_wool") },
    }
    RedShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "red_wool") },
    }
    BlackShulkerBox {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::Down,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        offset Some(facing.offset()),
        model { ("minecraft", "black_wool") },
    }
    WhiteGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "white_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    OrangeGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "orange_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    MagentaGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "magenta_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    LightBlueGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "light_blue_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    YellowGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "yellow_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    LimeGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "lime_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    PinkGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "pink_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    GrayGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "gray_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    LightGrayGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "silver_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    CyanGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "cyan_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    PurpleGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "purple_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    BlueGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "blue_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    BrownGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "brown_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    GreenGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "green_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    RedGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "red_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    BlackGlazedTerracotta {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.horizontal_index()),
        offset Some(facing.horizontal_offset()),
        model { ("minecraft", "black_glazed_terracotta") },
        variant format!("facing={}", facing.as_string()),
    }
    Concrete {
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data Some(color.data()),
        model { ("minecraft", format!("{}_concrete", color.as_string()) ) },
    }
    ConcretePowder {
        props {
            color: ColoredVariant = [
                ColoredVariant::White,
                ColoredVariant::Orange,
                ColoredVariant::Magenta,
                ColoredVariant::LightBlue,
                ColoredVariant::Yellow,
                ColoredVariant::Lime,
                ColoredVariant::Pink,
                ColoredVariant::Gray,
                ColoredVariant::Silver,
                ColoredVariant::Cyan,
                ColoredVariant::Purple,
                ColoredVariant::Blue,
                ColoredVariant::Brown,
                ColoredVariant::Green,
                ColoredVariant::Red,
                ColoredVariant::Black
            ],
        },
        data Some(color.data()),
        model { ("minecraft", format!("{}_concrete_powder", color.as_string()) ) },
    }
    Kelp {
        props {
            age: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25],
        },
        data None::<usize>,
        offset Some(age as usize),
        model { ("minecraft", "kelp") },
    }
    KelpPlant {
        props {},
        data None::<usize>,
        offset Some(0),
        model { ("minecraft", "kelp_plant") },
    }
    DriedKelpBlock {
        props {},
        data None::<usize>,
        offset Some(0),
        model { ("minecraft", "dried_kelp_block") },
    }
    TurtleEgg {
        props {
            age: u8 = [1, 2, 3, 4],
            hatch: u8 = [0, 1, 2],
        },
        data None::<usize>,
        offset Some((hatch as usize) + ((age - 1) as usize) * 3),
        model { ("minecraft", "turtle_egg") },
    }
    CoralBlock {
        props {
            variant: CoralVariant = [
                CoralVariant::DeadTube,
                CoralVariant::DeadBrain,
                CoralVariant::DeadBubble,
                CoralVariant::DeadFire,
                CoralVariant::DeadHorn,
                CoralVariant::Tube,
                CoralVariant::Brain,
                CoralVariant::Bubble,
                CoralVariant::Fire,
                CoralVariant::Horn
            ],
        },
        data None::<usize>,
        offset Some(variant.offset()),
        model { ("minecraft", format!("{}_block", variant.as_string())) },
    }
    Coral {
        props {
            waterlogged: bool = [true, false],
            variant: CoralVariant = [
                CoralVariant::DeadTube,
                CoralVariant::DeadBrain,
                CoralVariant::DeadBubble,
                CoralVariant::DeadFire,
                CoralVariant::DeadHorn,
                CoralVariant::Tube,
                CoralVariant::Brain,
                CoralVariant::Bubble,
                CoralVariant::Fire,
                CoralVariant::Horn
            ],
        },
        data None::<usize>,
        offset Some(if waterlogged { 0 } else { 1 } + variant.offset() * 2),
        model { ("minecraft", variant.as_string()) },
    }
    CoralWallFan {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            waterlogged: bool = [true, false],
            variant: CoralVariant = [
                CoralVariant::DeadTube,
                CoralVariant::DeadBrain,
                CoralVariant::DeadBubble,
                CoralVariant::DeadFire,
                CoralVariant::DeadHorn,
                CoralVariant::Tube,
                CoralVariant::Brain,
                CoralVariant::Bubble,
                CoralVariant::Fire,
                CoralVariant::Horn
            ],
        },
        data None::<usize>,
        offset Some(if waterlogged { 0 } else { 1 } +
                    facing.horizontal_offset() * 2 +
                    variant.offset() * (2 * 4)),
        model { ("minecraft", format!("{}_wall_fan", variant.as_string())) },
    }
    CoralFan {
        props {
            waterlogged: bool = [true, false],
            variant: CoralVariant = [
                CoralVariant::DeadTube,
                CoralVariant::DeadBrain,
                CoralVariant::DeadBubble,
                CoralVariant::DeadFire,
                CoralVariant::DeadHorn,
                CoralVariant::Tube,
                CoralVariant::Brain,
                CoralVariant::Bubble,
                CoralVariant::Fire,
                CoralVariant::Horn
            ],
        },
        data None::<usize>,
        offset Some(if waterlogged { 0 } else { 1 } +
                    variant.offset() * 2),
        model { ("minecraft", format!("{}_fan", variant.as_string())) },
    }
    SeaPickle {
        props {
            age: u8 = [1, 2, 3, 4],
            waterlogged: bool = [true, false],
        },
        data None::<usize>,
        offset Some(if waterlogged { 0 } else { 1 } +
                    ((age - 1) as usize) * 2),
        model { ("minecraft", "sea_pickle") },
        variant format!("age={}", age),
    }
    BlueIce {
        props {},
        data None::<usize>,
        offset Some(0),
        model { ("minecraft", "blue_ice") },
    }
    Conduit {
        props {
            waterlogged: bool = [true, false],
        },
        data None::<usize>,
        offset Some(if waterlogged { 0 } else { 1 }),
        material material::NON_SOLID,
        model { ("minecraft", "conduit") },
    }
    VoidAir {
        props {},
        data None::<usize>,
        offset Some(0),
        material material::Material {
            collidable: false,
            .. material::INVISIBLE
        },
        model { ("minecraft", "air") },
        collision vec![],
    }
    CaveAir {
        props {},
        data None::<usize>,
        offset Some(0),
        material material::Material {
            collidable: false,
            .. material::INVISIBLE
        },
        model { ("minecraft", "air") },
        collision vec![],
    }
    BubbleColumn {
        props {
            drag: bool = [true, false],
        },
        data None::<usize>,
        offset Some(if drag { 0 } else { 1 }),
        model { ("minecraft", "bubble_column") },
    }
    Missing253 {
        props {},
        data Some(0),
        offset None,
        model { ("steven", "missing_block") },
    }
    Missing254 {
        props {},
        data Some(0),
        offset None,
        model { ("steven", "missing_block") },
    }
    StructureBlock {
        props {
            mode: StructureBlockMode = [
                StructureBlockMode::Save,
                StructureBlockMode::Load,
                StructureBlockMode::Corner,
                StructureBlockMode::Data
            ],
        },
        data Some(mode.data()),
        model { ("minecraft", "structure_block") },
        variant format!("mode={}", mode.as_string()),
    }

    Missing {
        props {},
        data None::<usize>,
        model { ("steven", "missing_block") },
    }
}

fn can_burn<W: WorldAccess>(world: &W, pos: Position) -> bool {
    match world.get_block(pos) {
        Block::Planks { .. }
        | Block::DoubleWoodenSlab { .. }
        | Block::WoodenSlab { .. }
        | Block::FenceGate { .. }
        | Block::SpruceFenceGate { .. }
        | Block::BirchFenceGate { .. }
        | Block::JungleFenceGate { .. }
        | Block::DarkOakFenceGate { .. }
        | Block::AcaciaFenceGate { .. }
        | Block::Fence { .. }
        | Block::SpruceFence { .. }
        | Block::BirchFence { .. }
        | Block::JungleFence { .. }
        | Block::DarkOakFence { .. }
        | Block::AcaciaFence { .. }
        | Block::OakStairs { .. }
        | Block::BirchStairs { .. }
        | Block::SpruceStairs { .. }
        | Block::JungleStairs { .. }
        | Block::AcaciaStairs { .. }
        | Block::DarkOakStairs { .. }
        | Block::Log { .. }
        | Block::Log2 { .. }
        | Block::Leaves { .. }
        | Block::Leaves2 { .. }
        | Block::BookShelf { .. }
        | Block::TNT { .. }
        | Block::TallGrass { .. }
        | Block::DoublePlant { .. }
        | Block::YellowFlower { .. }
        | Block::RedFlower { .. }
        | Block::DeadBush { .. }
        | Block::Wool { .. }
        | Block::Vine { .. }
        | Block::CoalBlock { .. }
        | Block::HayBlock { .. }
        | Block::Carpet { .. } => true,
        _ => false,
    }
}

fn is_snowy<W: WorldAccess>(world: &W, pos: Position) -> bool {
    match world.get_block(pos.shift(Direction::Up)) {
        Block::Snow { .. } | Block::SnowLayer { .. } => true,
        _ => false,
    }
}

fn can_connect_sides<F: Fn(Block) -> bool, W: WorldAccess>(
    world: &W,
    pos: Position,
    f: &F,
) -> (bool, bool, bool, bool) {
    (
        can_connect(world, pos.shift(Direction::North), f),
        can_connect(world, pos.shift(Direction::South), f),
        can_connect(world, pos.shift(Direction::West), f),
        can_connect(world, pos.shift(Direction::East), f),
    )
}

fn can_connect<F: Fn(Block) -> bool, W: WorldAccess>(world: &W, pos: Position, f: &F) -> bool {
    let block = world.get_block(pos);
    f(block) || (block.get_material().renderable && block.get_material().should_cull_against)
}

fn can_connect_fence(block: Block) -> bool {
    match block {
        Block::Fence { .. }
        | Block::SpruceFence { .. }
        | Block::BirchFence { .. }
        | Block::JungleFence { .. }
        | Block::DarkOakFence { .. }
        | Block::AcaciaFence { .. }
        | Block::FenceGate { .. }
        | Block::SpruceFenceGate { .. }
        | Block::BirchFenceGate { .. }
        | Block::JungleFenceGate { .. }
        | Block::DarkOakFenceGate { .. }
        | Block::AcaciaFenceGate { .. } => true,
        _ => false,
    }
}

fn can_connect_glasspane(block: Block) -> bool {
    match block {
        Block::Glass { .. }
        | Block::StainedGlass { .. }
        | Block::GlassPane { .. }
        | Block::StainedGlassPane { .. } => true,
        _ => false,
    }
}

fn can_connect_redstone<W: WorldAccess>(world: &W, pos: Position, dir: Direction) -> RedstoneSide {
    let shift_pos = pos.shift(dir);
    let block = world.get_block(shift_pos);

    if block.get_material().should_cull_against {
        let side_up = world.get_block(shift_pos.shift(Direction::Up));
        let up = world.get_block(pos.shift(Direction::Up));

        if match side_up {
            Block::RedstoneWire { .. } => true,
            _ => false,
        } && !up.get_material().should_cull_against
        {
            return RedstoneSide::Up;
        }

        return RedstoneSide::None;
    }

    let side_down = world.get_block(shift_pos.shift(Direction::Down));
    if match block {
        Block::RedstoneWire { .. } => true,
        _ => false,
    } || match side_down {
        Block::RedstoneWire { .. } => true,
        _ => false,
    } {
        return RedstoneSide::Side;
    }
    RedstoneSide::None
}

fn fence_gate_data(facing: Direction, in_wall: bool, open: bool, powered: bool) -> Option<usize> {
    if in_wall || powered {
        return None;
    }

    Some(facing.horizontal_index() | (if open { 0x4 } else { 0x0 }))
}

fn fence_gate_offset(facing: Direction, in_wall: bool, open: bool, powered: bool) -> Option<usize> {
    Some(
        if powered { 0 } else { 1 << 0 }
            + if open { 0 } else { 1 << 1 }
            + if in_wall { 0 } else { 1 << 2 }
            + facing.horizontal_offset() * (1 << 3),
    )
}

fn fence_gate_collision(facing: Direction, in_wall: bool, open: bool) -> Vec<Aabb3<f64>> {
    if open {
        return vec![];
    }

    let (min_x, min_y, min_z, max_x, max_y, max_z) = if in_wall {
        match facing.axis() {
            Axis::Z => (0.0, 0.0, 3.0 / 8.0, 1.0, 13.0 / 16.0, 5.0 / 8.0),
            Axis::X => (3.0 / 8.0, 0.0, 0.0, 5.0 / 8.0, 13.0 / 16.0, 1.0),
            _ => unreachable!(),
        }
    } else {
        match facing.axis() {
            Axis::Z => (0.0, 0.0, 3.0 / 8.0, 1.0, 1.0, 5.0 / 8.0),
            Axis::X => (3.0 / 8.0, 0.0, 0.0, 5.0 / 8.0, 1.0, 1.0),
            _ => unreachable!(),
        }
    };

    vec![Aabb3::new(
        Point3::new(min_x, min_y, min_z),
        Point3::new(max_x, max_y, max_z),
    )]
}

fn fence_gate_update_state<W: WorldAccess>(world: &W, pos: Position, facing: Direction) -> bool {
    if let Block::CobblestoneWall { .. } = world.get_block(pos.shift(facing.clockwise())) {
        return true;
    }

    if let Block::CobblestoneWall { .. } = world.get_block(pos.shift(facing.counter_clockwise())) {
        return true;
    }

    false
}

fn door_data(
    facing: Direction,
    half: DoorHalf,
    hinge: Side,
    open: bool,
    powered: bool,
) -> Option<usize> {
    match half {
        DoorHalf::Upper => {
            if facing == Direction::North && open {
                Some(
                    0x8 | (if hinge == Side::Right { 0x1 } else { 0x0 })
                        | (if powered { 0x2 } else { 0x0 }),
                )
            } else {
                None
            }
        }
        DoorHalf::Lower => {
            if hinge == Side::Left && !powered {
                Some(facing.clockwise().horizontal_index() | (if open { 0x4 } else { 0x0 }))
            } else {
                None
            }
        }
    }
}

fn door_offset(
    facing: Direction,
    half: DoorHalf,
    hinge: Side,
    open: bool,
    powered: bool,
) -> Option<usize> {
    Some(
        if powered { 0 } else { 1 << 0 }
            + if open { 0 } else { 1 << 1 }
            + if hinge == Side::Left { 0 } else { 1 << 2 }
            + if half == DoorHalf::Upper { 0 } else { 1 << 3 }
            + facing.horizontal_offset() * (1 << 4),
    )
}

fn update_door_state<W: WorldAccess>(
    world: &W,
    pos: Position,
    ohalf: DoorHalf,
    ofacing: Direction,
    ohinge: Side,
    oopen: bool,
    opowered: bool,
) -> (Direction, Side, bool, bool) {
    let oy = if ohalf == DoorHalf::Upper { -1 } else { 1 };

    match world.get_block(pos + (0, oy, 0)) {
        Block::WoodenDoor {
            half,
            facing,
            hinge,
            open,
            powered,
        }
        | Block::SpruceDoor {
            half,
            facing,
            hinge,
            open,
            powered,
        }
        | Block::BirchDoor {
            half,
            facing,
            hinge,
            open,
            powered,
        }
        | Block::JungleDoor {
            half,
            facing,
            hinge,
            open,
            powered,
        }
        | Block::AcaciaDoor {
            half,
            facing,
            hinge,
            open,
            powered,
        }
        | Block::DarkOakDoor {
            half,
            facing,
            hinge,
            open,
            powered,
        }
        | Block::IronDoor {
            half,
            facing,
            hinge,
            open,
            powered,
        } => {
            if half != ohalf {
                if ohalf == DoorHalf::Upper {
                    return (facing, ohinge, open, opowered);
                } else {
                    return (ofacing, hinge, oopen, powered);
                }
            }
        }
        _ => {}
    }

    (ofacing, ohinge, oopen, opowered)
}

fn door_collision(facing: Direction, hinge: Side, open: bool) -> Vec<Aabb3<f64>> {
    use std::f64::consts::PI;
    let mut bounds = Aabb3::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 3.0 / 16.0),
    );
    let mut angle = match facing {
        Direction::South => 0.0,
        Direction::West => PI * 0.5,
        Direction::North => PI,
        Direction::East => PI * 1.5,
        _ => 0.0,
    };
    angle += if open { PI * 0.5 } else { 0.0 }
        * match hinge {
            Side::Left => 1.0,
            Side::Right => -1.0,
        };

    let c = angle.cos();
    let s = angle.sin();

    let x = bounds.min.x - 0.5;
    let z = bounds.min.z - 0.5;
    bounds.min.x = 0.5 + (x * c - z * s);
    bounds.min.z = 0.5 + (z * c + x * s);
    let x = bounds.max.x - 0.5;
    let z = bounds.max.z - 0.5;
    bounds.max.x = 0.5 + (x * c - z * s);
    bounds.max.z = 0.5 + (z * c + x * s);

    vec![bounds]
}

fn update_repeater_state<W: WorldAccess>(world: &W, pos: Position, facing: Direction) -> bool {
    let f = |dir| match world.get_block(pos.shift(dir)) {
        Block::RepeaterPowered { .. } => true,
        _ => false,
    };

    f(facing.clockwise()) || f(facing.counter_clockwise())
}

fn update_double_plant_state<W: WorldAccess>(
    world: &W,
    pos: Position,
    ohalf: BlockHalf,
    ovariant: DoublePlantVariant,
) -> (BlockHalf, DoublePlantVariant) {
    if ohalf != BlockHalf::Upper {
        return (ohalf, ovariant);
    }

    match world.get_block(pos.shift(Direction::Down)) {
        Block::DoublePlant { variant, .. } => (ohalf, variant),
        _ => (ohalf, ovariant),
    }
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
        Point3::new(max_x, max_y, max_z),
    )]
}

fn trapdoor_collision(facing: Direction, half: BlockHalf, open: bool) -> Vec<Aabb3<f64>> {
    let (min_x, min_y, min_z, max_x, max_y, max_z) = if open {
        match facing {
            Direction::North => (0.0, 0.0, 3.0 / 16.0, 1.0, 1.0, 1.0),
            Direction::South => (0.0, 0.0, 0.0, 1.0, 1.0, 3.0 / 16.0),
            Direction::West => (3.0 / 16.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            Direction::East => (0.0, 0.0, 0.0, 3.0 / 16.0, 1.0, 1.0),
            _ => unreachable!(),
        }
    } else {
        match half {
            BlockHalf::Bottom => (0.0, 0.0, 0.0, 1.0, 3.0 / 16.0, 1.0),
            BlockHalf::Top => (0.0, 3.0 / 16.0, 0.0, 1.0, 1.0, 1.0),
            _ => unreachable!(),
        }
    };

    vec![Aabb3::new(
        Point3::new(min_x, min_y, min_z),
        Point3::new(max_x, max_y, max_z),
    )]
}

fn fence_collision(north: bool, south: bool, west: bool, east: bool) -> Vec<Aabb3<f64>> {
    let mut collision = vec![Aabb3::new(
        Point3::new(3.0 / 8.0, 0.0, 3.0 / 8.0),
        Point3::new(5.0 / 8.0, 1.5, 5.0 / 8.0),
    )];

    if north {
        collision.push(Aabb3::new(
            Point3::new(3.0 / 8.0, 0.0, 0.0),
            Point3::new(5.0 / 8.0, 1.5, 3.0 / 8.0),
        ));
    }

    if south {
        collision.push(Aabb3::new(
            Point3::new(3.0 / 8.0, 0.0, 5.0 / 8.0),
            Point3::new(5.0 / 8.0, 1.5, 1.0),
        ));
    }

    if west {
        collision.push(Aabb3::new(
            Point3::new(0.0, 0.0, 3.0 / 8.0),
            Point3::new(3.0 / 8.0, 1.5, 5.0 / 8.0),
        ));
    }

    if east {
        collision.push(Aabb3::new(
            Point3::new(5.0 / 8.0, 0.0, 3.0 / 8.0),
            Point3::new(1.0, 1.5, 5.0 / 8.0),
        ));
    }

    collision
}

fn pane_collision(north: bool, south: bool, east: bool, west: bool) -> Vec<Aabb3<f64>> {
    let mut collision = vec![Aabb3::new(
        Point3::new(7.0 / 16.0, 0.0, 7.0 / 16.0),
        Point3::new(9.0 / 16.0, 1.0, 9.0 / 16.0),
    )];

    if north {
        collision.push(Aabb3::new(
            Point3::new(7.0 / 16.0, 0.0, 0.0),
            Point3::new(9.0 / 16.0, 1.0, 9.0 / 16.0),
        ));
    }

    if south {
        collision.push(Aabb3::new(
            Point3::new(7.0 / 16.0, 0.0, 7.0 / 16.0),
            Point3::new(9.0 / 16.0, 1.0, 1.0),
        ));
    }

    if west {
        collision.push(Aabb3::new(
            Point3::new(0.0, 0.0, 7.0 / 16.0),
            Point3::new(9.0 / 16.0, 1.0, 9.0 / 16.0),
        ));
    }

    if east {
        collision.push(Aabb3::new(
            Point3::new(7.0 / 16.0, 0.0, 7.0 / 16.0),
            Point3::new(1.0, 1.0, 9.0 / 16.0),
        ));
    }

    collision
}

fn get_stair_info<W: WorldAccess>(world: &W, pos: Position) -> Option<(Direction, BlockHalf)> {
    match world.get_block(pos) {
        Block::OakStairs { facing, half, .. }
        | Block::StoneStairs { facing, half, .. }
        | Block::BrickStairs { facing, half, .. }
        | Block::StoneBrickStairs { facing, half, .. }
        | Block::NetherBrickStairs { facing, half, .. }
        | Block::SandstoneStairs { facing, half, .. }
        | Block::SpruceStairs { facing, half, .. }
        | Block::BirchStairs { facing, half, .. }
        | Block::JungleStairs { facing, half, .. }
        | Block::QuartzStairs { facing, half, .. }
        | Block::AcaciaStairs { facing, half, .. }
        | Block::DarkOakStairs { facing, half, .. }
        | Block::RedSandstoneStairs { facing, half, .. }
        | Block::PurpurStairs { facing, half, .. } => Some((facing, half)),
        _ => None,
    }
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

fn stair_data(
    facing: Direction,
    half: BlockHalf,
    shape: StairShape,
    waterlogged: bool,
) -> Option<usize> {
    if shape != StairShape::Straight {
        return None;
    }
    if waterlogged {
        return None;
    }

    Some((5 - facing.index()) | (if half == BlockHalf::Top { 0x4 } else { 0x0 }))
}

fn stair_offset(
    facing: Direction,
    half: BlockHalf,
    shape: StairShape,
    waterlogged: bool,
) -> Option<usize> {
    Some(
        if waterlogged { 0 } else { 1 }
            + shape.offset() * 2
            + if half == BlockHalf::Top { 0 } else { 2 * 5 }
            + facing.horizontal_offset() * 2 * 5 * 2,
    )
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
        bound.min.x = 0.5 + (x * c - z * s);
        bound.min.z = 0.5 + (z * c + x * s);
        let x = bound.max.x - 0.5;
        let z = bound.max.z - 0.5;
        bound.max.x = 0.5 + (x * c - z * s);
        bound.max.z = 0.5 + (z * c + x * s);

        if half == BlockHalf::Top {
            let c = PI.cos();
            let s = PI.sin();
            let z = bound.min.z - 0.5;
            let y = bound.min.y - 0.5;
            bound.min.z = 0.5 + (z * c - y * s);
            bound.min.y = 0.5 + (y * c + z * s);
            let z = bound.max.z - 0.5;
            let y = bound.max.y - 0.5;
            bound.max.z = 0.5 + (z * c - y * s);
            bound.max.y = 0.5 + (y * c + z * s);

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
        Point3::new(max_x, max_y, max_z),
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
    Foot,
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

fn mushroom_block_data(
    is_stem: bool,
    west: bool,
    up: bool,
    south: bool,
    north: bool,
    east: bool,
    down: bool,
) -> Option<usize> {
    Some(match (is_stem, west, up, south, north, east, down) {
        (false, false, false, false, false, false, false) => 0,
        (false, true, false, false, true, false, false) => 1,
        (false, false, false, false, true, false, false) => 2,
        (false, false, false, false, true, true, false) => 3,
        (false, true, false, false, false, false, false) => 4,
        (false, false, true, false, false, false, false) => 5,
        (false, false, false, false, false, true, false) => 6,
        (false, true, false, true, false, false, false) => 7,
        (false, false, false, true, false, false, false) => 8,
        (false, false, false, true, false, true, false) => 9,
        (false, true, false, true, true, true, false) => 10,
        (false, true, true, true, true, true, true) => 14,
        (true, false, false, false, false, false, false) => 15,
        _ => return None,
    })
}

fn mushroom_block_offset(
    is_stem: bool,
    west: bool,
    up: bool,
    south: bool,
    north: bool,
    east: bool,
    down: bool,
) -> Option<usize> {
    if is_stem {
        None
    } else {
        Some(
            if west { 0 } else { 1 << 0 }
                + if up { 0 } else { 1 << 1 }
                + if south { 0 } else { 1 << 2 }
                + if north { 0 } else { 1 << 3 }
                + if east { 0 } else { 1 << 4 }
                + if down { 0 } else { 1 << 5 },
        )
    }
}

fn mushroom_block_variant(
    is_stem: bool,
    west: bool,
    up: bool,
    south: bool,
    north: bool,
    east: bool,
    down: bool,
) -> String {
    (if is_stem {
        "all_stem"
    } else {
        match (west, up, south, north, east, down) {
            (false, false, false, false, false, false) => "all_inside",
            (true, false, false, true, false, false) => "north_west",
            (false, false, false, true, false, false) => "north",
            (false, false, false, true, true, false) => "north_east",
            (true, false, false, false, false, false) => "west",
            (false, true, false, false, false, false) => "center",
            (false, false, false, false, true, false) => "east",
            (true, false, true, false, false, false) => "south_west",
            (false, false, true, false, false, false) => "south",
            (false, false, true, false, true, false) => "south_east",
            (true, false, true, true, true, false) => "stem",
            (true, true, true, true, true, true) => "all_outside",
            _ => "all_stem",
        }
    })
    .to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoorHalf {
    Upper,
    Lower,
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
            StoneSlabVariant::Stone | StoneSlabVariant::RedSandstone | StoneSlabVariant::Purpur => {
                0
            }
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
        }
        .to_owned()
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
            TreeVariant::StrippedOak => "stripped_oak_log",
        }
    }

    pub fn data(self) -> usize {
        match self {
            TreeVariant::Oak | TreeVariant::Acacia => 0,
            TreeVariant::Spruce | TreeVariant::DarkOak => 1,
            TreeVariant::Birch => 2,
            TreeVariant::Jungle => 3,
            _ => panic!("TreeVariant {:?} has no data (1.13+ only)", self),
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
            _ => panic!("TreeVariant {:?} has no plank data (1.13+ only)", self),
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
