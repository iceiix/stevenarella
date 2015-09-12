extern crate byteorder;

use std::fmt;
use protocol::{Serializable};
use std::io;
use std::io::{Read, Write};
use self::byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

#[derive(Clone, Copy)]
pub struct Position(u64);

impl Position {
    fn new(x: i32, y: i32, z: i32) -> Position {
        Position(
            (((x as u64) & 0x3FFFFFF) << 38) |
            (((y as u64) & 0xFFF) << 26) |
            ((z as u64) & 0x3FFFFFF)
        )
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
