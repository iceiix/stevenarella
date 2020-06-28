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
            0x01 => QueryBlockNBT
            0x02 => SetDifficulty
            0x03 => ChatMessage
            0x04 => ClientStatus
            0x05 => ClientSettings
            0x06 => TabComplete
            0x07 => ConfirmTransactionServerbound
            0x08 => ClickWindowButton
            0x09 => ClickWindow
            0x0a => CloseWindow
            0x0b => PluginMessageServerbound
            0x0c => EditBook
            0x0d => QueryEntityNBT
            0x0e => UseEntity_Hand
            0x0f => KeepAliveServerbound_i64
            0x10 => LockDifficulty
            0x11 => PlayerPosition
            0x12 => PlayerPositionLook
            0x13 => PlayerLook
            0x14 => Player
            0x15 => VehicleMove
            0x16 => SteerBoat
            0x17 => PickItem
            0x18 => CraftRecipeRequest
            0x19 => ClientAbilities_f32
            0x1a => PlayerDigging
            0x1b => PlayerAction
            0x1c => SteerVehicle
            0x1d => CraftingBookData
            0x1e => NameItem
            0x1f => ResourcePackStatus
            0x20 => AdvancementTab
            0x21 => SelectTrade
            0x22 => SetBeaconEffect
            0x23 => HeldItemChange
            0x24 => UpdateCommandBlock
            0x25 => UpdateCommandBlockMinecart
            0x26 => CreativeInventoryAction
            0x27 => UpdateJigsawBlock_Type
            0x28 => UpdateStructureBlock
            0x29 => SetSign
            0x2a => ArmSwing
            0x2b => SpectateTeleport
            0x2c => PlayerBlockPlacement_insideblock
            0x2d => UseItem
        }
        clientbound Clientbound {
            0x00 => SpawnObject_VarInt
            0x01 => SpawnExperienceOrb
            0x02 => SpawnGlobalEntity
            0x03 => SpawnMob_NoMeta
            0x04 => SpawnPainting_VarInt
            0x05 => SpawnPlayer_f64_NoMeta
            0x06 => Animation
            0x07 => Statistics
            0x08 => AcknowledgePlayerDigging
            0x09 => BlockBreakAnimation
            0x0a => UpdateBlockEntity
            0x0b => BlockAction
            0x0c => BlockChange_VarInt
            0x0d => BossBar
            0x0e => ServerDifficulty_Locked
            0x0f => ServerMessage_Position
            0x10 => MultiBlockChange_VarInt
            0x11 => TabCompleteReply
            0x12 => DeclareCommands
            0x13 => ConfirmTransaction
            0x14 => WindowClose
            0x15 => WindowItems
            0x16 => WindowProperty
            0x17 => WindowSetSlot
            0x18 => SetCooldown
            0x19 => PluginMessageClientbound
            0x1a => NamedSoundEffect
            0x1b => Disconnect
            0x1c => EntityAction
            0x1d => Explosion
            0x1e => ChunkUnload
            0x1f => ChangeGameState
            0x20 => WindowOpenHorse
            0x21 => KeepAliveClientbound_i64
            0x22 => ChunkData_Biomes3D
            0x23 => Effect
            0x24 => Particle_f64
            0x25 => UpdateLight_NoTrust
            0x26 => JoinGame_HashedSeed_Respawn
            0x27 => Maps
            0x28 => TradeList_WithRestock
            0x29 => EntityMove_i16
            0x2a => EntityLookAndMove_i16
            0x2b => EntityLook_VarInt
            0x2c => Entity
            0x2d => VehicleTeleport
            0x2e => OpenBook
            0x2f => WindowOpen_VarInt
            0x30 => SignEditorOpen
            0x31 => CraftRecipeResponse
            0x32 => PlayerAbilities
            0x33 => CombatEvent
            0x34 => PlayerInfo
            0x35 => FacePlayer
            0x36 => TeleportPlayer_WithConfirm
            0x37 => UnlockRecipes_WithSmelting
            0x38 => EntityDestroy
            0x39 => EntityRemoveEffect
            0x3a => ResourcePackSend
            0x3b => Respawn_HashedSeed
            0x3c => EntityHeadLook
            0x3d => SelectAdvancementTab
            0x3e => WorldBorder
            0x3f => Camera
            0x40 => SetCurrentHotbarSlot
            0x41 => UpdateViewPosition
            0x42 => UpdateViewDistance
            0x43 => ScoreboardDisplay
            0x44 => EntityMetadata
            0x45 => EntityAttach
            0x46 => EntityVelocity
            0x47 => EntityEquipment_VarInt
            0x48 => SetExperience
            0x49 => UpdateHealth
            0x4a => ScoreboardObjective
            0x4b => SetPassengers
            0x4c => Teams_VarInt
            0x4d => UpdateScore
            0x4e => SpawnPosition
            0x4f => TimeUpdate
            0x50 => Title
            0x51 => EntitySoundEffect
            0x52 => SoundEffect
            0x53 => StopSound
            0x54 => PlayerListHeaderFooter
            0x55 => NBTQueryResponse
            0x56 => CollectItem
            0x57 => EntityTeleport_f64
            0x58 => Advancements
            0x59 => EntityProperties
            0x5a => EntityEffect
            0x5b => DeclareRecipes
            0x5c => TagsWithEntities
        }
    }
    login Login {
        serverbound Serverbound {
            0x00 => LoginStart
            0x01 => EncryptionResponse
            0x02 => LoginPluginResponse
        }
        clientbound Clientbound {
            0x00 => LoginDisconnect
            0x01 => EncryptionRequest
            0x02 => LoginSuccess_String
            0x03 => SetInitialCompression
            0x04 => LoginPluginRequest
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
