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
            0x00 => TabComplete_NoAssume
            0x01 => ChatMessage
            0x02 => ClientStatus
            0x03 => ClientSettings_u8
            0x04 => ConfirmTransactionServerbound
            0x05 => EnchantItem
            0x06 => ClickWindow_u8
            0x07 => CloseWindow
            0x08 => PluginMessageServerbound
            0x09 => UseEntity_Hand
            0x0a => KeepAliveServerbound_VarInt
            0x0b => PlayerPosition
            0x0c => PlayerPositionLook
            0x0d => PlayerLook
            0x0e => Player
            0x0f => ClientAbilities_f32
            0x10 => PlayerDigging_u8
            0x11 => PlayerAction
            0x12 => SteerVehicle
            0x13 => ResourcePackStatus
            0x14 => HeldItemChange
            0x15 => CreativeInventoryAction
            0x16 => SetSign
            0x17 => ArmSwing
            0x18 => SpectateTeleport
            0x19 => PlayerBlockPlacement_u8
            0x1a => UseItem
        }
        clientbound Clientbound {
            0x00 => SpawnObject_i32
            0x01 => SpawnExperienceOrb_i32
            0x02 => SpawnGlobalEntity_i32
            0x03 => SpawnMob_u8_i32
            0x04 => SpawnPainting_NoUUID
            0x05 => SpawnPlayer_i32
            0x06 => Animation
            0x07 => Statistics
            0x08 => BlockBreakAnimation
            0x09 => UpdateBlockEntity
            0x0a => BlockAction
            0x0b => BlockChange_VarInt
            0x0c => BossBar
            0x0d => ServerDifficulty
            0x0e => TabCompleteReply
            0x0f => ServerMessage_Position
            0x10 => MultiBlockChange_VarInt
            0x11 => ConfirmTransaction
            0x12 => WindowClose
            0x13 => WindowOpen
            0x14 => WindowItems
            0x15 => WindowProperty
            0x16 => WindowSetSlot
            0x17 => SetCooldown
            0x18 => PluginMessageClientbound
            0x19 => Disconnect
            0x1a => EntityAction
            0x1b => Explosion
            0x1c => ChunkUnload
            0x1d => SetCompression
            0x1e => ChangeGameState
            0x1f => KeepAliveClientbound_VarInt
            0x20 => ChunkData_NoEntities
            0x21 => Effect
            0x22 => Particle_VarIntArray
            0x23 => NamedSoundEffect_u8_NoCategory
            0x24 => JoinGame_i8
            0x25 => Maps
            0x26 => EntityMove_i8
            0x27 => EntityLookAndMove_i8
            0x28 => EntityLook_VarInt
            0x29 => Entity
            0x2a => SignEditorOpen
            0x2b => PlayerAbilities
            0x2c => CombatEvent
            0x2d => PlayerInfo
            0x2e => TeleportPlayer_NoConfirm
            0x2f => EntityUsedBed
            0x30 => EntityDestroy
            0x31 => EntityRemoveEffect
            0x32 => ResourcePackSend
            0x33 => Respawn_Gamemode
            0x34 => EntityHeadLook
            0x35 => WorldBorder
            0x36 => Camera
            0x37 => SetCurrentHotbarSlot
            0x38 => ScoreboardDisplay
            0x39 => EntityMetadata
            0x3a => EntityAttach_leashed
            0x3b => EntityVelocity
            0x3c => EntityEquipment_VarInt
            0x3d => SetExperience
            0x3e => UpdateHealth
            0x3f => ScoreboardObjective
            0x40 => Teams_u8
            0x41 => UpdateScore
            0x42 => SpawnPosition
            0x43 => TimeUpdate
            0x44 => Title_notext_component
            0x45 => UpdateSign
            0x46 => PlayerListHeaderFooter
            0x47 => CollectItem_nocount
            0x48 => EntityTeleport_i32
            0x49 => EntityProperties
            0x4a => EntityEffect
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
