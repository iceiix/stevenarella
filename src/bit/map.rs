
pub struct Map {
    bits: Vec<u64>,
    bitSize: usize,
    length: usize
}

#[test]
fn test_map() {
    let mut map = Map::new(4096, 4);
    for i in 0 .. 4096 {
        for j in 0 .. 16 {
            map.set(i, j);
            if map.get(i) != j {
                panic!("Fail");
            }
        }
    }
}

impl Map {
    pub fn new(len: usize, size: usize) -> Map {
        let mut map = Map {
            bitSize: size,
            length: len,
            bits: Vec::with_capacity((len*size)/64)
        };
        for _ in 0 .. len {
            map.bits.push(0)
        }
        map
    }

    pub fn set(&mut self, i: usize, val: usize) {
        let mut i = i * self.bitSize;
        let pos = i / 64;
        let mask = (1 << self.bitSize) - 1;
        i %= 64;
        self.bits[pos] = (self.bits[pos] & !(mask << i )) | ((val << i) as u64)
    }

    pub fn get(&mut self, i: usize) -> usize {
        let mut i = i * self.bitSize;
        let pos = i / 64;
        let mask = (1 << self.bitSize) - 1;
        i %= 64;
        ((self.bits[pos] >> i) & mask) as usize
    }
}
