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
            0x00 => TeleportConfirm
            0x01 => TabComplete
            0x02 => ChatMessage
            0x03 => ClientStatus
            0x04 => ClientSettings
            0x05 => ConfirmTransactionServerbound
            0x06 => EnchantItem
            0x07 => ClickWindow
            0x08 => CloseWindow
            0x09 => PluginMessageServerbound
            0x0a => UseEntity_Hand
            0x0b => KeepAliveServerbound_VarInt
            0x0c => PlayerPosition
            0x0d => PlayerPositionLook
            0x0e => PlayerLook
            0x0f => Player
            0x10 => VehicleMove
            0x11 => SteerBoat
            0x12 => ClientAbilities_f32
            0x13 => PlayerDigging
            0x14 => PlayerAction
            0x15 => SteerVehicle
            0x16 => ResourcePackStatus
            0x17 => HeldItemChange
            0x18 => CreativeInventoryAction
            0x19 => SetSign
            0x1a => ArmSwing
            0x1b => SpectateTeleport
            0x1c => PlayerBlockPlacement_f32
            0x1d => UseItem
        }
        clientbound Clientbound {
            0x00 => SpawnObject
            0x01 => SpawnExperienceOrb
            0x02 => SpawnGlobalEntity
            0x03 => SpawnMob_WithMeta
            0x04 => SpawnPainting_String
            0x05 => SpawnPlayer_f64
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
            0x19 => NamedSoundEffect
            0x1a => Disconnect
            0x1b => EntityAction
            0x1c => Explosion
            0x1d => ChunkUnload
            0x1e => ChangeGameState
            0x1f => KeepAliveClientbound_VarInt
            0x20 => ChunkData
            0x21 => Effect
            0x22 => Particle_VarIntArray
            0x23 => JoinGame_i32
            0x24 => Maps
            0x25 => EntityMove_i16
            0x26 => EntityLookAndMove_i16
            0x27 => EntityLook_VarInt
            0x28 => Entity
            0x29 => VehicleTeleport
            0x2a => SignEditorOpen
            0x2b => PlayerAbilities
            0x2c => CombatEvent
            0x2d => PlayerInfo
            0x2e => TeleportPlayer_WithConfirm
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
            0x3a => EntityAttach
            0x3b => EntityVelocity
            0x3c => EntityEquipment_VarInt
            0x3d => SetExperience
            0x3e => UpdateHealth
            0x3f => ScoreboardObjective
            0x40 => SetPassengers
            0x41 => Teams_u8
            0x42 => UpdateScore
            0x43 => SpawnPosition
            0x44 => TimeUpdate
            0x45 => Title
            0x46 => SoundEffect
            0x47 => PlayerListHeaderFooter
            0x48 => CollectItem
            0x49 => EntityTeleport_f64
            0x4a => EntityProperties
            0x4b => EntityEffect
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
