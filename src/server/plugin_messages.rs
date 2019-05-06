
use crate::protocol::packet::play::serverbound::PluginMessageServerbound;
use crate::protocol::packet::play::serverbound::PluginMessageServerbound_i16;
use crate::protocol::{Serializable, VarShort};

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
            data: crate::protocol::LenPrefixedBytes::<VarShort>::new(data),
        }
    }

}
