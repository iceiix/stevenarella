
use std::collections::HashMap;
use std::io;
use byteorder::WriteBytesExt;

use crate::protocol::packet::play::serverbound::PluginMessageServerbound;
use crate::protocol::packet::play::serverbound::PluginMessageServerbound_i16;
use crate::protocol::{Serializable, Error};

#[derive(Debug)]
pub enum FmlHs<'a> {
    ServerHello {
        fml_protocol_version: i8,
        override_dimension: Option<i32>,
    },
    ClientHello {
        fml_protocol_version: i8,
    },
    ModList {
        mods: HashMap<&'a str, &'a str>,
    },
    RegistryData {
        has_more: bool,
        name: String,
        ids: HashMap<&'a str, i32>,
        substitutions: Vec<&'a str>,
        dummies: Vec<&'a str>,
    },
    HandshakeAck {
        phase: i8,
    },
    HandshakeReset,
}

impl<'a> Serializable for FmlHs<'a> {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        // https://wiki.vg/Minecraft_Forge_Handshake
        let discriminator: u8 = Serializable::read_from(buf)?;

        match discriminator {
            0 => {
                // ServerHello
                let fml_protocol_version: i8 = Serializable::read_from(buf)?;
                let override_dimension = if fml_protocol_version > 1 {
                    let dimension: i32 = Serializable::read_from(buf)?;
                    Some(dimension)
                } else {
                    None
                };

                println!("FML|HS ServerHello: fml_protocol_version={}, override_dimension={:?}", fml_protocol_version, override_dimension);

                Ok(FmlHs::ServerHello {
                    fml_protocol_version,
                    override_dimension,
                })
            },
            1 => panic!("Received unexpected FML|HS ClientHello from server"),
            2 => {
                //TODO let number_of_mods = VarInt::read_from(&mut data[1..].to_vec());
                let mods: HashMap<&'a str, &'a str> = HashMap::new();
                // TODO: read mods

                Ok(FmlHs::ModList {
                    mods,
                })
            },
            _ => panic!("Unhandled FML|HS packet: discriminator={}", discriminator),
        }
    }


    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        match self {
            FmlHs::ClientHello { fml_protocol_version } => {
                buf.write_u8(1)?;
                fml_protocol_version.write_to(buf)
            },
            FmlHs::ModList { mods } => {
                buf.write_u8(2)?;
                Ok(())

                //let number_of_mods = VarInt(mods.len() as i32);
                //number_of_mods.write_to(&mut buf).unwrap();
                // TODO: write mods
            },
            _ => unimplemented!()
        }
    }
}

impl<'a> FmlHs<'a> {
    // TODO: remove this wrapper and call write_to directly
    pub fn as_message(&'a self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        self.write_to(&mut buf).unwrap();
        buf
    }
}

pub struct Brand {
    pub brand: String,
}

impl Brand {
    pub fn as_message(self) -> PluginMessageServerbound {
        let protocol_version = unsafe { crate::protocol::CURRENT_PROTOCOL_VERSION };

        let channel_name = if protocol_version >= 404 {
            "minecraft:brand"
        } else {
            "MC|Brand"
        };

        let mut data = vec![];
        Serializable::write_to(&self.brand, &mut data).unwrap();
        PluginMessageServerbound {
            channel: channel_name.into(),
            data,
        }
    }

    // TODO: cleanup this duplication for 1.7, return either message dynamically
    pub fn as_message17(self) -> PluginMessageServerbound_i16 {
        let mut data = vec![];
        Serializable::write_to(&self.brand, &mut data).unwrap();
        PluginMessageServerbound_i16 {
            channel: "MC|Brand".into(),
            data: crate::protocol::LenPrefixedBytes::<i16>::new(data),
        }
    }

}
