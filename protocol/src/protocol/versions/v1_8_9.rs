protocol_packet_ids!(
    handshake Handshaking {
        serverbound Serverbound {
            0x00 => Handshake
        }
        clientbound Clientbound {
        }
    }
    play Play {
        serverbound Serverbound {
            0x00 => KeepAliveServerbound_VarInt
            0x01 => ChatMessage
            0x02 => UseEntity_Handsfree
            0x03 => Player
            0x04 => PlayerPosition
            0x05 => PlayerLook
            0x06 => PlayerPositionLook
            0x07 => PlayerDigging_u8
            0x08 => PlayerBlockPlacement_u8_Item
            0x09 => HeldItemChange
            0x0a => ArmSwing_Handsfree
            0x0b => PlayerAction
            0x0c => SteerVehicle
            0x0d => CloseWindow
            0x0e => ClickWindow_u8
            0x0f => ConfirmTransactionServerbound
            0x10 => CreativeInventoryAction
            0x11 => EnchantItem
            0x12 => SetSign
            0x13 => ClientAbilities_f32
            0x14 => TabComplete_NoAssume
            0x15 => ClientSettings_u8_Handsfree
            0x16 => ClientStatus
            0x17 => PluginMessageServerbound
            0x18 => SpectateTeleport
            0x19 => ResourcePackStatus
        }
        clientbound Clientbound {
            0x00 => KeepAliveClientbound_VarInt
            0x01 => JoinGame_i8
            0x02 => ServerMessage_Position
            0x03 => TimeUpdate
            0x04 => EntityEquipment_u16
            0x05 => SpawnPosition
            0x06 => UpdateHealth
            0x07 => Respawn_Gamemode
            0x08 => TeleportPlayer_NoConfirm
            0x09 => SetCurrentHotbarSlot
            0x0a => EntityUsedBed
            0x0b => Animation
            0x0c => SpawnPlayer_i32_HeldItem
            0x0d => CollectItem_nocount
            0x0e => SpawnObject_i32_NoUUID
            0x0f => SpawnMob_u8_i32_NoUUID
            0x10 => SpawnPainting_NoUUID
            0x11 => SpawnExperienceOrb_i32
            0x12 => EntityVelocity
            0x13 => EntityDestroy
            0x14 => Entity
            0x15 => EntityMove_i8
            0x16 => EntityLook_VarInt
            0x17 => EntityLookAndMove_i8
            0x18 => EntityTeleport_i32
            0x19 => EntityHeadLook
            0x1a => EntityStatus
            0x1b => EntityAttach_leashed
            0x1c => EntityMetadata
            0x1d => EntityEffect
            0x1e => EntityRemoveEffect
            0x1f => SetExperience
            0x20 => EntityProperties
            0x21 => ChunkData_NoEntities_u16
            0x22 => MultiBlockChange_VarInt
            0x23 => BlockChange_VarInt
            0x24 => BlockAction
            0x25 => BlockBreakAnimation
            0x26 => ChunkDataBulk
            0x27 => Explosion
            0x28 => Effect
            0x29 => NamedSoundEffect_u8_NoCategory
            0x2a => Particle_VarIntArray
            0x2b => ChangeGameState
            0x2c => SpawnGlobalEntity_i32
            0x2d => WindowOpen
            0x2e => WindowClose
            0x2f => WindowSetSlot
            0x30 => WindowItems
            0x31 => WindowProperty
            0x32 => ConfirmTransaction
            0x33 => UpdateSign
            0x34 => Maps_NoTracking
            0x35 => UpdateBlockEntity
            0x36 => SignEditorOpen
            0x37 => Statistics
            0x38 => PlayerInfo
            0x39 => PlayerAbilities
            0x3a => TabCompleteReply
            0x3b => ScoreboardObjective
            0x3c => UpdateScore
            0x3d => ScoreboardDisplay
            0x3e => Teams_u8
            0x3f => PluginMessageClientbound
            0x40 => Disconnect
            0x41 => ServerDifficulty
            0x42 => CombatEvent
            0x43 => Camera
            0x44 => WorldBorder
            0x45 => Title_notext_component
            0x46 => SetCompression
            0x47 => PlayerListHeaderFooter
            0x48 => ResourcePackSend
            0x49 => EntityUpdateNBT
        }
    }
    login Login {
        serverbound Serverbound {
            0x00 => LoginStart
            0x01 => EncryptionResponse
        }
        clientbound Clientbound {
            0x00 => LoginDisconnect
            0x01 => EncryptionRequest
            0x02 => LoginSuccess_String
            0x03 => SetInitialCompression
        }
    }
    status Status {
        serverbound Serverbound {
            0x00 => StatusRequest
            0x01 => StatusPing
        }
        clientbound Clientbound {
            0x00 => StatusResponse
            0x01 => StatusPong
        }
    }
);
