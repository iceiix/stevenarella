
use std::collections::HashMap;
use std::cell::UnsafeCell;

pub trait BlockSet {
    fn plugin(&self) -> &'static str {
        "minecraft"
    }

    fn name(&self) -> &'static str;
    fn blocks(&'static self) -> Vec<&'static Block>;

    fn base(&'static self) -> &'static Block {
        self.blocks()[0]
    }
}

pub trait Block: Sync + ::std::fmt::Debug {
    fn steven_id(&'static self) -> usize;
    fn vanilla_id(&'static self) -> Option<usize>;
    fn set_steven_id(&'static self, id: usize);
    fn set_vanilla_id(&'static self, id: usize);
    fn plugin(&self) -> &'static str;
    fn name(&self) -> &'static str;

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

        fn plugin(&self) -> &'static str {
            self.plugin
        }

        fn name(&self) -> &'static str {
            self.name
        }
    )
}

macro_rules! block_combos {
    (
        $set:ty, params($($pname:ident : $pty:ty),*),
        $bty:ident {
            $(
                $fname:ident : $fval:expr,
            )*
        }
    ) => (
        struct $bty {
            steven_id_storage: UnsafeCell<usize>,
            vanilla_id_storage: UnsafeCell<Option<usize>>,
            plugin: &'static str,
            name: &'static str,
            $(
                $fname: $fty,
            )*
        }
        unsafe impl Sync for $bty {}

        impl ::std::fmt::Debug for $bty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                write!(f, "{}:{}", self.plugin(), self.name())
            }
        }

        impl $set {
            fn gen_combos(&mut self, $($pname: $pty),*) {
                let val = $bty {
                    steven_id_storage: UnsafeCell::new(0),
                    vanilla_id_storage: UnsafeCell::new(None),
                    name: self.name(),
                    plugin: self.plugin(),
                    $(
                        $fname: $fval,
                    )*
                };
                self.sub_blocks.push(val);
            }
        }
    );
    (
        $set:ty, params($($pname:ident : $pty:ty),*),
        types (
            $(
                $tname:ident : $tty:ty = [$($val:expr),+]
            ),+
        ),
        $bty:ident {
            $(
                $fname:ident : $fty:ty = $fval:expr,
            )*
        }
    ) => (
        struct $bty {
            steven_id_storage: UnsafeCell<usize>,
            vanilla_id_storage: UnsafeCell<Option<usize>>,
            plugin: &'static str,
            name: &'static str,
            $(
                $fname: $fty,
            )*
        }
        unsafe impl Sync for $bty {}

        impl ::std::fmt::Debug for $bty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                try!(write!(f, "{}:{}[", self.plugin(), self.name()));
                let mut s = String::new();
                $(
                    s.push_str(&format!("{}={},", stringify!($tname), self.$tname));
                )+
                s.pop();
                write!(f, "{}]", s)
            }
        }

        impl $set {
            fn gen_combos(&mut self, $($pname: $pty),*) {
                use std::iter::Iterator;
                #[allow(dead_code)]
                #[allow(non_camel_case_types)]
                struct CombinationIter<$($tname),+> {
                    $(
                        $tname: $tname,
                    )+
                    orig: CombinationIterOrig<$($tname),+> ,
                    last: Option<Wrapper>,
                    done: bool,
                }
                #[allow(non_camel_case_types)]
                struct CombinationIterOrig<$($tname),+> {
                    $(
                        $tname: $tname,
                    )+
                }

                #[derive(Clone)]
                struct Wrapper {
                    $(
                        $tname: $tty,
                    )+
                }

                #[allow(non_camel_case_types)]
                impl <$($tname: Iterator<Item=$tty> + Clone),+> CombinationIter<$($tname),+> {
                    fn new($($tname: $tname),+) -> CombinationIter<$($tname),+> {
                        let orig = CombinationIterOrig {
                            $(
                                $tname: $tname
                            ),+
                        };
                        CombinationIter {
                            $(
                                $tname: orig.$tname.clone(),
                            )+
                            orig: orig,
                            last: None,
                            done: false,
                        }
                    }
                }

                #[allow(non_camel_case_types)]
                impl <$($tname: Iterator<Item=$tty> + Clone),+> Iterator for CombinationIter<$($tname),+> {
                    type Item = Wrapper;

                    fn next(&mut self) -> Option<Self::Item> {
                        if self.done {
                            return None
                        }
                        if self.last.is_none() {
                            let wrapper = Wrapper {
                                $(
                                    $tname: self.$tname.next().unwrap() // Shouldn't ever fail the first iter
                                ),+
                            };
                            self.last = Some(wrapper.clone());
                            return Some(wrapper);
                        }
                        let mut ret = self.last.take().unwrap();
                        $(
                            if let Some(val) = self.$tname.next() {
                                ret.$tname = val;
                                self.last = Some(ret.clone());
                                return Some(ret)
                            }
                            self.$tname = self.orig.$tname.clone();
                        )+
                        self.done = true;
                        None
                    }
                }

                let iter = CombinationIter::new($(vec![$($val),+].into_iter()),+);
                for val in iter {
                    $(
                    let $tname = val.$tname;
                    )+
                    let val = $bty {
                        steven_id_storage: UnsafeCell::new(0),
                        vanilla_id_storage: UnsafeCell::new(None),
                        name: self.name(),
                        plugin: self.plugin(),
                        $(
                            $fname: $fval,
                        )*
                    };
                    self.sub_blocks.push(val);
                }
            }
        }
    )
}

#[test]
fn temp_test() {
    force_init();

    println!("{:?}", STONE.blocks());
    unimplemented!()
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

    fn get_block_by_vanilla_id(&self, id: usize) -> &'static Block {
        self.vanilla_id.get(id).and_then(|v| *v).unwrap_or(MISSING.base())
    }
}

pub fn force_init() {
    MANAGER.force_init();
}

pub fn get_block_by_steven_id(id: usize) -> &'static Block {
    MANAGER.get_block_by_steven_id(id)
}

pub fn get_block_by_vanilla_id(id: usize) -> &'static Block {
    MANAGER.get_block_by_vanilla_id(id)
}

pub mod simple;
pub mod stone;

define_blocks! {
    AIR InvisibleBlockSet = InvisibleBlockSet::new("air");
    STONE stone::StoneBlockSet = stone::StoneBlockSet::new("stone");
    GRASS simple::SimpleBlockSet = simple::SimpleBlockSet::new("grass");
    DIRT simple::SimpleBlockSet = simple::SimpleBlockSet::new("dirt");
    MISSING simple::SimpleBlockSet = simple::SimpleBlockSet::new("missing");
}

/// A block set that contains blocks which cannot be rendered.
pub struct InvisibleBlockSet {
    name: &'static str,
    sub_blocks: Vec<InvisibleBlock>,
}

block_combos!(InvisibleBlockSet, params(),
    InvisibleBlock {
    }
);

impl InvisibleBlockSet {
    fn new(name: &'static str) -> InvisibleBlockSet {
        let mut set = InvisibleBlockSet {
            name: name,
            sub_blocks: vec![],
        };
        set.gen_combos();
        set
    }
}

impl BlockSet for InvisibleBlockSet {
    fn name(&self) -> &'static str {
        self.name
    }

    fn blocks(&'static self) -> Vec<&'static Block> {
        self.sub_blocks.iter().map(|v| v as &Block).collect()
    }
}

impl Block for InvisibleBlock {
    block_impl!();
    fn renderable(&'static self) -> bool {
        false
    }
}
