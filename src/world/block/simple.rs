
use super::{Block, BlockSet};
use std::cell::UnsafeCell;

/// A block set for blocks which don't have any special features about them.
pub struct SimpleBlockSet {
    name: &'static str,
    sub_blocks: Vec<SimpleBlock>,
}

block_combos!(SimpleBlockSet, params(),
    SimpleBlock {
    }
);

impl SimpleBlockSet {
    pub fn new(name: &'static str) -> SimpleBlockSet {
        let mut set = SimpleBlockSet {
            name: name,
            sub_blocks: vec![],
        };
        set.gen_combos();
        set
    }
}

impl BlockSet for SimpleBlockSet {
    fn name(&self) -> &'static str {
        self.name
    }

    fn blocks(&'static self) -> Vec<&'static Block> {
        self.sub_blocks.iter().map(|v| v as &Block).collect()
    }
}

impl Block for SimpleBlock {
    block_impl!();
}
