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

use nbt;
use protocol::{self, Serializable};
use std::io;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

#[derive(Debug)]
pub struct Stack {
    id: isize,
    count: isize,
    damage: isize,
    tag: Option<nbt::NamedTag>,
}


impl Default for Stack {
    fn default() -> Stack {
        Stack {
            id: -1,
            count: 0,
            damage: 0,
            tag: None,
        }
    }
}

impl Serializable for Option<Stack> {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Option<Stack>, protocol::Error> {
        let id = try!(buf.read_i16::<BigEndian>());
        if id == -1 {
            return Ok(None);
        }
        Ok(Some(Stack {
            id: id as isize,
            count: try!(buf.read_u8()) as isize,
            damage: try!(buf.read_i16::<BigEndian>()) as isize,
            tag: try!(Serializable::read_from(buf)),
        }))
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), protocol::Error> {
        match *self {
            Some(ref val) => {
                try!(buf.write_i16::<BigEndian>(val.id as i16));
                try!(buf.write_u8(val.count as u8));
                try!(buf.write_i16::<BigEndian>(val.damage as i16));
                try!(val.tag.write_to(buf));
            }
            None => try!(buf.write_i16::<BigEndian>(-1)),
        }
        Result::Ok(())
    }
}
