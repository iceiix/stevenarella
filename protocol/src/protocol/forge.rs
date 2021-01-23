use byteorder::WriteBytesExt;
use log::debug;
/// Implements https://wiki.vg/Minecraft_Forge_Handshake
use std::io;

use super::{Error, LenPrefixed, Serializable, VarInt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    // Client handshake states (written)
    Start,
    WaitingServerData,
    WaitingServerComplete,
    PendingComplete,

    // Server handshake states (read)
    WaitingCAck,

    // Both client and server handshake states (different values on the wire)
    Complete,
}

impl Serializable for Phase {
    /// Read server handshake state from server
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let phase: i8 = Serializable::read_from(buf)?;
        Ok(match phase {
            2 => Phase::WaitingCAck,
            3 => Phase::Complete,
            _ => panic!("bad FML|HS server phase: {}", phase),
        })
    }

    /// Send client handshake state from client
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_u8(match self {
            Phase::WaitingServerData => 2,
            Phase::WaitingServerComplete => 3,
            Phase::PendingComplete => 4,
            Phase::Complete => 5,
            _ => panic!("bad FML|HS client phase: {:?}", self),
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
pub struct ModIdMapping {
    pub name: String,
    pub id: VarInt,
}

impl Serializable for ModIdMapping {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(ModIdMapping {
            name: Serializable::read_from(buf)?,
            id: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.name.write_to(buf)?;
        self.id.write_to(buf)
    }
}

pub static BLOCK_NAMESPACE: &str = "\u{1}";
pub static ITEM_NAMESPACE: &str = "\u{2}";

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
        ids: LenPrefixed<VarInt, ModIdMapping>,
        substitutions: LenPrefixed<VarInt, String>,
        dummies: LenPrefixed<VarInt, String>,
    },
    ModIdData {
        mappings: LenPrefixed<VarInt, ModIdMapping>,
        block_substitutions: LenPrefixed<VarInt, String>,
        item_substitutions: LenPrefixed<VarInt, String>,
    },
    HandshakeAck {
        phase: Phase,
    },
    HandshakeReset,
}

impl Serializable for FmlHs {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let discriminator: u8 = Serializable::read_from(buf)?;

        match discriminator {
            0 => {
                let fml_protocol_version: i8 = Serializable::read_from(buf)?;
                let override_dimension = if fml_protocol_version > 1 {
                    let dimension: i32 = Serializable::read_from(buf)?;
                    Some(dimension)
                } else {
                    None
                };

                debug!(
                    "FML|HS ServerHello: fml_protocol_version={}, override_dimension={:?}",
                    fml_protocol_version, override_dimension
                );

                Ok(FmlHs::ServerHello {
                    fml_protocol_version,
                    override_dimension,
                })
            }
            1 => panic!("Received unexpected FML|HS ClientHello from server"),
            2 => Ok(FmlHs::ModList {
                mods: Serializable::read_from(buf)?,
            }),
            3 => {
                let protocol_version = super::current_protocol_version();

                if protocol_version >= 47 {
                    Ok(FmlHs::RegistryData {
                        has_more: Serializable::read_from(buf)?,
                        name: Serializable::read_from(buf)?,
                        ids: Serializable::read_from(buf)?,
                        substitutions: Serializable::read_from(buf)?,
                        dummies: Serializable::read_from(buf)?,
                    })
                } else {
                    Ok(FmlHs::ModIdData {
                        mappings: Serializable::read_from(buf)?,
                        block_substitutions: Serializable::read_from(buf)?,
                        item_substitutions: Serializable::read_from(buf)?,
                    })
                }
            }
            255 => Ok(FmlHs::HandshakeAck {
                phase: Serializable::read_from(buf)?,
            }),
            _ => panic!("Unhandled FML|HS packet: discriminator={}", discriminator),
        }
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        match self {
            FmlHs::ClientHello {
                fml_protocol_version,
            } => {
                buf.write_u8(1)?;
                fml_protocol_version.write_to(buf)
            }
            FmlHs::ModList { mods } => {
                buf.write_u8(2)?;
                mods.write_to(buf)
            }
            FmlHs::HandshakeAck { phase } => {
                buf.write_u8(255)?;
                phase.write_to(buf)
            }
            _ => unimplemented!(),
        }
    }
}

pub mod fml2 {
    // https://wiki.vg/Minecraft_Forge_Handshake#FML2_protocol_.281.13_-_Current.29
    use super::*;

    #[derive(Clone, Default, Debug)]
    pub struct Channel {
        pub name: String,
        pub version: String,
    }

    impl Serializable for Channel {
        fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
            Ok(Channel {
                name: Serializable::read_from(buf)?,
                version: Serializable::read_from(buf)?,
            })
        }

        fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
            self.name.write_to(buf)?;
            self.version.write_to(buf)
        }
    }

    #[derive(Clone, Default, Debug)]
    pub struct Registry {
        pub name: String,
        pub marker: String,
    }

    impl Serializable for Registry {
        fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
            Ok(Registry {
                name: Serializable::read_from(buf)?,
                marker: "".to_string(), // not in ModList
            })
        }

        fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
            self.name.write_to(buf)?;
            self.marker.write_to(buf)
        }
    }

    #[derive(Debug)]
    pub enum FmlHandshake {
        ModList {
            mod_names: LenPrefixed<VarInt, String>,
            channels: LenPrefixed<VarInt, Channel>,
            registries: LenPrefixed<VarInt, Registry>,
        },

        ModListReply {
            mod_names: LenPrefixed<VarInt, String>,
            channels: LenPrefixed<VarInt, Channel>,
            registries: LenPrefixed<VarInt, Registry>,
        },

        ServerRegistry {
            name: String,
            snapshot_present: bool,
            snapshot: Vec<u8>,
        },

        ConfigurationData {
            filename: String,
            contents: Vec<u8>,
        },

        Acknowledgement,
    }

    impl FmlHandshake {
        pub fn packet_by_id<R: io::Read>(id: i32, buf: &mut R) -> Result<Self, Error> {
            Ok(match id {
                1 => FmlHandshake::ModList {
                    mod_names: Serializable::read_from(buf)?,
                    channels: Serializable::read_from(buf)?,
                    registries: Serializable::read_from(buf)?,
                },
                3 => FmlHandshake::ServerRegistry {
                    name: Serializable::read_from(buf)?,
                    snapshot_present: Serializable::read_from(buf)?,
                    snapshot: Serializable::read_from(buf)?,
                },
                4 => FmlHandshake::ConfigurationData {
                    filename: Serializable::read_from(buf)?,
                    contents: Serializable::read_from(buf)?,
                },
                _ => unimplemented!(),
            })
        }
    }

    impl Serializable for FmlHandshake {
        fn read_from<R: io::Read>(_buf: &mut R) -> Result<Self, Error> {
            unimplemented!()
        }

        fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
            match self {
                FmlHandshake::ModListReply {
                    mod_names,
                    channels,
                    registries,
                } => {
                    VarInt(2).write_to(buf)?;
                    mod_names.write_to(buf)?;
                    channels.write_to(buf)?;
                    registries.write_to(buf)
                }
                FmlHandshake::Acknowledgement => VarInt(99).write_to(buf),
                _ => unimplemented!(),
            }
        }
    }
}
