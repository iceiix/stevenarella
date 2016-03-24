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

extern crate byteorder;

use std::fmt;
use protocol::Serializable;
use std::io;
use std::io::Write;
use self::byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

#[derive(Clone, Copy)]
pub struct Position(u64);

impl Position {
    #[allow(dead_code)]
    fn new(x: i32, y: i32, z: i32) -> Position {
        Position((((x as u64) & 0x3FFFFFF) << 38) | (((y as u64) & 0xFFF) << 26) |
                 ((z as u64) & 0x3FFFFFF))
    }

    fn get_x(&self) -> i32 {
        ((self.0 as i64) >> 38) as i32
    }

    fn get_y(&self) -> i32 {
        (((self.0 as i64) >> 26) & 0xFFF) as i32
    }

    fn get_z(&self) -> i32 {
        ((self.0 as i64) << 38 >> 38) as i32
    }
}

impl Default for Position {
    fn default() -> Position {
        Position(0)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{},{},{}>", self.get_x(), self.get_y(), self.get_z())
    }
}

impl Serializable for Position {
    fn read_from(buf: &mut io::Read) -> Result<Position, io::Error> {
        Result::Ok(Position(try!(buf.read_u64::<BigEndian>())))
    }
    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(buf.write_u64::<BigEndian>(self.0));
        Result::Ok(())
    }
}
