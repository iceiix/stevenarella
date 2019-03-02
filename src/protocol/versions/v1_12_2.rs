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
            0x0a => UseEntity
            0x0b => KeepAliveServerbound_i64
            0x0c => Player
            0x0d => PlayerPosition
            0x0e => PlayerPositionLook
            0x0f => PlayerLook
            0x10 => VehicleMove
            0x11 => SteerBoat
            0x12 => CraftRecipeRequest
            0x13 => ClientAbilities
            0x14 => PlayerDigging
            0x15 => PlayerAction
            0x16 => SteerVehicle
            0x17 => CraftingBookData
            0x18 => ResourcePackStatus
            0x19 => AdvancementTab
            0x1a => HeldItemChange
            0x1b => CreativeInventoryAction
            0x1c => SetSign
            0x1d => ArmSwing
            0x1e => SpectateTeleport
            0x1f => PlayerBlockPlacement_f32
            0x20 => UseItem
        }
        clientbound Clientbound {
            0x00 => SpawnObject
            0x01 => SpawnExperienceOrb
            0x02 => SpawnGlobalEntity
            0x03 => SpawnMob
            0x04 => SpawnPainting
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
            0x0f => ServerMessage
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
            0x1f => KeepAliveClientbound_i64
            0x20 => ChunkData
            0x21 => Effect
            0x22 => Particle_VarIntArray
            0x23 => JoinGame_i32
            0x24 => Maps
            0x25 => Entity
            0x26 => EntityMove_i16
            0x27 => EntityLookAndMove_i16
            0x28 => EntityLook_VarInt
            0x29 => VehicleTeleport
            0x2a => SignEditorOpen
            0x2b => CraftRecipeResponse
            0x2c => PlayerAbilities
            0x2d => CombatEvent
            0x2e => PlayerInfo
            0x2f => TeleportPlayer_WithConfirm
            0x30 => EntityUsedBed
            0x31 => UnlockRecipes_NoSmelting
            0x32 => EntityDestroy
            0x33 => EntityRemoveEffect
            0x34 => ResourcePackSend
            0x35 => Respawn
            0x36 => EntityHeadLook
            0x37 => SelectAdvancementTab
            0x38 => WorldBorder
            0x39 => Camera
            0x3a => SetCurrentHotbarSlot
            0x3b => ScoreboardDisplay
            0x3c => EntityMetadata
            0x3d => EntityAttach
            0x3e => EntityVelocity
            0x3f => EntityEquipment
            0x40 => SetExperience
            0x41 => UpdateHealth
            0x42 => ScoreboardObjective
            0x43 => SetPassengers
            0x44 => Teams
            0x45 => UpdateScore
            0x46 => SpawnPosition
            0x47 => TimeUpdate
            0x48 => Title
            0x49 => SoundEffect
            0x4a => PlayerListHeaderFooter
            0x4b => CollectItem
            0x4c => EntityTeleport_f64
            0x4d => Advancements
            0x4e => EntityProperties
            0x4f => EntityEffect
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
            0x02 => LoginSuccess
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


