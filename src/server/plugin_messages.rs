
use std::io;
use byteorder::WriteBytesExt;

use crate::protocol::packet::play::serverbound::PluginMessageServerbound;
use crate::protocol::packet::play::serverbound::PluginMessageServerbound_i16;
use crate::protocol::{Serializable, Error, LenPrefixed, VarInt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Start,
    WaitingServerData,
    WaitingServerComplete,
    PendingComplete,
    Complete,
}

impl Serializable for Phase {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let phase: i8 = Serializable::read_from(buf)?;
        Ok(match phase {
            2 => Phase::WaitingServerData,
            3 => Phase::WaitingServerComplete,
            4 => Phase::PendingComplete,
            5 => Phase::Complete,
            _ => panic!("bad FML|HS phase: {}", phase),
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_u8(match self {
            Phase::Start => panic!("attempted to send FML|HS start unexpectedly soon"),
            Phase::WaitingServerData => 2,
            Phase::WaitingServerComplete => 3,
            Phase::PendingComplete => 4,
            Phase::Complete => 5,
        })?;
        Ok(())
    }
}


#[derive(Clone, Debug, Default)]
pub struct ForgeMod {
    pub modid: String,
    pub version: String,
}

impl Serializable for ForgeMod {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(ForgeMod {
            modid: Serializable::read_from(buf)?,
            version: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.modid.write_to(buf)?;
        self.version.write_to(buf)
    }
}

#[derive(Debug)]
pub struct Id {
    pub name: String,
    pub id: VarInt,
}

impl Serializable for Id {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(Id {
            name: Serializable::read_from(buf)?,
            id: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.name.write_to(buf)?;
        self.id.write_to(buf)
    }
}

#[derive(Debug)]
pub enum FmlHs {
    ServerHello {
        fml_protocol_version: i8,
        override_dimension: Option<i32>,
    },
    ClientHello {
        fml_protocol_version: i8,
    },
    ModList {
        mods: LenPrefixed<VarInt, ForgeMod>,
    },
    RegistryData {
        has_more: bool,
        name: String,
        ids: LenPrefixed<VarInt, Id>,
        substitutions: LenPrefixed<VarInt, String>,
        dummies: LenPrefixed<VarInt, String>,
    },
    HandshakeAck {
        phase: Phase,
    },
    HandshakeReset,
}

impl Serializable for FmlHs {
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
                Ok(FmlHs::ModList {
                    mods: Serializable::read_from(buf)?,
                })
            },
            3 => {
                Ok(FmlHs::RegistryData {
                    has_more: Serializable::read_from(buf)?,
                    name: Serializable::read_from(buf)?,
                    ids: Serializable::read_from(buf)?,
                    substitutions: Serializable::read_from(buf)?,
                    dummies: Serializable::read_from(buf)?, 
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
                mods.write_to(buf)
            },
            FmlHs::HandshakeAck { phase } => {
                buf.write_u8(255)?;
                phase.write_to(buf)
            },
            _ => unimplemented!()
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
