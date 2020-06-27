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
            0x00 => KeepAliveServerbound_i32
            0x01 => ChatMessage
            0x02 => UseEntity_Handsfree_i32
            0x03 => Player
            0x04 => PlayerPosition_HeadY
            0x05 => PlayerLook
            0x06 => PlayerPositionLook_HeadY
            0x07 => PlayerDigging_u8_u8y
            0x08 => PlayerBlockPlacement_u8_Item_u8y
            0x09 => HeldItemChange
            0x0a => ArmSwing_Handsfree_ID
            0x0b => PlayerAction_i32
            0x0c => SteerVehicle_jump_unmount
            0x0d => CloseWindow
            0x0e => ClickWindow_u8
            0x0f => ConfirmTransactionServerbound
            0x10 => CreativeInventoryAction
            0x11 => EnchantItem
            0x12 => SetSign_i16y
            0x13 => ClientAbilities_f32
            0x14 => TabComplete_NoAssume_NoTarget
            0x15 => ClientSettings_u8_Handsfree_Difficulty
            0x16 => ClientStatus_u8
            0x17 => PluginMessageServerbound_i16
        }
        clientbound Clientbound {
            0x00 => KeepAliveClientbound_i32
            0x01 => JoinGame_i8_NoDebug
            0x02 => ServerMessage_NoPosition
            0x03 => TimeUpdate
            0x04 => EntityEquipment_u16_i32
            0x05 => SpawnPosition_i32
            0x06 => UpdateHealth_u16
            0x07 => Respawn_Gamemode
            0x08 => TeleportPlayer_OnGround
            0x09 => SetCurrentHotbarSlot
            0x0a => EntityUsedBed_i32
            0x0b => Animation
            0x0c => SpawnPlayer_i32_HeldItem_String
            0x0d => CollectItem_nocount_i32
            0x0e => SpawnObject_i32_NoUUID
            0x0f => SpawnMob_u8_i32_NoUUID
            0x10 => SpawnPainting_NoUUID_i32
            0x11 => SpawnExperienceOrb_i32
            0x12 => EntityVelocity_i32
            0x13 => EntityDestroy_u8
            0x14 => Entity_i32
            0x15 => EntityMove_i8_i32_NoGround
            0x16 => EntityLook_i32_NoGround
            0x17 => EntityLookAndMove_i8_i32_NoGround
            0x18 => EntityTeleport_i32_i32_NoGround
            0x19 => EntityHeadLook_i32
            0x1a => EntityStatus
            0x1b => EntityAttach_leashed
            0x1c => EntityMetadata_i32
            0x1d => EntityEffect_i32
            0x1e => EntityRemoveEffect_i32
            0x1f => SetExperience_i16
            0x20 => EntityProperties_i32
            0x21 => ChunkData_17
            0x22 => MultiBlockChange_u16
            0x23 => BlockChange_u8
            0x24 => BlockAction_u16
            0x25 => BlockBreakAnimation_i32
            0x26 => ChunkDataBulk_17
            0x27 => Explosion
            0x28 => Effect_u8y
            0x29 => NamedSoundEffect_u8_NoCategory
            0x2a => Particle_Named
            0x2b => ChangeGameState
            0x2c => SpawnGlobalEntity_i32
            0x2d => WindowOpen_u8
            0x2e => WindowClose
            0x2f => WindowSetSlot
            0x30 => WindowItems
            0x31 => WindowProperty
            0x32 => ConfirmTransaction
            0x33 => UpdateSign_u16
            0x34 => Maps_NoTracking_Data
            0x35 => UpdateBlockEntity_Data
            0x36 => SignEditorOpen_i32
            0x37 => Statistics
            0x38 => PlayerInfo_String
            0x39 => PlayerAbilities
            0x3a => TabCompleteReply
            0x3b => ScoreboardObjective_NoMode
            0x3c => UpdateScore_i32
            0x3d => ScoreboardDisplay
            0x3e => Teams_NoVisColor
            0x3f => PluginMessageClientbound_i16
            0x40 => Disconnect
            -0x1a => CoFHLib_SendUUID
        }
    }
    login Login {
        serverbound Serverbound {
            0x00 => LoginStart
            0x01 => EncryptionResponse_i16
        }
        clientbound Clientbound {
            0x00 => LoginDisconnect
            0x01 => EncryptionRequest_i16
            0x02 => LoginSuccess_String
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
