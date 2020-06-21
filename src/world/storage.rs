use crate::types::bit;
use crate::types::hash::FNVHash;
use crate::world::block;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;

pub struct BlockStorage {
    blocks: bit::Map,
    block_map: Vec<(block::Block, u32)>,
    rev_block_map: HashMap<block::Block, usize, BuildHasherDefault<FNVHash>>,
}

impl BlockStorage {
    pub fn new(size: usize) -> BlockStorage {
        Self::new_default(size, block::Air {})
    }

    pub fn new_default(size: usize, def: block::Block) -> BlockStorage {
        let mut storage = BlockStorage {
            blocks: bit::Map::new(size, 4),
            block_map: vec![(def, size as u32)],
            rev_block_map: HashMap::with_hasher(BuildHasherDefault::default()),
        };
        storage.rev_block_map.insert(def, 0);
        storage
    }

    pub fn get(&self, idx: usize) -> block::Block {
        let idx = self.blocks.get(idx);
        self.block_map[idx].0
    }

    pub fn set(&mut self, idx: usize, b: block::Block) -> bool {
        use std::collections::hash_map::Entry;
        let old = self.get(idx);
        if old == b {
            return false;
        }
        // Clean up the old block
        {
            let idx = *self.rev_block_map.get(&old).unwrap();
            let info = &mut self.block_map[idx];
            info.1 -= 1;
            if info.1 == 0 {
                // None left of this type
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
}
