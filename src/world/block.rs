
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
