extern crate byteorder;

use nbt;
use protocol::{Serializable};
use std::io;
use std::io::{Read, Write};
use self::byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

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
    fn read_from(buf: &mut io::Read) -> Result<Option<Stack>, io::Error> {
        let id = try!(buf.read_i16::<BigEndian>());
        if id == -1 {
            return Ok(None);
        }
        Ok(Some(Stack{
            id: id as isize,
            count: try!(buf.read_u8()) as isize,
            damage: try!(buf.read_i16::<BigEndian>()) as isize,
            tag: try!(Serializable::read_from(buf)),
        }))
    }
    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        match *self {
            Some(ref val) => {
                try!(buf.write_i16::<BigEndian>(val.id as i16));
                try!(buf.write_u8(val.count as u8));
                try!(buf.write_i16::<BigEndian>(val.damage as i16));
                try!(val.tag.write_to(buf));
            },
            None => try!(buf.write_i16::<BigEndian>(-1)),
        }
        Result::Ok(())
    }
}
