
use std::fmt::{Display, Formatter, Error};

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
                data $datafunc:expr,
                material $mat:expr,
                model $model:expr,
                variant $variant:expr,
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

            #[allow(unused_variables)]
            pub fn get_vanilla_id(&self) -> Option<usize> {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)*
                        } => {
                            ($datafunc).map(|v| v + (internal_ids::$name << 4))
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

            #[allow(unused_variables)]
            pub fn get_model_variant(&self) -> String {
                match *self {
                    $(
                        Block::$name {
                            $($fname,)*
                        } => {
                            String::from($variant)
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
    pub force_shade: bool,
}

define_blocks! {
    Air {
        props {},
        data { Some(0) },
        material Material {
            renderable: false,
            should_cull_against: false,
            force_shade: false,
        },
        model { ("minecraft", "air" ) },
        variant "normal",
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
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", variant.as_string() ) },
        variant "normal",
    }
    Grass {
        props {
            snowy: bool = [false, true],
        },
        data { if snowy { None } else { Some(0) } },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "grass" ) },
        variant format!("snowy={}", snowy),
    }
    Dirt {
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "dirt" ) },
        variant "normal",
    }
    Cobblestone {
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "cobblestone" ) },
        variant "normal",
    }
    Planks { // TODO
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "planks" ) },
        variant "normal",
    }
    Sapling { // TODO
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: false,
            force_shade: false,
        },
        model { ("minecraft", "sapling" ) },
        variant "normal",
    }
    Bedrock {
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "bedrock" ) },
        variant "normal",
    }
    FlowingWater { // TODO
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "water" ) },
        variant "normal",
    }
    Water { // TODO
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "water" ) },
        variant "normal",
    }
    FlowingLava { // TODO
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "lava" ) },
        variant "normal",
    }
    Lava {
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "lava" ) },
        variant "normal",
    }
    Sand { // TODO
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "sand" ) },
        variant "normal",
    }
    Gravel {
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "gravel" ) },
        variant "normal",
    }
    GoldOre {
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "gold_ore" ) },
        variant "normal",
    }
    IronOre {
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "iron_ore" ) },
        variant "normal",
    }
    CoalOre {
        props {
        },
        data { Some(0) },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("minecraft", "coal_ore" ) },
        variant "normal",
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
            should_cull_against: true,
            force_shade: false,
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
            should_cull_against: false,
            force_shade: true,
        },
        model { ("minecraft", format!("{}_leaves", variant.as_string()) ) },
        variant "normal",
    }
    Missing {
        props {},
        data { None::<usize> },
        material Material {
            renderable: true,
            should_cull_against: true,
            force_shade: false,
        },
        model { ("steven", "missing_block" ) },
        variant "normal",
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
            Axis::Z => 1,
            Axis::X => 2,
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
