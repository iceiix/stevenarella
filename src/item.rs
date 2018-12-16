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

use crate::nbt;
use crate::protocol::{self, Serializable};
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
        let id = buf.read_i16::<BigEndian>()?;
        if id == -1 {
            return Ok(None);
        }
        let count = buf.read_u8()? as isize;
        let damage = buf.read_i16::<BigEndian>()? as isize;

        let protocol_version = unsafe { protocol::CURRENT_PROTOCOL_VERSION };

        let tag: Option<nbt::NamedTag> = if protocol_version >= 47 {
            Serializable::read_from(buf)?
        } else {
            // 1.7 uses a different slot data format described on https://wiki.vg/index.php?title=Slot_Data&diff=6056&oldid=4753
            let tag_size = buf.read_i16::<BigEndian>()?;
            if tag_size != -1 {
                for _ in 0..tag_size {
                    let _ = buf.read_u8()?;
                }
                // TODO: decompress zlib NBT for 1.7
                None
            } else {
                None
            }
        };

        Ok(Some(Stack {
            id: id as isize,
            count,
            damage,
            tag,
        }))
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), protocol::Error> {
        match *self {
            Some(ref val) => {
                buf.write_i16::<BigEndian>(val.id as i16)?;
                buf.write_u8(val.count as u8)?;
                buf.write_i16::<BigEndian>(val.damage as i16)?;
                // TODO: compress zlib NBT if 1.7
                val.tag.write_to(buf)?;
            }
            None => buf.write_i16::<BigEndian>(-1)?,
        }
        Result::Ok(())
    }
}
