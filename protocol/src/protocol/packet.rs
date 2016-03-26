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

use format;

state_packets!(
    handshake Handshaking {
         serverbound Serverbound {
            // Handshake is the first packet sent in the protocol.
            // Its used for deciding if the request is a client
            // is requesting status information about the server
            // (MOTD, players etc) or trying to login to the server.
            //
            // The host and port fields are not used by the vanilla
            // server but are there for virtual server hosting to
            // be able to redirect a client to a target server with
            // a single address + port.
            //
            // Some modified servers/proxies use the handshake field
            // differently, packing information into the field other
            // than the hostname due to the protocol not providing
            // any system for custom information to be transfered
            // by the client to the server until after login.
            Handshake {
                // The protocol version of the connecting client
                protocol_version: VarInt =,
                // The hostname the client connected to
                host: String =,
                // The port the client connected to
                port: u16 =,
                // The next protocol state the client wants
                next: VarInt =,
            }
        }
        clientbound Clientbound {
        }
    }
    play Play {
        serverbound Serverbound {
            // TeleportConfirm is sent by the client as a reply to a telport from
            // the server.
            TeleportConfirm {
                teleport_id: VarInt =,
            }
            // TabComplete is sent by the client when the client presses tab in
            // the chat box.
            TabComplete {
                text: String =,
                assume_command: bool =,
                has_target: bool =,
                target: Option<types::Position> = when(|p: &TabComplete| p.has_target),
            }
            // ChatMessage is sent by the client when it sends a chat message or
            // executes a command (prefixed by '/').
            ChatMessage {
                message: String =,
            }
            // ClientStatus is sent to update the client's status
            ClientStatus {
                action_id: VarInt =,
            }
            // ClientSettings is sent by the client to update its current settings.
            ClientSettings {
                locale: String =,
                view_distance: u8 =,
                chat_mode: VarInt =,
                chat_colors: bool =,
                displayed_skin_parts: u8 =,
                main_hand: VarInt =,
            }
            // ConfirmTransactionServerbound is a reply to ConfirmTransaction.
            ConfirmTransactionServerbound {
                id: u8 =,
                action_number: i16 =,
                accepted: bool =,
            }
            // EnchantItem is sent when the client enchants an item.
            EnchantItem {
                id: u8 =,
                enchantment: u8 =,
            }
            // ClickWindow is sent when the client clicks in a window.
            ClickWindow {
                id: u8 =,
                slot: i16 =,
                button: u8 =,
                action_number: u16 =,
                mode: VarInt =,
                clicked_item: Option<item::Stack> =,
            }
            // CloseWindow is sent when the client closes a window.
            CloseWindow {
                id: u8 =,
            }
            // PluginMessageServerbound is used for custom messages between the client
            // and server. This is mainly for plugins/mods but vanilla has a few channels
            // registered too.
            PluginMessageServerbound {
                channel: String =,
                data: Vec<u8> =,
            }
            // UseEntity is sent when the user interacts (right clicks) or attacks
            // (left clicks) an entity.
            UseEntity {
                target_id: VarInt =,
                ty: VarInt =,
                target_x: f32 = when(|p: &UseEntity| p.ty.0 == 2),
                target_y: f32 = when(|p: &UseEntity| p.ty.0 == 2),
                target_z: f32 = when(|p: &UseEntity| p.ty.0 == 2),
                hand: VarInt = when(|p: &UseEntity| p.ty.0 == 0 || p.ty.0 == 2),
            }
            // KeepAliveServerbound is sent by a client as a response to a
            // KeepAliveClientbound. If the client doesn't reply the server
            // may disconnect the client.
            KeepAliveServerbound {
                id: VarInt =,
            }
            // PlayerPosition is used to update the player's position.
            PlayerPosition {
                x: f64 =,
                y: f64 =,
                z: f64 =,
                on_ground: bool =,
            }
            // PlayerPositionLook is a combination of PlayerPosition and
            // PlayerLook.
            PlayerPositionLook {
                x: f64 =,
                y: f64 =,
                z: f64 =,
                yaw: f32 =,
                pitch: f32 =,
                on_ground: bool =,
            }
            // PlayerLook is used to update the player's rotation.
            PlayerLook {
                yaw: f32 =,
                pitch: f32 =,
                on_ground: bool =,
            }
            // Player is used to update whether the player is on the ground or not.
            Player {
                on_ground: bool =,
            }
            // Sent by the client when in a vehicle instead of the normal move packet.
            VehicleMove {
                x: f64 =,
                y: f64 =,
                z: f64 =,
                yaw: f32 =,
                pitch: f32 =,
            }
            // TODO: Document
            SteerBoat {
                unknown: bool =,
                unknown2: bool =,
            }
            // ClientAbilities is used to modify the players current abilities.
            // Currently flying is the only one
            ClientAbilities {
                flags: u8 =,
                flying_speed: f32 =,
                walking_speed: f32 =,
            }
            // PlayerDigging is sent when the client starts/stops digging a block.
            // It also can be sent for droppping items and eating/shooting.
            PlayerDigging {
                status: VarInt =,
                location: types::Position =,
                face: u8 =,
            }
            // PlayerAction is sent when a player preforms various actions.
            PlayerAction {
                entity_id: VarInt =,
                action_id: VarInt =,
                jump_boost: VarInt =,
            }
            // SteerVehicle is sent by the client when steers or preforms an action
            // on a vehicle.
            SteerVehicle {
                sideways: f32 =,
                forward: f32 =,
                flags: u8 =,
            }
            // ResourcePackStatus informs the server of the client's current progress
            // in activating the requested resource pack
            ResourcePackStatus {
                hash: String =,
                result: VarInt =,
            }
            // HeldItemChange is sent when the player changes the currently active
            // hotbar slot.
            HeldItemChange {
                slot: i16 =,
            }
            // CreativeInventoryAction is sent when the client clicks in the creative
            // inventory. This is used to spawn items in creative.
            CreativeInventoryAction {
                slot: i16 =,
                clicked_item: Option<item::Stack> =,
            }
            // SetSign sets the text on a sign after placing it.
            SetSign {
                location: types::Position =,
                line1: String =,
                line2: String =,
                line3: String =,
                line4: String =,
            }
            // ArmSwing is sent by the client when the player left clicks (to swing their
            // arm).
            ArmSwing {
                hand: VarInt =,
            }
            // SpectateTeleport is sent by clients in spectator mode to teleport to a player.
            SpectateTeleport {
                target: UUID =,
            }
            // PlayerBlockPlacement is sent when the client tries to place a block.
            PlayerBlockPlacement {
                location: types::Position =,
                face: VarInt =,
                hand: VarInt =,
                cursor_x: u8 =,
                cursor_y: u8 =,
                cursor_z: u8 =,
            }
            // UseItem is sent when the client tries to use an item.
            UseItem {
                hand: VarInt =,
            }
        }
        clientbound Clientbound {
            // SpawnObject is used to spawn an object or vehicle into the world when it
            // is in range of the client.
            SpawnObject {
                entity_id: VarInt =,
                uuid: UUID =,
                ty: u8 =,
                x: f64 =,
                y: f64 =,
                z: f64 =,
                pitch: i8 =,
                yaw: i8 =,
                data: i32 =,
                velocity_x: i16 =,
                velocity_y: i16 =,
                velocity_z: i16 =,
            }
            // SpawnExperienceOrb spawns a single experience orb into the world when
            // it is in range of the client. The count controls the amount of experience
            // gained when collected.
            SpawnExperienceOrb {
                entity_id: VarInt =,
                x: f64 =,
                y: f64 =,
                z: f64 =,
                count: i16 =,
            }
            // SpawnGlobalEntity spawns an entity which is visible from anywhere in the
            // world. Currently only used for lightning.
            SpawnGlobalEntity {
                entity_id: VarInt =,
                ty: u8 =,
                x: f64 =,
                y: f64 =,
                z: f64 =,
            }
            // SpawnMob is used to spawn a living entity into the world when it is in
            // range of the client.
            SpawnMob {
                entity_id: VarInt =,
                uuid: UUID =,
                ty: u8 =,
                x: f64 =,
                y: f64 =,
                z: f64 =,
                yaw: i8 =,
                pitch: i8 =,
                head_pitch: i8 =,
                velocity_x: i16 =,
                velocity_y: i16 =,
                velocity_z: i16 =,
                metadata: types::Metadata =,
            }
            // SpawnPainting spawns a painting into the world when it is in range of
            // the client. The title effects the size and the texture of the painting.
            SpawnPainting {
                entity_id: VarInt =,
                uuid: UUID =,
                title: String =,
                location: types::Position =,
                direction: u8 =,
            }
            // SpawnPlayer is used to spawn a player when they are in range of the client.
            // This packet alone isn't enough to display the player as the skin and username
            // information is in the player information packet.
            SpawnPlayer {
                entity_id: VarInt =,
                uuid: UUID =,
                x: f64 =,
                y: f64 =,
                z: f64 =,
                yaw: i8 =,
                pitch: i8 =,
                metadata: types::Metadata =,
            }
            // Animation is sent by the server to play an animation on a specific entity.
            Animation {
                entity_id: VarInt =,
                animation_id: u8 =,
            }
            // Statistics is used to update the statistics screen for the client.
            Statistics {
                statistices: LenPrefixed<VarInt, packet::Statistic> =,
            }
            // BlockBreakAnimation is used to create and update the block breaking
            // animation played when a player starts digging a block.
            BlockBreakAnimation {
                entity_id: VarInt =,
                location: types::Position =,
                stage: i8 =,
            }
            // UpdateBlockEntity updates the nbt tag of a block entity in the
            // world.
            UpdateBlockEntity {
                location: types::Position =,
                action: u8 =,
                nbt: Option<nbt::NamedTag> =,
            }
            // BlockAction triggers different actions depending on the target block.
            BlockAction {
                location: types::Position =,
                byte1: u8 =,
                byte2: u8 =,
                block_type: VarInt =,
            }
            // BlockChange is used to update a single block on the client.
            BlockChange {
                location: types::Position =,
                block_id: VarInt =,
            }
            // BossBar displays and/or changes a boss bar that is displayed on the
            // top of the client's screen. This is normally used for bosses such as
            // the ender dragon or the wither.
            BossBar {
                uuid: UUID =,
                action: VarInt =,
                title: format::Component = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 3),
                health: f32 = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 2),
                color: VarInt = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 4),
                style: VarInt = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 4),
                flags: u8 = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 5),
            }
            // ServerDifficulty changes the displayed difficulty in the client's menu
            // as well as some ui changes for hardcore.
            ServerDifficulty {
                difficulty: u8 =,
            }
            // TabCompleteReply is sent as a reply to a tab completion request.
            // The matches should be possible completions for the command/chat the
            // player sent.
            TabCompleteReply {
                matches: LenPrefixed<VarInt, String> =,
            }
            // ServerMessage is a message sent by the server. It could be from a player
            // or just a system message. The Type field controls the location the
            // message is displayed at and when the message is displayed.
            ServerMessage {
                message: format::Component =,
                // 0 - Chat message, 1 - System message, 2 - Action bar message
                position: u8 =,
            }
            // MultiBlockChange is used to update a batch of blocks in a single packet.
            MultiBlockChange {
                chunk_x: i32 =,
                chunk_y: i32 =,
                records: LenPrefixed<VarInt, packet::BlockChangeRecord> =,
            }
            // ConfirmTransaction notifies the client whether a transaction was successful
            // or failed (e.g. due to lag).
            ConfirmTransaction {
                id: u8 =,
                action_number: i16 =,
                accepted: bool =,
            }
            // WindowClose forces the client to close the window with the given id,
            // e.g. a chest getting destroyed.
            WindowClose {
                id: u8 =,
            }
            // WindowOpen tells the client to open the inventory window of the given
            // type. The ID is used to reference the instance of the window in
            // other packets.
            WindowOpen {
                id: u8 =,
                ty: String =,
                title: format::Component =,
                slot_count: u8 =,
                entity_id: i32 = when(|p: &WindowOpen| p.ty == "EntityHorse"),
            }
            // WindowItems sets every item in a window.
            WindowItems {
                id: u8 =,
                items: LenPrefixed<i16, Option<item::Stack>> =,
            }
            // WindowProperty changes the value of a property of a window. Properties
            // vary depending on the window type.
            WindowProperty {
                id: u8 =,
                property: i16 =,
                value: i16 =,
            }
            // WindowSetSlot changes an itemstack in one of the slots in a window.
            WindowSetSlot {
                id: u8 =,
                property: i16 =,
                item: Option<item::Stack> =,
            }
            // SetCooldown disables a set item (by id) for the set number of ticks
            SetCooldown {
                item_id: VarInt =,
                ticks: VarInt =,
            }
            // PluginMessageClientbound is used for custom messages between the client
            // and server. This is mainly for plugins/mods but vanilla has a few channels
            // registered too.
            PluginMessageClientbound {
                channel: String =,
                data: Vec<u8> =,
            }
            // Plays a sound by name on the client
            NamedSoundEffect {
                name: String =,
                category: VarInt =,
                x: i32 =,
                y: i32 =,
                z: i32 =,
                volume: f32 =,
                pitch: u8 =,
            }
            // Disconnect causes the client to disconnect displaying the passed reason.
            Disconnect {
                reason: format::Component =,
            }
            // EntityAction causes an entity to preform an action based on the passed
            // id.
            EntityAction {
                entity_id: i32 =,
                action_id: u8 =,
            }
            // Explosion is sent when an explosion is triggered (tnt, creeper etc).
            // This plays the effect and removes the effected blocks.
            Explosion {
                x: f32 =,
                y: f32 =,
                z: f32 =,
                radius: f32 =,
                records: LenPrefixed<i32, packet::ExplosionRecord> =,
                velocity_x: f32 =,
                velocity_y: f32 =,
                velocity_z: f32 =,
            }
            // ChunkUnload tells the client to unload the chunk at the specified
            // position.
            ChunkUnload {
                x: i32 =,
                z: i32 =,
            }
            // ChangeGameState is used to modify the game's state like gamemode or
            // weather.
            ChangeGameState {
                reason: u8 =,
                value: f32 =,
            }
            // KeepAliveClientbound is sent by a server to check if the
            // client is still responding and keep the connection open.
            // The client should reply with the KeepAliveServerbound
            // packet setting ID to the same as this one.
            KeepAliveClientbound {
                id: VarInt =,
            }
            // ChunkData sends or updates a single chunk on the client. If New is set
            // then biome data should be sent too.
            ChunkData {
                chunk_x: i32 =,
                chunk_z: i32 =,
                new: bool =,
                bitmask: VarInt =,
                data: LenPrefixedBytes<VarInt> =,
            }
            // Effect plays a sound effect or particle at the target location with the
            // volume (of sounds) being relative to the player's position unless
            // DisableRelative is set to true.
            Effect {
                effect_id: i32 =,
                location: types::Position =,
                data: i32 =,
                disable_relative: bool =,
            }
            // Particle spawns particles at the target location with the various
            // modifiers.
            Particle {
                particle_id: i32 =,
                long_distance: bool =,
                x: f32 =,
                y: f32 =,
                z: f32 =,
                offset_x: f32 =,
                offset_y: f32 =,
                offset_z: f32 =,
                speed: f32 =,
                count: i32 =,
                data1: VarInt = when(|p: &Particle| p.particle_id == 36 || p.particle_id == 37 || p.particle_id == 38),
                data2: VarInt = when(|p: &Particle| p.particle_id == 36),
            }
            // JoinGame is sent after completing the login process. This
            // sets the initial state for the client.
            JoinGame {
                // The entity id the client will be referenced by
                entity_id: i32 =,
                // The starting gamemode of the client
                gamemode: u8 =,
                // The dimension the client is starting in
                dimension: i8 =,
                // The difficuilty setting for the server
                difficulty: u8 =,
                // The max number of players on the server
                max_players: u8 =,
                // The level type of the server
                level_type: String =,
                // Whether the client should reduce the amount of debug
                // information it displays in F3 mode
                reduced_debug_info: bool =,
            }
            // Maps updates a single map's contents
            Maps {
                item_damage: VarInt =,
                scale: i8 =,
                tracking_position: bool =,
                icons: LenPrefixed<VarInt, packet::MapIcon> =,
                columns: u8 =,
                rows: Option<u8> = when(|p: &Maps| p.columns > 0),
                x: Option<u8> = when(|p: &Maps| p.columns > 0),
                z: Option<u8> = when(|p: &Maps| p.columns > 0),
                data: Option<LenPrefixedBytes<VarInt>> = when(|p: &Maps| p.columns > 0),
            }
            // EntityMove moves the entity with the id by the offsets provided.
            EntityMove {
                entity_id: VarInt =,
                delta_x: i16 =,
                delta_y: i16 =,
                delta_z: i16 =,
                on_ground: bool =,
            }
            // EntityLookAndMove is a combination of EntityMove and EntityLook.
            EntityLookAndMove {
                entity_id: VarInt =,
                delta_x: i16 =,
                delta_y: i16 =,
                delta_z: i16 =,
                yaw: i8 =,
                pitch: i8 =,
                on_ground: bool =,
            }
            // EntityLook rotates the entity to the new angles provided.
            EntityLook {
                entity_id: VarInt =,
                yaw: i8 =,
                pitch: i8 =,
                on_ground: bool =,
            }
            // Entity does nothing. It is a result of subclassing used in Minecraft.
            Entity {
                entity_id: VarInt =,
            }
            // Teleports the player's vehicle
            VehicleTeleport {
                x: f64 =,
                y: f64 =,
                z: f64 =,
                yaw: f32 =,
                pitch: f32 =,
            }
            // SignEditorOpen causes the client to open the editor for a sign so that
            // it can write to it. Only sent in vanilla when the player places a sign.
            SignEditorOpen {
                location: types::Position =,
            }
            // PlayerAbilities is used to modify the players current abilities. Flying,
            // creative, god mode etc.
            PlayerAbilities {
                flags: u8 =,
                flying_speed: f32 =,
                walking_speed: f32 =,
            }
            // CombatEvent is used for... you know, I never checked. I have no
            // clue.
            CombatEvent {
                event: VarInt =,
                direction: Option<VarInt> = when(|p: &CombatEvent| p.event.0 == 1),
                player_id: Option<VarInt> = when(|p: &CombatEvent| p.event.0 == 2),
                entity_id: Option<i32> = when(|p: &CombatEvent| p.event.0 == 1 || p.event.0 == 2),
                message: Option<format::Component> = when(|p: &CombatEvent| p.event.0 == 2),
            }
            // PlayerInfo is sent by the server for every player connected to the server
            // to provide skin and username information as well as ping and gamemode info.
            PlayerInfo {
                inner: packet::PlayerInfoData =, // Ew but watcha gonna do?
            }
            // TeleportPlayer is sent to change the player's position. The client is expected
            // to reply to the server with the same positions as contained in this packet
            // otherwise will reject future packets.
            TeleportPlayer {
                x: f64 =,
                y: f64 =,
                z: f64 =,
                yaw: f32 =,
                pitch: f32 =,
                flags: u8 =,
                teleport_id: VarInt =,
            }
            // EntityUsedBed is sent by the server when a player goes to bed.
            EntityUsedBed {
                entity_id: VarInt =,
                location: types::Position =,
            }
            // EntityDestroy destroys the entities with the ids in the provided slice.
            EntityDestroy {
                entity_ids: LenPrefixed<VarInt, VarInt> =,
            }
            // EntityRemoveEffect removes an effect from an entity.
            EntityRemoveEffect {
                entity_id: VarInt =,
                effect_id: i8 =,
            }
            // ResourcePackSend causes the client to check its cache for the requested
            // resource packet and download it if its missing. Once the resource pack
            // is obtained the client will use it.
            ResourcePackSend {
                url: String =,
                hash: String =,
            }
            // Respawn is sent to respawn the player after death or when they move worlds.
            Respawn {
                dimension: i32 =,
                difficulty: u8 =,
                gamemode: u8 =,
                level_type: String =,
            }
            // EntityHeadLook rotates an entity's head to the new angle.
            EntityHeadLook {
                entity_id: VarInt =,
                head_yaw: i8 =,
            }
            // WorldBorder configures the world's border.
            WorldBorder {
                action: VarInt =,
                old_radius: Option<f64> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 1),
                new_radius: Option<f64> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 1 || p.action.0 == 0),
                speed: Option<VarLong> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 1),
                x: Option<f64> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 2),
                z: Option<f64> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 2),
                portal_boundary: Option<VarInt> = when(|p: &WorldBorder| p.action.0 == 3),
                warning_time: Option<VarInt> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 4),
                warning_blocks: Option<VarInt> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 5),
            }
            // Camera causes the client to spectate the entity with the passed id.
            // Use the player's id to de-spectate.
            Camera {
                target_id: VarInt =,
            }
            // SetCurrentHotbarSlot changes the player's currently selected hotbar item.
            SetCurrentHotbarSlot {
                slot: u8 =,
            }
            // ScoreboardDisplay is used to set the display position of a scoreboard.
            ScoreboardDisplay {
                position: u8 =,
                name: String =,
            }
            // EntityMetadata updates the metadata for an entity.
            EntityMetadata {
                entity_id: VarInt =,
                metadata: types::Metadata =,
            }
            // EntityAttach attaches to entities together, either by mounting or leashing.
            // -1 can be used at the EntityID to deattach.
            EntityAttach {
                entity_id: i32 =,
                vehicle: i32 =,
            }
            // EntityVelocity sets the velocity of an entity in 1/8000 of a block
            // per a tick.
            EntityVelocity {
                entity_id: VarInt =,
                velocity_x: i16 =,
                velocity_y: i16 =,
                velocity_z: i16 =,
            }
            // EntityEquipment is sent to display an item on an entity, like a sword
            // or armor. Slot 0 is the held item and slots 1 to 4 are boots, leggings
            // chestplate and helmet respectively.
            EntityEquipment {
                entity_id: VarInt =,
                slot: VarInt =,
                item: Option<item::Stack> =,
            }
            // SetExperience updates the experience bar on the client.
            SetExperience {
                experience_bar: f32 =,
                level: VarInt =,
                total_experience: VarInt =,
            }
            // UpdateHealth is sent by the server to update the player's health and food.
            UpdateHealth {
                health: f32 =,
                food: VarInt =,
                food_saturation: f32 =,
            }
            // ScoreboardObjective creates/updates a scoreboard objective.
            ScoreboardObjective {
                name: String =,
                mode: u8 =,
                value: String = when(|p: &ScoreboardObjective| p.mode == 0 || p.mode == 2),
                ty: String = when(|p: &ScoreboardObjective| p.mode == 0 || p.mode == 2),
            }
            // SetPassengers mounts entities to an entity
            SetPassengers {
                entity_id: VarInt =,
                passengers: LenPrefixed<VarInt, VarInt> =,
            }
            // Teams creates and updates teams
            Teams {
                name: String =,
                mode: u8 =,
                display_name: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                prefix: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                suffix: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                flags: Option<u8> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                name_tag_visibility: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                collision_rule: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                color: Option<u8> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                players: Option<LenPrefixed<VarInt, String>> = when(|p: &Teams| p.mode == 0 || p.mode == 3 || p.mode == 4),
            }
            // UpdateScore is used to update or remove an item from a scoreboard
            // objective.
            UpdateScore {
                name: String =,
                action: u8 =,
                object_name: String =,
                value: Option<VarInt> = when(|p: &UpdateScore| p.action != 1),
            }
            // SpawnPosition is sent to change the player's current spawn point. Currently
            // only used by the client for the compass.
            SpawnPosition {
                location: types::Position =,
            }
            // TimeUpdate is sent to sync the world's time to the client, the client
            // will manually tick the time itself so this doesn't need to sent repeatedly
            // but if the server or client has issues keeping up this can fall out of sync
            // so it is a good idea to send this now and again
            TimeUpdate {
                world_age: i64 =,
                time_of_day: i64 =,
            }
            // Title configures an on-screen title.
            Title {
                action: VarInt =,
                title: Option<format::Component> = when(|p: &Title| p.action.0 == 0),
                sub_title: Option<format::Component> = when(|p: &Title| p.action.0 == 1),
                fade_in: Option<i32> = when(|p: &Title| p.action.0 == 2),
                fade_stay: Option<i32> = when(|p: &Title| p.action.0 == 2),
                fade_out: Option<i32> = when(|p: &Title| p.action.0 == 2),
            }
            // UpdateSign sets or changes the text on a sign.
            UpdateSign {
                location: types::Position =,
                line1: format::Component =,
                line2: format::Component =,
                line3: format::Component =,
                line4: format::Component =,
            }
            // SoundEffect plays the named sound at the target location.
            SoundEffect {
                name: VarInt =,
                category: VarInt =,
                x: i32 =,
                y: i32 =,
                z: i32 =,
                volume: f32 =,
                pitch: u8 =,
            }
            // PlayerListHeaderFooter updates the header/footer of the player list.
            PlayerListHeaderFooter {
                header: format::Component =,
                footer: format::Component =,
            }
            // CollectItem causes the collected item to fly towards the collector. This
            // does not destroy the entity.
            CollectItem {
                collected_entity_id: VarInt =,
                collector_entity_id: VarInt =,
            }
            // EntityTeleport teleports the entity to the target location. This is
            // sent if the entity moves further than EntityMove allows.
            EntityTeleport {
                entity_id: VarInt =,
                x: f64 =,
                y: f64 =,
                z: f64 =,
                yaw: i8 =,
                pitch: i8 =,
                on_ground: bool =,
            }
            // EntityProperties updates the properties for an entity.
            EntityProperties {
                entity_id: VarInt =,
                properties: LenPrefixed<i32, packet::EntityProperty> =,
            }
            // EntityEffect applies a status effect to an entity for a given duration.
            EntityEffect {
                entity_id: VarInt =,
                effect_id: i8 =,
                amplifier: i8 =,
                duration: VarInt =,
                hide_particles: bool =,
            }
       }
    }
    login Login {
        serverbound Serverbound {
            // LoginStart is sent immeditately after switching into the login
            // state. The passed username is used by the server to authenticate
            // the player in online mode.
            LoginStart {
                username: String =,
            }
            // EncryptionResponse is sent as a reply to EncryptionRequest. All
            // packets following this one must be encrypted with AES/CFB8
            // encryption.
            EncryptionResponse {
                // The key for the AES/CFB8 cipher encrypted with the
                // public key
                shared_secret: LenPrefixedBytes<VarInt> =,
                // The verify token from the request encrypted with the
                // public key
                verify_token: LenPrefixedBytes<VarInt> =,
            }
        }
        clientbound Clientbound {
            // LoginDisconnect is sent by the server if there was any issues
            // authenticating the player during login or the general server
            // issues (e.g. too many players).
            LoginDisconnect {
                reason: format::Component =,
            }
            // EncryptionRequest is sent by the server if the server is in
            // online mode. If it is not sent then its assumed the server is
            // in offline mode.
            EncryptionRequest {
                // Generally empty, left in from legacy auth
                // but is still used by the client if provided
                server_id: String =,
                // A RSA Public key serialized in x.509 PRIX format
                public_key: LenPrefixedBytes<VarInt> =,
                // Token used by the server to verify encryption is working
                // correctly
                verify_token: LenPrefixedBytes<VarInt> =,
            }
            // LoginSuccess is sent by the server if the player successfully
            // authenicates with the session servers (online mode) or straight
            // after LoginStart (offline mode).
            LoginSuccess {
                // String encoding of a uuid (with hyphens)
                uuid: String =,
                username: String =,
            }
            // SetInitialCompression sets the compression threshold during the
            // login state.
            SetInitialCompression {
                // Threshold where a packet should be sent compressed
                threshold: VarInt =,
            }
        }
    }
    status Status {
        serverbound Serverbound {
            // StatusRequest is sent by the client instantly after
            // switching to the Status protocol state and is used
            // to signal the server to send a StatusResponse to the
            // client
            StatusRequest {
                empty: () =,
            }
            // StatusPing is sent by the client after recieving a
            // StatusResponse. The client uses the time from sending
            // the ping until the time of recieving a pong to measure
            // the latency between the client and the server.
            StatusPing {
                ping: i64 =,
            }
        }
        clientbound Clientbound {
            // StatusResponse is sent as a reply to a StatusRequest.
            // The Status should contain a json encoded structure with
            // version information, a player sample, a description/MOTD
            // and optionally a favicon.
            //
            // The structure is as follows
            //     {
            //         "version": {
            //             "name": "1.8.3",
            //             "protocol": 47,
            //         },
            //         "players": {
            //             "max": 20,
            //             "online": 1,
            //             "sample": [
            //                 {"name": "Thinkofdeath", "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20"}
            //             ]
            //         },
            //         "description": "Hello world",
            //         "favicon": "data:image/png;base64,<data>"
            //     }
            StatusResponse {
                status: String =,
            }
            // StatusPong is sent as a reply to a StatusPing.
            // The Time field should be exactly the same as the
            // one sent by the client.
            StatusPong {
                ping: i64 =,
            }
       }
    }
);

#[derive(Debug, Default)]
pub struct Statistic {
    pub name: String,
    pub value: VarInt,
}

impl Serializable for Statistic {
    fn read_from(buf: &mut io::Read) -> Result<Self, io::Error> {
        Ok(Statistic {
            name: try!(Serializable::read_from(buf)),
            value: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(self.name.write_to(buf));
        self.value.write_to(buf)
    }
}

#[derive(Debug, Default)]
pub struct BlockChangeRecord {
    pub xz: u8,
    pub y: u8,
    pub block_id: VarInt,
}

impl Serializable for BlockChangeRecord {
    fn read_from(buf: &mut io::Read) -> Result<Self, io::Error> {
        Ok(BlockChangeRecord {
            xz: try!(Serializable::read_from(buf)),
            y: try!(Serializable::read_from(buf)),
            block_id: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(self.xz.write_to(buf));
        try!(self.y.write_to(buf));
        self.block_id.write_to(buf)
    }
}

#[derive(Debug, Default)]
pub struct ExplosionRecord {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl Serializable for ExplosionRecord {
    fn read_from(buf: &mut io::Read) -> Result<Self, io::Error> {
        Ok(ExplosionRecord {
            x: try!(Serializable::read_from(buf)),
            y: try!(Serializable::read_from(buf)),
            z: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(self.x.write_to(buf));
        try!(self.y.write_to(buf));
        self.z.write_to(buf)
    }
}

#[derive(Debug)]
pub struct MapIcon {
    pub direction_type: i8,
    pub x: i8,
    pub z: i8,
}

impl Serializable for MapIcon {
    fn read_from(buf: &mut io::Read) -> Result<Self, io::Error> {
        Ok(MapIcon {
            direction_type: try!(Serializable::read_from(buf)),
            x: try!(Serializable::read_from(buf)),
            z: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(self.direction_type.write_to(buf));
        try!(self.x.write_to(buf));
        self.z.write_to(buf)
    }
}

impl Default for MapIcon {
    fn default() -> Self {
        MapIcon {
            direction_type: 0,
            x: 0,
            z: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct EntityProperty {
    pub key: String,
    pub value: f64,
    pub modifiers: LenPrefixed<VarInt, PropertyModifier>,
}

impl Serializable for EntityProperty {
    fn read_from(buf: &mut io::Read) -> Result<Self, io::Error> {
        Ok(EntityProperty {
            key: try!(Serializable::read_from(buf)),
            value: try!(Serializable::read_from(buf)),
            modifiers: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(self.key.write_to(buf));
        try!(self.value.write_to(buf));
        self.modifiers.write_to(buf)
    }
}

#[derive(Debug, Default)]
pub struct PropertyModifier {
    pub uuid: UUID,
    pub amount: f64,
    pub operation: i8,
}

impl Serializable for PropertyModifier {
    fn read_from(buf: &mut io::Read) -> Result<Self, io::Error> {
        Ok(PropertyModifier {
            uuid: try!(Serializable::read_from(buf)),
            amount: try!(Serializable::read_from(buf)),
            operation: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to(&self, buf: &mut io::Write) -> Result<(), io::Error> {
        try!(self.uuid.write_to(buf));
        try!(self.amount.write_to(buf));
        self.operation.write_to(buf)
    }
}

#[derive(Debug)]
pub struct PlayerInfoData {
    pub action: VarInt,
    pub players: Vec<PlayerDetail>,
}

impl Serializable for PlayerInfoData {
    fn read_from(buf: &mut io::Read) -> Result<Self, io::Error> {
        let mut m = PlayerInfoData {
            action: try!(Serializable::read_from(buf)),
            players: Vec::new(),
        };
        let len = try!(VarInt::read_from(buf));
        for _ in 0..len.0 {
            let uuid = try!(UUID::read_from(buf));
            match m.action.0 {
                0 => {
                    let name = try!(String::read_from(buf));
                    let mut props = Vec::new();
                    let plen = try!(VarInt::read_from(buf)).0;
                    for _ in 0..plen {
                        let mut prop = PlayerProperty {
                            name: try!(String::read_from(buf)),
                            value: try!(String::read_from(buf)),
                            signature: Default::default(),
                        };
                        if try!(bool::read_from(buf)) {
                            prop.signature = Some(try!(String::read_from(buf)));
                        }
                        props.push(prop);
                    }
                    let p = PlayerDetail::Add {
                        uuid: uuid,
                        name: name,
                        properties: props,
                        gamemode: try!(Serializable::read_from(buf)),
                        ping: try!(Serializable::read_from(buf)),
                        display: {
                            if try!(bool::read_from(buf)) {
                                Some(try!(Serializable::read_from(buf)))
                            } else {
                                None
                            }
                        },
                    };
                    m.players.push(p);
                }
                1 => {
                    m.players.push(PlayerDetail::UpdateGamemode {
                        uuid: uuid,
                        gamemode: try!(Serializable::read_from(buf)),
                    })
                }
                2 => {
                    m.players.push(PlayerDetail::UpdateLatency {
                        uuid: uuid,
                        ping: try!(Serializable::read_from(buf)),
                    })
                }
                3 => {
                    m.players.push(PlayerDetail::UpdateDisplayName {
                        uuid: uuid,
                        display: {
                            if try!(bool::read_from(buf)) {
                                Some(try!(Serializable::read_from(buf)))
                            } else {
                                None
                            }
                        },
                    })
                }
                4 => {
                    m.players.push(PlayerDetail::Remove { uuid: uuid })
                }
                _ => panic!(),
            }
        }
        Ok(m)
    }

    fn write_to(&self, _: &mut io::Write) -> Result<(), io::Error> {
        unimplemented!() // I'm lazy
    }
}

impl Default for PlayerInfoData {
    fn default() -> Self {
        PlayerInfoData {
            action: VarInt(0),
            players: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum PlayerDetail {
    Add {
        uuid: UUID,
        name: String,
        properties: Vec<PlayerProperty>,
        gamemode: VarInt,
        ping: VarInt,
        display: Option<format::Component>,
    },
    UpdateGamemode {
        uuid: UUID,
        gamemode: VarInt,
    },
    UpdateLatency {
        uuid: UUID,
        ping: VarInt,
    },
    UpdateDisplayName {
        uuid: UUID,
        display: Option<format::Component>,
    },
    Remove {
        uuid: UUID,
    },
}

#[derive(Debug)]
pub struct PlayerProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}
