
use crate::protocol::Serializable;
use crate::protocol::packet::play::serverbound::PluginMessageServerbound;
use crate::protocol::packet::play::serverbound::PluginMessageServerbound_i16;

pub struct PluginMessageHandler {
}

impl PluginMessageHandler {
    pub fn on_plugin_message_clientbound(&mut self, channel: &str, data: &[u8]) {
        println!("Received plugin message: channel={}, data={:?}", channel, data);

        match channel {
            // TODO: "REGISTER" => 
            // TODO: "UNREGISTER" =>
            "FML|HS" => {
                // https://wiki.vg/Minecraft_Forge_Handshake
                let discriminator = data[0];

                match discriminator {
                    0 => {
                        // ServerHello
                        let fml_protocol_version = data[1];
                        let dimension = if fml_protocol_version > 1 {
                            use byteorder::{BigEndian, ReadBytesExt};
                            let dimension = (&data[2..2 + 4]).read_u32::<BigEndian>().unwrap();
                            Some(dimension)
                        } else {
                            None
                        };

                        println!("FML|HS ServerHello: fml_protocol_version={}, dimension={:?}", fml_protocol_version, dimension);

                        // TODO: send reply
                    },
                    _ => {
                        println!("Unhandled FML|HS packet: discriminator={}", discriminator);
                    }
                }
            }
            _ => ()
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
