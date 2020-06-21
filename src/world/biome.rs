use image::Rgba;
use lazy_static::lazy_static;

#[derive(Clone, Copy)]
pub struct Biome {
    pub id: usize,
    pub temperature: i16,
    pub moisture: i16,
}

impl Biome {
    const fn new(id: usize, t: i16, m: i16) -> Biome {
        Biome {
            id,
            temperature: t,
            moisture: m * t,
        }
    }

    pub fn by_id(id: usize) -> Biome {
        *BY_ID.get(id).unwrap_or(&INVALID)
    }

    pub fn get_color_index(self) -> usize {
        let t = (self.temperature as f64 / 100f64).min(1.0).max(0.0);
        let m = (self.moisture as f64 / 100f64).min(1.0).max(0.0);
        (((1.0 - t) * 255.0) as usize) | ((((1.0 - (m * t)) * 255.0) as usize) << 8)
    }

    pub fn process_color(self, col: Rgba<u8>) -> Rgba<u8> {
        if self.id == ROOFED_FOREST.id || self.id == ROOFED_FOREST_MOUNTAINS.id {
            Rgba([
                ((col.0[0] as u32 + 0x28) / 2) as u8,
                ((col.0[1] as u32 + 0x34) / 2) as u8,
                ((col.0[2] as u32 + 0x0A) / 2) as u8,
                255,
            ])
        } else {
            col
        }
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
pub const OCEAN: Biome = Biome::new(0, 50, 50);
pub const PLAINS: Biome = Biome::new(1, 80, 40);
pub const DESERT: Biome = Biome::new(2, 200, 0);
pub const EXTREME_HILLS: Biome = Biome::new(3, 20, 30);
pub const FOREST: Biome = Biome::new(4, 70, 80);
pub const TAIGA: Biome = Biome::new(5, 5, 80);
pub const SWAMPLAND: Biome = Biome::new(6, 80, 90);
pub const RIVER: Biome = Biome::new(7, 50, 50);
pub const HELL: Biome = Biome::new(8, 200, 0);
pub const THE_END: Biome = Biome::new(9, 50, 50);
pub const FROZEN_OCEAN: Biome = Biome::new(10, 0, 50);
pub const FROZEN_RIVER: Biome = Biome::new(11, 0, 50);
pub const ICE_PLAINS: Biome = Biome::new(12, 0, 50);
pub const ICE_MOUNTAINS: Biome = Biome::new(13, 0, 50);
pub const MUSHROOM_ISLAND: Biome = Biome::new(14, 90, 100);
pub const MUSHROOM_ISLAND_SHORE: Biome = Biome::new(15, 90, 100);
pub const BEACH: Biome = Biome::new(16, 80, 40);
pub const DESERT_HILLS: Biome = Biome::new(17, 200, 0);
pub const FOREST_HILLS: Biome = Biome::new(18, 70, 80);
pub const TAIGA_HILLS: Biome = Biome::new(19, 20, 70);
pub const EXTREME_HILLS_EDGE: Biome = Biome::new(20, 20, 30);
pub const JUNGLE: Biome = Biome::new(21, 120, 90);
pub const JUNGLE_HILLS: Biome = Biome::new(22, 120, 90);
pub const JUNGLE_EDGE: Biome = Biome::new(23, 95, 80);
pub const DEEP_OCEAN: Biome = Biome::new(24, 50, 50);
pub const STONE_BEACH: Biome = Biome::new(25, 20, 30);
pub const COLD_BEACH: Biome = Biome::new(26, 5, 30);
pub const BIRCH_FOREST: Biome = Biome::new(27, 60, 60);
pub const BIRCH_FOREST_HILLS: Biome = Biome::new(28, 60, 60);
pub const ROOFED_FOREST: Biome = Biome::new(29, 70, 80);
pub const COLD_TAIGA: Biome = Biome::new(30, -50, 40);
pub const COLD_TAIGA_HILLS: Biome = Biome::new(31, -50, 40);
pub const MEGA_TAIGA: Biome = Biome::new(32, 30, 80);
pub const MEGA_TAIGA_HILLS: Biome = Biome::new(33, 30, 80);
pub const EXTREME_HILLS_PLUS: Biome = Biome::new(34, 20, 30);
pub const SAVANNA: Biome = Biome::new(35, 120, 0);
pub const SAVANNA_PLATEAU: Biome = Biome::new(36, 100, 0);
pub const MESA: Biome = Biome::new(37, 200, 0);
pub const MESA_PLATEAU_FOREST: Biome = Biome::new(38, 200, 0);
pub const MESA_PLATEAU: Biome = Biome::new(39, 200, 0);

pub const SUNFLOWER_PLAINS: Biome = Biome::new(129, 80, 40);
pub const DESERT_MOUNTAIN: Biome = Biome::new(130, 200, 0);
pub const EXTREME_HILLS_MOUNTAINS: Biome = Biome::new(131, 20, 30);
pub const FLOWER_FOREST: Biome = Biome::new(132, 70, 80);
pub const TAIGA_M: Biome = Biome::new(133, 5, 80);
pub const SWAMPLAND_MOUNTAINS: Biome = Biome::new(134, 80, 90);
pub const ICE_PLAINS_SPIKES: Biome = Biome::new(140, 0, 50);
pub const JUNGLE_MOUNTAINS: Biome = Biome::new(149, 120, 90);
pub const JUNGLE_EDGE_MOUNTAINS: Biome = Biome::new(151, 95, 80);
pub const BIRCH_FOREST_MOUNTAINS: Biome = Biome::new(155, 60, 60);
pub const BIRCH_FOREST_HILLS_MOUNTAINS: Biome = Biome::new(156, 60, 60);
pub const ROOFED_FOREST_MOUNTAINS: Biome = Biome::new(157, 70, 80);
pub const COLD_TAIGA_MOUNTAINS: Biome = Biome::new(158, -50, 40);
pub const MEGA_SPRUCE_TAIGA: Biome = Biome::new(160, 25, 80);
pub const MEGA_SPRUCE_TAIGA_HILLS: Biome = Biome::new(161, 30, 80);
pub const EXTREME_HILLS_PLUS_MOUNTAINS: Biome = Biome::new(162, 20, 30);
pub const SAVANNA_MOUNTAINS: Biome = Biome::new(163, 120, 0);
pub const SAVANNA_PLATEAU_MOUNTAINS: Biome = Biome::new(164, 100, 0);
pub const MESA_BRYCE: Biome = Biome::new(165, 200, 0);
pub const MESA_PLATEAU_FOREST_MOUNTAINS: Biome = Biome::new(166, 200, 0);
pub const MESA_PLATEAU_MOUNTAINS: Biome = Biome::new(167, 200, 0);

pub const INVALID: Biome = Biome::new(255, 0, 0);
}
