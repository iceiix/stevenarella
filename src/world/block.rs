
use std::collections::HashMap;
use std::cell::UnsafeCell;

pub trait BlockSet {
    fn plugin(&'static self) -> &'static str {
        "minecraft"
    }

    fn name(&'static self) -> &'static str;
    fn blocks(&'static self) -> Vec<&'static Block>;

    fn base(&'static self) -> &'static Block {
        self.blocks()[0]
    }
}

pub trait Block: Sync {
    fn steven_id(&'static self) -> usize;
    fn vanilla_id(&'static self) -> Option<usize>;
    fn set_steven_id(&'static self, id: usize);
    fn set_vanilla_id(&'static self, id: usize);

    fn equals(&'static self, other: &'static Block) -> bool {
        self.steven_id() == other.steven_id()
    }

    fn in_set(&'static self, set: &'static BlockSet) -> bool {
        // TODO: Make faster
        for block in set.blocks() {
            if self.equals(block) {
                return true
            }
        }
        false
    }

    fn renderable(&'static self) -> bool {
        true
    }

    fn data(&'static self) -> Option<u8> {
        Some(0)
    }
}

pub struct BlockManager {
    vanilla_id: Vec<Option<&'static Block>>,
    steven_id: Vec<&'static Block>,
    next_id: usize,
}

macro_rules! define_blocks {
    (
        $(
            $internal_name:ident $ty:ty = $bl:expr;
        )*
    ) => (
        lazy_static! {
            $(
                pub static ref $internal_name: $ty = $bl;
            )*
            static ref MANAGER: BlockManager = {
                let mut manager = BlockManager {
                    vanilla_id: vec![None; 0xFFFF],
                    steven_id: vec![],
                    next_id: 0,
                };
                $(
                    manager.register_set(&*$internal_name);
                )*
                manager
            };
        }
    )
}

// TODO: Replace this with trait fields when supported by rust
macro_rules! block_impl {
    () => (
        fn steven_id(&'static self) -> usize {
            unsafe { *self.steven_id_storage.get() }
        }
        fn vanilla_id(&'static self) -> Option<usize> {
            unsafe { *self.vanilla_id_storage.get() }
        }
        fn set_steven_id(&'static self, id: usize) {
            unsafe { *self.steven_id_storage.get() = id; }
        }
        fn set_vanilla_id(&'static self, id: usize) {
            unsafe { *self.vanilla_id_storage.get() = Some(id); }
        }
    )
}

impl BlockManager {
    fn force_init(&self) {}
    fn register_set(&mut self, set: &'static BlockSet) {
        for block in set.blocks() {
            if let Some(data) = block.data() {
                let id = (self.next_id<<4) | (data as usize);
                self.vanilla_id[id] = Some(block);
                block.set_vanilla_id(id);
            }
            block.set_steven_id(self.steven_id.len());
            self.steven_id.push(block);
        }
        self.next_id += 1;
    }

    fn get_block_by_steven_id(&self, id: usize) -> &'static Block {
        self.steven_id[id]
    }
}

pub fn force_init() {
    MANAGER.force_init();
}

pub fn get_block_by_steven_id(id: usize) -> &'static Block {
    MANAGER.get_block_by_steven_id(id)
}

define_blocks! {
    AIR InvisibleBlockSet = InvisibleBlockSet::new("air");
    MISSING SimpleBlockSet = SimpleBlockSet::new("missing");
}

pub struct InvisibleBlockSet {
    name: &'static str,
    sub_blocks: Vec<InvisibleBlock>,
}

impl InvisibleBlockSet {
    fn new(name: &'static str) -> InvisibleBlockSet {
        let sub_blocks = vec![InvisibleBlock {
            steven_id_storage: UnsafeCell::new(0),
            vanilla_id_storage: UnsafeCell::new(None),
        }];
        InvisibleBlockSet {
            name: name,
            sub_blocks: sub_blocks,
        }
    }
}

impl BlockSet for InvisibleBlockSet {
    fn name(&'static self) -> &'static str {
        self.name
    }

    fn blocks(&'static self) -> Vec<&'static Block> {
        self.sub_blocks.iter().map(|v| v as &Block).collect()
    }
}

struct InvisibleBlock {
    steven_id_storage: UnsafeCell<usize>,
    vanilla_id_storage: UnsafeCell<Option<usize>>,
}

unsafe impl Sync for InvisibleBlock {}
impl Block for InvisibleBlock {
    block_impl!();
    fn renderable(&'static self) -> bool {
        false
    }
}

pub struct SimpleBlockSet {
    name: &'static str,
    sub_blocks: Vec<SimpleBlock>,
}

impl SimpleBlockSet {
    fn new(name: &'static str) -> SimpleBlockSet {
        let sub_blocks = vec![SimpleBlock {
            steven_id_storage: UnsafeCell::new(0),
            vanilla_id_storage: UnsafeCell::new(None),
        }];
        SimpleBlockSet {
            name: name,
            sub_blocks: sub_blocks,
        }
    }
}

impl BlockSet for SimpleBlockSet {
    fn name(&'static self) -> &'static str {
        self.name
    }

    fn blocks(&'static self) -> Vec<&'static Block> {
        self.sub_blocks.iter().map(|v| v as &Block).collect()
    }
}

struct SimpleBlock {
    steven_id_storage: UnsafeCell<usize>,
    vanilla_id_storage: UnsafeCell<Option<usize>>,
}

unsafe impl Sync for SimpleBlock {}
impl Block for SimpleBlock {
    block_impl!();
    fn renderable(&'static self) -> bool {
        true
    }
}

/*
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Block {
    pub render: bool,
}

macro_rules! define_blocks {
    (
        $(
            $name:ident $bl:expr
        )*
    ) => (
        const BLOCKS: &'static [Block] = &[
        $(
                $bl
        ),*
        ];
        mod internal_ids { create_ids!(usize, $($name),*); }
        $(
            pub const $name: &'static Block = &BLOCKS[internal_ids::$name];
        )*

        impl Block {
            pub fn get_id(&self) -> usize {
                $(
                    if self == $name { return internal_ids::$name; }
                )*
                unreachable!()
            }
        }
    )
}

define_blocks! {
    AIR Block {
        render: false,
    }
    MISSING Block {
        render: true,
    }
}

pub fn get_block_by_id(id: usize) -> &'static Block {
    if id >= BLOCKS.len() {
        return MISSING;
    }
    &BLOCKS[id]
}
*/
