
use types::bit;
use types::hash::FNVHash;
use world::block;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;

pub struct BlockStorage {
    blocks: bit::Map,
    block_map: Vec<(block::Block, u32)>,
    rev_block_map: HashMap<block::Block, usize, BuildHasherDefault<FNVHash>>,
}

impl BlockStorage {
    pub fn new(size: usize) -> BlockStorage {
        let mut storage = BlockStorage {
            blocks: bit::Map::new(size, 4),
            block_map: vec![
                (block::Air{}, 0xFFFFFFFF)
            ],
            rev_block_map: HashMap::with_hasher(BuildHasherDefault::default()),
        };
        storage.rev_block_map.insert(block::Air{}, 0);
        storage
    }

    pub fn get(&self, idx: usize) -> block::Block {
        let idx = self.blocks.get(idx);
        self.block_map.get(idx).map_or(block::Missing{}, |v| v.0)
    }

    pub fn set(&mut self, idx: usize, b: block::Block) -> bool {
        use std::collections::hash_map::Entry;
        let old = self.get(idx);
        if old == b {
            return false;
        }
        // Clean up old block
        {
            let idx = self.rev_block_map[&old];
            let info = &mut self.block_map[idx];
            info.1 -= 1;
            if info.1 == 0 { // None left of this type
                self.rev_block_map.remove(&old);
            }
        }

        if let Entry::Vacant(entry) = self.rev_block_map.entry(b) {
            let mut found = false;
            let id = entry.insert(self.block_map.len());
            for (i, ref mut info) in self.block_map.iter_mut().enumerate() {
                if info.1 == 0 {
                    info.0 = b;
                    *id = i;
                    found = true;
                    break;
                }
            }
            if !found {
                if self.block_map.len() >= 1 << self.blocks.bit_size {
                    let new_size = self.blocks.bit_size + 1;
                    self.blocks = self.blocks.resize(new_size);
                }
                self.block_map.push((b, 0));
            }
        }

        {
            let b_idx = self.rev_block_map[&b];
            let info = &mut self.block_map[b_idx];
            info.1 += 1;
            self.blocks.set(idx, b_idx);
        }
        true
    }

    // Quick chunk loading support helpers

    pub fn clear(&mut self) {
        self.block_map.clear();
        self.rev_block_map.clear();
    }

    pub fn force_mapping(&mut self, idx: usize, b: block::Block) {
        if self.block_map.len() < idx {
            self.block_map[idx] = (b, 0);
        } else if self.block_map.len() == idx {
            self.block_map.push((b, 0));
        } else {
            panic!("Out of bounds force mapping")
        }
        self.rev_block_map.insert(b, idx);
    }

    pub fn use_raw(&mut self, data: bit::Map) {
        self.blocks = data;
        // Recount blocks
        for bi in 0 .. 4096 {
            let bl_id = self.blocks.get(bi);
            if self.blocks.bit_size == 13 { // Global palette
                if self.block_map.get(bl_id)
                        .map(|v| v.1)
                        .unwrap_or(0) == 0 {
                    if bl_id >= self.block_map.len() {
                        self.block_map.resize(bl_id + 1, (block::Air{}, 0xFFFFF)); // Impossible to reach this value normally
                    }
                    let bl = block::Block::by_vanilla_id(bl_id as usize);
                    self.block_map[bl_id] = (bl, 0);
                    self.rev_block_map.insert(bl, bl_id);
                }
            }
            let bmap = self.block_map.get_mut(bl_id).unwrap();
            if bmap.1 == 0xFFFFF {
                bmap.1 = 0;
            }
            bmap.1 += 1;
        }
        self.gc_entries();
    }

    fn gc_entries(&mut self) {
        for entry in &mut self.block_map {
            if entry.1 == 0xFFFFF {
                println!("GC'd block");
                self.rev_block_map.remove(&entry.0);
                entry.1 = 0;
            }
        }
    }
}
