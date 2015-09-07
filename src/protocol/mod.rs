#![allow(dead_code)]
extern crate byteorder;

use std::net::TcpStream;
use std::io;
use std::io::{Write, Read};
use std::convert;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

/// Serializes types into a buffer
macro_rules! serialize_type {
    ($dst:expr, $name:expr, u16) => {
        $dst.write_u16::<BigEndian>($name).unwrap();
    };
    ($dst:expr, $name:expr, i64) => {
        $dst.write_i64::<BigEndian>($name).unwrap();
    };
    ($dst:expr, $name:expr, VarInt) => {
        try!(write_varint($dst, $name));
    };
    ($dst:expr, $name:expr, String) => {
        try!(write_varint($dst, $name.len() as i32));
        $dst.extend($name.bytes());
    };
    ($dst:expr, $name:expr, Empty) => {

    };
    ($dst:expr, $name:expr, $ftype:ident) => {
        unimplemented!()
    };
}

/// Deserializes types from a buffer
macro_rules! deserialize_type {
    ($src:expr, String) => {
        {
            let len = read_variant(&mut $src).unwrap();
            let mut ret = String::new();
            (&mut $src).take(len as u64).read_to_string(&mut ret).unwrap();
            ret
        }
    };
    ($src:expr, i64) => {
        $src.read_i64::<BigEndian>().unwrap()
    };
    ($src:expr, Empty) => { Empty };
    ($src:expr, $ftype:ident) => {
        unimplemented!()
    };
}

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
        use std::io::{Read, Write};
        use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

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
                use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
                use protocol::*;
                use std::io;
                use std::io::{Read, Write};

                $(
                    pub struct $name {
                        $(pub $field: $field_type),+,
                    }

                    impl PacketType for $name {

                        fn packet_id(&self) -> i32{ $id }

                        fn write(self, buf: &mut Vec<u8>) -> Result<(), io::Error> {
                            $(
                                serialize_type!(buf, self.$field, $field_type);
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
        pub fn packet_by_id(state: State, dir: Direction, id: i32, mut buf: &mut io::Cursor<Vec<u8>>) -> Option<Packet> {
            match state {
                $(
                    State::$stateName => {
                        match dir {
                            $(
                                Direction::$dirName => {
                                    match id {
                                    $(
                                        $id => Option::Some(Packet::$name($state::$dir::$name {
                                            $($field: deserialize_type!(buf, $field_type)),+,
                                        })),
                                    )*
                                        _ => Option::None
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

/// VarInt have a variable size (between 1 and 5 bytes) when encoded based
/// on the size of the number
pub type VarInt = i32;

/// Encodes a VarInt into the Writer
pub fn write_varint(buf: &mut io::Write, v: VarInt) -> Result<(), io::Error> {
    const PART : u32 = 0x7F;
    let mut val = v as u32;
    loop {
        if (val & !PART) == 0 {
            try!(buf.write_u8(val as u8));
            return Result::Ok(());
        }
        try!(buf.write_u8(((val & PART) | 0x80) as u8));
        val >>= 7;
    }
}

/// Decodes a VarInt from the Reader
pub fn read_variant(buf: &mut io::Read) -> Result<VarInt, io::Error> {
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

    Result::Ok(val as VarInt)
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
        try!(write_varint(&mut buf, packet.packet_id()));
        try!(packet.write(&mut buf));
        try!(write_varint(&mut self.stream, buf.len() as i32));
        try!(self.stream.write_all(&buf.into_boxed_slice()));

        Result::Ok(())
    }

    pub fn read_packet(&mut self) -> Result<packet::Packet, Error> {
        let len = try!(read_variant(&mut self.stream)) as usize;
        let mut ibuf = Vec::with_capacity(len);
        try!((&mut self.stream).take(len as u64).read_to_end(&mut ibuf));

        let mut buf = io::Cursor::new(ibuf);
        let id = try!(read_variant(&mut buf));

        let dir = match self.direction {
            Direction::Clientbound => Direction::Serverbound,
            Direction::Serverbound => Direction::Clientbound,
        };

        let packet = packet::packet_by_id(self.state, dir, id, &mut buf);

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
    return; // Skip
    let mut c = Conn::new("localhost:25565").unwrap();

    c.write_packet(packet::handshake::serverbound::Handshake{
        protocol_version: 69,
        host: "localhost".to_string(),
        port: 25565,
        next: 1,
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
