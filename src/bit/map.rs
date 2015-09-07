
pub struct Map {
    bits: Vec<u64>,
    bit_size: usize,
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

#[test]
fn test_map_odd() {
    for size in 1 .. 16 {
        let mut map = Map::new(64*3, size);
        let max = (1 << size) - 1;
        for i in 0 .. 64*3 {
            for j in 0 .. max {
                map.set(i, j);
                if map.get(i) != j {
                    panic!("Index: {} wanted {} and got {}", i, j, map.get(i));
                }
            }
        }
    }
}

impl Map {
    pub fn new(len: usize, size: usize) -> Map {
        let mut map = Map {
            bit_size: size,
            length: len,
            bits: Vec::with_capacity((len*size)/64)
        };
        for _ in 0 .. len {
            map.bits.push(0)
        }
        map
    }

    pub fn set(&mut self, i: usize, val: usize) {
        let i = i * self.bit_size;
        let pos = i / 64;
        let mask = (1 << self.bit_size) - 1;
        let ii = i % 64;
        self.bits[pos] = (self.bits[pos] & !(mask << ii )) | ((val << ii) as u64);
        let pos2 = (i + self.bit_size - 1) / 64;
        if pos2 != pos {
            let used = 64 - ii;
            let rem = self.bit_size - used;
            self.bits[pos2] = self.bits[pos2] >> rem << rem | (val as u64 >> used);
        }
    }

    pub fn get(&mut self, i: usize) -> usize {
        let i = i * self.bit_size;
        let pos = i / 64;
        let mask = (1 << self.bit_size) - 1;
        let ii = i % 64;
        let pos2 = (i + self.bit_size - 1) / 64;
        if pos2 != pos {
            let used = 64 - ii;
            (((self.bits[pos] >> ii) | (self.bits[pos2] << used)) & mask) as usize
        } else {
            ((self.bits[pos] >> ii) & mask) as usize
        }
    }
}
