
#![recursion_limit="300"]

extern crate cgmath;
extern crate collision;
#[macro_use]
extern crate lazy_static;
extern crate steven_shared as shared;

use shared::{Axis, Direction, Position};
use collision::{Aabb, Aabb3};
use cgmath::Point3;

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


macro_rules! consume_token { ($i:tt) => (0) }

macro_rules! offsets {
    ($first:ident, $($other:ident),*) => (
        #[allow(non_upper_case_globals)]
        pub const $first: usize = 0;
        offsets!(prev($first), $($other),*);
    );
    (prev($prev:ident), $first:ident, $($other:ident),*) => (
        #[allow(non_upper_case_globals)]
        pub const $first: usize = $prev + internal_sizes::$prev;
        offsets!(prev($first), $($other),*);
    );
    (prev($prev:ident), $first:ident) => (
        #[allow(non_upper_case_globals)]
        pub const $first: usize = $prev + internal_sizes::$prev;
    )
}

macro_rules! define_blocks {
    (
        $(
            $name:ident {
                props {
                    $(
                        $fname:ident : $ftype:ty = [$($val:expr),+],
                    )*
                },
                $(data $datafunc:expr,)*
                material $mat:expr,
                model $model:expr,
                $(variant $variant:expr,)*
                $(tint $tint:expr,)*
                $(collision $collision:expr,)*
                $(update_state ($world:ident, $pos:ident) => $update_state:expr,)*
                $(multipart ($mkey:ident, $mval:ident) => $multipart:expr,)*
            }
        )+
    ) => (
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Block {
            $(
                $name {
                    $(
                        $fname : $ftype,
                    )*
                },
            )+
        }
        mod internal_ids {
            create_ids!(usize, $($name),+);
        }
        mod internal_sizes {
            $(
                #[allow(non_upper_case_globals)]
                pub const $name : usize = $(($(1 + consume_token!($val) + )+ 0) *  )* 1;
            )+
        }
        mod internal_offsets {
            use super::internal_sizes;
            offsets!($($name),+);
        }
        mod internal_offset_max {
            use super::internal_sizes;
            use super::internal_offsets;
            $(
                #[allow(non_upper_case_globals)]
                pub const $name: usize = internal_offsets::$name + internal_sizes::$name - 1;
            )+
        }

        impl Block {
            #[allow(unused_variables, unused_mut, unused_assignments)]
            pub fn get_steven_id(&self) -> usize {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)*
                        } => {
                            let mut offset = internal_offsets::$name;
                            let mut mul = 1;
                            $(
                                offset += [$($val),+].into_iter().position(|v| *v == $fname).unwrap() * mul;
                                mul *= $(1 + consume_token!($val) + )+ 0;
                            )*
                            offset
                        },
                    )+
                }
            }

            #[allow(unused_variables, unused_assignments)]
            pub fn by_steven_id(id: usize) -> Block {
                match id {
                    $(
                        mut data @ internal_offsets::$name ... internal_offset_max::$name=> {
                            data -= internal_offsets::$name;
                            $(
                                let vals = [$($val),+];
                                let $fname = vals[data % vals.len()];
                                data /= vals.len();
                            )*
                            Block::$name {
                                $(
                                    $fname: $fname,
                                )*
                            }
                        },
                    )*
                    _ => Block::Missing {}
                }
            }

            #[allow(unused_variables, unreachable_code)]
            pub fn get_vanilla_id(&self) -> Option<usize> {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)*
                        } => {
                            $(
                                let data: Option<usize> = ($datafunc).map(|v| v + (internal_ids::$name << 4));
                                return data;
                            )*
                            Some(internal_ids::$name << 4)
                        }
                    )+
                }
            }

            pub fn by_vanilla_id(id: usize) -> Block {
                VANILLA_ID_MAP.get(id).and_then(|v| *v).unwrap_or(Block::Missing{})
            }

            #[allow(unused_variables)]
            pub fn get_material(&self) -> Material {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)*
                        } => {
                            $mat
                        }
                    )+
                }
            }

            #[allow(unused_variables)]
            pub fn get_model(&self) -> (String, String) {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)*
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
                            $($fname,)*
                        } => {
                            $(return String::from($variant);)*
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
                            $($fname,)*
                        } => {
                            $(return $tint;)*
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
                            $($fname,)*
                        } => {
                            $(return $collision;)*
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
                            $($fname,)*
                        } => {
                            $(
                                let $world = world;
                                let $pos = pos;
                                return $update_state;
                            )*
                            Block::$name {
                                $($fname: $fname,)*
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
                            $($fname,)*
                        } => {
                            $(
                                let $mkey = key;
                                let $mval = val;
                                return $multipart;
                            )*
                            false
                        }
                    )+
                }
            }
        }

        lazy_static! {
            static ref VANILLA_ID_MAP: Vec<Option<Block>> = {
                let mut blocks = vec![];
                for i in 0 .. internal_offsets::Missing {
                    let block = Block::by_steven_id(i);
                    if let Some(id) = block.get_vanilla_id() {
                        if blocks.len() <= id {
                            blocks.resize(id + 1, None);
                        }
                        if blocks[id].is_none() {
                            blocks[id] = Some(block);
                        } else {
                            panic!(
                                "Tried to register {:#?} to {}:{} but {:#?} was already registered",
                                block,
                                id >> 4,
                                id & 0xF,
                                blocks[id]
                            );
                        }
                    }
                }
                blocks
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

define_blocks! {
    Air {
        props {},
        material material::INVISIBLE,
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
        material material::SOLID,
        model { ("minecraft", variant.as_string() ) },
    }
    Grass {
        props {
            snowy: bool = [false, true],
        },
        data { if snowy { None } else { Some(0) } },
        material material::SOLID,
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
        material material::SOLID,
        model { ("minecraft", variant.as_string()) },
        variant {
            if variant == DirtVariant::Podzol {
                format!("snowy={}", snowy)
            } else {
                "normal".to_owned()
            }
        },
        update_state (world, pos) => if variant == DirtVariant::Podzol {
            Block::Dirt{snowy: is_snowy(world, pos), variant: variant}
        } else {
            Block::Dirt{snowy: snowy, variant: variant}
        },
    }
    Cobblestone {
        props {},
        material material::SOLID,
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
        material material::SOLID,
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
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_sapling", variant.as_string()) ) },
        variant format!("stage={}", stage),
        collision vec![],
    }
    Bedrock {
        props {},
        material material::SOLID,
        model { ("minecraft", "bedrock") },
    }
    FlowingWater {
        props {
            level: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(level as usize),
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
        material material::SOLID,
        model { ("minecraft", if red { "red_sand" } else { "sand" } ) },
    }
    Gravel {
        props {},
        material material::SOLID,
        model { ("minecraft", "gravel") },
    }
    GoldOre {
        props {},
        material material::SOLID,
        model { ("minecraft", "gold_ore") },
    }
    IronOre {
        props {},
        material material::SOLID,
        model { ("minecraft", "iron_ore") },
    }
    CoalOre {
        props {},
        material material::SOLID,
        model { ("minecraft", "coal_ore") },
    }
    Log {
        props {
            variant: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle
            ],
            axis: Axis = [Axis::Y, Axis::Z, Axis::X, Axis::None],
        },
        data Some(variant.data() | (axis.index() << 2)),
        material material::SOLID,
        model { ("minecraft", format!("{}_log", variant.as_string()) ) },
        variant format!("axis={}", axis.as_string()),
    }
    Leaves {
        props {
            variant: TreeVariant = [
                TreeVariant::Oak,
                TreeVariant::Spruce,
                TreeVariant::Birch,
                TreeVariant::Jungle
            ],
            decayable: bool = [false, true],
            check_decay: bool = [false, true],
        },
        data Some(variant.data()
                  | (if decayable { 0x4 } else { 0x0 })
                  | (if check_decay { 0x8 } else { 0x0 })),
        material material::LEAVES,
        model { ("minecraft", format!("{}_leaves", variant.as_string()) ) },
        tint TintType::Foliage,
    }
    Sponge {
        props {
            wet: bool = [false, true],
        },
        data Some(if wet { 1 } else { 0 }),
        material material::SOLID,
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
        material material::SOLID,
        model { ("minecraft", "lapis_ore") },
    }
    LapisBlock {
        props {},
        material material::SOLID,
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
        material material::SOLID,
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
        material material::SOLID,
        model { ("minecraft", variant.as_string() ) },
    }
    NoteBlock {
        props {},
        material material::SOLID,
        model { ("minecraft", "noteblock") },
    }
    Bed {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
            occupied: bool = [false, true],
            part: BedPart = [BedPart::Head, BedPart::Foot],
        },
        data Some(facing.horizontal_index()
                  | (if occupied { 0x4 } else { 0x0 })
                  | (if part == BedPart::Head { 0x8 } else { 0x0 })),
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
    DeadBush {
        props {},
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
        material material::SOLID,
        model { ("minecraft", format!("{}_wool", color.as_string()) ) },
    }
    PistonExtension {
        props {},
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
        material material::SOLID,
        model { ("minecraft", "gold_block") },
    }
    IronBlock {
        props {},
        material material::SOLID,
        model { ("minecraft", "iron_block") },
    }
    DoubleStoneSlab {
        props {
            seamless: bool = [false, true],
            variant: StoneSlabVariant = [
                StoneSlabVariant::Stone,
                StoneSlabVariant::Sandstone,
                StoneSlabVariant::Wood,
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
        material material::SOLID,
        model { ("minecraft", format!("{}_double_slab", variant.as_string()) ) },
        variant if seamless { "all" } else { "normal" },
    }
    StoneSlab {
        props {
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            variant: StoneSlabVariant = [
                StoneSlabVariant::Stone,
                StoneSlabVariant::Sandstone,
                StoneSlabVariant::Wood,
                StoneSlabVariant::Cobblestone,
                StoneSlabVariant::Brick,
                StoneSlabVariant::StoneBrick,
                StoneSlabVariant::NetherBrick,
                StoneSlabVariant::Quartz
            ],
        },
        data Some(variant.data() | (if half == BlockHalf::Top { 0x8 } else { 0x0 })),
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("half={}", half.as_string()),
        collision match half {
            BlockHalf::Bottom => vec![Aabb3::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.5, 1.0)
            )],
            BlockHalf::Top => vec![Aabb3::new(
                Point3::new(0.0, 0.5, 0.0),
                Point3::new(1.0, 1.0, 1.0)
            )],
            _ => unreachable!(),
        },
    }
    BrickBlock {
        props {},
        material material::SOLID,
        model { ("minecraft", "brick_block") },
    }
    TNT {
        props {
            explode: bool = [false, true],
        },
        data Some(if explode { 1 } else { 0 }),
        material material::SOLID,
        model { ("minecraft", "tnt") },
    }
    BookShelf {
        props {},
        material material::SOLID,
        model { ("minecraft", "bookshelf") },
    }
    MossyCobblestone {
        props {},
        material material::SOLID,
        model { ("minecraft", "mossy_cobblestone") },
    }
    Obsidian {
        props {},
        material material::SOLID,
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
        material Material {
            emitted_light: 15,
            ..material::NON_SOLID
        },
        model { ("minecraft", "fire") },
        collision vec![],
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "oak_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::OakStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
    }
    Chest {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
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
        material material::NON_SOLID,
        model { ("minecraft", "redstone_wire") },
        tint TintType::Color{r: ((255.0 / 30.0) * (power as f64) + 14.0) as u8, g: 0, b: 0},
        collision vec![],
        update_state (world, pos) => Block::RedstoneWire {
            north: can_connect_redstone(world, pos, Direction::North),
            south: can_connect_redstone(world, pos, Direction::South),
            west: can_connect_redstone(world, pos, Direction::West),
            east: can_connect_redstone(world, pos, Direction::East),
            power: power
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
        material material::SOLID,
        model { ("minecraft", "diamond_ore") },
    }
    DiamondBlock {
        props {},
        material material::SOLID,
        model { ("minecraft", "diamond_block") },
    }
    CraftingTable {
        props {},
        material material::SOLID,
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
        },
        data Some(facing.index()),
        material material::SOLID,
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
        },
        data Some(rotation.data()),
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
        material material::NON_SOLID,
        model { ("minecraft", "wooden_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::WoodenDoor{facing: facing, half: half, hinge: hinge, open: open, powered: powered}
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
        },
        data Some(facing.index()),
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "stone_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::StoneStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
    }
    WallSign {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ],
        },
        data Some(facing.index()),
        material material::INVISIBLE,
        model { ("minecraft", "wall_sign") },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
    }
    Lever {
        props {
            facing: LeverDirection = [
                LeverDirection::North,
                LeverDirection::South,
                LeverDirection::East,
                LeverDirection::West,
                LeverDirection::UpX,
                LeverDirection::DownX,
                LeverDirection::UpZ,
                LeverDirection::DownZ
            ],
            powered: bool = [false, true],
        },
        data Some(facing.data() | (if powered { 0x8 } else { 0x0 })),
        material material::NON_SOLID,
        model { ("minecraft", "lever") },
        variant format!("facing={},powered={}", facing.as_string(), powered),
        collision vec![],
    }
    StonePressurePlate {
        props {
            powered: bool = [false, true],
        },
        data Some(if powered { 1 } else { 0 }),
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
        material material::NON_SOLID,
        model { ("minecraft", "iron_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::IronDoor{facing: facing, half: half, hinge: hinge, open: open, powered: powered}
        },
    }
    WoodenPressurePlate {
        props {
            powered: bool = [false, true],
        },
        data Some(if powered { 1 } else { 0 }),
        material material::NON_SOLID,
        model { ("minecraft", "wooden_pressure_plate") },
        variant format!("powered={}", powered),
        collision vec![],
    }
    RedstoneOre {
        props {},
        material material::SOLID,
        model { ("minecraft", "redstone_ore") },
    }
    RedstoneOreLit {
        props {},
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
        material material::NON_SOLID,
        model { ("minecraft", "unlit_redstone_torch") },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
    }
    RedstoneTorch {
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
        material Material {
            emitted_light: 7,
            ..material::NON_SOLID
        },
        model { ("minecraft", "redstone_torch") },
        variant format!("facing={}", facing.as_string()),
        collision vec![],
    }
    StoneButton {
        props {
            facing: Direction = [
                Direction::Down,
                Direction::East,
                Direction::West,
                Direction::South,
                Direction::North,
                Direction::Up
            ],
            powered: bool = [false, true],
        },
        data Some(match facing {
            Direction::Down => 0,
            Direction::East => 1,
            Direction::West => 2,
            Direction::South => 3,
            Direction::North => 4,
            Direction::Up => 5,
            _ => unreachable!(),
        } | (if powered { 0x8 } else { 0x0 })),
        material material::NON_SOLID,
        model { ("minecraft", "stone_button") },
        variant format!("facing={},powered={}", facing.as_string(), powered),
        collision vec![],
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
            Point3::new(1.0, (layers as f64)/8.0, 1.0),
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
        material material::SOLID,
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
        material material::SOLID,
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
        material material::SOLID,
        model { ("minecraft", "jukebox") },
    }
    Fence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::Fence{north: north, south: south, west: west, east: east}
        },
        multipart (key, val) => match key {
            "north" => north == (val == "true"),
            "south" => south == (val == "true"),
            "west" => west == (val == "true"),
            "east" => east == (val == "true"),
            _ => false,
        },
    }
    Pumpkin {
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
        material material::SOLID,
        model { ("minecraft", "pumpkin") },
        variant format!("facing={}", facing.as_string()),
    }
    Netherrack {
        props {},
        material material::SOLID,
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
        material Material {
            emitted_light: 11,
            ..material::TRANSPARENT
        },
        model { ("minecraft", "portal") },
        variant format!("axis={}", axis.as_string()),
        collision vec![],
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
            Point3::new((1.0 + (bites as f64 * 2.0)) / 16.0, 0.0, 1.0/16.0),
            Point3::new(1.0 - (1.0/16.0), 0.5, 1.0 - (1.0/16.0))
        )],
    }
    RepeaterUnpowered {
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
        material material::NON_SOLID,
        model { ("minecraft", "unpowered_repeater") },
        variant format!("delay={},facing={},locked={}", delay, facing.as_string(), locked),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0/8.0, 1.0)
        )],
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
        material material::NON_SOLID,
        model { ("minecraft", "powered_repeater") },
        variant format!("delay={},facing={},locked={}", delay, facing.as_string(), locked),
        collision vec![Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0/8.0, 1.0)
        )],
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
        },
        data Some(match facing {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            _ => unreachable!(),
        } | (if open { 0x4 } else { 0x0 }) | (if half == BlockHalf::Top { 0x8 } else { 0x0 })),
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
        material material::SOLID,
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
        material material::SOLID,
        model { ("minecraft", variant.as_string() ) },
    }
    BrownMushroomBlock {
        props {
            variant: MushroomVariant = [
                MushroomVariant::East,
                MushroomVariant::North,
                MushroomVariant::NorthEast,
                MushroomVariant::NorthWest,
                MushroomVariant::South,
                MushroomVariant::SouthEast,
                MushroomVariant::SouthWest,
                MushroomVariant::West,
                MushroomVariant::Center,
                MushroomVariant::Stem,
                MushroomVariant::AllInside,
                MushroomVariant::AllOutside,
                MushroomVariant::AllStem
            ],
        },
        data Some(variant.data()),
        material material::SOLID,
        model { ("minecraft", "brown_mushroom_block") },
        variant format!("variant={}", variant.as_string()),
    }
    RedMushroomBlock {
        props {
            variant: MushroomVariant = [
                MushroomVariant::East,
                MushroomVariant::North,
                MushroomVariant::NorthEast,
                MushroomVariant::NorthWest,
                MushroomVariant::South,
                MushroomVariant::SouthEast,
                MushroomVariant::SouthWest,
                MushroomVariant::West,
                MushroomVariant::Center,
                MushroomVariant::Stem,
                MushroomVariant::AllInside,
                MushroomVariant::AllOutside,
                MushroomVariant::AllStem
            ],
        },
        data Some(variant.data()),
        material material::SOLID,
        model { ("minecraft", "red_mushroom_block") },
        variant format!("variant={}", variant.as_string()),
    }
    IronBars {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
        },
        data if !north && !south && !west && !east { Some(0) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "iron_bars") },
        collision pane_collision(north, south, east, west),
        update_state (world, pos) => {
            let f = |block| match block {
                Block::IronBars{..} => true,
                _ => false,
            };

            let (north, south, west, east) = can_connect_sides(world, pos, &f);
            Block::IronBars{north: north, south: south, west: west, east: east}
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
        },
        data if !north && !south && !west && !east { Some(0) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "glass_pane") },
        collision pane_collision(north, south, east, west),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_glasspane);
            Block::GlassPane{north: north, south: south, west: west, east: east}
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
        material material::SOLID,
        model { ("minecraft", "melon_block") },
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

            Block::PumpkinStem{age: age, facing: facing}
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

            Block::MelonStem{age: age, facing: facing}
        },
    }
    Vine {
        props {
             south: bool = [false, true],
             west: bool = [false, true],
             north: bool = [false, true],
             east: bool = [false, true],
             up: bool = [false, true],
        },
        data if !up {
            Some((if south { 0x1 } else { 0x0 })
                | (if west { 0x2 } else { 0x0 })
                | (if north { 0x4 } else { 0x0 })
                | (if east { 0x8 } else { 0x0 }))
        } else {
            None
        },
        material material::NON_SOLID,
        model { ("minecraft", "vine") },
        variant format!("east={},north={},south={},up={},west={}", east, north, south, up, west),
        tint TintType::Foliage,
        collision vec![],
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
        material material::NON_SOLID,
        model { ("minecraft", "fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "brick_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::BrickStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "stone_brick_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::StoneBrickStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
    }
    Mycelium {
        props {
            snowy: bool = [false, true],
        },
        data if snowy { None } else { Some(0) },
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
        material material::SOLID,
        model { ("minecraft", "nether_brick") },
    }
    NetherBrickFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
        },
        data if !north && !south && !west && !east { Some(0) } else { None },
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
            Block::NetherBrickFence{north: north, south: south, west: west, east: east}
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "nether_brick_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::NetherBrickStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
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
        material material::SOLID,
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
        material material::SOLID,
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
        material material::SOLID,
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
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("half={}", half.as_string()),
        collision match half {
            BlockHalf::Bottom => vec![Aabb3::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.5, 1.0)
            )],
            BlockHalf::Top => vec![Aabb3::new(
                Point3::new(0.0, 0.5, 0.0),
                Point3::new(1.0, 1.0, 1.0))
            ],
            _ => unreachable!(),
        },
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
        material material::NON_SOLID,
        model { ("minecraft", "cocoa") },
        variant format!("age={},facing={}", age, facing.as_string()),
        collision {
            let i = 4.0 + age as f64 * 2.0;
            let j = 5.0 + age as f64 * 2.0;
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "sandstone_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::SandstoneStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
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
        },
        data Some(facing.index()),
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
        material material::NON_SOLID,
        model { ("minecraft", "tripwire_hook") },
        variant format!("attached={},facing={},powered={}", attached, facing.as_string(), powered),
        collision vec![],
    }
    Tripwire {
        props {
            attached: bool = [false, true],
            disarmed: bool = [false, true],
            east: bool = [false, true],
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            powered: bool = [false, true],
        },
        data if !north && !south && !east && !west {
            Some((if powered { 0x1 } else { 0x0 })
                 | (if attached { 0x4 } else { 0x0 })
                 | (if disarmed { 0x8 } else { 0x0 }))
        } else {
            None
        },
        material material::NON_SOLID,
        model { ("minecraft", "tripwire") },
        variant format!("attached={},east={},north={},south={},west={}", attached, east, north, south, west),
        collision vec![],
    }
    EmeraldBlock {
        props {},
        material material::SOLID,
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "spruce_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::SpruceStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "birch_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::BirchStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "jungle_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::JungleStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
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
        material material::SOLID,
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
            east: bool = [false, true],
            west: bool = [false, true],
            variant: CobblestoneWallVariant = [
                CobblestoneWallVariant::Normal,
                CobblestoneWallVariant::Mossy
            ],
        },
        data if !north && !south && !east && !west && !up { Some(variant.data()) } else { None },
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

            let up = can_connect(world, pos.shift(Direction::Up), &f);
            let (north, south, west, east) = can_connect_sides(world, pos, &f);
            Block::CobblestoneWall{up: up, north: north, south: south, west: west, east: east, variant: variant}
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
                FlowerPotVariant::DarkOak,
                FlowerPotVariant::BlueOrchid,
                FlowerPotVariant::Allium,
                FlowerPotVariant::AzureBluet,
                FlowerPotVariant::RedTulip,
                FlowerPotVariant::OrangeTulip,
                FlowerPotVariant::WhiteTulip,
                FlowerPotVariant::PinkTulip,
                FlowerPotVariant::Oxeye,
                FlowerPotVariant::Dandelion
            ],
            legacy_data: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data if contents == FlowerPotVariant::Empty { Some(legacy_data as usize) } else { None },
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
            facing: Direction = [
                Direction::Down,
                Direction::East,
                Direction::West,
                Direction::South,
                Direction::North,
                Direction::Up
            ],
            powered: bool = [false, true],
        },
        data Some(match facing {
            Direction::Down => 0,
            Direction::East => 1,
            Direction::West => 2,
            Direction::South => 3,
            Direction::North => 4,
            Direction::Up => 5,
            _ => unreachable!(),
        } | (if powered { 0x8 } else { 0x0 })),
        material material::NON_SOLID,
        model { ("minecraft", "wooden_button") },
        variant format!("facing={},powered={}", facing.as_string(), powered),
        collision vec![],
    }
    Skull {
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
        },
        data Some(facing.index()),
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
        },
        data Some(power as usize),
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
        material material::SOLID,
        model { ("minecraft", "redstone_block") },
    }
    QuartzOre {
        props {},
        material material::SOLID,
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
        material material::SOLID,
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "quartz_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::QuartzStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
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
        material material::SOLID,
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
        material material::SOLID,
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
        },
        data if !north && !south && !east && !west { Some(color.data()) } else { None },
        material material::TRANSPARENT,
        model { ("minecraft", format!("{}_stained_glass_pane", color.as_string()) ) },
        collision pane_collision(north, south, east, west),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_glasspane);
            Block::StainedGlassPane{color: color, north: north, south: south, west: west, east: east}
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
        material material::SOLID,
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "acacia_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::AcaciaStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "dark_oak_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::DarkOakStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
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
        },
        data Some(match facing {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            _ => unreachable!(),
        } | (if open { 0x4 } else { 0x0 }) | (if half == BlockHalf::Top { 0x8 } else { 0x0 })),
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
        material material::SOLID,
        model { ("minecraft", variant.as_string() ) },
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
        material material::SOLID,
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
        material material::SOLID,
        model { ("minecraft", "hardened_clay") },
    }
    CoalBlock {
        props {},
        material material::SOLID,
        model { ("minecraft", "coal_block") },
    }
    PackedIce {
        props {},
        material material::SOLID,
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
        material material::NON_SOLID,
        model { ("minecraft", variant.as_string()) },
        variant format!("half={}", half.as_string()),
        tint TintType::Foliage,
        collision vec![],
        update_state (world, pos) => {
            let (half, variant) = update_double_plant_state(world, pos, half, variant);
            Block::DoublePlant{half: half, variant: variant}
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
        },
        data Some(rotation.data()),
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
        },
        data Some(facing.index()),
        material material::NON_SOLID,
        model { ("minecraft", "wall_banner") },
        variant format!("facing={}", facing.as_string()),
    }
    DaylightDetectorInverted {
        props {
            power: u8 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(power as usize),
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
        material material::SOLID,
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "red_sandstone_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::RedSandstoneStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
    }
    DoubleStoneSlab2 {
        props {
            seamless: bool = [false, true],
            variant: StoneSlabVariant = [
                StoneSlabVariant::RedSandstone
            ],
        },
        data Some(variant.data() | (if seamless { 0x8 } else { 0x0 })),
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
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("half={}", half.as_string()),
        collision match half {
            BlockHalf::Bottom => vec![Aabb3::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.5, 1.0)
            )],
            BlockHalf::Top => vec![Aabb3::new(
                Point3::new(0.0, 0.5, 0.0),
                Point3::new(1.0, 1.0, 1.0)
            )],
            _ => unreachable!(),
        },
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
        material material::NON_SOLID,
        model { ("minecraft", "spruce_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
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
        material material::NON_SOLID,
        model { ("minecraft", "birch_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
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
        material material::NON_SOLID,
        model { ("minecraft", "jungle_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
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
        material material::NON_SOLID,
        model { ("minecraft", "dark_oak_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
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
        material material::NON_SOLID,
        model { ("minecraft", "acacia_fence_gate") },
        variant format!("facing={},in_wall={},open={}", facing.as_string(), in_wall, open),
        collision fence_gate_collision(facing, in_wall, open),
    }
    SpruceFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            west: bool = [false, true],
            east: bool = [false, true],
        },
        data if !north && !south && !west && !east { Some(0) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "spruce_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::SpruceFence{north: north, south: south, west: west, east: east}
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
        },
        data if !north && !south && !west && !east { Some(0) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "birch_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::BirchFence{north: north, south: south, west: west, east: east}
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
        },
        data if !north && !south && !west && !east { Some(0) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "jungle_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::JungleFence{north: north, south: south, west: west, east: east}
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
        },
        data if !north && !south && !west && !east { Some(0) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "dark_oak_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::DarkOakFence{north: north, south: south, east: east, west: west}
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
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material material::NON_SOLID,
        model { ("minecraft", "acacia_fence") },
        collision fence_collision(north, south, west, east),
        update_state (world, pos) => {
            let (north, south, west, east) = can_connect_sides(world, pos, &can_connect_fence);
            Block::AcaciaFence{north: north, south: south, east: east, west: west}
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
        material material::NON_SOLID,
        model { ("minecraft", "spruce_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::SpruceDoor{facing: facing, half: half, hinge: hinge, open: open, powered: powered}
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
        material material::NON_SOLID,
        model { ("minecraft", "birch_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::BirchDoor{facing: facing, half: half, hinge: hinge, open: open, powered: powered}
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
        material material::NON_SOLID,
        model { ("minecraft", "jungle_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::JungleDoor{facing: facing, half: half, hinge: hinge, open: open, powered: powered}
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
        material material::NON_SOLID,
        model { ("minecraft", "acacia_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::AcaciaDoor{facing: facing, half: half, hinge: hinge, open: open, powered: powered}
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
        material material::NON_SOLID,
        model { ("minecraft", "dark_oak_door") },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
        collision door_collision(facing, hinge, open),
        update_state (world, pos) => {
            let (facing, hinge, open, powered) = update_door_state(world, pos, half, facing, hinge, open, powered);
            Block::DarkOakDoor{facing: facing, half: half, hinge: hinge, open: open, powered: powered}
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
        material material::SOLID,
        model { ("minecraft", "purpur_block") },
    }
    PurpurPillar {
        props {
            axis: Axis = [Axis::X, Axis::Y, Axis::Z],
        },
        data Some(match axis { Axis::X => 0x4, Axis::Y => 0x0, Axis::Z => 0x8, _ => unreachable!() }),
        material material::SOLID,
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
        },
        data stair_data(facing, half, shape),
        material material::NON_SOLID,
        model { ("minecraft", "purpur_stairs") },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, pos) => Block::PurpurStairs{facing: facing, half: half, shape: update_stair_shape(world, pos, facing)},
    }
    PurpurDoubleSlab {
        props {
            variant: StoneSlabVariant = [StoneSlabVariant::Purpur],
        },
        material material::SOLID,
        model { ("minecraft", format!("{}_double_slab", variant.as_string()) ) },
    }
    PurpurSlab {
        props {
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            variant: StoneSlabVariant = [StoneSlabVariant::Purpur],
        },
        data if half == BlockHalf::Top { Some(0x8) } else { Some(0) },
        material material::NON_SOLID,
        model { ("minecraft", format!("{}_slab", variant.as_string()) ) },
        variant format!("half={},variant=default", half.as_string()),
        collision match half {
            BlockHalf::Bottom => vec![Aabb3::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.5, 1.0)
            )],
            BlockHalf::Top => vec![Aabb3::new(
                Point3::new(0.0, 0.5, 0.0),
                Point3::new(1.0, 1.0, 1.0)
            )],
            _ => unreachable!(),
        },
    }
    EndBricks {
        props {},
        material material::SOLID,
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
        material material::SOLID,
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
        material material::SOLID,
        model { ("minecraft", "chain_command_block") },
        variant format!("conditional={},facing={}", conditional, facing.as_string()),
    }
    FrostedIce {
        props {},
        material material::SOLID,
        model { ("minecraft", "frosted_ice") },
    }
    Missing {
        props {},
        data None::<usize>,
        material material::SOLID,
        model { ("steven", "missing_block") },
    }
}

fn is_snowy<W: WorldAccess>(world: &W, pos: Position) -> bool {
    match world.get_block(pos.shift(Direction::Up)) {
        Block::Snow{..} | Block::SnowLayer{..} => true,
        _ => false,
    }
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
    match block {
        Block::Fence{..} |
        Block::SpruceFence{..} |
        Block::BirchFence{..} |
        Block::JungleFence{..} |
        Block::DarkOakFence{..} |
        Block::AcaciaFence{..} |
        Block::FenceGate{..} |
        Block::SpruceFenceGate{..} |
        Block::BirchFenceGate{..} |
        Block::JungleFenceGate{..} |
        Block::DarkOakFenceGate{..} |
        Block::AcaciaFenceGate{..} => true,
        _ => false,
    }
}

fn can_connect_glasspane(block: Block) -> bool {
    match block {
        Block::Glass{..} |
        Block::StainedGlass{..} |
        Block::GlassPane{..} |
        Block::StainedGlassPane{..} => true,
        _ => false,
    }
}

fn can_connect_redstone<W: WorldAccess>(world: &W, pos: Position, dir: Direction) -> RedstoneSide {
    let shift_pos = pos.shift(dir);
    let block = world.get_block(shift_pos);

    if block.get_material().should_cull_against {
        let side_up = world.get_block(shift_pos.shift(Direction::Up));
        let up = world.get_block(pos.shift(Direction::Up));

        if match side_up { Block::RedstoneWire{..} => true, _ => false,} && !up.get_material().should_cull_against {
            return RedstoneSide::Up;
        }

        return RedstoneSide::None;
    }

    let side_down = world.get_block(shift_pos.shift(Direction::Down));
    if match block { Block::RedstoneWire{..} => true, _ => false,} || match side_down { Block::RedstoneWire{..} => true, _ => false,} {
        return RedstoneSide::Side;
    }
    RedstoneSide::None
}

fn fence_gate_data(facing: Direction, in_wall: bool, open: bool, powered: bool) -> Option<usize> {
    if in_wall || powered { return None; }

    Some(facing.horizontal_index() | (if open { 0x4 } else { 0x0 }))
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

fn update_door_state<W: WorldAccess>(world: &W, pos: Position, ohalf: DoorHalf, ofacing: Direction, ohinge: Side, oopen: bool, opowered: bool) -> (Direction, Side, bool, bool) {
    let oy = if ohalf == DoorHalf::Upper { -1 } else { 1 };

    match world.get_block(pos + (0, oy, 0)) {
        Block::WoodenDoor{half, facing, hinge, open, powered} |
        Block::SpruceDoor{half, facing, hinge, open, powered} |
        Block::BirchDoor{half, facing, hinge, open, powered} |
        Block::JungleDoor{half, facing, hinge, open, powered} |
        Block::AcaciaDoor{half, facing, hinge, open, powered} |
        Block::DarkOakDoor{half, facing, hinge, open, powered} |
        Block::IronDoor{half, facing, hinge, open, powered} => {
            if half != ohalf {
                if ohalf == DoorHalf::Upper {
                    return (facing, ohinge, open, opowered);
                } else {
                    return (ofacing, hinge, oopen, powered);
                }
            }
        },
        _ => {},
    }

    (ofacing, ohinge, oopen, opowered)
}

fn update_double_plant_state<W: WorldAccess>(world: &W, pos: Position, ohalf: BlockHalf, ovariant: DoublePlantVariant) -> (BlockHalf, DoublePlantVariant) {
    if ohalf != BlockHalf::Upper { return (ohalf, ovariant); }

    match world.get_block(pos.shift(Direction::Down)) {
        Block::DoublePlant{variant, ..} => (ohalf, variant),
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
        Point3::new(max_x, max_y, max_z)
    )]
}

fn door_collision(facing: Direction, hinge: Side, open: bool) -> Vec<Aabb3<f64>> {
    let (min_x, min_y, min_z, max_x, max_y, max_z) = if open {
        if hinge == Side::Left {
            match facing {
                Direction::North => (1.0 - (3.0 / 16.0), 0.0, 0.0, 1.0, 1.0, 1.0),
                Direction::South => (0.0, 0.0, 0.0, 3.0 / 16.0, 1.0, 1.0),
                Direction::West => (0.0, 0.0, 0.0, 1.0, 1.0, 3.0 / 16.0),
                Direction::East => (0.0, 0.0, 1.0 - (3.0 / 16.0), 1.0, 1.0, 1.0),
                _ => unreachable!(),
            }
        } else {
            match facing {
                Direction::North => (0.0, 0.0, 0.0, 3.0/16.0, 1.0, 1.0),
                Direction::South => (1.0 - (3.0 / 16.0), 0.0, 0.0, 1.0, 1.0, 1.0),
                Direction::West => (0.0, 0.0, 1.0 - (3.0 / 16.0), 1.0, 1.0, 1.0),
                Direction::East => (0.0, 0.0, 0.0, 1.0, 1.0, 3.0 / 16.0),
                _ => unreachable!(),
            }
        }
    } else {
        match facing {
            Direction::North => (0.0, 0.0, 1.0 - (3.0 / 16.0), 1.0, 1.0, 1.0),
            Direction::South => (0.0, 0.0, 0.0, 1.0, 1.0, 3.0 / 16.0),
            Direction::West => (1.0 - (3.0 / 16.0), 0.0, 0.0, 1.0, 1.0, 1.0),
            Direction::East => (0.0, 0.0, 0.0, 3.0 / 16.0, 1.0, 1.0),
            _ => unreachable!(),
        }
    };

    vec![Aabb3::new(
        Point3::new(min_x, min_y, min_z),
        Point3::new(max_x, max_y, max_z))
    ]
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
    match world.get_block(pos) {
        Block::OakStairs{facing, half, ..} |
        Block::StoneStairs{facing, half, ..} |
        Block::BrickStairs{facing, half, ..} |
        Block::StoneBrickStairs{facing, half, ..} |
        Block::NetherBrickStairs{facing, half, ..} |
        Block::SandstoneStairs{facing, half, ..} |
        Block::SpruceStairs{facing, half, ..} |
        Block::BirchStairs{facing, half, ..} |
        Block::JungleStairs{facing, half, ..} |
        Block::QuartzStairs{facing, half, ..} |
        Block::AcaciaStairs{facing, half, ..} |
        Block::DarkOakStairs{facing, half, ..} |
        Block::RedSandstoneStairs{facing, half, ..} |
        Block::PurpurStairs{facing, half, ..} => Some((facing, half)),
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

fn stair_data(facing: Direction, half: BlockHalf, shape: StairShape) -> Option<usize> {
    if shape != StairShape::Straight { return None; }

    Some((5 - facing.index()) | (if half == BlockHalf::Top { 0x4 } else { 0x0 }))
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            StoneVariant::Normal => "stone",
            StoneVariant::Granite => "granite",
            StoneVariant::SmoothGranite => "smooth_granite",
            StoneVariant::Diorite => "diorite",
            StoneVariant::SmoothDiorite => "smooth_diorite",
            StoneVariant::Andesite => "andesite",
            StoneVariant::SmoothAndesite => "smooth_andesite",
        }
    }
    fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            DirtVariant::Normal => "dirt",
            DirtVariant::Coarse => "coarse_dirt",
            DirtVariant::Podzol => "podzol",
        }
    }

    fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            SandstoneVariant::Normal => "sandstone",
            SandstoneVariant::Chiseled => "chiseled_sandstone",
            SandstoneVariant::Smooth => "smooth_sandstone",
        }
    }

    fn data(&self) -> usize {
        match *self {
            SandstoneVariant::Normal => 0,
            SandstoneVariant::Chiseled => 1,
            SandstoneVariant::Smooth => 2,
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            RedSandstoneVariant::Normal => "red_sandstone",
            RedSandstoneVariant::Chiseled => "chiseled_red_sandstone",
            RedSandstoneVariant::Smooth => "smooth_red_sandstone",
        }
    }

    fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            QuartzVariant::Normal | QuartzVariant::Chiseled => "normal",
            QuartzVariant::PillarVertical => "axis=y",
            QuartzVariant::PillarNorthSouth => "axis=z",
            QuartzVariant::PillarEastWest => "axis=x",
        }
    }

    fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            PrismarineVariant::Normal => "prismarine",
            PrismarineVariant::Brick => "prismarine_bricks",
            PrismarineVariant::Dark => "dark_prismarine",
        }
    }

    fn data(&self) -> usize {
        match *self {
            PrismarineVariant::Normal => 0,
            PrismarineVariant::Brick => 1,
            PrismarineVariant::Dark => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MushroomVariant {
    East,
    North,
    NorthEast,
    NorthWest,
    South,
    SouthEast,
    SouthWest,
    West,
    Center,
    Stem,
    AllInside,
    AllOutside,
    AllStem,
}

impl MushroomVariant {
    pub fn as_string(&self) -> &'static str {
        match *self {
            MushroomVariant::East => "east",
            MushroomVariant::North => "north",
            MushroomVariant::NorthEast => "north_east",
            MushroomVariant::NorthWest => "north_west",
            MushroomVariant::South => "south",
            MushroomVariant::SouthEast => "south_east",
            MushroomVariant::SouthWest => "south_west",
            MushroomVariant::West => "west",
            MushroomVariant::Center => "center",
            MushroomVariant::Stem => "stem",
            MushroomVariant::AllInside => "all_inside",
            MushroomVariant::AllOutside => "all_outside",
            MushroomVariant::AllStem => "all_stem",
        }
    }

    fn data(&self) -> usize {
        match *self {
            MushroomVariant::AllInside => 0,
            MushroomVariant::NorthWest => 1,
            MushroomVariant::North => 2,
            MushroomVariant::NorthEast => 3,
            MushroomVariant::West => 4,
            MushroomVariant::Center => 5,
            MushroomVariant::East => 6,
            MushroomVariant::SouthWest => 7,
            MushroomVariant::South => 8,
            MushroomVariant::SouthEast => 9,
            MushroomVariant::Stem => 10,
            MushroomVariant::AllOutside => 14,
            MushroomVariant::AllStem => 15,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoorHalf {
    Upper,
    Lower
}

impl DoorHalf {
    pub fn as_string(&self) -> &'static str {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
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

    fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
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

    fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            MonsterEggVariant::Stone => "stone",
            MonsterEggVariant::Cobblestone => "cobblestone",
            MonsterEggVariant::StoneBrick => "stone_brick",
            MonsterEggVariant::MossyBrick => "mossy_brick",
            MonsterEggVariant::CrackedBrick => "cracked_brick",
            MonsterEggVariant::ChiseledBrick => "chiseled_brick",
        }
    }

    fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            StoneBrickVariant::Normal => "stonebrick",
            StoneBrickVariant::Mossy => "mossy_stonebrick",
            StoneBrickVariant::Cracked => "cracked_stonebrick",
            StoneBrickVariant::Chiseled => "chiseled_stonebrick",
        }
    }

    fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
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

    pub fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            ComparatorMode::Compare => "compare",
            ComparatorMode::Subtract => "subtract",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LeverDirection {
    North,
    South,
    East,
    West,
    UpX,
    DownX,
    UpZ,
    DownZ,
}

impl LeverDirection {
    pub fn as_string(&self) -> &'static str {
        match *self {
            LeverDirection::North => "north",
            LeverDirection::South => "south",
            LeverDirection::East => "east",
            LeverDirection::West => "west",
            LeverDirection::UpX => "up_x",
            LeverDirection::DownX => "down_x",
            LeverDirection::UpZ => "up_z",
            LeverDirection::DownZ => "down_z",
        }
    }

    pub fn data(&self) -> usize {
        match *self {
            LeverDirection::DownX => 0,
            LeverDirection::East => 1,
            LeverDirection::West => 2,
            LeverDirection::South => 3,
            LeverDirection::North => 4,
            LeverDirection::UpZ => 5,
            LeverDirection::UpX => 6,
            LeverDirection::DownZ => 7,
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            RedstoneSide::None => "none",
            RedstoneSide::Side => "side",
            RedstoneSide::Up => "up",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PistonType {
    Normal,
    Sticky,
}

impl PistonType {
    pub fn as_string(&self) -> &'static str {
        match *self {
            PistonType::Normal => "normal",
            PistonType::Sticky => "sticky",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoneSlabVariant {
    Stone,
    Sandstone,
    Wood,
    Cobblestone,
    Brick,
    StoneBrick,
    NetherBrick,
    Quartz,
    RedSandstone,
    Purpur,
}

impl StoneSlabVariant {
    pub fn as_string(&self) -> &'static str {
        match *self {
            StoneSlabVariant::Stone => "stone",
            StoneSlabVariant::Sandstone => "sandstone",
            StoneSlabVariant::Wood => "wood_old",
            StoneSlabVariant::Cobblestone => "cobblestone",
            StoneSlabVariant::Brick => "brick",
            StoneSlabVariant::StoneBrick => "stone_brick",
            StoneSlabVariant::NetherBrick => "nether_brick",
            StoneSlabVariant::Quartz => "quartz",
            StoneSlabVariant::RedSandstone => "red_sandstone",
            StoneSlabVariant::Purpur => "purpur",
        }
    }

    fn data(&self) -> usize {
        match *self {
            StoneSlabVariant::Stone |
            StoneSlabVariant::RedSandstone |
            StoneSlabVariant::Purpur => 0,
            StoneSlabVariant::Sandstone => 1,
            StoneSlabVariant::Wood => 2,
            StoneSlabVariant::Cobblestone => 3,
            StoneSlabVariant::Brick => 4,
            StoneSlabVariant::StoneBrick => 5,
            StoneSlabVariant::NetherBrick => 6,
            StoneSlabVariant::Quartz => 7,
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            WoodSlabVariant::Oak => "oak",
            WoodSlabVariant::Spruce => "spruce",
            WoodSlabVariant::Birch => "birch",
            WoodSlabVariant::Jungle => "jungle",
            WoodSlabVariant::Acacia => "acacia",
            WoodSlabVariant::DarkOak => "dark_oak",
        }
    }

    fn data(&self) -> usize {
        match *self {
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
}

impl BlockHalf {
    pub fn as_string(&self) -> &'static str {
        match *self {
            BlockHalf::Top => "top",
            BlockHalf::Bottom => "bottom",
            BlockHalf::Upper => "upper",
            BlockHalf::Lower => "lower",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CobblestoneWallVariant {
    Normal,
    Mossy,
}

impl CobblestoneWallVariant {
    pub fn as_string(&self) -> &'static str {
        match *self {
            CobblestoneWallVariant::Normal => "cobblestone",
            CobblestoneWallVariant::Mossy => "mossy_cobblestone",
        }
    }

    pub fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
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

    pub fn data(&self) -> usize {
        match *self {
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            StairShape::Straight => "straight",
            StairShape::InnerLeft => "inner_left",
            StairShape::InnerRight => "inner_right",
            StairShape::OuterLeft => "outer_left",
            StairShape::OuterRight => "outer_right",
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
    DarkOak
}

impl TreeVariant {
    pub fn as_string(&self) -> &'static str {
        match *self {
            TreeVariant::Oak => "oak",
            TreeVariant::Spruce => "spruce",
            TreeVariant::Birch => "birch",
            TreeVariant::Jungle => "jungle",
            TreeVariant::Acacia => "acacia",
            TreeVariant::DarkOak => "dark_oak",
        }
    }

    pub fn data(&self) -> usize {
        match *self {
            TreeVariant::Oak | TreeVariant::Acacia => 0,
            TreeVariant::Spruce | TreeVariant::DarkOak => 1,
            TreeVariant::Birch => 2,
            TreeVariant::Jungle => 3,
        }
    }

    pub fn plank_data(&self) -> usize {
        match *self {
            TreeVariant::Oak => 0,
            TreeVariant::Spruce => 1,
            TreeVariant::Birch => 2,
            TreeVariant::Jungle => 3,
            TreeVariant::Acacia => 4,
            TreeVariant::DarkOak => 5,
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            TallGrassVariant::DeadBush => "dead_bush",
            TallGrassVariant::TallGrass => "tall_grass",
            TallGrassVariant::Fern => "fern",
        }
    }

    fn data(&self) -> usize {
        match *self {
            TallGrassVariant::DeadBush => 0,
            TallGrassVariant::TallGrass => 1,
            TallGrassVariant::Fern => 2,
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
    pub fn as_string(&self) -> &'static str {
        match *self {
            DoublePlantVariant::Sunflower => "sunflower",
            DoublePlantVariant::Lilac => "syringa",
            DoublePlantVariant::DoubleTallgrass => "double_grass",
            DoublePlantVariant::LargeFern => "double_fern",
            DoublePlantVariant::RoseBush => "double_rose",
            DoublePlantVariant::Peony => "paeonia",
        }
    }

    pub fn data(&self) -> usize {
        match *self {
            DoublePlantVariant::Sunflower => 0,
            DoublePlantVariant::Lilac => 1,
            DoublePlantVariant::DoubleTallgrass => 2,
            DoublePlantVariant::LargeFern => 3,
            DoublePlantVariant::RoseBush => 4,
            DoublePlantVariant::Peony => 5,
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
    DarkOak,
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
    pub fn as_string(&self) -> &'static str {
        match *self {
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
            FlowerPotVariant::DarkOak => "dark_oak_sapling",
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
}
