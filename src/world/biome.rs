
#[derive(Clone, Copy)]
pub struct Biome {
    pub id: usize,
    pub temperature: f64,
    pub moisture: f64,
    pub color_index: usize,
}

impl Biome {
    const fn new(id: usize, t: f64, m: f64) -> Biome{
        Biome {
            id: id,
            temperature: t,
            moisture: m*t,
            color_index: (((1.0 - t) * 255.0) as usize) | ((((1.0 - (m*t)) * 255.0) as usize) << 8),
        }
    }

    pub fn by_id(id: usize) -> Biome {
        *BY_ID.get(id).unwrap_or(&INVALID)
    }
}

macro_rules! define_biomes {
    (
        $(pub const $name:ident : Biome = $cr:expr;)*
    ) => (
        $(
            pub const $name: Biome = $cr;
        )*

        lazy_static! {
            static ref BY_ID: [Biome; 256] = {
                let mut by_id = [INVALID; 256];
                $(
                    by_id[$name.id] = $name;
                )*
                by_id
            };
        }
    )
}

define_biomes! {
pub const OCEAN: Biome = Biome::new(0, 0.5, 0.5);
pub const PLAINS: Biome = Biome::new(1, 0.8, 0.4);
pub const DESERT: Biome = Biome::new(2, 2.0, 0.0);
pub const EXTREME_HILLS: Biome = Biome::new(3, 0.2, 0.3);
pub const FOREST: Biome = Biome::new(4, 0.7, 0.8);
pub const TAIGA: Biome = Biome::new(5, 0.05, 0.8);
pub const SWAMPLAND: Biome = Biome::new(6, 0.8, 0.9);
pub const RIVER: Biome = Biome::new(7, 0.5, 0.5);
pub const HELL: Biome = Biome::new(8, 2.0, 0.0);
pub const THE_END: Biome = Biome::new(9, 0.5, 0.5);
pub const FROZEN_OCEAN: Biome = Biome::new(10, 0.0, 0.5);
pub const FROZEN_RIVER: Biome = Biome::new(11, 0.0, 0.5);
pub const ICE_PLAINS: Biome = Biome::new(12, 0.0, 0.5);
pub const ICE_MOUNTAINS: Biome = Biome::new(13, 0.0, 0.5);
pub const MUSHROOM_ISLAND: Biome = Biome::new(14, 0.9, 1.0);
pub const MUSHROOM_ISLAND_SHORE: Biome = Biome::new(15, 0.9, 1.0);
pub const BEACH: Biome = Biome::new(16, 0.8, 0.4);
pub const DESERT_HILLS: Biome = Biome::new(17, 2.0, 0.0);
pub const FOREST_HILLS: Biome = Biome::new(18, 0.7, 0.8);
pub const TAIGA_HILLS: Biome = Biome::new(19, 0.2, 0.7);
pub const EXTREME_HILLS_EDGE: Biome = Biome::new(20, 0.2, 0.3);
pub const JUNGLE: Biome = Biome::new(21, 1.2, 0.9);
pub const JUNGLE_HILLS: Biome = Biome::new(22, 1.2, 0.9);
pub const JUNGLE_EDGE: Biome = Biome::new(23, 0.95, 0.8);
pub const DEEP_OCEAN: Biome = Biome::new(24, 0.5, 0.5);
pub const STONE_BEACH: Biome = Biome::new(25, 0.2, 0.3);
pub const COLD_BEACH: Biome = Biome::new(26, 0.05, 0.3);
pub const BIRCH_FOREST: Biome = Biome::new(27, 0.6, 0.6);
pub const BIRCH_FOREST_HILLS: Biome = Biome::new(28, 0.6, 0.6);
pub const ROOFED_FOREST: Biome = Biome::new(29, 0.7, 0.8);
pub const COLD_TAIGA: Biome = Biome::new(30, -0.5, 0.4);
pub const COLD_TAIGA_HILLS: Biome = Biome::new(31, -0.5, 0.4);
pub const MEGA_TAIGA: Biome = Biome::new(32, 0.3, 0.8);
pub const MEGA_TAIGA_HILLS: Biome = Biome::new(33, 0.3, 0.8);
pub const EXTREME_HILLS_PLUS: Biome = Biome::new(34, 0.2, 0.3);
pub const SAVANNA: Biome = Biome::new(35, 1.2, 0.0);
pub const SAVANNA_PLATEAU: Biome = Biome::new(36, 1.0, 0.0);
pub const MESA: Biome = Biome::new(37, 2.0, 0.0);
pub const MESA_PLATEAU_FOREST: Biome = Biome::new(38, 2.0, 0.0);
pub const MESA_PLATEAU: Biome = Biome::new(39, 2.0, 0.0);

pub const SUNFLOWER_PLAINS: Biome = Biome::new(129, 0.8, 0.4);
pub const DESERT_MOUNTAIN: Biome = Biome::new(130, 2.0, 0.0);
pub const EXTREME_HILLS_MOUNTAINS: Biome = Biome::new(131, 0.2, 0.3);
pub const FLOWER_FOREST: Biome = Biome::new(132, 0.7, 0.8);
pub const TAIGA_M: Biome = Biome::new(133, 0.05, 0.8);
pub const SWAMPLAND_MOUNTAINS: Biome = Biome::new(134, 0.8, 0.9);
pub const ICE_PLAINS_SPIKES: Biome = Biome::new(140, 0.0, 0.5);
pub const JUNGLE_MOUNTAINS: Biome = Biome::new(149, 1.2, 0.9);
pub const JUNGLE_EDGE_MOUNTAINS: Biome = Biome::new(151, 0.95, 0.8);
pub const BIRCH_FOREST_MOUNTAINS: Biome = Biome::new(155, 0.6, 0.6);
pub const BIRCH_FOREST_HILLS_MOUNTAINS: Biome = Biome::new(156, 0.6, 0.6);
pub const ROOFED_FOREST_MOUNTAINS: Biome = Biome::new(157, 0.7, 0.8);
pub const COLD_TAIGA_MOUNTAINS: Biome = Biome::new(158, -0.5, 0.4);
pub const MEGA_SPRUCE_TAIGA: Biome = Biome::new(160, 0.25, 0.8);
pub const MEGA_SPRUCE_TAIGA_HILLS: Biome = Biome::new(161, 0.3, 0.8);
pub const EXTREME_HILLS_PLUS_MOUNTAINS: Biome = Biome::new(162, 0.2, 0.3);
pub const SAVANNA_MOUNTAINS: Biome = Biome::new(163, 1.2, 0.0);
pub const SAVANNA_PLATEAU_MOUNTAINS: Biome = Biome::new(164, 1.0, 0.0);
pub const MESA_BRYCE: Biome = Biome::new(165, 2.0, 0.0);
pub const MESA_PLATEAU_FOREST_MOUNTAINS: Biome = Biome::new(166, 2.0, 0.0);
pub const MESA_PLATEAU_MOUNTAINS: Biome = Biome::new(167, 2.0, 0.0);

pub const INVALID: Biome = Biome::new(255, 0.0, 0.0);
}
