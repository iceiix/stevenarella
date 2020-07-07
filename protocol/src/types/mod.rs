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

mod metadata;
pub use self::metadata::*;

pub mod bit;
pub mod hash;
pub mod nibble;

#[derive(Clone, Copy, Debug)]
pub enum Gamemode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

impl Gamemode {
    pub fn from_int(val: i32) -> Gamemode {
        match val {
            3 => Gamemode::Spectator,
            2 => Gamemode::Adventure,
            1 => Gamemode::Creative,
            0 => Gamemode::Survival,
            _ => Gamemode::Survival,
        }
    }

    pub fn can_fly(&self) -> bool {
        match *self {
            Gamemode::Creative | Gamemode::Spectator => true,
            _ => false,
        }
    }

    pub fn always_fly(&self) -> bool {
        match *self {
            Gamemode::Spectator => true,
            _ => false,
        }
    }

    pub fn noclip(&self) -> bool {
        match *self {
            Gamemode::Spectator => true,
            _ => false,
        }
    }
}
