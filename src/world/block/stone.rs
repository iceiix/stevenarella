
use super::{Block, BlockSet};
use std::cell::UnsafeCell;
use std::fmt::{Display, Formatter, Error};

/// A block set for stone blocks.
pub struct StoneBlockSet {
    name: &'static str,
    sub_blocks: Vec<StoneBlock>,
}

block_combos!(StoneBlockSet, params(),
    types (
        variant: Variant = [
            Variant::Normal,
            Variant::Granite, Variant::SmoothGranite,
            Variant::Diorite, Variant::SmoothDiorite,
            Variant::Andesite, Variant::SmoothAndesite
        ],
        test_num: u32 = [0, 1, 2, 3, 4, 5, 6]
    ),
    StoneBlock {
        variant: Variant = variant,
        test_num: u32 = test_num,
    }
);

#[derive(Clone, Copy, Debug)]
pub enum Variant {
    Normal,
    Granite,
    SmoothGranite,
    Diorite,
    SmoothDiorite,
    Andesite,
    SmoothAndesite,
}

impl Display for Variant {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", match *self {
            Variant::Normal => "stone",
            Variant::Granite => "granite",
            Variant::SmoothGranite => "smooth_granite",
            Variant::Diorite => "diorite",
            Variant::SmoothDiorite => "smooth_diorite",
            Variant::Andesite => "andesite",
            Variant::SmoothAndesite => "smooth_andesite",
        })
    }
}

impl StoneBlockSet {
    pub fn new(name: &'static str) -> StoneBlockSet {
        let mut set = StoneBlockSet {
            name: name,
            sub_blocks: vec![],
        };
        set.gen_combos();
        set
    }
}

impl BlockSet for StoneBlockSet {
    fn name(&self) -> &'static str {
        self.name
    }

    fn blocks(&'static self) -> Vec<&'static Block> {
        self.sub_blocks.iter().map(|v| v as &Block).collect()
    }
}

impl Block for StoneBlock {
    block_impl!();
}
