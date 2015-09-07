
pub struct Set {
    data : Vec<u64>
}

#[test]
fn test_set() {
    let mut set = Set::new(200);
    for i in 0 .. 200 {
        if i % 3 == 0 {
            set.set(i, true)
        }
    }
    for i in 0 .. 200 {
        if set.get(i) != (i%3 == 0) {
            panic!("Fail")
        }
    }
}

impl Set {
    pub fn new(size: usize) -> Set {
        let mut set = Set {
            data: Vec::with_capacity(size)
        };
        for _ in 0 .. size {
            set.data.push(0)
        }
        set
    }

    pub fn set(&mut self, i: usize, v: bool) {
        if v {
            self.data[i>>6] |= 1 << (i & 0x3F)
        } else {
            self.data[i>>6] &= !(1 << (i & 0x3F))
        }
    }

    pub fn get(&mut self, i: usize) -> bool {
        return (self.data[i>>6] & (1 << (i & 0x3F))) != 0
    }
}
