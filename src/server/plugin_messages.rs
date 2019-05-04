
use std::collections::HashMap;

use crate::protocol::Serializable;
use crate::protocol::packet::play::serverbound::PluginMessageServerbound;
use crate::protocol::packet::play::serverbound::PluginMessageServerbound_i16;

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

impl<'a> FmlHs<'a> {
    pub fn from_message(data: &[u8]) -> FmlHs<'a> {
        // https://wiki.vg/Minecraft_Forge_Handshake
        let discriminator = data[0];

        match discriminator {
            0 => {
                // ServerHello
                let fml_protocol_version = data[1] as i8;
                let override_dimension = if fml_protocol_version > 1 {
                    use byteorder::{BigEndian, ReadBytesExt};
                    let dimension = (&data[2..2 + 4]).read_i32::<BigEndian>().unwrap();
                    Some(dimension)
                } else {
                    None
                };

                println!("FML|HS ServerHello: fml_protocol_version={}, override_dimension={:?}", fml_protocol_version, override_dimension);

                FmlHs::ServerHello {
                    fml_protocol_version,
                    override_dimension,
                }

                // TODO: send reply
            },
            _ => {
                panic!("Unhandled FML|HS packet: discriminator={}", discriminator);
            }
        }
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
