use std::fmt::{Display, Formatter, Error};
use collision::{Aabb, Aabb3};
use cgmath::Point3;
use types::Direction;

pub use self::Block::*;

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
                $(update_state ($world:ident, $x:ident, $y:ident, $z:ident) => $update_state:expr,)*
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
            pub fn update_state(&self, world: &super::World, x: i32, y: i32, z: i32) -> Block {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)*
                        } => {
                            $(
                                let $world = world;
                                let $x = x;
                                let $y = y;
                                let $z = z;
                                return $update_state;
                            )*
                            Block::$name {
                                $($fname: $fname,)*
                            }
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

pub struct Material {
    pub renderable: bool,
    pub should_cull_against: bool,
    pub never_cull: bool, // Because leaves suck
    pub force_shade: bool,
    pub transparent: bool,
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
        material Material {
            renderable: false,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "air" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", variant.as_string() ) },
    }
    Grass {
        props {
            snowy: bool = [false, true],
        },
        data { if snowy { None } else { Some(0) } },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "grass" ) },
        variant format!("snowy={}", snowy),
        tint TintType::Grass,
    }
    Dirt {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dirt" ) },
    }
    Cobblestone {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cobblestone" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
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
            stage: i32 = [0, 1],
        },
        data Some(variant.plank_data() | ((stage as usize) << 3)),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", format!("{}_sapling", variant.as_string()) ) },
    }
    Bedrock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "bedrock" ) },
    }
    FlowingWater {
        props {
            level: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(level as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", "water" ) },
    }
    Water {
        props {
            level: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(level as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", "water" ) },
    }
    FlowingLava {
        props {
            level: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(level as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "lava" ) },
    }
    Lava {
        props {
            level: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(level as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "lava" ) },
    }
    Sand {
        props {
            red: bool = [false, true],
        },
        data Some(if red { 1 } else { 0 }),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", if red { "red_sand" } else { "sand" } ) },
    }
    Gravel {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "gravel" ) },
    }
    GoldOre {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "gold_ore" ) },
    }
    IronOre {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "iron_ore" ) },
    }
    CoalOre {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "coal_ore" ) },
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
        data Some(variant.data() | (axis.data() << 2)),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
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
        material Material {
            renderable: true,
            never_cull: true,
            should_cull_against: false,
            force_shade: true,
            transparent: false,
        },
        model { ("minecraft", format!("{}_leaves", variant.as_string()) ) },
        tint TintType::Foliage,
    }
    Sponge {
        props {
            wet: bool = [false, true],
        },
        data Some(if wet { 1 } else { 0 }),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sponge" ) },
        variant format!("wet={}", wet),
    }
    Glass {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "glass" ) },
    }
    LapisOre {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "lapis_ore" ) },
    }
    LapisBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "lapis_block" ) },
    }
    Dispenser {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
            triggered: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::Down => 0,
                Direction::Up => 1,
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => unreachable!(),
            };

            Some(data | (if triggered { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dispenser" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", variant.as_string() ) },
    }
    NoteBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "noteblock" ) },
    }
    Bed {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            occupied: bool = [false, true],
            part: BedPart = [BedPart::Head, BedPart::Foot],
        },
        data {
            let data = match facing {
                Direction::South => 0,
                Direction::West => 1,
                Direction::North => 2,
                Direction::East => 3,
                _ => unreachable!(),
            };

            Some(data
                 | (if occupied { 0x4 } else { 0x0 })
                 | (if part == BedPart::Head { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "bed" ) },
        variant format!("facing={},part={}", facing.as_string(), part.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "golden_rail" ) },
        variant format!("powered={},shape={}", powered, shape.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "detector_rail" ) },
        variant format!("powered={},shape={}", powered, shape.as_string()),
    }
    StickyPiston {
        props {
            extended: bool = [false, true],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
        },
        data {
            let data = match facing {
                Direction::Down => 0,
                Direction::Up => 1,
                Direction::North => 2,
                Direction::South => 3,
                Direction::East => 5,
                Direction::West => 4,
                _ => unreachable!(),
            };

            Some(data | (if extended { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: !extended,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stick_piston" ) },
        variant format!("extended={},facing={}", extended, facing.as_string()),
    }
    Web {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "web" ) },
    }
    TallGrass {
        props {
            variant: TallGrassVariant = [
                TallGrassVariant::DeadBush,
                TallGrassVariant::TallGrass,
                TallGrassVariant::Fern
            ],
        },
        data Some(match variant {
            TallGrassVariant::DeadBush => 0,
            TallGrassVariant::TallGrass => 1,
            TallGrassVariant::Fern => 2,
        }),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", variant.as_string() ) },
        tint TintType::Grass,
    }
    DeadBush {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dead_bush" ) },
    }
    Piston {
        props {
            extended: bool = [false, true],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
        },
        data {
            let data = match facing {
                Direction::Down => 0,
                Direction::Up => 1,
                Direction::North => 2,
                Direction::South => 3,
                Direction::East => 5,
                Direction::West => 4,
                _ => unreachable!(),
            };

            Some(data | (if extended { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: !extended,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "piston" ) },
        variant format!("extended={},facing={}", extended, facing.as_string()),
    }
    PistonHead {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
            short: bool = [false, true],
            variant: PistonType = [PistonType::Normal, PistonType::Sticky],
        },
        data if !short { Some(
            match facing {
                Direction::Down => 0x0,
                Direction::Up => 0x1,
                Direction::North => 0x2,
                Direction::South => 0x3,
                Direction::West => 0x4,
                Direction::East => 0x5,
                _ => unreachable!(),
            } | if variant == PistonType::Sticky { 0x8 } else { 0x0 }
        )} else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "piston_head" ) },
        variant format!("facing={},short={},type={}", facing.as_string(), short, variant.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", format!("{}_wool", color.as_string()) ) },
    }
    PistonExtension {
        props {},
        data Some(0),
        material Material {
            renderable: false,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "piston_extension" ) },
    }
    YellowFlower {
        props { // TODO: Define dandelion? Condense all flower types into one enum?
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "yellow_flower" ) },
        variant "type=dandelion",
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "red_flower" ) },
        variant format!("type={}", variant.as_string()),
    }
    BrownMushroom {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "brown_mushroom" ) },
    }
    RedMushroom {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "red_mushroom" ) },
    }
    GoldBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "gold_block" ) },
    }
    IronBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "iron_block" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "double_stone_slab" ) },
        variant format!("seamless={},variant={}", seamless, variant.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_slab" ) },
        variant format!("half={},variant={}", half.as_string(), variant.as_string()),
    }
    BrickBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "brick_block" ) },
    }
    TNT {
        props {
            explode: bool = [false, true],
        },
        data Some(if explode { 1 } else { 0 }),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "tnt" ) },
        variant format!("explode={}", explode),
    }
    BookShelf {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "bookshelf" ) },
    }
    MossyCobblestone {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "mossy_cobblestone" ) },
    }
    Obsidian {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "obsidian" ) },
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
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "torch" ) },
        variant format!("facing={}", facing.as_string()),
    }
    Fire {
        props {
            age: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            up: bool = [false, true],
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
        },
        data if !up && !north && !south && !east && !west { Some(age as usize) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "fire" ) },
        variant format!("age={},up={},north={},south={},east={},west={}", age, up, north, south, east, west),
    }
    MobSpawner {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "mob_spawner" ) },
    }
    OakStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "oak_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::OakStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
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
        data {
            Some(match facing {
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => 2,
            })
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "chest" ) },
        variant format!("facing={}", facing.as_string()),
    }
    RedstoneWire {
        props {
            north: RedstoneSide = [RedstoneSide::None, RedstoneSide::Side, RedstoneSide::Up],
            south: RedstoneSide = [RedstoneSide::None, RedstoneSide::Side, RedstoneSide::Up],
            east: RedstoneSide = [RedstoneSide::None, RedstoneSide::Side, RedstoneSide::Up],
            west: RedstoneSide = [RedstoneSide::None, RedstoneSide::Side, RedstoneSide::Up],
            power: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data {
            if north == RedstoneSide::None && south == RedstoneSide::None
                && east == RedstoneSide::None && west == RedstoneSide::None  {
                Some(power as usize)
            } else {
                None
            }
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "redstone_wire" ) },
        variant format!("north={},south={},east={},west={},power={}", north.as_string(), south.as_string(), east.as_string(), west.as_string(), power),
    }
    DiamondOre {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "diamond_ore" ) },
    }
    DiamondBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "diamond_block" ) },
    }
    CraftingTable {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "crafting_table" ) },
    }
    Wheat {
        props {
            age: i32 = [0, 1, 2, 3, 4, 5, 6, 7],
        },
        data Some(age as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wheat" ) },
        variant format!("age={}", age),
    }
    Farmland {
        props {
            moisture: i32 = [0, 1, 2, 3, 4, 5, 6, 7],
        },
        data Some(moisture as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "farmland" ) },
        variant format!("moisture={}", moisture),
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
        data {
            Some(match facing {
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => 2,
            })
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "furnace" ) },
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
        data {
            Some(match facing {
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => 2,
            })
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "furnace_lit" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "standing_sign" ) },
        variant format!("rotation={}", rotation.as_string()),
    }
    WoodenDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wooden_door" ) },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
    }
    Ladder {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
        },
        data {
            Some(match facing {
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => 2,
            })
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "ladder" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "rail" ) },
        variant format!("shape={}", shape.as_string()),
    }
    StoneStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::StoneStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    WallSign {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
        },
        data {
            Some(match facing {
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => 2,
            })
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wall_sign" ) },
        variant format!("facing={}", facing.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "lever" ) },
        variant format!("facing={},powered={}", facing.as_string(), powered),
    }
    StonePressurePlate {
        props {
            powered: bool = [false, true],
        },
        data Some(if powered { 1 } else { 0 }),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_pressure_plate" ) },
        variant format!("powered={}", powered),
    }
    IronDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "iron_door" ) },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
    }
    WoodenPressurePlate {
        props {
            powered: bool = [false, true],
        },
        data Some(if powered { 1 } else { 0 }),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wooden_pressure_plate" ) },
        variant format!("powered={}", powered),
    }
    RedstoneOre {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "redstone_ore" ) },
    }
    RedstoneOreLit {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "redstone_ore_lit" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "unlit_redstone_torch" ) },
        variant format!("facing={}", facing.as_string()),
    }
    RedstoneTorch {
        props {
            facing: Direction = [
                Direction::Up,
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West
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
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "redstone_torch" ) },
        variant format!("facing={}", facing.as_string()),
    }
    StoneButton {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
            powered: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::Down => 0,
                Direction::East => 1,
                Direction::West => 2,
                Direction::South => 3,
                Direction::North => 4,
                Direction::Up => 5,
                _ => unreachable!(),
            };

            Some(data | (if powered { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_button" ) },
        variant format!("facing={},powered={}", facing.as_string(), powered),
    }
    SnowLayer {
        props {
            layers: i32 = [1, 2, 3, 4, 5, 6, 7, 8],
        },
        data Some(layers as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "snow_layer" ) },
        variant format!("layers={}", layers),
    }
    Ice {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", "ice" ) },
    }
    Snow {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "snow" ) },
    }
    Cactus {
        props {
            age: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(age as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cactus" ) },
        variant format!("age={}", age),
    }
    Clay {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "clay" ) },
    }
    Reeds {
        props {
            age: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(age as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "reeds" ) },
        variant format!("age={}", age),
    }
    Jukebox {
        props {
            has_record: bool = [false, true],
        },
        data Some(if has_record { 1 } else { 0 }),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jukebox" ) },
        variant format!("has_record={}", has_record),
    }
    Fence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "fence" ) },
        variant format!("north={},south={},east={},west={}", north, south, east, west),
    }
    Pumpkin {
        props {
            facing: Direction = [
                Direction::South,
                Direction::West,
                Direction::North,
                Direction::East
            ],
            without_face: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::South => 0,
                Direction::West => 1,
                Direction::North => 2,
                Direction::East => 3,
                _ => unreachable!(),
            };

            Some(data | (if without_face { 0x4 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "pumpkin" ) },
        variant format!("facing={}", facing.as_string()),
    }
    Netherrack {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "netherrack" ) },
    }
    SoulSand {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "soul_sand" ) },
    }
    Glowstone {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "glowstone" ) },
    }
    Portal {
        props {
            axis: Axis = [Axis::X, Axis::Z],
        },
        data if axis == Axis::X { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", "portal" ) },
        variant format!("axis={}", axis.as_string()),
    }
    PumpkinLit {
        props {
            facing: Direction = [
                Direction::South,
                Direction::West,
                Direction::North,
                Direction::East
            ],
            without_face: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::South => 0,
                Direction::West => 1,
                Direction::North => 2,
                Direction::East => 3,
                _ => unreachable!(),
            };

            Some(data | (if without_face { 0x4 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "pumpkin_lit" ) },
        variant format!("facing={}", facing.as_string()),
    }
    Cake {
        props {
            bites: i32 = [0, 1, 2, 3, 4, 5, 6],
        },
        data Some(bites as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cake" ) },
        variant format!("bites={}", bites),
    }
    RepeaterUnpowered {
        props {
            delay: i32 = [0, 1, 2, 3],
            facing: Direction = [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West
            ],
            locked: bool = [false, true],
        },
        data if !locked {
            let data = match facing {
                Direction::North => 0,
                Direction::East => 1,
                Direction::South => 2,
                Direction::West => 3,
                _ => unreachable!(),
            };

            Some(data | (delay as usize) << 2)
        } else {
            None
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "repeater_unpowered" ) },
        variant format!("delay={},facing={},locked={}", delay, facing.as_string(), locked),
    }
    RepeaterPowered {
        props {
            delay: i32 = [0, 1, 2, 3],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            locked: bool = [false, true],
        },
        data if !locked {
            let data = match facing {
                Direction::North => 0,
                Direction::East => 1,
                Direction::South => 2,
                Direction::West => 3,
                _ => unreachable!(),
            };

            Some(data | (delay as usize) << 2)
        } else {
            None
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "repeater_powered" ) },
        variant format!("delay={},facing={},locked={}", delay, facing.as_string(), locked),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", format!("{}_stained_glass", color.as_string()) ) },
    }
    TrapDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            open: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::South => 0,
                Direction::North => 1,
                Direction::East => 2,
                Direction::West => 3,
                _ => unreachable!(),
            };

            Some(data
                 | (if open { 0x4 } else { 0x0 })
                 | (if half == BlockHalf::Top { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "trap_door" ) },
        variant format!("facing={},half={},open={}", facing.as_string(), half.as_string(), open),
    }
    MonsterEgg {
        props {
            variant: MonsterEggVariant = [
                MonsterEggVariant::Stone,
                MonsterEggVariant::Cobblestone,
                MonsterEggVariant::StoneBrick,
                MonsterEggVariant::MossyStoneBrick,
                MonsterEggVariant::CrackedStoneBrick,
                MonsterEggVariant::ChiseledStoneBrick
            ],
        },
        data Some(variant.data()),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "monster_egg" ) },
        variant format!("variant={}", variant.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stonebrick" ) },
        variant format!("variant={}", variant.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "brown_mushroom_block" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "red_mushroom_block" ) },
        variant format!("variant={}", variant.as_string()),
    }
    IronBars {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "iron_bars" ) },
        variant format!("north={},south={},east={},west={}", north, south, east, west),
    }
    GlassPane {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "glass_pane" ) },
    }
    MelonBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "melon_block" ) },
    }
    PumpkinStem {
        props {
            age: i32 = [0, 1, 2, 3, 4, 5, 6, 7],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up
            ],
        },
        data if facing == Direction::North { Some(age as usize) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "pumpkin_stem" ) },
        variant format!("age={},facing={}", age, facing.as_string()),
    }
    MelonStem {
        props {
            age: i32 = [0, 1, 2, 3, 4, 5, 6, 7],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up
            ],
        },
        data if facing == Direction::North { Some(age as usize) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "melon_stem" ) },
        variant format!("age={},facing={}", age, facing.as_string()),
    }
    Vine {
        props {
             north: bool = [false, true],
             south: bool = [false, true],
             east: bool = [false, true],
             west: bool = [false, true],
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "vine" ) },
        variant format!("east={},north={},south={},up={},west={}", east, north, south, up, west),
        tint TintType::Foliage,
    }
    FenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "fence_gate" ) },
        variant format!("facing={},in_wall={},open={},powered={}", facing.as_string(), in_wall, open, powered),
    }
    BrickStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "brick_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::BrickStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    StoneBrickStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_brick_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::StoneBrickStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    Mycelium {
        props {
            snowy: bool = [false, true],
        },
        data if snowy { None } else { Some(0) },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "mycelium" ) },
        variant format!("snowy={}", snowy),
    }
    Waterlily {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "waterlily" ) },
        tint TintType::Foliage,
    }
    NetherBrick {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "nether_brick" ) },
    }
    NetherBrickFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "nether_brick_fence" ) },
        variant format!("north={},south={},east={},west={}", north, south, east, west),
    }
    NetherBrickStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "nether_brick_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::NetherBrickStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    NetherWart {
        props {
            age: i32 = [0, 1, 2, 3],
        },
        data Some(age as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "nether_wart" ) },
        variant format!("age={}", age),
    }
    EnchantingTable {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "enchanting_table" ) },
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
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "brewing_stand" ) },
        variant format!("has_bottle_0={},has_bottle_1={},has_bottle_2={}", has_bottle_0, has_bottle_1, has_bottle_2),
    }
    Cauldron {
        props {
            level: i32 = [0, 1, 2, 3],
        },
        data Some(level as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cauldron" ) },
        variant format!("level={}", level),
    }
    EndPortal {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "end_portal" ) },
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
        data {
            let data = match facing {
                Direction::South => 0,
                Direction::West => 1,
                Direction::North => 2,
                Direction::East => 3,
                _ => unreachable!(),
            };

            Some(data | (if eye { 0x4 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "end_portal_frame" ) },
        variant format!("eye={},facing={}", eye, facing.as_string()),
    }
    EndStone {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "end_stone" ) },
    }
    DragonEgg {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dragon_egg" ) },
    }
    RedstoneLamp {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "redstone_lamp" ) },
    }
    RedstoneLampLit {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "redstone_lamp_lit" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "double_wooden_slab" ) },
        variant format!("variant={}", variant.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wooden_slab" ) },
        variant format!("half={},variant={}", half.as_string(), variant.as_string()),
    }
    Cocoa {
        props {
            age: i32 = [0, 1, 2],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
        },
        data {
            let data = match facing {
                Direction::South => 0,
                Direction::West => 1,
                Direction::North => 2,
                Direction::East => 3,
                _ => unreachable!(),
            };

            Some(data | (age as usize) << 2)
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cocoa" ) },
        variant format!("age={},facing={}", age, facing.as_string()),
    }
    SandstoneStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sandstone_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::SandstoneStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    EmeraldOre {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "emerald_ore" ) },
    }
    EnderChest {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
        },
        data {
            Some(match facing {
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => 2,
            })
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "ender_chest" ) },
        variant format!("facing={}", facing.as_string()),
    }
    TripwireHook {
        props {
            attached: bool = [false, true],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            powered: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::South => 0,
                Direction::West => 1,
                Direction::North => 2,
                Direction::East => 3,
                _ => unreachable!(),
            };

            Some(data
                 | (if attached { 0x4 } else { 0x0 })
                 | (if powered { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "tripwire_hook" ) },
        variant format!("attached={},facing={},powered={}", attached, facing.as_string(), powered),
    }
    Tripwire {
        props {
            attached: bool = [false, true],
            disarmed: bool = [false, true],
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
            powered: bool = [false, true],
        },
        data {
            if !north && !south && !east && !west {
                Some((if powered { 0x1 } else { 0x0 })
                     | (if attached { 0x4 } else { 0x0 })
                     | (if disarmed { 0x8 } else { 0x0 }))
            } else {
                None
            }
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "tripwire" ) },
        variant format!("attached={},disarmed={},north={},south={},east={},west={},powered={}", attached, disarmed, north, south, east, west, powered),
    }
    EmeraldBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "emerald_block" ) },
    }
    SpruceStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "spruce_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::SpruceStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    BirchStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "birch_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::BirchStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    JungleStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jungle_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::JungleStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    CommandBlock {
        props {
            conditional: bool = [false, true],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
        },
        data command_block_data(conditional, facing),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "command_block" ) },
        variant format!("conditional={},facing={}", conditional, facing.as_string()),
    }
    Beacon {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "beacon" ) },
    }
    CobblestoneWall {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
            up: bool = [false, true],
            variant: CobblestoneWallVariant = [
                CobblestoneWallVariant::Normal,
                CobblestoneWallVariant::Mossy
            ],
        },
        data if !north && !south && !east && !west && !up { Some(variant.data()) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cobblestone_wall" ) },
        variant format!("north={},south={},east={},west={},up={},variant={}", north, south, east, west, up, variant.data()),
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
            legacy_data: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data if contents == FlowerPotVariant::Empty { Some(legacy_data as usize) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "flower_pot" ) },
    }
    Carrots {
        props {
            age: i32 = [0, 1, 2, 3, 4, 5, 6, 7],
        },
        data Some(age as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "carrots" ) },
        variant format!("age={}", age),
    }
    Potatoes {
        props {
            age: i32 = [0, 1, 2, 3, 4, 5, 6, 7],
        },
        data Some(age as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "potatoes" ) },
        variant format!("age={}", age),
    }
    WoodenButton {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
            powered: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::Down => 0,
                Direction::East => 1,
                Direction::West => 2,
                Direction::South => 3,
                Direction::North => 4,
                Direction::Up => 5,
                _ => unreachable!(),
            };

            Some(data | (if powered { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wooden_button" ) },
        variant format!("facing={},powered={}", facing.as_string(), powered),
    }
    Skull {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up
            ],
            nodrop: bool = [false, true],
        },
        data if !nodrop {
            Some(match facing {
                Direction::Up => 1,
                Direction::North => 2,
                Direction::South => 3,
                Direction::East => 4,
                Direction::West => 5,
                _ => unreachable!(),
            })
        } else {
            None
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "skull" ) },
        variant format!("facing={},nodrop={}", facing.as_string(), nodrop),
    }
    Anvil {
        props {
            damage: i32 = [0, 1, 2],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
        },
        data {
            let data = match facing {
                Direction::North => 0,
                Direction::East => 1,
                Direction::South => 2,
                Direction::West => 3,
                _ => unreachable!(),
            };

            Some(data | (if damage == 0 { 0x0 }
                 else if damage == 1 { 0x4 }
                 else if damage == 2 { 0x8 }
                 else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "anvil" ) },
        variant format!("damage={},facing={}", damage, facing.as_string()),
    }
    TrappedChest {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
        },
        data {
            Some(match facing {
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => 2,
            })
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "trapped_chest" ) },
        variant format!("facing={}", facing.as_string()),
    }
    LightWeightedPressurePlate {
        props {
            power: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(power as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "light_weighted_pressure_plate" ) },
        variant format!("power={}", power),
    }
    HeavyWeightedPressurePlate {
        props {
            power: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(power as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "heavy_weighted_pressure_plate" ) },
        variant format!("power={}", power),
    }
    ComparatorUnpowered {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            mode: ComparatorMode = [ComparatorMode::Compare, ComparatorMode::Subtract],
            powered: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::North => 0,
                Direction::East => 1,
                Direction::South => 2,
                Direction::West => 3,
                _ => unreachable!(),
            };

            Some(data
                 | (if mode == ComparatorMode::Subtract { 0x4 } else { 0x0 })
                 | (if powered { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "comparator_unpowered" ) },
        variant format!("facing={},mode={},powered={}", facing.as_string(), mode.as_string(), powered),
    }
    ComparatorPowered {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            mode: ComparatorMode = [ComparatorMode::Compare, ComparatorMode::Subtract],
            powered: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::North => 0,
                Direction::East => 1,
                Direction::South => 2,
                Direction::West => 3,
                _ => unreachable!(),
            };

            Some(data
                 | (if mode == ComparatorMode::Subtract { 0x4 } else { 0x0 })
                 | (if powered { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "comparator_powered" ) },
        variant format!("facing={},mode={},powered={}", facing.as_string(), mode.as_string(), powered),
    }
    DaylightDetector {
        props {
            power: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(power as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "daylight_detector" ) },
        variant format!("power={}", power),
    }
    RedstoneBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "redstone_block" ) },
    }
    QuartzOre {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "quartz_ore" ) },
    }
    Hopper {
        props {
            enabled: bool = [false, true],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Down
            ],
        },
        data {
            let data = match facing {
                Direction::Down => 0,
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => unreachable!(),
            };

            Some(data | (if enabled { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "hopper" ) },
        variant format!("enabled={},facing={}", enabled, facing.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "quartz_block" ) },
        variant format!("variant={}", variant.as_string()),
    }
    QuartzStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "quartz_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::QuartzStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    ActivatorRail {
        props {
            shape: RailShape = [
                RailShape::NorthSouth,
                RailShape::EastWest,
                RailShape::AscendingNorth,
                RailShape::AscendingSouth,
                RailShape::AscendingEast,
                RailShape::AscendingWest,
                RailShape::NorthEast,
                RailShape::NorthWest,
                RailShape::SouthEast,
                RailShape::SouthWest
            ],
        },
        data Some(shape.data()),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "activator_rail" ) },
        variant format!("shape={}", shape.as_string()),
    }
    Dropper {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
            triggered: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::Down => 0,
                Direction::Up => 1,
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => unreachable!(),
            };

            Some(data | (if triggered { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dropper" ) },
        variant format!("facing={},triggered={}", facing.as_string(), triggered),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
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
        },
        data Some(color.data()),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", format!("{}_stained_glass_pane", color.as_string()) ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: true,
            transparent: false,
        },
        model { ("minecraft", "leaves2" ) },
        variant format!("check_decay={},decayable={},variant={}", check_decay, decayable, variant.as_string()),
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
        data Some(variant.data() | (axis.data() << 2)),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "log2" ) },
        variant format!("axis={}", axis.as_string()),
    }
    AcaciaStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "acacia_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::AcaciaStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    DarkOakStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dark_oak_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::DarkOakStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    Slime {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", "slime" ) },
    }
    Barrier {
        props {},
        material Material {
            renderable: false,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "barrier" ) },
    }
    IronTrapDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            open: bool = [false, true],
        },
        data {
            let data = match facing {
                Direction::South => 0,
                Direction::North => 1,
                Direction::East => 2,
                Direction::West => 3,
                _ => unreachable!(),
            };

            Some(data
                 | (if open { 0x4 } else { 0x0 })
                 | (if half == BlockHalf::Top { 0x8 } else { 0x0 }))
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sponge" ) },
        variant format!("facing={},half={},open={}", facing.as_string(), half.as_string(), open),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "prismarine" ) },
        variant format!("variant={}", variant.as_string()),
    }
    SeaLantern {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sea_lantern" ) },
    }
    HayBlock {
        props {
            axis: Axis = [Axis::X, Axis::Y, Axis::Z],
        },
        data if axis == Axis::X { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "hay_block" ) },
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", format!("{}_carpet", color.as_string()) ) },
    }
    HardenedClay {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "hardened_clay" ) },
    }
    CoalBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "coal_block" ) },
    }
    PackedIce {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "packed_ice" ) },
    }
    DoublePlant {
        props {
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            variant: DoublePlantVariant = [
                DoublePlantVariant::Sunflower,
                DoublePlantVariant::Lilac,
                DoublePlantVariant::DoubleTallgrass,
                DoublePlantVariant::LargeFern,
                DoublePlantVariant::Peony
            ],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
        },
        data if facing == Direction::North {
            Some((if half == DoorHalf::Upper { 0x8 } else { 0x0 }) | variant.data())
        } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "double_plant" ) },
        variant format!("half={},variant={},facing={}", half.as_string(), variant.as_string(), facing.as_string()),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "standing_banner" ) },
        variant format!("rotation={}", rotation.as_string()),
    }
    WallBanner {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
        },
        data Some(match facing {
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => unreachable!(),
             }),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wall_banner" ) },
        variant format!("facing={}", facing.as_string()),
    }
    DaylightDetectorInverted {
        props {
            power: i32 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        },
        data Some(power as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "daylight_detector_inverted" ) },
        variant format!("power={}", power),
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
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "red_sandstone" ) },
        variant format!("type={}", variant.as_string()),
    }
    RedSandstoneStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "red_sandstone_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::RedSandstoneStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    DoubleStoneSlab2 {
        props {
            seamless: bool = [false, true],
            variant: StoneSlabVariant = [
                StoneSlabVariant::RedSandstone
            ],
        },
        data Some(variant.data() | (if seamless { 0x8 } else { 0x0 })),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "double_stone_slab2" ) },
        variant format!("seamless={},variant={}", seamless, variant.as_string()),
    }
    StoneSlab2 {
        props {
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            variant: StoneSlabVariant = [StoneSlabVariant::RedSandstone],
        },
        data Some(variant.data() | (if half == BlockHalf::Top { 0x8 } else { 0x0 })),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_slab2" ) },
        variant format!("half={},variant={}", half.as_string(), variant.as_string()),
    }
    SpruceFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "spruce_fence_gate" ) },
        variant format!("facing={},in_wall={},open={},powered={}", facing.as_string(), in_wall, open, powered),
    }
    BirchFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "birch_fence_gate" ) },
        variant format!("facing={},in_wall={},open={},powered={}", facing.as_string(), in_wall, open, powered),
    }
    JungleFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jungle_fence_gate" ) },
        variant format!("facing={},in_wall={},open={},powered={}", facing.as_string(), in_wall, open, powered),
    }
    DarkOakFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dark_oak_fence_gate" ) },
        variant format!("facing={},in_wall={},open={},powered={}", facing.as_string(), in_wall, open, powered),
    }
    AcaciaFenceGate {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            in_wall: bool = [false, true],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data fence_gate_data(facing, in_wall, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "acacia_fence_gate" ) },
        variant format!("facing={},in_wall={},open={},powered={}", facing.as_string(), in_wall, open, powered),
    }
    SpruceFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "spruce_fence" ) },
        variant format!("north={},south={},east={},west={}", north, south, east, west),
    }
    BirchFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "birch_fence" ) },
        variant format!("north={},south={},east={},west={}", north, south, east, west),
    }
    JungleFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jungle_fence" ) },
        variant format!("north={},south={},east={},west={}", north, south, east, west),
    }
    DarkOakFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dark_oak_fence" ) },
        variant format!("north={},south={},east={},west={}", north, south, east, west),
    }
    AcaciaFence {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
        },
        data if !north && !south && !east && !west { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "acacia_fence" ) },
        variant format!("north={},south={},east={},west={}", north, south, east, west),
    }
    SpruceDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "spruce_door" ) },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
    }
    BirchDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "birch_door" ) },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
    }
    JungleDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jungle_door" ) },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
    }
    AcaciaDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "acacia_door" ) },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
    }
    DarkOakDoor {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: DoorHalf = [DoorHalf::Upper, DoorHalf::Lower],
            hinge: Side = [Side::Left, Side::Right],
            open: bool = [false, true],
            powered: bool = [false, true],
        },
        data door_data(facing, half, hinge, open, powered),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dark_oak_door" ) },
        variant format!("facing={},half={},hinge={},open={}", facing.as_string(), half.as_string(), hinge.as_string(), open),
    }
    EndRod {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
        },
        data {
            Some(match facing {
                Direction::Down => 0,
                Direction::Up => 1,
                Direction::North => 2,
                Direction::South => 3,
                Direction::West => 4,
                Direction::East => 5,
                _ => unreachable!(),
            })
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "end_rod" ) },
        variant format!("facing={}", facing.as_string()),
    }
    ChorusPlant {
        props {
            north: bool = [false, true],
            south: bool = [false, true],
            east: bool = [false, true],
            west: bool = [false, true],
            up: bool = [false, true],
            down: bool = [false, true],
        },
        data if !north && !south && !east && !west && !up && !down { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "chorus_plant" ) },
        variant format!("north={},south={},east={},west={},up={},down={}", north, south, east, west, up, down),
    }
    ChorusFlower {
        props {
            age: i32 = [0, 1, 2, 3, 4, 5],
        },
        data Some(age as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "chorus_flower" ) },
        variant format!("age={}", age),
    }
    PurpurBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "purpur_block" ) },
    }
    PurpurPillar {
        props {
            axis: Axis = [Axis::X, Axis::Y, Axis::Z],
        },
        data if axis == Axis::X { Some(0) } else { None },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "purpur_pillar" ) },
        variant format!("axis={}", axis.as_string()),
    }
    PurpurStairs {
        props {
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West
            ],
            half: StairHalf = [StairHalf::Top, StairHalf::Bottom],
            shape: StairShape = [
                StairShape::Straight,
                StairShape::InnerLeft, StairShape::InnerRight,
                StairShape::OuterLeft, StairShape::OuterRight
            ],
        },
        data stair_data(facing, half, shape),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "purpur_stairs" ) },
        variant format!("facing={},half={},shape={}", facing.as_string(), half.as_string(), shape.as_string()),
        update_state (world, x, y, z) => Block::PurpurStairs{facing: facing, half: half, shape: update_stair_shape(world, x, y, z, facing)},
    }
    PurpurDoubleSlab {
        props {
            variant: StoneSlabVariant = [StoneSlabVariant::Purpur],
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "purpur_double_slab" ) },
    }
    PurpurSlab {
        props {
            half: BlockHalf = [BlockHalf::Top, BlockHalf::Bottom],
            variant: StoneSlabVariant = [StoneSlabVariant::Purpur],
        },
        data {
            if half == BlockHalf::Top {
                Some(0x8)
            } else {
                Some(0x0)
            }
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "purpur_slab" ) },
        variant format!("half={},variant={}", half.as_string(), variant.as_string()),
    }
    EndBricks {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "end_bricks" ) },
    }
    Beetroots {
        props {
            age: i32 = [0, 1, 2, 3],
        },
        data Some(age as usize),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "beetroots" ) },
        variant format!("age={}", age),
    }
    GrassPath {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "grass_path" ) },
    }
    EndGateway {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "end_gateway" ) },
    }
    RepeatingCommandBlock {
        props {
            conditional: bool = [false, true],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
        },
        data command_block_data(conditional, facing),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "repeating_command_block" ) },
        variant format!("conditional={},facing={}", conditional, facing.as_string()),
    }
    ChainCommandBlock {
        props {
            conditional: bool = [false, true],
            facing: Direction = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
                Direction::Up,
                Direction::Down
            ],
        },
        data command_block_data(conditional, facing),
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "chain_command_block" ) },
        variant format!("conditional={},facing={}", conditional, facing.as_string()),
    }
    FrostedIce {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "frosted_ice" ) },
    }
    Missing {
        props {},
        data { None::<usize> },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("steven", "missing_block" ) },
    }
}

fn fence_gate_data(facing: Direction, in_wall: bool, open: bool, powered: bool) -> Option<usize> {
    if in_wall || powered {
        return None;
    }

    let data = match facing {
        Direction::South => 0,
        Direction::West => 1,
        Direction::North => 2,
        Direction::East => 3,
        _ => unreachable!(),
    };

    Some(data | (if open { 0x4 } else { 0x0 }))
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
                let data = match facing {
                    Direction::West => 0,
                    Direction::North => 1,
                    Direction::East => 2,
                    Direction::South => 3,
                    _ => unreachable!(),
                };

                Some(data | (if open { 0x4 } else { 0x0 }))
            } else {
                None
            }
        }
    }
}

fn command_block_data(conditional: bool, facing: Direction) -> Option<usize> {
    let data = match facing {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
        _ => unreachable!(),
    };

    Some(data | (if conditional { 0x8 } else { 0x0 }))
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

impl Display for StoneVariant {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.as_string())
    }
}

impl StoneVariant {
    fn as_string(&self) -> &'static str {
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
            QuartzVariant::Normal => "default",
            QuartzVariant::Chiseled => "chiseled",
            QuartzVariant::PillarVertical => "lines_x",
            QuartzVariant::PillarNorthSouth => "lines_y",
            QuartzVariant::PillarEastWest => "lines_z",
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
            PrismarineVariant::Brick => "prismarine_brick",
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
            ColoredVariant::LightBlue => "lightBlue",
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
    MossyStoneBrick,
    CrackedStoneBrick,
    ChiseledStoneBrick,
}

impl MonsterEggVariant {
    pub fn as_string(&self) -> &'static str {
        match *self {
            MonsterEggVariant::Stone => "stone",
            MonsterEggVariant::Cobblestone => "cobblestone",
            MonsterEggVariant::StoneBrick => "stone_brick",
            MonsterEggVariant::MossyStoneBrick => "mossy_brick",
            MonsterEggVariant::CrackedStoneBrick => "cracked_brick",
            MonsterEggVariant::ChiseledStoneBrick => "chiseled_brick",
        }
    }

    fn data(&self) -> usize {
        match *self {
            MonsterEggVariant::Stone => 0,
            MonsterEggVariant::Cobblestone => 1,
            MonsterEggVariant::StoneBrick => 2,
            MonsterEggVariant::MossyStoneBrick => 3,
            MonsterEggVariant::CrackedStoneBrick => 4,
            MonsterEggVariant::ChiseledStoneBrick => 5,
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
            StoneSlabVariant::Stone => 0,
            StoneSlabVariant::Sandstone => 1,
            StoneSlabVariant::Wood => 2,
            StoneSlabVariant::Cobblestone => 3,
            StoneSlabVariant::Brick => 4,
            StoneSlabVariant::StoneBrick => 5,
            StoneSlabVariant::NetherBrick => 6,
            StoneSlabVariant::Quartz => 7,
            StoneSlabVariant::RedSandstone => 0,
            StoneSlabVariant::Purpur => 0,
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
}

impl BlockHalf {
    pub fn as_string(&self) -> &'static str {
        match *self {
            BlockHalf::Top => "top",
            BlockHalf::Bottom => "bottom",
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

    fn data(&self) -> usize {
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
pub enum Axis {
    Y,
    Z,
    X,
    None
}

impl Axis {
    pub fn as_string(&self) -> &'static str {
        match *self {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Z => "z",
            Axis::None => "none",
        }
    }

    fn data(&self) -> usize {
        match *self {
            Axis::Y => 0,
            Axis::Z => 2,
            Axis::X => 1,
            Axis::None => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StairHalf {
    Top,
    Bottom,
}

impl StairHalf {
    pub fn as_string(&self) -> &'static str {
        match *self {
            StairHalf::Top => "top",
            StairHalf::Bottom => "bottom",
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

fn get_stair_info(world: &super::World, x: i32, y: i32, z: i32) -> Option<(Direction, StairHalf)> {
    use self::Block::*;
    match world.get_block(x, y, z) {
        OakStairs{facing, half, ..} |
        StoneStairs{facing, half, ..} |
        BrickStairs{facing, half, ..} |
        StoneBrickStairs{facing, half, ..} |
        NetherBrickStairs{facing, half, ..} |
        SandstoneStairs{facing, half, ..} |
        SpruceStairs{facing, half, ..} |
        BirchStairs{facing, half, ..} |
        JungleStairs{facing, half, ..} |
        QuartzStairs{facing, half, ..} |
        AcaciaStairs{facing, half, ..} |
        DarkOakStairs{facing, half, ..} |
        RedSandstoneStairs{facing, half, ..} |
        PurpurStairs{facing, half, ..} => Some((facing, half)),
        _ => None,
    }
}

fn update_stair_shape(world: &super::World, x: i32, y: i32, z: i32, facing: Direction) -> StairShape {
    let (ox, oy, oz) = facing.get_offset();
    if let Some((other_facing, _)) = get_stair_info(world, x+ox, y+oy, z+oz) {
        if other_facing != facing && other_facing != facing.opposite() {
            if other_facing == facing.clockwise() {
                return StairShape::OuterRight;
            }

            return StairShape::OuterLeft;
        }
    }

    let (ox, oy, oz) = facing.opposite().get_offset();
    if let Some((other_facing, _)) = get_stair_info(world, x+ox, y+oy, z+oz) {
        if other_facing != facing && other_facing != facing.opposite() {
            if other_facing == facing.clockwise() {
                return StairShape::InnerRight;
            }

            return StairShape::InnerLeft;
        }
    }

    StairShape::Straight
}

fn stair_data(facing: Direction, half: StairHalf, shape: StairShape) -> Option<usize> {
    if shape != StairShape::Straight {
        return None;
    }

    let data = match facing {
        Direction::East => 0,
        Direction::West => 1,
        Direction::South => 2,
        Direction::North => 3,
        _ => unreachable!(),
    };

    Some(data | (if half == StairHalf::Top { 0x4 } else { 0x0 }))
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
            TreeVariant::Oak => 0,
            TreeVariant::Spruce => 1,
            TreeVariant::Birch => 2,
            TreeVariant::Jungle => 3,
            TreeVariant::Acacia => 0,
            TreeVariant::DarkOak => 1,
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
    // Not included in the data value:
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

    fn data(&self) -> usize { // TODO: How should this be implemented?
        0
    }
}
