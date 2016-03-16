// Copyright 2016 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub struct Set {
    data: Vec<u64>,
}

#[test]
fn test_set() {
    let mut set = Set::new(200);
    for i in 0..200 {
        if i % 3 == 0 {
            set.set(i, true)
        }
    }
    for i in 0..200 {
        if set.get(i) != (i % 3 == 0) {
            panic!("Fail")
        }
    }
}

impl Set {
    pub fn new(size: usize) -> Set {
        let mut set = Set { data: Vec::with_capacity(size) };
        for _ in 0..size {
            set.data.push(0)
        }
        set
    }

    pub fn set(&mut self, i: usize, v: bool) {
        if v {
            self.data[i >> 6] |= 1 << (i & 0x3F)
        } else {
            self.data[i >> 6] &= !(1 << (i & 0x3F))
        }
    }

    pub fn get(&mut self, i: usize) -> bool {
        (self.data[i >> 6] & (1 << (i & 0x3F))) != 0
    }
}
