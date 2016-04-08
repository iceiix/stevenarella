
use protocol::Serializable;
use protocol::packet::play::serverbound::PluginMessageServerbound;

pub struct Brand {
    pub brand: String,
}

impl Brand {
    pub fn as_message(self) -> PluginMessageServerbound {
        let mut data = vec![];
        Serializable::write_to(&self.brand, &mut data).unwrap();
        PluginMessageServerbound {
            channel: "MC|Brand".into(),
            data: data,
        }
    }
}
