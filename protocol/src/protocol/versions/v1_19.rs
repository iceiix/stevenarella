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
            //TODO 0x03 => ChatCommand 
            0x04 => ChatMessage
            //TODO 0x05 => ChatPreview
            //TODO 0x06 => ClientCommand
            0x07 => ClientSettings_Filtering
            0x08 => TabComplete
            0x09 => ClickWindowButton
            0x0a => ClickWindow_State
            0x0b => CloseWindow
            0x0c => PluginMessageServerbound
            0x0d => EditBook_Pages
            0x0e => QueryEntityNBT
            0x0f => UseEntity_Sneakflag
            0x10 => GenerateStructure
            0x11 => KeepAliveServerbound_i64
            0x12 => LockDifficulty
            0x13 => PlayerPosition
            0x14 => PlayerPositionLook
            0x15 => PlayerLook
            0x16 => Player
            0x17 => VehicleMove
            0x18 => SteerBoat
            0x19 => PickItem
            0x1a => CraftRecipeRequest
            0x1b => ClientAbilities_u8
            0x1c => PlayerDigging
            0x1d => PlayerAction
            0x1e => SteerVehicle
            0x1f => WindowPong
            0x20 => SetDisplayedRecipe
            0x21 => SetRecipeBookState
            0x22 => NameItem
            0x23 => ResourcePackStatus
            0x24 => AdvancementTab
            0x25 => SelectTrade
            0x26 => SetBeaconEffect
            0x27 => HeldItemChange
            0x28 => UpdateCommandBlock
            0x29 => UpdateCommandBlockMinecart
            0x2a => CreativeInventoryAction
            0x2b => UpdateJigsawBlock_Joint
            0x2c => UpdateStructureBlock
            0x2d => SetSign
            0x2e => ArmSwing
            0x2f => SpectateTeleport
            0x30 => PlayerBlockPlacement_insideblock
            0x31 => UseItem
        }
        clientbound Clientbound {
            0x00 => SpawnObject_VarInt
            0x01 => SpawnExperienceOrb
            0x02 => SpawnPlayer_f64_NoMeta
            0x03 => Animation
            0x04 => Statistics
            0x05 => AcknowledgePlayerDigging // TODO: Acknowledge Block Change
            0x06 => BlockBreakAnimation
            0x07 => UpdateBlockEntity_VarInt
            0x08 => BlockAction
            0x09 => BlockChange_VarInt
            0x0a => BossBar
            0x0b => ServerDifficulty_Locked
            0x0c => ServerMessage_Sender // TODO: Chat Preview
            0x0d => ClearTitles
            0x0e => TabCompleteReply
            0x0f => DeclareCommands
            0x10 => WindowClose
            0x11 => WindowItems_StateCarry
            0x12 => WindowProperty
            0x13 => WindowSetSlot_State
            0x14 => SetCooldown
            0x15 => PluginMessageClientbound
            0x16 => NamedSoundEffect
            0x17 => Disconnect
            0x18 => EntityAction
            0x19 => Explosion_VarInt
            0x1a => ChunkUnload
            0x1b => ChangeGameState
            0x1c => WindowOpenHorse
            0x1d => WorldBorderInit
            0x1e => KeepAliveClientbound_i64
            0x1f => ChunkData_AndLight
            0x20 => Effect
            0x21 => Particle_f64
            0x22 => UpdateLight_Arrays
            0x23 => JoinGame_WorldNames_IsHard_SimDist
            0x24 => Maps
            0x25 => TradeList_WithRestock
            0x26 => EntityMove_i16
            0x27 => EntityLookAndMove_i16
            0x28 => EntityLook_VarInt
            0x29 => VehicleTeleport
            0x2a => OpenBook
            0x2b => WindowOpen_VarInt
            0x2c => SignEditorOpen
            0x2d => WindowPing
            0x2e => CraftRecipeResponse
            0x2f => PlayerAbilities
            //0x30 => PlayerChatMessage // TODO
            0x31 => CombatEventEnd
            0x32 => CombatEventEnter
            0x33 => CombatEventDeath
            0x34 => PlayerInfo
            0x35 => FacePlayer
            0x36 => TeleportPlayer_WithDismount
            0x37 => UnlockRecipes_WithBlastSmoker
            0x38 => EntityDestroy
            0x39 => EntityRemoveEffect_VarInt
            0x3a => ResourcePackSend_Prompt
            0x3b => Respawn_NBT
            0x3c => EntityHeadLook
            0x3d => MultiBlockChange_Packed
            0x3e => SelectAdvancementTab
            //0x3f => ServerData // TODO
            0x40 => ActionBar
            0x41 => WorldBorderCenter
            0x42 => WorldBorderLerpSize
            0x43 => WorldBorderSize
            0x44 => WorldBorderWarningDelay
            0x45 => WorldBorderWarningReach
            0x46 => Camera
            0x47 => SetCurrentHotbarSlot
            0x48 => UpdateViewPosition
            0x49 => UpdateViewDistance
            0x4a => SpawnPosition_Angle
            //0x4b => SetDisplayChatPreview // TODO
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
            0x56 => UpdateScore_VarInt
            0x57 => UpdateSimulationDistance
            0x58 => TitleSubtitle
            0x59 => TimeUpdate
            0x5a => Title
            0x5b => TitleTimes
            0x5c => EntitySoundEffect
            0x5d => SoundEffect
            0x5e => StopSound
            //0x5f => SystemChatMessage // TODO
            0x60 => PlayerListHeaderFooter
            0x61 => NBTQueryResponse
            0x62 => CollectItem
            0x63 => EntityTeleport_f64
            0x64 => Advancements
            0x65 => EntityProperties_VarIntVarInt
            0x66 => EntityEffect_VarInt
            0x67 => DeclareRecipes
            0x68 => Tags_Nested
        }
    }
    login Login {
        serverbound Serverbound {
            0x00 => LoginStart_Sig
            0x01 => EncryptionResponse_Sig
            0x02 => LoginPluginResponse
        }
        clientbound Clientbound {
            0x00 => LoginDisconnect
            0x01 => EncryptionRequest
            0x02 => LoginSuccess_Sig
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
