// Copyright 2015 Matthew Collins
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


pub struct Array {
    pub data: Vec<u8>,
}

impl Array {
    pub fn new(size: usize) -> Array {
        Array {
            data: vec![0; (size + 1) >> 1],
        }
    }

    pub fn get(&self, idx: usize) -> u8 {
        let val = self.data[idx>>1];
        if idx&1 == 0 {
            val & 0xF
        } else {
            val >> 4
        }
    }

    pub fn set(&mut self, idx: usize, val: u8) {
        let i = idx >> 1;
        let old = self.data[i];
        if idx&1 == 0 {
            self.data[i] = (old & 0xF0) | (val & 0xF);
        } else {
            self.data[i] = (old & 0x0F) | ((val & 0xF) << 4);
        }
    }
}
