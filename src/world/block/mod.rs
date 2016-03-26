
use std::fmt::{Display, Formatter, Error};
use collision::{Aabb, Aabb3};
use cgmath::{Point3, Point};

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
                $(update_state $update_state:expr,)*
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
                            return Some(internal_ids::$name << 4);
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
                            return "normal".to_owned();
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
                            return TintType::Default;
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
                            return vec![Aabb3::new(
                                Point3::new(0.0, 0.0, 0.0),
                                Point3::new(1.0, 1.0, 1.0)
                            )];
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
                            $(return $update_state;)*
                            return Block::$name {
                                $($fname: $fname,)*
                            };
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
                        blocks[id] = Some(block);
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
                StoneVariant::Granite, StoneVariant::SmoothGranite,
                StoneVariant::Diorite, StoneVariant::SmoothDiorite,
                StoneVariant::Andesite, StoneVariant::SmoothAndesite
            ],
        },
        data { Some(variant.data()) },
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
        props {
        },
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
        props {
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cobblestone" ) },
    }
    Planks { // TODO
        props {
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "planks" ) },
    }
    Sapling { // TODO
        props {
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sapling" ) },
    }
    Bedrock {
        props {
        },
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
    Sand { // TODO
        props {
        },
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sand" ) },
    }
    Gravel {
        props {
        },
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
        props {
        },
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
        props {
        },
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
        props {
        },
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
                TreeVariant::Oak, TreeVariant::Spruce,
                TreeVariant::Birch, TreeVariant::Jungle
            ],
            axis: Axis = [Axis::Y, Axis::Z, Axis::X, Axis::None],
        },
        data { Some(variant.data() | (axis.data() << 2)) },
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
                TreeVariant::Oak, TreeVariant::Spruce,
                TreeVariant::Birch, TreeVariant::Jungle
            ],
            decayable: bool = [false, true],
            check_decay: bool = [false, true],
        },
        data { Some(variant.data()
                    | (if decayable { 0x4 } else { 0x0 })
                    | (if check_decay { 0x8 } else { 0x0 })
        ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sponge" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dispenser" ) },
    }
    Sandstone {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sandstone" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "bed" ) },
    }
    GoldenRail {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "golden_rail" ) },
    }
    DetectorRail {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "detector_rail" ) },
    }
    StickyPiston {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stick_piston" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "tallgrass" ) },
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
        model { ("minecraft", "deadbush" ) },
    }
    Piston {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "piston" ) },
    }
    PistonHead {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "piston_head" ) },
    }
    Wool {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wool" ) },
    }
    PistonExtension {
        props {},
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "yellow_flower" ) },
    }
    RedFlower {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "red_flower" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "double_stone_slab" ) },
    }
    StoneSlab {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_slab" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "tnt" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "torch" ) },
    }
    Fire {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "fire" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "oak_stairs" ) },
    }
    Chest {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "chest" ) },
    }
    RedstoneWire {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "redstone_wire" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wheat" ) },
    }
    Farmland {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "farmland" ) },
    }
    Furnace {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "furnace" ) },
    }
    FurnaceLit {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "furnace_lit" ) },
    }
    StandingSign {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "standing_sign" ) },
    }
    WoodenDoor {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wooden_door" ) },
    }
    Ladder {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "ladder" ) },
    }
    Rail {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "rail" ) },
    }
    StoneStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_stairs" ) },
    }
    WallSign {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wall_sign" ) },
    }
    Lever {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "lever" ) },
    }
    StonePressurePlate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_pressure_plate" ) },
    }
    IronDoor {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "iron_door" ) },
    }
    WoodenPressurePlate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wooden_pressure_plate" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "unlit_redstone_torch" ) },
    }
    RedstoneTorch {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "redstone_torch" ) },
    }
    StoneButton {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_button" ) },
    }
    SnowLayer {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "snow_layer" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cactus" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "reeds" ) },
    }
    Jukebox {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jukebox" ) },
    }
    Fence {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "fence" ) },
    }
    Pumpkin {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "pumpkin" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", "portal" ) },
    }
    PumpkinLit {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "pumpkin_lit" ) },
    }
    Cake {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cake" ) },
    }
    RepeaterUnpowered {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "repeater_unpowered" ) },
    }
    RepeaterPowered {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "repeater_powered" ) },
    }
    StainedGlass {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", "stained_glass" ) },
    }
    TrapDoor {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "trap_door" ) },
    }
    MonsterEgg {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "monster_egg" ) },
    }
    StoneBrick {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stonebrick" ) },
    }
    BrownMushroomBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "brown_mushroom_block" ) },
    }
    RedMushroomBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "red_mushroom_block" ) },
    }
    IronBars {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "iron_bars" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "pumpkin_stem" ) },
    }
    MelonStem {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "melon_stem" ) },
    }
    Vine {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "vine" ) },
    }
    FenceGate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "fence_gate" ) },
    }
    BrickStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "brick_stairs" ) },
    }
    StoneBrickStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_brick_stairs" ) },
    }
    Mycelium {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "mycelium" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "nether_brick_fence" ) },
    }
    NetherBrickStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "nether_brick_stairs" ) },
    }
    NetherWart {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "nether_wart" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "brewing_stand" ) },
    }
    Cauldron {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cauldron" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "end_portal_frame" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "double_wooden_slab" ) },
    }
    WoodenSlab {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wooden_slab" ) },
    }
    Cocoa {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cocoa" ) },
    }
    SandstoneStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sandstone_stairs" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "ender_chest" ) },
    }
    TripwireHook {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "tripwire_hook" ) },
    }
    Tripwire {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "tripwire" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "spruce_stairs" ) },
    }
    BirchStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "birch_stairs" ) },
    }
    JungleStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jungle_stairs" ) },
    }
    CommandBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "command_block" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "cobblestone_wall" ) },
    }
    FlowerPot {
        props {},
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "carrots" ) },
    }
    Potatoes {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "potatoes" ) },
    }
    WoodenButton {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wooden_button" ) },
    }
    Skull {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "skull" ) },
    }
    Anvil {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "anvil" ) },
    }
    TrappedChest {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "trapped_chest" ) },
    }
    LightWeightedPressurePlate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "light_weighted_pressure_plate" ) },
    }
    HeavyWeightedPressurePlate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "heavy_weighted_pressure_plate" ) },
    }
    ComparatorUnpowered {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "comparator_unpowered" ) },
    }
    ComparatorPowered {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "comparator_powered" ) },
    }
    DaylightDetector {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "daylight_detector" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "hopper" ) },
    }
    QuartzBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "quartz_block" ) },
    }
    QuartzStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "quartz_stairs" ) },
    }
    ActivatorRail {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "activator_rail" ) },
    }
    Dropper {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dropper" ) },
    }
    StainedHardenedClay {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stained_hardened_clay" ) },
    }
    StainedGlassPane {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: true,
        },
        model { ("minecraft", "stained_glass_pane" ) },
    }
    Leaves2 {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: true,
            transparent: false,
        },
        model { ("minecraft", "leaves2" ) },
    }
    Log2 {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "log2" ) },
    }
    AcaciaStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "acacia-stairs" ) },
    }
    DarkOakStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dark_oak_stairs" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "sponge" ) },
    }
    Prismarine {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "prismarine" ) },
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
        props {},
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "carpet" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "double_plant" ) },
    }
    StandingBanner {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "standing_banner" ) },
    }
    WallBanner {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "wall_banner" ) },
    }
    DaylightDetectorInverted {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "daylight_detector_inverted" ) },
    }
    RedStonestone {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "red_sandstone" ) },
    }
    RedSandstoneStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "red_sandstone_stairs" ) },
    }
    DoubleStoneSlab2 {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "double_stone_slab2" ) },
    }
    StoneSlab2 {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "stone_slab2" ) },
    }
    SpruceFenceGate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "spruce_fence_gate" ) },
    }
    BirchFenceGate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "birch_fence_gate" ) },
    }
    JungleFenceGate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jungle_fence_gate" ) },
    }
    DarkOakFenceGate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dark_oak_fence_gate" ) },
    }
    AcaciaFenceGate {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "acacia_fence_gate" ) },
    }
    SpruceFence {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "spruce_fence" ) },
    }
    BirchFence {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "birch_fence" ) },
    }
    JungleFence {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jungle_fence" ) },
    }
    DarkOakFence {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dark_oak_fence" ) },
    }
    AcaciaFence {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "acacia_fence" ) },
    }
    SpruceDoor {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "spruce_door" ) },
    }
    BirchDoor {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "birch_door" ) },
    }
    JungleDoor {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "jungle_door" ) },
    }
    AcaciaDoor {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "acacia_door" ) },
    }
    DarkOakDoor {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "dark_oak_door" ) },
    }
    EndRod {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "end_rod" ) },
    }
    ChorusPlant {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "chorus_plant" ) },
    }
    ChorusFlower {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "chorus_flower" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "purpur_pillar" ) },
    }
    PurpurStairs {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "purpur_stairs" ) },
    }
    PurpurDoubleSlab {
        props {},
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "purpur_slab" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: false,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "beetroots" ) },
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
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "repeating_command_block" ) },
    }
    ChainCommandBlock {
        props {},
        material Material {
            renderable: true,
            never_cull: false,
            should_cull_against: true,
            force_shade: false,
            transparent: false,
        },
        model { ("minecraft", "chain_command_block" ) },
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
