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

mod blockpos;
pub use self::blockpos::*;

mod metadata;
pub use self::metadata::*;

pub mod bit;
pub mod nibble;
pub mod hash;

use model::{PRECOMPUTED_VERTS, BlockVertex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Invalid,
    Up,
    Down,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn all() -> Vec<Direction> {
        vec![
            Direction::Up, Direction::Down,
            Direction::North, Direction::South,
            Direction::West, Direction::East,
        ]
    }

    pub fn from_string(val: &str) -> Direction {
        match val {
            "up" => Direction::Up,
            "down" => Direction::Down,
            "north" => Direction::North,
            "south" => Direction::South,
            "west" => Direction::West,
            "east" => Direction::East,
            _ => Direction::Invalid,
        }
    }

    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
            _ => unreachable!(),
        }
    }

    pub fn get_verts(&self) -> &'static [BlockVertex; 4] {
        match *self {
            Direction::Up => PRECOMPUTED_VERTS[0],
            Direction::Down => PRECOMPUTED_VERTS[1],
            Direction::North => PRECOMPUTED_VERTS[2],
            Direction::South => PRECOMPUTED_VERTS[3],
            Direction::West => PRECOMPUTED_VERTS[4],
            Direction::East => PRECOMPUTED_VERTS[5],
            _ => unreachable!(),
        }
    }

    pub fn get_offset(&self) -> (i32, i32, i32) {
        match *self {
            Direction::Up => (0, 1, 0),
            Direction::Down => (0, -1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
            _ => unreachable!(),
        }
    }

    pub fn as_string(&self) -> &'static str {
        match *self {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::North => "north",
            Direction::South => "south",
            Direction::West => "west",
            Direction::East => "east",
            Direction::Invalid => "invalid",
        }
    }

    pub fn index(&self) -> usize {
        match *self {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
            _ => unreachable!(),
        }
    }
}
