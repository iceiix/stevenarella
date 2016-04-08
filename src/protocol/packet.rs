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
            /// Handshake is the first packet sent in the protocol.
            /// Its used for deciding if the request is a client
            /// is requesting status information about the server
            /// (MOTD, players etc) or trying to login to the server.
            ///
            /// The host and port fields are not used by the vanilla
            /// server but are there for virtual server hosting to
            /// be able to redirect a client to a target server with
            /// a single address + port.
            ///
            /// Some modified servers/proxies use the handshake field
            /// differently, packing information into the field other
            /// than the hostname due to the protocol not providing
            /// any system for custom information to be transfered
            /// by the client to the server until after login.
            packet Handshake {
                /// The protocol version of the connecting client
                field protocol_version: VarInt =,
                /// The hostname the client connected to
                field host: String =,
                /// The port the client connected to
                field port: u16 =,
                /// The next protocol state the client wants
                field next: VarInt =,
            }
        }
        clientbound Clientbound {
        }
    }
    play Play {
        serverbound Serverbound {
            /// TeleportConfirm is sent by the client as a reply to a telport from
            /// the server.
            packet TeleportConfirm {
                field teleport_id: VarInt =,
            }
            /// TabComplete is sent by the client when the client presses tab in
            /// the chat box.
            packet TabComplete {
                field text: String =,
                field assume_command: bool =,
                field has_target: bool =,
                field target: Option<Position> = when(|p: &TabComplete| p.has_target),
            }
            /// ChatMessage is sent by the client when it sends a chat message or
            /// executes a command (prefixed by '/').
            packet ChatMessage {
                field message: String =,
            }
            /// ClientStatus is sent to update the client's status
            packet ClientStatus {
                field action_id: VarInt =,
            }
            /// ClientSettings is sent by the client to update its current settings.
            packet ClientSettings {
                field locale: String =,
                field view_distance: u8 =,
                field chat_mode: VarInt =,
                field chat_colors: bool =,
                field displayed_skin_parts: u8 =,
                field main_hand: VarInt =,
            }
            /// ConfirmTransactionServerbound is a reply to ConfirmTransaction.
            packet ConfirmTransactionServerbound {
                field id: u8 =,
                field action_number: i16 =,
                field accepted: bool =,
            }
            /// EnchantItem is sent when the client enchants an item.
            packet EnchantItem {
                field id: u8 =,
                field enchantment: u8 =,
            }
            /// ClickWindow is sent when the client clicks in a window.
            packet ClickWindow {
                field id: u8 =,
                field slot: i16 =,
                field button: u8 =,
                field action_number: u16 =,
                field mode: VarInt =,
                field clicked_item: Option<item::Stack> =,
            }
            /// CloseWindow is sent when the client closes a window.
            packet CloseWindow {
                field id: u8 =,
            }
            /// PluginMessageServerbound is used for custom messages between the client
            /// and server. This is mainly for plugins/mods but vanilla has a few channels
            /// registered too.
            packet PluginMessageServerbound {
                field channel: String =,
                field data: Vec<u8> =,
            }
            /// UseEntity is sent when the user interacts (right clicks) or attacks
            /// (left clicks) an entity.
            packet UseEntity {
                field target_id: VarInt =,
                field ty: VarInt =,
                field target_x: f32 = when(|p: &UseEntity| p.ty.0 == 2),
                field target_y: f32 = when(|p: &UseEntity| p.ty.0 == 2),
                field target_z: f32 = when(|p: &UseEntity| p.ty.0 == 2),
                field hand: VarInt = when(|p: &UseEntity| p.ty.0 == 0 || p.ty.0 == 2),
            }
            /// KeepAliveServerbound is sent by a client as a response to a
            /// KeepAliveClientbound. If the client doesn't reply the server
            /// may disconnect the client.
            packet KeepAliveServerbound {
                field id: VarInt =,
            }
            /// PlayerPosition is used to update the player's position.
            packet PlayerPosition {
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field on_ground: bool =,
            }
            /// PlayerPositionLook is a combination of PlayerPosition and
            /// PlayerLook.
            packet PlayerPositionLook {
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: f32 =,
                field pitch: f32 =,
                field on_ground: bool =,
            }
            /// PlayerLook is used to update the player's rotation.
            packet PlayerLook {
                field yaw: f32 =,
                field pitch: f32 =,
                field on_ground: bool =,
            }
            /// Player is used to update whether the player is on the ground or not.
            packet Player {
                field on_ground: bool =,
            }
            /// Sent by the client when in a vehicle instead of the normal move packet.
            packet VehicleMove {
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: f32 =,
                field pitch: f32 =,
            }
            /// TODO: Document
            packet SteerBoat {
                field unknown: bool =,
                field unknown2: bool =,
            }
            /// ClientAbilities is used to modify the players current abilities.
            /// Currently flying is the only one
            packet ClientAbilities {
                field flags: u8 =,
                field flying_speed: f32 =,
                field walking_speed: f32 =,
            }
            /// PlayerDigging is sent when the client starts/stops digging a block.
            /// It also can be sent for droppping items and eating/shooting.
            packet PlayerDigging {
                field status: VarInt =,
                field location: Position =,
                field face: u8 =,
            }
            /// PlayerAction is sent when a player preforms various actions.
            packet PlayerAction {
                field entity_id: VarInt =,
                field action_id: VarInt =,
                field jump_boost: VarInt =,
            }
            /// SteerVehicle is sent by the client when steers or preforms an action
            /// on a vehicle.
            packet SteerVehicle {
                field sideways: f32 =,
                field forward: f32 =,
                field flags: u8 =,
            }
            /// ResourcePackStatus informs the server of the client's current progress
            /// in activating the requested resource pack
            packet ResourcePackStatus {
                field hash: String =,
                field result: VarInt =,
            }
            /// HeldItemChange is sent when the player changes the currently active
            /// hotbar slot.
            packet HeldItemChange {
                field slot: i16 =,
            }
            /// CreativeInventoryAction is sent when the client clicks in the creative
            /// inventory. This is used to spawn items in creative.
            packet CreativeInventoryAction {
                field slot: i16 =,
                field clicked_item: Option<item::Stack> =,
            }
            /// SetSign sets the text on a sign after placing it.
            packet SetSign {
                field location: Position =,
                field line1: String =,
                field line2: String =,
                field line3: String =,
                field line4: String =,
            }
            /// ArmSwing is sent by the client when the player left clicks (to swing their
            /// arm).
            packet ArmSwing {
                field hand: VarInt =,
            }
            /// SpectateTeleport is sent by clients in spectator mode to teleport to a player.
            packet SpectateTeleport {
                field target: UUID =,
            }
            /// PlayerBlockPlacement is sent when the client tries to place a block.
            packet PlayerBlockPlacement {
                field location: Position =,
                field face: VarInt =,
                field hand: VarInt =,
                field cursor_x: u8 =,
                field cursor_y: u8 =,
                field cursor_z: u8 =,
            }
            /// UseItem is sent when the client tries to use an item.
            packet UseItem {
                field hand: VarInt =,
            }
        }
        clientbound Clientbound {
            /// SpawnObject is used to spawn an object or vehicle into the world when it
            /// is in range of the client.
            packet SpawnObject {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field ty: u8 =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field pitch: i8 =,
                field yaw: i8 =,
                field data: i32 =,
                field velocity_x: i16 =,
                field velocity_y: i16 =,
                field velocity_z: i16 =,
            }
            /// SpawnExperienceOrb spawns a single experience orb into the world when
            /// it is in range of the client. The count controls the amount of experience
            /// gained when collected.
            packet SpawnExperienceOrb {
                field entity_id: VarInt =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field count: i16 =,
            }
            /// SpawnGlobalEntity spawns an entity which is visible from anywhere in the
            /// world. Currently only used for lightning.
            packet SpawnGlobalEntity {
                field entity_id: VarInt =,
                field ty: u8 =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
            }
            /// SpawnMob is used to spawn a living entity into the world when it is in
            /// range of the client.
            packet SpawnMob {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field ty: u8 =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: i8 =,
                field pitch: i8 =,
                field head_pitch: i8 =,
                field velocity_x: i16 =,
                field velocity_y: i16 =,
                field velocity_z: i16 =,
                field metadata: types::Metadata =,
            }
            /// SpawnPainting spawns a painting into the world when it is in range of
            /// the client. The title effects the size and the texture of the painting.
            packet SpawnPainting {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field title: String =,
                field location: Position =,
                field direction: u8 =,
            }
            /// SpawnPlayer is used to spawn a player when they are in range of the client.
            /// This packet alone isn't enough to display the player as the skin and username
            /// information is in the player information packet.
            packet SpawnPlayer {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: i8 =,
                field pitch: i8 =,
                field metadata: types::Metadata =,
            }
            /// Animation is sent by the server to play an animation on a specific entity.
            packet Animation {
                field entity_id: VarInt =,
                field animation_id: u8 =,
            }
            /// Statistics is used to update the statistics screen for the client.
            packet Statistics {
                field statistices: LenPrefixed<VarInt, packet::Statistic> =,
            }
            /// BlockBreakAnimation is used to create and update the block breaking
            /// animation played when a player starts digging a block.
            packet BlockBreakAnimation {
                field entity_id: VarInt =,
                field location: Position =,
                field stage: i8 =,
            }
            /// UpdateBlockEntity updates the nbt tag of a block entity in the
            /// world.
            packet UpdateBlockEntity {
                field location: Position =,
                field action: u8 =,
                field nbt: Option<nbt::NamedTag> =,
            }
            /// BlockAction triggers different actions depending on the target block.
            packet BlockAction {
                field location: Position =,
                field byte1: u8 =,
                field byte2: u8 =,
                field block_type: VarInt =,
            }
            /// BlockChange is used to update a single block on the client.
            packet BlockChange {
                field location: Position =,
                field block_id: VarInt =,
            }
            /// BossBar displays and/or changes a boss bar that is displayed on the
            /// top of the client's screen. This is normally used for bosses such as
            /// the ender dragon or the wither.
            packet BossBar {
                field uuid: UUID =,
                field action: VarInt =,
                field title: format::Component = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 3),
                field health: f32 = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 2),
                field color: VarInt = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 4),
                field style: VarInt = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 4),
                field flags: u8 = when(|p: &BossBar| p.action.0 == 0 || p.action.0 == 5),
            }
            /// ServerDifficulty changes the displayed difficulty in the client's menu
            /// as well as some ui changes for hardcore.
            packet ServerDifficulty {
                field difficulty: u8 =,
            }
            /// TabCompleteReply is sent as a reply to a tab completion request.
            /// The matches should be possible completions for the command/chat the
            /// player sent.
            packet TabCompleteReply {
                field matches: LenPrefixed<VarInt, String> =,
            }
            /// ServerMessage is a message sent by the server. It could be from a player
            /// or just a system message. The Type field controls the location the
            /// message is displayed at and when the message is displayed.
            packet ServerMessage {
                field message: format::Component =,
                /// 0 - Chat message, 1 - System message, 2 - Action bar message
                field position: u8 =,
            }
            /// MultiBlockChange is used to update a batch of blocks in a single packet.
            packet MultiBlockChange {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field records: LenPrefixed<VarInt, packet::BlockChangeRecord> =,
            }
            /// ConfirmTransaction notifies the client whether a transaction was successful
            /// or failed (e.g. due to lag).
            packet ConfirmTransaction {
                field id: u8 =,
                field action_number: i16 =,
                field accepted: bool =,
            }
            /// WindowClose forces the client to close the window with the given id,
            /// e.g. a chest getting destroyed.
            packet WindowClose {
                field id: u8 =,
            }
            /// WindowOpen tells the client to open the inventory window of the given
            /// type. The ID is used to reference the instance of the window in
            /// other packets.
            packet WindowOpen {
                field id: u8 =,
                field ty: String =,
                field title: format::Component =,
                field slot_count: u8 =,
                field entity_id: i32 = when(|p: &WindowOpen| p.ty == "EntityHorse"),
            }
            /// WindowItems sets every item in a window.
            packet WindowItems {
                field id: u8 =,
                field items: LenPrefixed<i16, Option<item::Stack>> =,
            }
            /// WindowProperty changes the value of a property of a window. Properties
            /// vary depending on the window type.
            packet WindowProperty {
                field id: u8 =,
                field property: i16 =,
                field value: i16 =,
            }
            /// WindowSetSlot changes an itemstack in one of the slots in a window.
            packet WindowSetSlot {
                field id: u8 =,
                field property: i16 =,
                field item: Option<item::Stack> =,
            }
            /// SetCooldown disables a set item (by id) for the set number of ticks
            packet SetCooldown {
                field item_id: VarInt =,
                field ticks: VarInt =,
            }
            /// PluginMessageClientbound is used for custom messages between the client
            /// and server. This is mainly for plugins/mods but vanilla has a few channels
            /// registered too.
            packet PluginMessageClientbound {
                field channel: String =,
                field data: Vec<u8> =,
            }
            /// Plays a sound by name on the client
            packet NamedSoundEffect {
                field name: String =,
                field category: VarInt =,
                field x: i32 =,
                field y: i32 =,
                field z: i32 =,
                field volume: f32 =,
                field pitch: u8 =,
            }
            /// Disconnect causes the client to disconnect displaying the passed reason.
            packet Disconnect {
                field reason: format::Component =,
            }
            /// EntityAction causes an entity to preform an action based on the passed
            /// id.
            packet EntityAction {
                field entity_id: i32 =,
                field action_id: u8 =,
            }
            /// Explosion is sent when an explosion is triggered (tnt, creeper etc).
            /// This plays the effect and removes the effected blocks.
            packet Explosion {
                field x: f32 =,
                field y: f32 =,
                field z: f32 =,
                field radius: f32 =,
                field records: LenPrefixed<i32, packet::ExplosionRecord> =,
                field velocity_x: f32 =,
                field velocity_y: f32 =,
                field velocity_z: f32 =,
            }
            /// ChunkUnload tells the client to unload the chunk at the specified
            /// position.
            packet ChunkUnload {
                field x: i32 =,
                field z: i32 =,
            }
            /// ChangeGameState is used to modify the game's state like gamemode or
            /// weather.
            packet ChangeGameState {
                field reason: u8 =,
                field value: f32 =,
            }
            /// KeepAliveClientbound is sent by a server to check if the
            /// client is still responding and keep the connection open.
            /// The client should reply with the KeepAliveServerbound
            /// packet setting ID to the same as this one.
            packet KeepAliveClientbound {
                field id: VarInt =,
            }
            /// ChunkData sends or updates a single chunk on the client. If New is set
            /// then biome data should be sent too.
            packet ChunkData {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field new: bool =,
                field bitmask: VarInt =,
                field data: LenPrefixedBytes<VarInt> =,
            }
            /// Effect plays a sound effect or particle at the target location with the
            /// volume (of sounds) being relative to the player's position unless
            /// DisableRelative is set to true.
            packet Effect {
                field effect_id: i32 =,
                field location: Position =,
                field data: i32 =,
                field disable_relative: bool =,
            }
            /// Particle spawns particles at the target location with the various
            /// modifiers.
            packet Particle {
                field particle_id: i32 =,
                field long_distance: bool =,
                field x: f32 =,
                field y: f32 =,
                field z: f32 =,
                field offset_x: f32 =,
                field offset_y: f32 =,
                field offset_z: f32 =,
                field speed: f32 =,
                field count: i32 =,
                field data1: VarInt = when(|p: &Particle| p.particle_id == 36 || p.particle_id == 37 || p.particle_id == 38),
                field data2: VarInt = when(|p: &Particle| p.particle_id == 36),
            }
            /// JoinGame is sent after completing the login process. This
            /// sets the initial state for the client.
            packet JoinGame {
                /// The entity id the client will be referenced by
                field entity_id: i32 =,
                /// The starting gamemode of the client
                field gamemode: u8 =,
                /// The dimension the client is starting in
                field dimension: i32 =,
                /// The difficuilty setting for the server
                field difficulty: u8 =,
                /// The max number of players on the server
                field max_players: u8 =,
                /// The level type of the server
                field level_type: String =,
                /// Whether the client should reduce the amount of debug
                /// information it displays in F3 mode
                field reduced_debug_info: bool =,
            }
            /// Maps updates a single map's contents
            packet Maps {
                field item_damage: VarInt =,
                field scale: i8 =,
                field tracking_position: bool =,
                field icons: LenPrefixed<VarInt, packet::MapIcon> =,
                field columns: u8 =,
                field rows: Option<u8> = when(|p: &Maps| p.columns > 0),
                field x: Option<u8> = when(|p: &Maps| p.columns > 0),
                field z: Option<u8> = when(|p: &Maps| p.columns > 0),
                field data: Option<LenPrefixedBytes<VarInt>> = when(|p: &Maps| p.columns > 0),
            }
            /// EntityMove moves the entity with the id by the offsets provided.
            packet EntityMove {
                field entity_id: VarInt =,
                field delta_x: i16 =,
                field delta_y: i16 =,
                field delta_z: i16 =,
                field on_ground: bool =,
            }
            /// EntityLookAndMove is a combination of EntityMove and EntityLook.
            packet EntityLookAndMove {
                field entity_id: VarInt =,
                field delta_x: i16 =,
                field delta_y: i16 =,
                field delta_z: i16 =,
                field yaw: i8 =,
                field pitch: i8 =,
                field on_ground: bool =,
            }
            /// EntityLook rotates the entity to the new angles provided.
            packet EntityLook {
                field entity_id: VarInt =,
                field yaw: i8 =,
                field pitch: i8 =,
                field on_ground: bool =,
            }
            /// Entity does nothing. It is a result of subclassing used in Minecraft.
            packet Entity {
                field entity_id: VarInt =,
            }
            /// Teleports the player's vehicle
            packet VehicleTeleport {
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: f32 =,
                field pitch: f32 =,
            }
            /// SignEditorOpen causes the client to open the editor for a sign so that
            /// it can write to it. Only sent in vanilla when the player places a sign.
            packet SignEditorOpen {
                field location: Position =,
            }
            /// PlayerAbilities is used to modify the players current abilities. Flying,
            /// creative, god mode etc.
            packet PlayerAbilities {
                field flags: u8 =,
                field flying_speed: f32 =,
                field walking_speed: f32 =,
            }
            /// CombatEvent is used for... you know, I never checked. I have no
            /// clue.
            packet CombatEvent {
                field event: VarInt =,
                field direction: Option<VarInt> = when(|p: &CombatEvent| p.event.0 == 1),
                field player_id: Option<VarInt> = when(|p: &CombatEvent| p.event.0 == 2),
                field entity_id: Option<i32> = when(|p: &CombatEvent| p.event.0 == 1 || p.event.0 == 2),
                field message: Option<format::Component> = when(|p: &CombatEvent| p.event.0 == 2),
            }
            /// PlayerInfo is sent by the server for every player connected to the server
            /// to provide skin and username information as well as ping and gamemode info.
            packet PlayerInfo {
                field inner: packet::PlayerInfoData =,
            }
            /// TeleportPlayer is sent to change the player's position. The client is expected
            /// to reply to the server with the same positions as contained in this packet
            /// otherwise will reject future packets.
            packet TeleportPlayer {
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: f32 =,
                field pitch: f32 =,
                field flags: u8 =,
                field teleport_id: VarInt =,
            }
            /// EntityUsedBed is sent by the server when a player goes to bed.
            packet EntityUsedBed {
                field entity_id: VarInt =,
                field location: Position =,
            }
            /// EntityDestroy destroys the entities with the ids in the provided slice.
            packet EntityDestroy {
                field entity_ids: LenPrefixed<VarInt, VarInt> =,
            }
            /// EntityRemoveEffect removes an effect from an entity.
            packet EntityRemoveEffect {
                field entity_id: VarInt =,
                field effect_id: i8 =,
            }
            /// ResourcePackSend causes the client to check its cache for the requested
            /// resource packet and download it if its missing. Once the resource pack
            /// is obtained the client will use it.
            packet ResourcePackSend {
                field url: String =,
                field hash: String =,
            }
            /// Respawn is sent to respawn the player after death or when they move worlds.
            packet Respawn {
                field dimension: i32 =,
                field difficulty: u8 =,
                field gamemode: u8 =,
                field level_type: String =,
            }
            /// EntityHeadLook rotates an entity's head to the new angle.
            packet EntityHeadLook {
                field entity_id: VarInt =,
                field head_yaw: i8 =,
            }
            /// WorldBorder configures the world's border.
            packet WorldBorder {
                field action: VarInt =,
                field old_radius: Option<f64> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 1),
                field new_radius: Option<f64> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 1 || p.action.0 == 0),
                field speed: Option<VarLong> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 1),
                field x: Option<f64> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 2),
                field z: Option<f64> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 2),
                field portal_boundary: Option<VarInt> = when(|p: &WorldBorder| p.action.0 == 3),
                field warning_time: Option<VarInt> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 4),
                field warning_blocks: Option<VarInt> = when(|p: &WorldBorder| p.action.0 == 3 || p.action.0 == 5),
            }
            /// Camera causes the client to spectate the entity with the passed id.
            /// Use the player's id to de-spectate.
            packet Camera {
                field target_id: VarInt =,
            }
            /// SetCurrentHotbarSlot changes the player's currently selected hotbar item.
            packet SetCurrentHotbarSlot {
                field slot: u8 =,
            }
            /// ScoreboardDisplay is used to set the display position of a scoreboard.
            packet ScoreboardDisplay {
                field position: u8 =,
                field name: String =,
            }
            /// EntityMetadata updates the metadata for an entity.
            packet EntityMetadata {
                field entity_id: VarInt =,
                field metadata: types::Metadata =,
            }
            /// EntityAttach attaches to entities together, either by mounting or leashing.
            /// -1 can be used at the EntityID to deattach.
            packet EntityAttach {
                field entity_id: i32 =,
                field vehicle: i32 =,
            }
            /// EntityVelocity sets the velocity of an entity in 1/8000 of a block
            /// per a tick.
            packet EntityVelocity {
                field entity_id: VarInt =,
                field velocity_x: i16 =,
                field velocity_y: i16 =,
                field velocity_z: i16 =,
            }
            /// EntityEquipment is sent to display an item on an entity, like a sword
            /// or armor. Slot 0 is the held item and slots 1 to 4 are boots, leggings
            /// chestplate and helmet respectively.
            packet EntityEquipment {
                field entity_id: VarInt =,
                field slot: VarInt =,
                field item: Option<item::Stack> =,
            }
            /// SetExperience updates the experience bar on the client.
            packet SetExperience {
                field experience_bar: f32 =,
                field level: VarInt =,
                field total_experience: VarInt =,
            }
            /// UpdateHealth is sent by the server to update the player's health and food.
            packet UpdateHealth {
                field health: f32 =,
                field food: VarInt =,
                field food_saturation: f32 =,
            }
            /// ScoreboardObjective creates/updates a scoreboard objective.
            packet ScoreboardObjective {
                field name: String =,
                field mode: u8 =,
                field value: String = when(|p: &ScoreboardObjective| p.mode == 0 || p.mode == 2),
                field ty: String = when(|p: &ScoreboardObjective| p.mode == 0 || p.mode == 2),
            }
            /// SetPassengers mounts entities to an entity
            packet SetPassengers {
                field entity_id: VarInt =,
                field passengers: LenPrefixed<VarInt, VarInt> =,
            }
            /// Teams creates and updates teams
            packet Teams {
                field name: String =,
                field mode: u8 =,
                field display_name: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                field prefix: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                field suffix: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                field flags: Option<u8> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                field name_tag_visibility: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                field collision_rule: Option<String> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                field color: Option<u8> = when(|p: &Teams| p.mode == 0 || p.mode == 2),
                field players: Option<LenPrefixed<VarInt, String>> = when(|p: &Teams| p.mode == 0 || p.mode == 3 || p.mode == 4),
            }
            /// UpdateScore is used to update or remove an item from a scoreboard
            /// objective.
            packet UpdateScore {
                field name: String =,
                field action: u8 =,
                field object_name: String =,
                field value: Option<VarInt> = when(|p: &UpdateScore| p.action != 1),
            }
            /// SpawnPosition is sent to change the player's current spawn point. Currently
            /// only used by the client for the compass.
            packet SpawnPosition {
                field location: Position =,
            }
            /// TimeUpdate is sent to sync the world's time to the client, the client
            /// will manually tick the time itself so this doesn't need to sent repeatedly
            /// but if the server or client has issues keeping up this can fall out of sync
            /// so it is a good idea to send this now and again
            packet TimeUpdate {
                field world_age: i64 =,
                field time_of_day: i64 =,
            }
            /// Title configures an on-screen title.
            packet Title {
                field action: VarInt =,
                field title: Option<format::Component> = when(|p: &Title| p.action.0 == 0),
                field sub_title: Option<format::Component> = when(|p: &Title| p.action.0 == 1),
                field fade_in: Option<i32> = when(|p: &Title| p.action.0 == 2),
                field fade_stay: Option<i32> = when(|p: &Title| p.action.0 == 2),
                field fade_out: Option<i32> = when(|p: &Title| p.action.0 == 2),
            }
            /// UpdateSign sets or changes the text on a sign.
            packet UpdateSign {
                field location: Position =,
                field line1: format::Component =,
                field line2: format::Component =,
                field line3: format::Component =,
                field line4: format::Component =,
            }
            /// SoundEffect plays the named sound at the target location.
            packet SoundEffect {
                field name: VarInt =,
                field category: VarInt =,
                field x: i32 =,
                field y: i32 =,
                field z: i32 =,
                field volume: f32 =,
                field pitch: u8 =,
            }
            /// PlayerListHeaderFooter updates the header/footer of the player list.
            packet PlayerListHeaderFooter {
                field header: format::Component =,
                field footer: format::Component =,
            }
            /// CollectItem causes the collected item to fly towards the collector. This
            /// does not destroy the entity.
            packet CollectItem {
                field collected_entity_id: VarInt =,
                field collector_entity_id: VarInt =,
            }
            /// EntityTeleport teleports the entity to the target location. This is
            /// sent if the entity moves further than EntityMove allows.
            packet EntityTeleport {
                field entity_id: VarInt =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: i8 =,
                field pitch: i8 =,
                field on_ground: bool =,
            }
            /// EntityProperties updates the properties for an entity.
            packet EntityProperties {
                field entity_id: VarInt =,
                field properties: LenPrefixed<i32, packet::EntityProperty> =,
            }
            /// EntityEffect applies a status effect to an entity for a given duration.
            packet EntityEffect {
                field entity_id: VarInt =,
                field effect_id: i8 =,
                field amplifier: i8 =,
                field duration: VarInt =,
                field hide_particles: bool =,
            }
       }
    }
    login Login {
        serverbound Serverbound {
            /// LoginStart is sent immeditately after switching into the login
            /// state. The passed username is used by the server to authenticate
            /// the player in online mode.
            packet LoginStart {
                field username: String =,
            }
            /// EncryptionResponse is sent as a reply to EncryptionRequest. All
            /// packets following this one must be encrypted with AES/CFB8
            /// encryption.
            packet EncryptionResponse {
                /// The key for the AES/CFB8 cipher encrypted with the
                /// public key
                field shared_secret: LenPrefixedBytes<VarInt> =,
                /// The verify token from the request encrypted with the
                /// public key
                field verify_token: LenPrefixedBytes<VarInt> =,
            }
        }
        clientbound Clientbound {
            /// LoginDisconnect is sent by the server if there was any issues
            /// authenticating the player during login or the general server
            /// issues (e.g. too many players).
            packet LoginDisconnect {
                field reason: format::Component =,
            }
            /// EncryptionRequest is sent by the server if the server is in
            /// online mode. If it is not sent then its assumed the server is
            /// in offline mode.
            packet EncryptionRequest {
                /// Generally empty, left in from legacy auth
                /// but is still used by the client if provided
                field server_id: String =,
                /// A RSA Public key serialized in x.509 PRIX format
                field public_key: LenPrefixedBytes<VarInt> =,
                /// Token used by the server to verify encryption is working
                /// correctly
                field verify_token: LenPrefixedBytes<VarInt> =,
            }
            /// LoginSuccess is sent by the server if the player successfully
            /// authenicates with the session servers (online mode) or straight
            /// after LoginStart (offline mode).
            packet LoginSuccess {
                /// String encoding of a uuid (with hyphens)
                field uuid: String =,
                field username: String =,
            }
            /// SetInitialCompression sets the compression threshold during the
            /// login state.
            packet SetInitialCompression {
                /// Threshold where a packet should be sent compressed
                field threshold: VarInt =,
            }
        }
    }
    status Status {
        serverbound Serverbound {
            /// StatusRequest is sent by the client instantly after
            /// switching to the Status protocol state and is used
            /// to signal the server to send a StatusResponse to the
            /// client
            packet StatusRequest {
                field empty: () =,
            }
            /// StatusPing is sent by the client after recieving a
            /// StatusResponse. The client uses the time from sending
            /// the ping until the time of recieving a pong to measure
            /// the latency between the client and the server.
            packet StatusPing {
                field ping: i64 =,
            }
        }
        clientbound Clientbound {
            /// StatusResponse is sent as a reply to a StatusRequest.
            /// The Status should contain a json encoded structure with
            /// version information, a player sample, a description/MOTD
            /// and optionally a favicon.
            //
            /// The structure is as follows
            ///
            /// ```json
            /// {
            ///     "version": {
            ///         "name": "1.8.3",
            ///         "protocol": 47,
            ///     },
            ///     "players": {
            ///         "max": 20,
            ///         "online": 1,
            ///         "sample": [
            ///            packet  {"name": "Thinkofdeath", "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20"}
            ///         ]
            ///     },
            ///     "description": "Hello world",
            ///     "favicon": "data:image/png;base64,<data>"
            /// }
            /// ```
            packet StatusResponse {
                field status: String =,
            }
            /// StatusPong is sent as a reply to a StatusPing.
            /// The Time field should be exactly the same as the
            /// one sent by the client.
            packet StatusPong {
                field ping: i64 =,
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
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(Statistic {
            name: try!(Serializable::read_from(buf)),
            value: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
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
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(BlockChangeRecord {
            xz: try!(Serializable::read_from(buf)),
            y: try!(Serializable::read_from(buf)),
            block_id: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
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
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(ExplosionRecord {
            x: try!(Serializable::read_from(buf)),
            y: try!(Serializable::read_from(buf)),
            z: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
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
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(MapIcon {
            direction_type: try!(Serializable::read_from(buf)),
            x: try!(Serializable::read_from(buf)),
            z: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
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
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(EntityProperty {
            key: try!(Serializable::read_from(buf)),
            value: try!(Serializable::read_from(buf)),
            modifiers: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
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
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(PropertyModifier {
            uuid: try!(Serializable::read_from(buf)),
            amount: try!(Serializable::read_from(buf)),
            operation: try!(Serializable::read_from(buf)),
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
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
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
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

    fn write_to<W: io::Write>(&self, _: &mut W) -> Result<(), Error> {
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
