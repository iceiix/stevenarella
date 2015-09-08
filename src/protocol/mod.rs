#![allow(dead_code)]
extern crate byteorder;

use std::default;
use std::net::TcpStream;
use std::io;
use std::io::{Write, Read};
use std::convert;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

/// Helper macro for defining packets
#[macro_export]
macro_rules! state_packets {
     ($($state:ident $stateName:ident {
        $($dir:ident $dirName:ident {
            $($name:ident => $id:expr {
                $($field:ident: $field_type:ident),+
            }),*
        })+
    })+) => {
        use protocol::*;
        use std::io;
        use protocol::{Serializable};

        pub enum Packet {
        $(
            $(
                $(
        $name($state::$dir::$name),
                )*
            )+
        )+
        }

        $(
        pub mod $state {
            $(
            pub mod $dir {
                use protocol::*;
                use std::io;
                use protocol::{Serializable};

                $(
                    #[derive(Default)]
                    pub struct $name {
                        $(pub $field: $field_type),+,
                    }

                    impl PacketType for $name {

                        fn packet_id(&self) -> i32{ $id }

                        fn write(self, buf: &mut Vec<u8>) -> Result<(), io::Error> {
                            $(
                                try!(self.$field.write_to(buf));
                            )+

                            Result::Ok(())
                        }
                    }
                )*
            }
            )+
        }
        )+

        /// Returns the packet for the given state, direction and id after parsing the fields
        /// from the buffer.
        pub fn packet_by_id(state: State, dir: Direction, id: i32, mut buf: &mut io::Cursor<Vec<u8>>) -> Result<Option<Packet>, io::Error> {
            match state {
                $(
                    State::$stateName => {
                        match dir {
                            $(
                                Direction::$dirName => {
                                    match id {
                                    $(
                                        $id => {
                                            let mut packet : $state::$dir::$name = $state::$dir::$name::default();
                                            $(
                                                packet.$field = try!($field_type::read_from(&mut buf));
                                            )+
                                            Result::Ok(Option::Some(Packet::$name(packet)))
                                        },
                                    )*
                                        _ => Result::Ok(Option::None)
                                    }
                                }
                            )+
                        }
                    }
                )+
            }
        }
    }
}

pub mod packet;

trait Serializable {
    fn read_from(buf: &mut io::Read) -> Result<Self, io::Error>;
    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error>;
}

impl Serializable for String {
    fn read_from(buf: &mut io::Read) -> Result<String, io::Error> {
        let len = try!(VarInt::read_from(buf)).0;
        let mut ret = String::new();
        try!(buf.take(len as u64).read_to_string(&mut ret));
        Result::Ok(ret)
    }
    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        let bytes = self.as_bytes();
        try!(VarInt(bytes.len() as i32).write_to(buf));
        try!(buf.write_all(bytes));
        Result::Ok(())
    }
}

impl Serializable for Empty {
    fn read_from(buf: &mut io::Read) -> Result<Empty, io::Error> {
        Result::Ok(Empty)
    }
    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        Result::Ok(())
    }
}

impl Default for Empty {
    fn default() -> Empty { Empty }
}

impl Serializable for i32 {
    fn read_from(buf: &mut io::Read) -> Result<i32, io::Error> {
        Result::Ok(try!(buf.read_i32::<BigEndian>()))
    }
    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(buf.write_i32::<BigEndian>(*self));
        Result::Ok(())
    }
}

impl Serializable for i64 {
    fn read_from(buf: &mut io::Read) -> Result<i64, io::Error> {
        Result::Ok(try!(buf.read_i64::<BigEndian>()))
    }
    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(buf.write_i64::<BigEndian>(*self));
        Result::Ok(())
    }
}

impl Serializable for u16 {
    fn read_from(buf: &mut io::Read) -> Result<u16, io::Error> {
        Result::Ok(try!(buf.read_u16::<BigEndian>()))
    }
    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(buf.write_u16::<BigEndian>(*self));
        Result::Ok(())
    }
}

/// VarInt have a variable size (between 1 and 5 bytes) when encoded based
/// on the size of the number
pub struct VarInt(i32);

impl Serializable for VarInt {
    /// Decodes a VarInt from the Reader
    fn read_from(buf: &mut io::Read) -> Result<VarInt, io::Error> {
        const PART : u32 = 0x7F;
        let mut size = 0;
        let mut val = 0u32;
        loop {
            let b = try!(buf.read_u8()) as u32;
            val |= (b & PART) << (size * 7);
            size+=1;
            if size > 5 {
                return Result::Err(io::Error::new(io::ErrorKind::InvalidInput, Error::Err("VarInt too big".to_string())))
            }
            if (b & 0x80) == 0 {
                break
            }
        }

        Result::Ok(VarInt(val as i32))
    }

    /// Encodes a VarInt into the Writer
    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        const PART : u32 = 0x7F;
        let mut val = self.0 as u32;
        loop {
            if (val & !PART) == 0 {
                try!(buf.write_u8(val as u8));
                return Result::Ok(());
            }
            try!(buf.write_u8(((val & PART) | 0x80) as u8));
            val >>= 7;
        }
    }
}

impl default::Default for VarInt {
    fn default() -> VarInt { VarInt(0) }
}

/// Direction is used to define whether packets are going to the
/// server or the client.
pub enum Direction {
    Serverbound,
    Clientbound
}

/// The protocol has multiple 'sub-protocols' or states which control which
/// packet an id points to.
#[derive(Clone, Copy)]
pub enum State {
    Handshaking,
    Play,
    Status,
    Login
}

/// Return for any protocol related error.
#[derive(Debug)]
pub enum Error {
    Err(String),
    IOError(io::Error),
}

impl convert::From<io::Error> for Error {
    fn from(e : io::Error) -> Error {
        Error::IOError(e)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Err(ref val) => &val[..],
            &Error::IOError(ref e) => e.description(),
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f : &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            &Error::Err(ref val) => write!(f, "protocol error: {}", val),
            &Error::IOError(ref e) => e.fmt(f)
        }
    }
}


/// Helper for empty structs
pub struct Empty;

pub struct Conn {
    stream: TcpStream,
    host: String,
    port: u16,
    direction: Direction,
    state: State,
}

impl Conn {
    pub fn new(target: &str) -> Result<Conn, Error>{
        // TODO SRV record support
        let stream = match TcpStream::connect(target) {
            Ok(val) => val,
            Err(err) => return Result::Err(Error::IOError(err))
        };
        let parts = target.split(":").collect::<Vec<&str>>();
        Result::Ok(Conn {
            stream: stream,
            host: parts[0].to_owned(),
            port: parts[1].parse().unwrap(),
            direction: Direction::Serverbound,
            state: State::Handshaking,
        })
    }

    // TODO: compression and encryption

    pub fn write_packet<T: PacketType>(&mut self, packet: T) -> Result<(), Error> {
        let mut buf = Vec::new();
        try!(VarInt(packet.packet_id()).write_to(&mut buf));
        try!(packet.write(&mut buf));
        try!(VarInt(buf.len() as i32).write_to(&mut self.stream));
        try!(self.stream.write_all(&buf.into_boxed_slice()));

        Result::Ok(())
    }

    pub fn read_packet(&mut self) -> Result<packet::Packet, Error> {
        let len = try!(VarInt::read_from(&mut self.stream)).0 as usize;
        let mut ibuf = Vec::with_capacity(len);
        try!((&mut self.stream).take(len as u64).read_to_end(&mut ibuf));

        let mut buf = io::Cursor::new(ibuf);
        let id = try!(VarInt::read_from(&mut buf)).0;

        let dir = match self.direction {
            Direction::Clientbound => Direction::Serverbound,
            Direction::Serverbound => Direction::Clientbound,
        };

        let packet = try!(packet::packet_by_id(self.state, dir, id, &mut buf));

        match packet {
            Some(val) => {
                let pos = buf.position() as usize;
                let ibuf = buf.into_inner();
                if ibuf.len() < pos {
                    return Result::Err(Error::Err(format!("Failed to read all of packet 0x{:X}, had {} bytes left", id, ibuf.len() - pos)))
                }
                Result::Ok(val)
            },
            None => Result::Err(Error::Err("missing packet".to_string()))
        }
    }
}

pub trait PacketType {
    fn packet_id(&self) -> i32;

    fn write(self, buf: &mut Vec<u8>) -> Result<(), io::Error>;
}

#[test]
fn test() {
    let mut c = Conn::new("localhost:25565").unwrap();

    c.write_packet(packet::handshake::serverbound::Handshake{
        protocol_version: VarInt(69),
        host: "localhost".to_string(),
        port: 25565,
        next: VarInt(1),
    }).unwrap();
    c.state = State::Status;
    c.write_packet(packet::status::serverbound::StatusRequest{empty: Empty}).unwrap();

    match c.read_packet().unwrap() {
        packet::Packet::StatusResponse(val) => println!("{}", val.status),
        _ => panic!("Wrong packet"),
    }

    c.write_packet(packet::status::serverbound::StatusPing{ping: 4433}).unwrap();

    match c.read_packet().unwrap() {
        packet::Packet::StatusPong(val) => println!("{}", val.ping),
        _ => panic!("Wrong packet"),
    }

    panic!("TODO!");
}
