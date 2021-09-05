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
            0x05 => ClientSettings_Filtering
            0x06 => TabComplete
            0x07 => ClickWindowButton
            0x08 => ClickWindow_State
            0x09 => CloseWindow
            0x0a => PluginMessageServerbound
            0x0b => EditBook_Pages
            0x0c => QueryEntityNBT
            0x0d => UseEntity_Sneakflag
            0x0e => GenerateStructure
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
            0x19 => ClientAbilities_u8
            0x1a => PlayerDigging
            0x1b => PlayerAction
            0x1c => SteerVehicle
            0x1d => WindowPong
            0x1e => SetDisplayedRecipe
            0x1f => SetRecipeBookState
            0x20 => NameItem
            0x21 => ResourcePackStatus
            0x22 => AdvancementTab
            0x23 => SelectTrade
            0x24 => SetBeaconEffect
            0x25 => HeldItemChange
            0x26 => UpdateCommandBlock
            0x27 => UpdateCommandBlockMinecart
            0x28 => CreativeInventoryAction
            0x29 => UpdateJigsawBlock_Joint
            0x2a => UpdateStructureBlock
            0x2b => SetSign
            0x2c => ArmSwing
            0x2d => SpectateTeleport
            0x2e => PlayerBlockPlacement_insideblock
            0x2f => UseItem
        }
        clientbound Clientbound {
            0x00 => SpawnObject_VarInt
            0x01 => SpawnExperienceOrb
            0x02 => SpawnMob_NoMeta
            0x03 => SpawnPainting_VarInt
            0x04 => SpawnPlayer_f64_NoMeta
            0x05 => SculkVibrationSignal
            0x06 => Animation
            0x07 => Statistics
            0x08 => AcknowledgePlayerDigging
            0x09 => BlockBreakAnimation
            0x0a => UpdateBlockEntity
            0x0b => BlockAction
            0x0c => BlockChange_VarInt
            0x0d => BossBar
            0x0e => ServerDifficulty_Locked
            0x0f => ServerMessage_Sender
            0x10 => ClearTitles
            0x11 => TabCompleteReply
            0x12 => DeclareCommands
            0x13 => WindowClose
            0x14 => WindowItems_StateCarry
            0x15 => WindowProperty
            0x16 => WindowSetSlot_State
            0x17 => SetCooldown
            0x18 => PluginMessageClientbound
            0x19 => NamedSoundEffect
            0x1a => Disconnect
            0x1b => EntityAction
            0x1c => Explosion_VarInt
            0x1d => ChunkUnload
            0x1e => ChangeGameState
            0x1f => WindowOpenHorse
            0x20 => WorldBorderInit
            0x21 => KeepAliveClientbound_i64
            0x22 => ChunkData_Biomes3D_Bitmasks
            0x23 => Effect
            0x24 => Particle_f64
            0x25 => UpdateLight_Arrays
            0x26 => JoinGame_WorldNames_IsHard
            0x27 => Maps
            0x28 => TradeList_WithRestock
            0x29 => EntityMove_i16
            0x2a => EntityLookAndMove_i16
            0x2b => EntityLook_VarInt
            0x2c => VehicleTeleport
            0x2d => OpenBook
            0x2e => WindowOpen_VarInt
            0x2f => SignEditorOpen
            0x30 => WindowPing
            0x31 => CraftRecipeResponse
            0x32 => PlayerAbilities
            0x33 => CombatEventEnd
            0x34 => CombatEventEnter
            0x35 => CombatEventDeath
            0x36 => PlayerInfo
            0x37 => FacePlayer
            0x38 => TeleportPlayer_WithDismount
            0x39 => UnlockRecipes_WithBlastSmoker
            0x3a => EntityDestroy
            0x3b => EntityRemoveEffect
            0x3c => ResourcePackSend_Prompt
            0x3d => Respawn_NBT
            0x3e => EntityHeadLook
            0x3f => MultiBlockChange_Packed
            0x40 => SelectAdvancementTab
            0x41 => ActionBar
            0x42 => WorldBorderCenter
            0x43 => WorldBorderLerpSize
            0x44 => WorldBorderSize
            0x45 => WorldBorderWarningDelay
            0x46 => WorldBorderWarningReach
            0x47 => Camera
            0x48 => SetCurrentHotbarSlot
            0x49 => UpdateViewPosition
            0x4a => UpdateViewDistance
            0x4b => SpawnPosition_Angle
            0x4c => ScoreboardDisplay
            0x4d => EntityMetadata
            0x4e => EntityAttach
            0x4f => EntityVelocity
            0x50 => EntityEquipment_Array
            0x51 => SetExperience
            0x52 => UpdateHealth
            0x53 => ScoreboardObjective
            0x54 => SetPassengers
            0x55 => Teams_VarInt
            0x56 => UpdateScore
            0x57 => TitleSubtitle
            0x58 => TimeUpdate
            0x59 => Title
            0x5a => TitleTimes
            0x5b => EntitySoundEffect
            0x5c => SoundEffect
            0x5d => StopSound
            0x5e => PlayerListHeaderFooter
            0x5f => NBTQueryResponse
            0x60 => CollectItem
            0x61 => EntityTeleport_f64
            0x62 => Advancements
            0x63 => EntityProperties_VarIntVarInt
            0x64 => EntityEffect
            0x65 => DeclareRecipes
            0x66 => Tags_Nested
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
            0x02 => LoginSuccess_UUID
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
