// Copyright 2016 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::format;
use crate::item;
use crate::nbt;
use crate::protocol;
use crate::protocol::LenPrefixed;
use crate::protocol::Serializable;
use crate::shared::Position;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MetadataKey<T: MetaValue> {
    index: i32,
    ty: PhantomData<T>,
}

impl<T: MetaValue> MetadataKey<T> {
    #[allow(dead_code)]
    fn new(index: i32) -> MetadataKey<T> {
        MetadataKey {
            index,
            ty: PhantomData,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Metadata {
    map: HashMap<i32, Value>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            map: HashMap::new(),
        }
    }

    pub fn get<T: MetaValue>(&self, key: &MetadataKey<T>) -> Option<&T> {
        self.map.get(&key.index).map(T::unwrap)
    }

    pub fn put<T: MetaValue>(&mut self, key: &MetadataKey<T>, val: T) {
        self.map.insert(key.index, val.wrap());
    }

    fn put_raw<T: MetaValue>(&mut self, index: i32, val: T) {
        self.map.insert(index, val.wrap());
    }

    fn read_from18<R: io::Read>(buf: &mut R) -> Result<Self, protocol::Error> {
        let mut m = Self::new();
        loop {
            let ty_index = u8::read_from(buf)? as i32;
            if ty_index == 0x7f {
                break;
            }
            let index = ty_index & 0x1f;
            let ty = ty_index >> 5;

            match ty {
                0 => m.put_raw(index, i8::read_from(buf)?),
                1 => m.put_raw(index, i16::read_from(buf)?),
                2 => m.put_raw(index, i32::read_from(buf)?),
                3 => m.put_raw(index, f32::read_from(buf)?),
                4 => m.put_raw(index, String::read_from(buf)?),
                5 => m.put_raw(index, Option::<item::Stack>::read_from(buf)?),
                6 => m.put_raw(
                    index,
                    [
                        i32::read_from(buf)?,
                        i32::read_from(buf)?,
                        i32::read_from(buf)?,
                    ],
                ),
                7 => m.put_raw(
                    index,
                    [
                        f32::read_from(buf)?,
                        f32::read_from(buf)?,
                        f32::read_from(buf)?,
                    ],
                ),
                _ => return Err(protocol::Error::Err("unknown metadata type".to_owned())),
            }
        }
        Ok(m)
    }

    fn write_to18<W: io::Write>(&self, buf: &mut W) -> Result<(), protocol::Error> {
        for (k, v) in &self.map {
            if (*k as u8) > 0x1f {
                panic!("write metadata index {:x} > 0x1f", *k as u8);
            }

            let ty_index: u8 = *k as u8;
            const TYPE_SHIFT: usize = 5;

            match *v {
                Value::Byte(ref val) => {
                    u8::write_to(&(ty_index | (0 << TYPE_SHIFT)), buf)?;
                    val.write_to(buf)?;
                }
                Value::Short(ref val) => {
                    u8::write_to(&(ty_index | (1 << TYPE_SHIFT)), buf)?;
                    val.write_to(buf)?;
                }

                Value::Int(ref val) => {
                    u8::write_to(&(ty_index | (2 << TYPE_SHIFT)), buf)?;
                    val.write_to(buf)?;
                }
                Value::Float(ref val) => {
                    u8::write_to(&(ty_index | (3 << TYPE_SHIFT)), buf)?;
                    val.write_to(buf)?;
                }
                Value::String(ref val) => {
                    u8::write_to(&(ty_index | (4 << TYPE_SHIFT)), buf)?;
                    val.write_to(buf)?;
                }
                Value::OptionalItemStack(ref val) => {
                    u8::write_to(&(ty_index | (5 << TYPE_SHIFT)), buf)?;
                    val.write_to(buf)?;
                }
                Value::Vector(ref val) => {
                    u8::write_to(&(ty_index | (6 << TYPE_SHIFT)), buf)?;
                    val[0].write_to(buf)?;
                    val[1].write_to(buf)?;
                    val[2].write_to(buf)?;
                }
                Value::Rotation(ref val) => {
                    u8::write_to(&(ty_index | (7 << TYPE_SHIFT)), buf)?;
                    val[0].write_to(buf)?;
                    val[1].write_to(buf)?;
                    val[2].write_to(buf)?;
                }

                _ => {
                    panic!("attempted to write 1.9+ metadata to 1.8");
                }
            }
        }
        u8::write_to(&0x7f, buf)?;
        Ok(())
    }

    fn read_from19<R: io::Read>(buf: &mut R) -> Result<Self, protocol::Error> {
        let mut m = Self::new();
        loop {
            let index = u8::read_from(buf)? as i32;
            if index == 0xFF {
                break;
            }
            let ty = protocol::VarInt::read_from(buf)?.0;
            match ty {
                0 => m.put_raw(index, i8::read_from(buf)?),
                1 => m.put_raw(index, protocol::VarInt::read_from(buf)?.0),
                2 => m.put_raw(index, f32::read_from(buf)?),
                3 => m.put_raw(index, String::read_from(buf)?),
                4 => m.put_raw(index, format::Component::read_from(buf)?),
                5 => m.put_raw(index, Option::<item::Stack>::read_from(buf)?),
                6 => m.put_raw(index, bool::read_from(buf)?),
                7 => m.put_raw(
                    index,
                    [
                        f32::read_from(buf)?,
                        f32::read_from(buf)?,
                        f32::read_from(buf)?,
                    ],
                ),
                8 => m.put_raw(index, Position::read_from(buf)?),
                9 => {
                    if bool::read_from(buf)? {
                        m.put_raw(index, Option::<Position>::read_from(buf)?);
                    } else {
                        m.put_raw::<Option<Position>>(index, None);
                    }
                }
                10 => m.put_raw(index, protocol::VarInt::read_from(buf)?),
                11 => {
                    if bool::read_from(buf)? {
                        m.put_raw(index, Option::<protocol::UUID>::read_from(buf)?);
                    } else {
                        m.put_raw::<Option<protocol::UUID>>(index, None);
                    }
                }
                12 => m.put_raw(index, protocol::VarInt::read_from(buf)?.0 as u16),
                13 => {
                    let ty = u8::read_from(buf)?;
                    if ty != 0 {
                        let name = nbt::read_string(buf)?;
                        let tag = nbt::Tag::read_from(buf)?;

                        m.put_raw(index, nbt::NamedTag(name, tag));
                    }
                }
                _ => return Err(protocol::Error::Err("unknown metadata type".to_owned())),
            }
        }
        Ok(m)
    }

    fn write_to19<W: io::Write>(&self, buf: &mut W) -> Result<(), protocol::Error> {
        for (k, v) in &self.map {
            (*k as u8).write_to(buf)?;
            match *v {
                Value::Byte(ref val) => {
                    u8::write_to(&0, buf)?;
                    val.write_to(buf)?;
                }
                Value::Int(ref val) => {
                    u8::write_to(&1, buf)?;
                    protocol::VarInt(*val).write_to(buf)?;
                }
                Value::Float(ref val) => {
                    u8::write_to(&2, buf)?;
                    val.write_to(buf)?;
                }
                Value::String(ref val) => {
                    u8::write_to(&3, buf)?;
                    val.write_to(buf)?;
                }
                Value::FormatComponent(ref val) => {
                    u8::write_to(&4, buf)?;
                    val.write_to(buf)?;
                }
                Value::OptionalItemStack(ref val) => {
                    u8::write_to(&5, buf)?;
                    val.write_to(buf)?;
                }
                Value::Bool(ref val) => {
                    u8::write_to(&6, buf)?;
                    val.write_to(buf)?;
                }
                Value::Vector(ref val) => {
                    u8::write_to(&7, buf)?;
                    val[0].write_to(buf)?;
                    val[1].write_to(buf)?;
                    val[2].write_to(buf)?;
                }
                Value::Position(ref val) => {
                    u8::write_to(&8, buf)?;
                    val.write_to(buf)?;
                }
                Value::OptionalPosition(ref val) => {
                    u8::write_to(&9, buf)?;
                    val.is_some().write_to(buf)?;
                    val.write_to(buf)?;
                }
                Value::Direction(ref val) => {
                    u8::write_to(&10, buf)?;
                    val.write_to(buf)?;
                }
                Value::OptionalUUID(ref val) => {
                    u8::write_to(&11, buf)?;
                    val.is_some().write_to(buf)?;
                    val.write_to(buf)?;
                }
                Value::Block(ref val) => {
                    u8::write_to(&11, buf)?;
                    protocol::VarInt(*val as i32).write_to(buf)?;
                }
                Value::NBTTag(ref _val) => {
                    u8::write_to(&13, buf)?;
                    // TODO: write NBT tags metadata
                    //nbt::Tag(*val).write_to(buf)?;
                }
                _ => panic!("unexpected metadata"),
            }
        }
        u8::write_to(&0xFF, buf)?;
        Ok(())
    }

    fn read_from113<R: io::Read>(buf: &mut R) -> Result<Self, protocol::Error> {
        let mut m = Self::new();
        loop {
            let index = u8::read_from(buf)? as i32;
            if index == 0xFF {
                break;
            }
            let ty = protocol::VarInt::read_from(buf)?.0;
            match ty {
                0 => m.put_raw(index, i8::read_from(buf)?),
                1 => m.put_raw(index, protocol::VarInt::read_from(buf)?.0),
                2 => m.put_raw(index, f32::read_from(buf)?),
                3 => m.put_raw(index, String::read_from(buf)?),
                4 => m.put_raw(index, format::Component::read_from(buf)?),
                5 => m.put_raw(
                    index,
                    LenPrefixed::<bool, format::Component>::read_from(buf)?,
                ),
                6 => m.put_raw(index, Option::<item::Stack>::read_from(buf)?),
                7 => m.put_raw(index, bool::read_from(buf)?),
                8 => m.put_raw(
                    index,
                    [
                        f32::read_from(buf)?,
                        f32::read_from(buf)?,
                        f32::read_from(buf)?,
                    ],
                ),
                9 => m.put_raw(index, Position::read_from(buf)?),
                10 => {
                    if bool::read_from(buf)? {
                        m.put_raw(index, Option::<Position>::read_from(buf)?);
                    } else {
                        m.put_raw::<Option<Position>>(index, None);
                    }
                }
                11 => m.put_raw(index, protocol::VarInt::read_from(buf)?),
                12 => {
                    if bool::read_from(buf)? {
                        m.put_raw(index, Option::<protocol::UUID>::read_from(buf)?);
                    } else {
                        m.put_raw::<Option<protocol::UUID>>(index, None);
                    }
                }
                13 => m.put_raw(index, protocol::VarInt::read_from(buf)?.0 as u16),
                14 => {
                    let ty = u8::read_from(buf)?;
                    if ty != 0 {
                        let name = nbt::read_string(buf)?;
                        let tag = nbt::Tag::read_from(buf)?;

                        m.put_raw(index, nbt::NamedTag(name, tag));
                    }
                }
                15 => panic!("TODO: particle"),
                16 => m.put_raw(index, VillagerData::read_from(buf)?),
                17 => {
                    if bool::read_from(buf)? {
                        m.put_raw(index, Option::<protocol::VarInt>::read_from(buf)?);
                    } else {
                        m.put_raw::<Option<protocol::VarInt>>(index, None);
                    }
                }
                18 => m.put_raw(index, PoseData::read_from(buf)?),
                _ => return Err(protocol::Error::Err("unknown metadata type".to_owned())),
            }
        }
        Ok(m)
    }

    fn write_to113<W: io::Write>(&self, buf: &mut W) -> Result<(), protocol::Error> {
        for (k, v) in &self.map {
            (*k as u8).write_to(buf)?;
            match *v {
                Value::Byte(ref val) => {
                    u8::write_to(&0, buf)?;
                    val.write_to(buf)?;
                }
                Value::Int(ref val) => {
                    u8::write_to(&1, buf)?;
                    protocol::VarInt(*val).write_to(buf)?;
                }
                Value::Float(ref val) => {
                    u8::write_to(&2, buf)?;
                    val.write_to(buf)?;
                }
                Value::String(ref val) => {
                    u8::write_to(&3, buf)?;
                    val.write_to(buf)?;
                }
                Value::FormatComponent(ref val) => {
                    u8::write_to(&4, buf)?;
                    val.write_to(buf)?;
                }
                Value::OptionalFormatComponent(ref val) => {
                    u8::write_to(&5, buf)?;
                    val.write_to(buf)?;
                }
                Value::OptionalItemStack(ref val) => {
                    u8::write_to(&6, buf)?;
                    val.write_to(buf)?;
                }
                Value::Bool(ref val) => {
                    u8::write_to(&7, buf)?;
                    val.write_to(buf)?;
                }
                Value::Vector(ref val) => {
                    u8::write_to(&8, buf)?;
                    val[0].write_to(buf)?;
                    val[1].write_to(buf)?;
                    val[2].write_to(buf)?;
                }
                Value::Position(ref val) => {
                    u8::write_to(&9, buf)?;
                    val.write_to(buf)?;
                }
                Value::OptionalPosition(ref val) => {
                    u8::write_to(&10, buf)?;
                    val.is_some().write_to(buf)?;
                    val.write_to(buf)?;
                }
                Value::Direction(ref val) => {
                    u8::write_to(&11, buf)?;
                    val.write_to(buf)?;
                }
                Value::OptionalUUID(ref val) => {
                    u8::write_to(&12, buf)?;
                    val.is_some().write_to(buf)?;
                    val.write_to(buf)?;
                }
                Value::Block(ref val) => {
                    u8::write_to(&13, buf)?;
                    protocol::VarInt(*val as i32).write_to(buf)?;
                }
                Value::NBTTag(ref _val) => {
                    u8::write_to(&14, buf)?;
                    // TODO: write NBT tags metadata
                    //nbt::Tag(*val).write_to(buf)?;
                }
                Value::Particle(ref val) => {
                    u8::write_to(&15, buf)?;
                    val.write_to(buf)?;
                }
                Value::Villager(ref val) => {
                    u8::write_to(&16, buf)?;
                    val.write_to(buf)?;
                }
                Value::OptionalVarInt(ref val) => {
                    u8::write_to(&17, buf)?;
                    val.write_to(buf)?;
                }
                Value::Pose(ref val) => {
                    u8::write_to(&18, buf)?;
                    val.write_to(buf)?;
                }
                _ => panic!("unexpected metadata"),
            }
        }
        u8::write_to(&0xFF, buf)?;
        Ok(())
    }
}

impl Serializable for Metadata {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, protocol::Error> {
        let protocol_version = protocol::current_protocol_version();

        if protocol_version >= 404 {
            Metadata::read_from113(buf)
        } else if protocol_version >= 74 {
            Metadata::read_from19(buf)
        } else {
            Metadata::read_from18(buf)
        }
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), protocol::Error> {
        let protocol_version = protocol::current_protocol_version();

        if protocol_version >= 404 {
            self.write_to113(buf)
        } else if protocol_version >= 74 {
            self.write_to19(buf)
        } else {
            self.write_to18(buf)
        }
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Metadata[ ")?;
        for (k, v) in &self.map {
            write!(f, "{:?}={:?}, ", k, v)?;
        }
        write!(f, "]")
    }
}

impl Default for Metadata {
    fn default() -> Metadata {
        Metadata::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    FormatComponent(format::Component),
    OptionalFormatComponent(LenPrefixed<bool, format::Component>),
    OptionalItemStack(Option<item::Stack>),
    Bool(bool),
    Vector([f32; 3]),
    Rotation([i32; 3]),
    Position(Position),
    OptionalPosition(Option<Position>),
    Direction(protocol::VarInt), // TODO: Proper type
    OptionalUUID(Option<protocol::UUID>),
    Block(u16), // TODO: Proper type
    NBTTag(nbt::NamedTag),
    Particle(ParticleData),
    Villager(VillagerData),
    OptionalVarInt(Option<protocol::VarInt>),
    Pose(PoseData),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticleData {
    AmbientEntityEffect,
    AngryVillager,
    Barrier,
    Block {
        block_state: protocol::VarInt,
    },
    Bubble,
    Cloud,
    Crit,
    DamageIndicator,
    DragonBreath,
    DrippingLava,
    DrippingWater,
    Dust {
        red: f32,
        green: f32,
        blue: f32,
        scale: f32,
    },
    Effect,
    ElderGuardian,
    EnchantedHit,
    Enchant,
    EndRod,
    EntityEffect,
    ExplosionEmitter,
    Explosion,
    FallingDust {
        block_state: protocol::VarInt,
    },
    Firework,
    Fishing,
    Flame,
    HappyVillager,
    Heart,
    InstantEffect,
    Item {
        item: Option<item::Stack>,
    },
    ItemSlime,
    ItemSnowball,
    LargeSmoke,
    Lava,
    Mycelium,
    Note,
    Poof,
    Portal,
    Rain,
    Smoke,
    Spit,
    SquidInk,
    SweepAttack,
    TotemOfUndying,
    Underwater,
    Splash,
    Witch,
    BubblePop,
    CurrentDown,
    BubbleColumnUp,
    Nautilus,
    Dolphin,
}

impl Serializable for ParticleData {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, protocol::Error> {
        let id = protocol::VarInt::read_from(buf)?.0;
        Ok(match id {
            0 => ParticleData::AmbientEntityEffect,
            1 => ParticleData::AngryVillager,
            2 => ParticleData::Barrier,
            3 => ParticleData::Block {
                block_state: Serializable::read_from(buf)?,
            },
            4 => ParticleData::Bubble,
            5 => ParticleData::Cloud,
            6 => ParticleData::Crit,
            7 => ParticleData::DamageIndicator,
            8 => ParticleData::DragonBreath,
            9 => ParticleData::DrippingLava,
            10 => ParticleData::DrippingWater,
            11 => ParticleData::Dust {
                red: Serializable::read_from(buf)?,
                green: Serializable::read_from(buf)?,
                blue: Serializable::read_from(buf)?,
                scale: Serializable::read_from(buf)?,
            },
            12 => ParticleData::Effect,
            13 => ParticleData::ElderGuardian,
            14 => ParticleData::EnchantedHit,
            15 => ParticleData::Enchant,
            16 => ParticleData::EndRod,
            17 => ParticleData::EntityEffect,
            18 => ParticleData::ExplosionEmitter,
            19 => ParticleData::Explosion,
            20 => ParticleData::FallingDust {
                block_state: Serializable::read_from(buf)?,
            },
            21 => ParticleData::Firework,
            22 => ParticleData::Fishing,
            23 => ParticleData::Flame,
            24 => ParticleData::HappyVillager,
            25 => ParticleData::Heart,
            26 => ParticleData::InstantEffect,
            27 => ParticleData::Item {
                item: Serializable::read_from(buf)?,
            },
            28 => ParticleData::ItemSlime,
            29 => ParticleData::ItemSnowball,
            30 => ParticleData::LargeSmoke,
            31 => ParticleData::Lava,
            32 => ParticleData::Mycelium,
            33 => ParticleData::Note,
            34 => ParticleData::Poof,
            35 => ParticleData::Portal,
            36 => ParticleData::Rain,
            37 => ParticleData::Smoke,
            38 => ParticleData::Spit,
            39 => ParticleData::SquidInk,
            40 => ParticleData::SweepAttack,
            41 => ParticleData::TotemOfUndying,
            42 => ParticleData::Underwater,
            43 => ParticleData::Splash,
            44 => ParticleData::Witch,
            45 => ParticleData::BubblePop,
            46 => ParticleData::CurrentDown,
            47 => ParticleData::BubbleColumnUp,
            48 => ParticleData::Nautilus,
            49 => ParticleData::Dolphin,
            _ => panic!("unrecognized particle data id {}", id),
        })
    }

    fn write_to<W: io::Write>(&self, _buf: &mut W) -> Result<(), protocol::Error> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct VillagerData {
    villager_type: protocol::VarInt,
    profession: protocol::VarInt,
    level: protocol::VarInt,
}

impl Serializable for VillagerData {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, protocol::Error> {
        let villager_type = protocol::VarInt::read_from(buf)?;
        let profession = protocol::VarInt::read_from(buf)?;
        let level = protocol::VarInt::read_from(buf)?;
        Ok(VillagerData {
            villager_type,
            profession,
            level,
        })
    }

    fn write_to<W: io::Write>(&self, _buf: &mut W) -> Result<(), protocol::Error> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoseData {
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Sneaking,
    Dying,
    LongJumping,
}

impl Serializable for PoseData {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, protocol::Error> {
        let n = protocol::VarInt::read_from(buf)?;
        Ok(match n.0 {
            0 => PoseData::Standing,
            1 => PoseData::FallFlying,
            2 => PoseData::Sleeping,
            3 => PoseData::Swimming,
            4 => PoseData::SpinAttack,
            5 => PoseData::Sneaking,
            6 => PoseData::Dying,
            7 => PoseData::LongJumping,
            _ => panic!("unknown pose data: {}", n.0),
        })
    }

    fn write_to<W: io::Write>(&self, _buf: &mut W) -> Result<(), protocol::Error> {
        unimplemented!()
    }
}

pub trait MetaValue {
    fn unwrap(_: &Value) -> &Self;
    fn wrap(self) -> Value;
}

impl MetaValue for i8 {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Byte(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Byte(self)
    }
}

impl MetaValue for i16 {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Short(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Short(self)
    }
}

impl MetaValue for i32 {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Int(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Int(self)
    }
}

impl MetaValue for f32 {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Float(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Float(self)
    }
}

impl MetaValue for String {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::String(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::String(self)
    }
}

impl MetaValue for format::Component {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::FormatComponent(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::FormatComponent(self)
    }
}

impl MetaValue for LenPrefixed<bool, format::Component> {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::OptionalFormatComponent(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::OptionalFormatComponent(self)
    }
}

impl MetaValue for Option<item::Stack> {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::OptionalItemStack(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::OptionalItemStack(self)
    }
}

impl MetaValue for bool {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Bool(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Bool(self)
    }
}

impl MetaValue for [i32; 3] {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Rotation(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Rotation(self)
    }
}

impl MetaValue for [f32; 3] {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Vector(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Vector(self)
    }
}

impl MetaValue for Position {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Position(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Position(self)
    }
}

impl MetaValue for Option<Position> {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::OptionalPosition(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::OptionalPosition(self)
    }
}

impl MetaValue for protocol::VarInt {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Direction(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Direction(self)
    }
}

impl MetaValue for Option<protocol::UUID> {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::OptionalUUID(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::OptionalUUID(self)
    }
}

impl MetaValue for u16 {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Block(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Block(self)
    }
}

impl MetaValue for nbt::NamedTag {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::NBTTag(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::NBTTag(self)
    }
}

impl MetaValue for VillagerData {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Villager(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Villager(self)
    }
}

impl MetaValue for Option<protocol::VarInt> {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::OptionalVarInt(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::OptionalVarInt(self)
    }
}

impl MetaValue for PoseData {
    fn unwrap(value: &Value) -> &Self {
        match *value {
            Value::Pose(ref val) => val,
            _ => panic!("incorrect key"),
        }
    }
    fn wrap(self) -> Value {
        Value::Pose(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::marker::PhantomData;

    const TEST: MetadataKey<String> = MetadataKey {
        index: 0,
        ty: PhantomData,
    };

    #[test]
    fn basic() {
        let mut m = Metadata::new();

        m.put(&TEST, "Hello world".to_owned());

        match m.get(&TEST) {
            Some(val) => {
                assert!(val == "Hello world");
            }
            None => panic!("failed"),
        }
    }
}
