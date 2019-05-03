
use std::collections::HashMap;

use crate::protocol::Serializable;
use crate::protocol::packet::play::serverbound::PluginMessageServerbound;
use crate::protocol::packet::play::serverbound::PluginMessageServerbound_i16;

pub enum FmlHs<'a> {
    ServerHello {
        fml_protocol_version: i8,
        override_dimension: i32,
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
