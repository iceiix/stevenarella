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

use crate::format;

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
            packet QueryBlockNBT {
                field transaction_id: VarInt =,
                field location: Position =,
            }
            packet SetDifficulty {
                field new_difficulty: u8 =,
            }
            /// TabComplete is sent by the client when the client presses tab in
            /// the chat box.
            packet TabComplete {
                field text: String =,
                field assume_command: bool =,
                field has_target: bool =,
                field target: Option<Position> = when(|p: &TabComplete| p.has_target),
            }
            packet TabComplete_NoAssume {
                field text: String =,
                field has_target: bool =,
                field target: Option<Position> = when(|p: &TabComplete_NoAssume| p.has_target),
            }
            packet TabComplete_NoAssume_NoTarget {
                field text: String =,
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
            packet ClientStatus_u8 {
                field action_id: u8=,
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
            packet ClientSettings_u8 {
                field locale: String =,
                field view_distance: u8 =,
                field chat_mode: u8 =,
                field chat_colors: bool =,
                field displayed_skin_parts: u8 =,
                field main_hand: VarInt =,
            }
            packet ClientSettings_u8_Handsfree {
                field locale: String =,
                field view_distance: u8 =,
                field chat_mode: u8 =,
                field chat_colors: bool =,
                field displayed_skin_parts: u8 =,
            }
            packet ClientSettings_u8_Handsfree_Difficulty {
                field locale: String =,
                field view_distance: u8 =,
                field chat_mode: u8 =,
                field chat_colors: bool =,
                field difficulty: u8 =,
                field displayed_skin_parts: u8 =,
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
            /// ClickWindowButton is used for clicking an enchantment, lectern, stonecutter, or loom.
            packet ClickWindowButton {
                field id: u8 =,
                field button: u8 =,
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
            packet ClickWindow_u8 {
                field id: u8 =,
                field slot: i16 =,
                field button: u8 =,
                field action_number: u16 =,
                field mode: u8 =,
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
            packet PluginMessageServerbound_i16 {
                field channel: String =,
                field data: LenPrefixedBytes<VarShort> =,
            }
            packet EditBook {
                field new_book: Option<item::Stack> =,
                field is_signing: bool =,
                field hand: VarInt =,
            }
            packet QueryEntityNBT {
                field transaction_id: VarInt =,
                field entity_id: VarInt =,
            }
            /// UseEntity is sent when the user interacts (right clicks) or attacks
            /// (left clicks) an entity.
            packet UseEntity_Sneakflag {
                field target_id: VarInt =,
                field ty: VarInt =,
                field target_x: f32 = when(|p: &UseEntity_Sneakflag| p.ty.0 == 2),
                field target_y: f32 = when(|p: &UseEntity_Sneakflag| p.ty.0 == 2),
                field target_z: f32 = when(|p: &UseEntity_Sneakflag| p.ty.0 == 2),
                field hand: VarInt = when(|p: &UseEntity_Sneakflag| p.ty.0 == 0 || p.ty.0 == 2),
                field sneaking: bool =,
            }
            packet UseEntity_Hand {
                field target_id: VarInt =,
                field ty: VarInt =,
                field target_x: f32 = when(|p: &UseEntity_Hand| p.ty.0 == 2),
                field target_y: f32 = when(|p: &UseEntity_Hand| p.ty.0 == 2),
                field target_z: f32 = when(|p: &UseEntity_Hand| p.ty.0 == 2),
                field hand: VarInt = when(|p: &UseEntity_Hand| p.ty.0 == 0 || p.ty.0 == 2),
            }
            packet UseEntity_Handsfree {
                field target_id: VarInt =,
                field ty: VarInt =,
                field target_x: f32 = when(|p: &UseEntity_Handsfree| p.ty.0 == 2),
                field target_y: f32 = when(|p: &UseEntity_Handsfree| p.ty.0 == 2),
                field target_z: f32 = when(|p: &UseEntity_Handsfree| p.ty.0 == 2),
            }
            packet UseEntity_Handsfree_i32 {
                field target_id: i32 =,
                field ty: u8 =,
            }
            /// Sent when Generate is pressed on the Jigsaw Block interface.
            packet GenerateStructure {
                field location: Position =,
                field levels: VarInt =,
                field keep_jigsaws: bool =,
            }
            /// KeepAliveServerbound is sent by a client as a response to a
            /// KeepAliveClientbound. If the client doesn't reply the server
            /// may disconnect the client.
            packet KeepAliveServerbound_i64 {
                field id: i64 =,
            }
            packet KeepAliveServerbound_VarInt {
                field id: VarInt =,
            }
            packet KeepAliveServerbound_i32 {
                field id: i32 =,
            }
            packet LockDifficulty {
                field locked: bool =,
            }
            /// PlayerPosition is used to update the player's position.
            packet PlayerPosition {
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field on_ground: bool =,
            }
            packet PlayerPosition_HeadY {
                field x: f64 =,
                field feet_y: f64 =,
                field head_y: f64 =,
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
            packet PlayerPositionLook_HeadY {
                field x: f64 =,
                field feet_y: f64 =,
                field head_y: f64 =,
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
            /// SteerBoat is used to visually update the boat paddles.
            packet SteerBoat {
                field left_paddle_turning: bool =,
                field right_paddle_turning: bool =,
            }
            packet PickItem {
                field slot_to_use: VarInt =,
            }
            /// CraftRecipeRequest is sent when player clicks a recipe in the crafting book.
            packet CraftRecipeRequest {
                field window_id: u8 =,
                field recipe: VarInt =,
                field make_all: bool =,
            }
            /// ClientAbilities is used to modify the players current abilities.
            /// Currently flying is the only one
            packet ClientAbilities_f32 {
                field flags: u8 =,
                field flying_speed: f32 =,
                field walking_speed: f32 =,
            }
            packet ClientAbilities_u8 {
                field flags: u8 =,
            }
            /// PlayerDigging is sent when the client starts/stops digging a block.
            /// It also can be sent for droppping items and eating/shooting.
            packet PlayerDigging {
                field status: VarInt =,
                field location: Position =,
                field face: u8 =,
            }
            packet PlayerDigging_u8 {
                field status: u8 =,
                field location: Position =,
                field face: u8 =,
            }
            packet PlayerDigging_u8_u8y {
                field status: u8 =,
                field x: i32 =,
                field y: u8 =,
                field z: i32 =,
                field face: u8 =,
            }
            /// PlayerAction is sent when a player preforms various actions.
            packet PlayerAction {
                field entity_id: VarInt =,
                field action_id: VarInt =,
                field jump_boost: VarInt =,
            }
            packet PlayerAction_i32 {
                field entity_id: i32 =,
                field action_id: i8 =,
                field jump_boost: i32 =,
            }
            /// SteerVehicle is sent by the client when steers or preforms an action
            /// on a vehicle.
            packet SteerVehicle {
                field sideways: f32 =,
                field forward: f32 =,
                field flags: u8 =,
            }
            packet SteerVehicle_jump_unmount {
                field sideways: f32 =,
                field forward: f32 =,
                field jump: bool =,
                field unmount: bool =,
            }
            /// CraftingBookData is sent when the player interacts with the crafting book.
            packet CraftingBookData {
                field action: VarInt =,
                field recipe_id: i32 = when(|p: &CraftingBookData| p.action.0 == 0),
                field crafting_book_open: bool = when(|p: &CraftingBookData| p.action.0 == 1),
                field crafting_filter: bool = when(|p: &CraftingBookData| p.action.0 == 1),
            }
            /// SetDisplayedRecipe replaces CraftingBookData, type 0.
            packet SetDisplayedRecipe {
                field recipe_id: String =,
            }
            /// SetRecipeBookState replaces CraftingBookData, type 1.
            packet SetRecipeBookState {
                field book_id: VarInt =, // TODO: enum, 0: crafting, 1: furnace, 2: blast furnace, 3: smoker
                field book_open: bool =,
                field filter_active: bool =,
            }
            packet NameItem {
                field item_name: String =,
            }
            /// ResourcePackStatus informs the server of the client's current progress
            /// in activating the requested resource pack
            packet ResourcePackStatus {
                field result: VarInt =,
            }
            packet ResourcePackStatus_hash {
                field hash: String =,
                field result: VarInt =,
            }
            // TODO: Document
            packet AdvancementTab {
                field action: VarInt =,
                field tab_id: String = when(|p: &AdvancementTab| p.action.0 == 0),
            }
            packet SelectTrade {
                field selected_slot: VarInt =,
            }
            packet SetBeaconEffect {
                field primary_effect: VarInt =,
                field secondary_effect: VarInt =,
            }
            /// HeldItemChange is sent when the player changes the currently active
            /// hotbar slot.
            packet HeldItemChange {
                field slot: i16 =,
            }
            packet UpdateCommandBlock {
                field location: Position =,
                field command: String =,
                field mode: VarInt =,
                field flags: u8 =,
            }
            packet UpdateCommandBlockMinecart {
                field entity_id: VarInt =,
                field command: String =,
                field track_output: bool =,
            }
            /// CreativeInventoryAction is sent when the client clicks in the creative
            /// inventory. This is used to spawn items in creative.
            packet CreativeInventoryAction {
                field slot: i16 =,
                field clicked_item: Option<item::Stack> =,
            }
            packet UpdateJigsawBlock_Joint {
                field location: Position =,
                field name: String =,
                field target: String =,
                field pool: String =,
                field final_state: String =,
                field joint_type: String =,
            }
            packet UpdateJigsawBlock_Type {
                field location: Position =,
                field attachment_type: String =,
                field target_pool: String =,
                field final_state: String =,
            }
            packet UpdateStructureBlock {
                field location: Position =,
                field action: VarInt =,
                field mode: VarInt =,
                field name: String =,
                field offset_x: i8 =,
                field offset_y: i8 =,
                field offset_z: i8 =,
                field size_x: i8 =,
                field size_y: i8 =,
                field size_z: i8 =,
                field mirror: VarInt =,
                field rotation: VarInt =,
                field metadata: String =,
                field integrity: f32 =,
                field seed: VarLong =,
                field flags: i8 =,
            }
            /// SetSign sets the text on a sign after placing it.
            packet SetSign {
                field location: Position =,
                field line1: String =,
                field line2: String =,
                field line3: String =,
                field line4: String =,
            }
            packet SetSign_i16y {
                field x: i32 =,
                field y: i16 =,
                field z: i32 =,
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
            packet ArmSwing_Handsfree {
                field empty: () =,
            }
            packet ArmSwing_Handsfree_ID {
                field entity_id: i32 =,
                field animation: u8 =,
            }
            /// SpectateTeleport is sent by clients in spectator mode to teleport to a player.
            packet SpectateTeleport {
                field target: UUID =,
            }
            /// PlayerBlockPlacement is sent when the client tries to place a block.
            packet PlayerBlockPlacement_f32 {
                field location: Position =,
                field face: VarInt =,
                field hand: VarInt =,
                field cursor_x: f32 =,
                field cursor_y: f32 =,
                field cursor_z: f32 =,
            }
            packet PlayerBlockPlacement_u8 {
                field location: Position =,
                field face: VarInt =,
                field hand: VarInt =,
                field cursor_x: u8 =,
                field cursor_y: u8 =,
                field cursor_z: u8 =,
            }
            packet PlayerBlockPlacement_u8_Item {
                field location: Position =,
                field face: u8 =,
                field hand: Option<item::Stack> =,
                field cursor_x: u8 =,
                field cursor_y: u8 =,
                field cursor_z: u8 =,
            }
            packet PlayerBlockPlacement_u8_Item_u8y {
                field x: i32 =,
                field y: u8 =,
                field z: i32 =,
                field face: u8 =,
                field hand: Option<item::Stack> =,
                field cursor_x: u8 =,
                field cursor_y: u8 =,
                field cursor_z: u8 =,
            }
            packet PlayerBlockPlacement_insideblock {
                field hand: VarInt =,
                field location: Position =,
                field face: VarInt =,
                field cursor_x: f32 =,
                field cursor_y: f32 =,
                field cursor_z: f32 =,
                field inside_block: bool =, //1.14 added insideblock
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
            packet SpawnObject_i32 {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field ty: u8 =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
                field pitch: i8 =,
                field yaw: i8 =,
                field data: i32 =,
                field velocity_x: i16 =,
                field velocity_y: i16 =,
                field velocity_z: i16 =,
            }
            packet SpawnObject_i32_NoUUID {
                field entity_id: VarInt =,
                field ty: u8 =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
                field pitch: i8 =,
                field yaw: i8 =,
                field data: i32 =,
                field velocity_x: i16 = when(|p: &SpawnObject_i32_NoUUID| p.data != 0),
                field velocity_y: i16 = when(|p: &SpawnObject_i32_NoUUID| p.data != 0),
                field velocity_z: i16 = when(|p: &SpawnObject_i32_NoUUID| p.data != 0),
            }
            packet SpawnObject_VarInt {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field ty: VarInt =, //1.14 changed u8 to VarInt
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
            packet SpawnExperienceOrb_i32 {
                field entity_id: VarInt =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
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
            packet SpawnGlobalEntity_i32 {
                field entity_id: VarInt =,
                field ty: u8 =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
            }
            /// SpawnMob is used to spawn a living entity into the world when it is in
            /// range of the client.
            packet SpawnMob_NoMeta {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field ty: VarInt =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: i8 =,
                field pitch: i8 =,
                field head_pitch: i8 =,
                field velocity_x: i16 =,
                field velocity_y: i16 =,
                field velocity_z: i16 =,
            }
            packet SpawnMob_WithMeta {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field ty: VarInt =,
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
            packet SpawnMob_u8 {
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
            packet SpawnMob_u8_i32 {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field ty: u8 =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
                field yaw: i8 =,
                field pitch: i8 =,
                field head_pitch: i8 =,
                field velocity_x: i16 =,
                field velocity_y: i16 =,
                field velocity_z: i16 =,
                field metadata: types::Metadata =,
            }
            packet SpawnMob_u8_i32_NoUUID {
                field entity_id: VarInt =,
                field ty: u8 =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
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
            packet SpawnPainting_VarInt {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field motive: VarInt =,
                field location: Position =,
                field direction: u8 =,
            }
            packet SpawnPainting_String {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field title: String =,
                field location: Position =,
                field direction: u8 =,
            }
            packet SpawnPainting_NoUUID {
                field entity_id: VarInt =,
                field title: String =,
                field location: Position =,
                field direction: u8 =,
            }
            packet SpawnPainting_NoUUID_i32 {
                field entity_id: VarInt =,
                field title: String =,
                field x: i32 =,
                field y: i32 =,
                field z: i32 =,
                field direction: i32 =,
            }
            /// SpawnPlayer is used to spawn a player when they are in range of the client.
            /// This packet alone isn't enough to display the player as the skin and username
            /// information is in the player information packet.
            packet SpawnPlayer_f64_NoMeta {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: i8 =,
                field pitch: i8 =,
            }
            packet SpawnPlayer_f64 {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: i8 =,
                field pitch: i8 =,
                field metadata: types::Metadata =,
            }
            packet SpawnPlayer_i32 {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
                field yaw: i8 =,
                field pitch: i8 =,
                field metadata: types::Metadata =,
            }
            packet SpawnPlayer_i32_HeldItem {
                field entity_id: VarInt =,
                field uuid: UUID =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
                field yaw: i8 =,
                field pitch: i8 =,
                field current_item: u16 =,
                field metadata: types::Metadata =,
            }
            packet SpawnPlayer_i32_HeldItem_String {
                field entity_id: VarInt =,
                field uuid: String =,
                field name: String =,
                field properties: LenPrefixed<VarInt, packet::SpawnProperty> =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
                field yaw: i8 =,
                field pitch: i8 =,
                field current_item: u16 =,
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
            packet BlockBreakAnimation_i32 {
                field entity_id: VarInt =,
                field x: i32 =,
                field y: i32 =,
                field z: i32 =,
                field stage: i8 =,
            }
            /// UpdateBlockEntity updates the nbt tag of a block entity in the
            /// world.
            packet UpdateBlockEntity {
                field location: Position =,
                field action: u8 =,
                field nbt: Option<nbt::NamedTag> =,
            }
            packet UpdateBlockEntity_Data {
                field x: i32 =,
                field y: i16 =,
                field z: i32 =,
                field action: u8 =,
                field data_length: i16 =,
                field gzipped_nbt: Vec<u8> =,
            }
            /// BlockAction triggers different actions depending on the target block.
            packet BlockAction {
                field location: Position =,
                field byte1: u8 =,
                field byte2: u8 =,
                field block_type: VarInt =,
            }
            packet BlockAction_u16 {
                field x: i32 =,
                field y: u16 =,
                field z: i32 =,
                field byte1: u8 =,
                field byte2: u8 =,
                field block_type: VarInt =,
            }
            /// BlockChange is used to update a single block on the client.
            packet BlockChange_VarInt {
                field location: Position =,
                field block_id: VarInt =,
            }
            packet BlockChange_u8 {
                field x: i32 =,
                field y: u8 =,
                field z: i32 =,
                field block_id: VarInt =,
                field block_metadata: u8 =,
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
            packet ServerDifficulty_Locked {
                field difficulty: u8 =,
                field locked: bool =,
            }
            /// TabCompleteReply is sent as a reply to a tab completion request.
            /// The matches should be possible completions for the command/chat the
            /// player sent.
            packet TabCompleteReply {
                field matches: LenPrefixed<VarInt, String> =,
            }
            packet DeclareCommands {
                field nodes: LenPrefixed<VarInt, packet::CommandNode> =,
                field root_index: VarInt =,
            }
            /// ServerMessage is a message sent by the server. It could be from a player
            /// or just a system message. The Type field controls the location the
            /// message is displayed at and when the message is displayed.
            packet ServerMessage_Sender {
                field message: format::Component =,
                /// 0 - Chat message, 1 - System message, 2 - Action bar message
                field position: u8 =,
                field sender: UUID =,
            }
            packet ServerMessage_Position {
                field message: format::Component =,
                /// 0 - Chat message, 1 - System message, 2 - Action bar message
                field position: u8 =,
            }
            packet ServerMessage_NoPosition {
                field message: format::Component =,
            }
            /// MultiBlockChange is used to update a batch of blocks in a single packet.
            packet MultiBlockChange_VarInt {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field records: LenPrefixed<VarInt, packet::BlockChangeRecord> =,
            }
            packet MultiBlockChange_u16 {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field record_count: u16 =,
                field data_size: i32 =,
                field data: Vec<u8> =,
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
            packet WindowOpenHorse {
                field window_id: u8 =,
                field number_of_slots: VarInt =,
                field entity_id: i32 =,
            }
            packet WindowOpen_u8 {
                field id: u8 =,
                field ty: u8 =,
                field title: format::Component =,
                field slot_count: u8 =,
                field use_provided_window_title: bool =,
                field entity_id: i32 = when(|p: &WindowOpen_u8| p.ty == 11),
            }
            packet WindowOpen_VarInt {
                field id: VarInt =,
                field ty: VarInt =,
                field title: format::Component =,
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
            packet PluginMessageClientbound_i16 {
                field channel: String =,
                field data: LenPrefixedBytes<VarShort> =,
            }
            /// Plays a sound by name on the client
            packet NamedSoundEffect {
                field name: String =,
                field category: VarInt =,
                field x: i32 =,
                field y: i32 =,
                field z: i32 =,
                field volume: f32 =,
                field pitch: f32 =,
            }
            packet NamedSoundEffect_u8 {
                field name: String =,
                field category: VarInt =,
                field x: i32 =,
                field y: i32 =,
                field z: i32 =,
                field volume: f32 =,
                field pitch: u8 =,
            }
            packet NamedSoundEffect_u8_NoCategory {
                field name: String =,
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
            /// SetCompression updates the compression threshold.
            packet SetCompression {
                field threshold: VarInt =,
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
            packet KeepAliveClientbound_i64 {
                field id: i64 =,
            }
            packet KeepAliveClientbound_VarInt {
                field id: VarInt =,
            }
            packet KeepAliveClientbound_i32 {
                field id: i32 =,
            }
            /// ChunkData sends or updates a single chunk on the client. If New is set
            /// then biome data should be sent too.
            packet ChunkData_Biomes3D_bool {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field new: bool =,
                field ignore_old_data: bool =,
                field bitmask: VarInt =,
                field heightmaps: Option<nbt::NamedTag> =,
                field biomes: Biomes3D = when(|p: &ChunkData_Biomes3D_bool| p.new),
                field data: LenPrefixedBytes<VarInt> =,
                field block_entities: LenPrefixed<VarInt, Option<nbt::NamedTag>> =,
            }
            packet ChunkData_Biomes3D {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field new: bool =,
                field bitmask: VarInt =,
                field heightmaps: Option<nbt::NamedTag> =,
                field biomes: Biomes3D = when(|p: &ChunkData_Biomes3D| p.new),
                field data: LenPrefixedBytes<VarInt> =,
                field block_entities: LenPrefixed<VarInt, Option<nbt::NamedTag>> =,
            }
            packet ChunkData_HeightMap {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field new: bool =,
                field bitmask: VarInt =,
                field heightmaps: Option<nbt::NamedTag> =,
                field data: LenPrefixedBytes<VarInt> =,
                field block_entities: LenPrefixed<VarInt, Option<nbt::NamedTag>> =,
            }
            packet ChunkData {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field new: bool =,
                field bitmask: VarInt =,
                field data: LenPrefixedBytes<VarInt> =,
                field block_entities: LenPrefixed<VarInt, Option<nbt::NamedTag>> =,
            }
            packet ChunkData_NoEntities {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field new: bool =,
                field bitmask: VarInt =,
                field data: LenPrefixedBytes<VarInt> =,
            }
            packet ChunkData_NoEntities_u16 {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field new: bool =,
                field bitmask: u16 =,
                field data: LenPrefixedBytes<VarInt> =,
            }
            packet ChunkData_17 {
                field chunk_x: i32 =,
                field chunk_z: i32 =,
                field new: bool =,
                field bitmask: u16 =,
                field add_bitmask: u16 =,
                field compressed_data: LenPrefixedBytes<i32> =,
            }
            packet ChunkDataBulk {
                field skylight: bool =,
                field chunk_meta: LenPrefixed<VarInt, packet::ChunkMeta> =,
                field chunk_data: Vec<u8> =,
            }
            packet ChunkDataBulk_17 {
                field chunk_column_count: u16 =,
                field data_length: i32 =,
                field skylight: bool =,
                field chunk_data_and_meta: Vec<u8> =,
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
            packet Effect_u8y {
                field effect_id: i32 =,
                field x: i32 =,
                field y: u8 =,
                field z: i32 =,
                field data: i32 =,
                field disable_relative: bool =,
            }
            /// Particle spawns particles at the target location with the various
            /// modifiers.
            packet Particle_f64 {
                field particle_id: i32 =,
                field long_distance: bool =,
                field x: f64 =,
                field y: f64=,
                field z: f64 =,
                field offset_x: f32 =,
                field offset_y: f32 =,
                field offset_z: f32 =,
                field speed: f32 =,
                field count: i32 =,
                field block_state: VarInt = when(|p: &Particle_f64| p.particle_id == 3 || p.particle_id == 20),
                field red: f32 = when(|p: &Particle_f64| p.particle_id == 11),
                field green: f32 = when(|p: &Particle_f64| p.particle_id == 11),
                field blue: f32 = when(|p: &Particle_f64| p.particle_id == 11),
                field scale: f32 = when(|p: &Particle_f64| p.particle_id == 11),
                field item: Option<nbt::NamedTag> = when(|p: &Particle_f64| p.particle_id == 27),
            }
            packet Particle_Data {
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
                field block_state: VarInt = when(|p: &Particle_Data| p.particle_id == 3 || p.particle_id == 20),
                field red: f32 = when(|p: &Particle_Data| p.particle_id == 11),
                field green: f32 = when(|p: &Particle_Data| p.particle_id == 11),
                field blue: f32 = when(|p: &Particle_Data| p.particle_id == 11),
                field scale: f32 = when(|p: &Particle_Data| p.particle_id == 11),
                field item: Option<nbt::NamedTag> = when(|p: &Particle_Data| p.particle_id == 27),
            }
            packet Particle_VarIntArray {
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
                field data1: VarInt = when(|p: &Particle_VarIntArray| p.particle_id == 36 || p.particle_id == 37 || p.particle_id == 38 || p.particle_id == 46),
                field data2: VarInt = when(|p: &Particle_VarIntArray| p.particle_id == 36),
            }
            packet Particle_Named {
                field particle_id: String =,
                field x: f32 =,
                field y: f32 =,
                field z: f32 =,
                field offset_x: f32 =,
                field offset_y: f32 =,
                field offset_z: f32 =,
                field speed: f32 =,
                field count: i32 =,
            }
            /// JoinGame is sent after completing the login process. This
            /// sets the initial state for the client.
            packet JoinGame_WorldNames {
                /// The entity id the client will be referenced by
                field entity_id: i32 =,
                /// The starting gamemode of the client
                field gamemode: u8 =,
                /// The previous gamemode of the client
                field previous_gamemode: u8 =,
                /// Identifiers for all worlds on the server
                field world_names: LenPrefixed<VarInt, String> =,
                /// Represents a dimension registry
                field dimension_codec: Option<nbt::NamedTag> =,
                /// The dimension the client is starting in
                field dimension: String =,
                /// The world being spawned into
                field world_name: String =,
                /// Truncated SHA-256 hash of world's seed
                field hashed_seed: i64 =,
                /// The max number of players on the server
                field max_players: u8 =,
                /// The render distance (2-32)
                field view_distance: VarInt =,
                /// Whether the client should reduce the amount of debug
                /// information it displays in F3 mode
                field reduced_debug_info: bool =,
                /// Whether to prompt or immediately respawn
                field enable_respawn_screen: bool =,
                /// Whether the world is in debug mode
                field is_debug: bool =,
                /// Whether the world is a superflat world
                field is_flat: bool =,
            }

            packet JoinGame_HashedSeed_Respawn {
                /// The entity id the client will be referenced by
                field entity_id: i32 =,
                /// The starting gamemode of the client
                field gamemode: u8 =,
                /// The dimension the client is starting in
                field dimension: i32 =,
                /// Truncated SHA-256 hash of world's seed
                field hashed_seed: i64 =,
                /// The max number of players on the server
                field max_players: u8 =,
                /// The level type of the server
                field level_type: String =,
                /// The render distance (2-32)
                field view_distance: VarInt =,
                /// Whether the client should reduce the amount of debug
                /// information it displays in F3 mode
                field reduced_debug_info: bool =,
                /// Whether to prompt or immediately respawn
                field enable_respawn_screen: bool =,
            }
            packet JoinGame_i32_ViewDistance {
                /// The entity id the client will be referenced by
                field entity_id: i32 =,
                /// The starting gamemode of the client
                field gamemode: u8 =,
                /// The dimension the client is starting in
                field dimension: i32 =,
                /// The max number of players on the server
                field max_players: u8 =,
                /// The level type of the server
                field level_type: String =,
                /// The render distance (2-32)
                field view_distance: VarInt =,
                /// Whether the client should reduce the amount of debug
                /// information it displays in F3 mode
                field reduced_debug_info: bool =,
            }
            packet JoinGame_i32 {
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
            packet JoinGame_i8 {
                /// The entity id the client will be referenced by
                field entity_id: i32 =,
                /// The starting gamemode of the client
                field gamemode: u8 =,
                /// The dimension the client is starting in
                field dimension: i8 =,
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
            packet JoinGame_i8_NoDebug {
                field entity_id: i32 =,
                field gamemode: u8 =,
                field dimension: i8 =,
                field difficulty: u8 =,
                field max_players: u8 =,
                field level_type: String =,
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
            packet Maps_NoTracking {
                field item_damage: VarInt =,
                field scale: i8 =,
                field icons: LenPrefixed<VarInt, packet::MapIcon> =,
                field columns: u8 =,
                field rows: Option<u8> = when(|p: &Maps_NoTracking| p.columns > 0),
                field x: Option<u8> = when(|p: &Maps_NoTracking| p.columns > 0),
                field z: Option<u8> = when(|p: &Maps_NoTracking| p.columns > 0),
                field data: Option<LenPrefixedBytes<VarInt>> = when(|p: &Maps_NoTracking| p.columns > 0),
            }
            packet Maps_NoTracking_Data {
                field item_damage: VarInt =,
                field data: LenPrefixedBytes<i16> =,
            }
            /// EntityMove moves the entity with the id by the offsets provided.
            packet EntityMove_i16 {
                field entity_id: VarInt =,
                field delta_x: FixedPoint12<i16> =,
                field delta_y: FixedPoint12<i16> =,
                field delta_z: FixedPoint12<i16> =,
                field on_ground: bool =,
            }
            packet EntityMove_i8 {
                field entity_id: VarInt =,
                field delta_x: FixedPoint5<i8> =,
                field delta_y: FixedPoint5<i8> =,
                field delta_z: FixedPoint5<i8> =,
                field on_ground: bool =,
            }
            packet EntityMove_i8_i32_NoGround {
                field entity_id: i32 =,
                field delta_x: FixedPoint5<i8> =,
                field delta_y: FixedPoint5<i8> =,
                field delta_z: FixedPoint5<i8> =,
            }
            /// EntityLookAndMove is a combination of EntityMove and EntityLook.
            packet EntityLookAndMove_i16 {
                field entity_id: VarInt =,
                field delta_x: FixedPoint12<i16> =,
                field delta_y: FixedPoint12<i16> =,
                field delta_z: FixedPoint12<i16> =,
                field yaw: i8 =,
                field pitch: i8 =,
                field on_ground: bool =,
            }
            packet EntityLookAndMove_i8 {
                field entity_id: VarInt =,
                field delta_x: FixedPoint5<i8> =,
                field delta_y: FixedPoint5<i8> =,
                field delta_z: FixedPoint5<i8> =,
                field yaw: i8 =,
                field pitch: i8 =,
                field on_ground: bool =,
            }
            packet EntityLookAndMove_i8_i32_NoGround {
                field entity_id: i32 =,
                field delta_x: FixedPoint5<i8> =,
                field delta_y: FixedPoint5<i8> =,
                field delta_z: FixedPoint5<i8> =,
                field yaw: i8 =,
                field pitch: i8 =,
            }
            /// EntityLook rotates the entity to the new angles provided.
            packet EntityLook_VarInt {
                field entity_id: VarInt =,
                field yaw: i8 =,
                field pitch: i8 =,
                field on_ground: bool =,
            }
            packet EntityLook_i32_NoGround {
                field entity_id: i32 =,
                field yaw: i8 =,
                field pitch: i8 =,
            }
            /// Entity does nothing. It is a result of subclassing used in Minecraft.
            packet Entity {
                field entity_id: VarInt =,
            }
            packet Entity_i32 {
                field entity_id: i32 =,
            }
            /// EntityUpdateNBT updates the entity named binary tag.
            packet EntityUpdateNBT {
                field entity_id: VarInt =,
                field nbt: Option<nbt::NamedTag> =,
            }
            /// Teleports the player's vehicle
            packet VehicleTeleport {
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: f32 =,
                field pitch: f32 =,
            }
            /// Opens the book GUI.
            packet OpenBook {
                field hand: VarInt =,
            }
            /// SignEditorOpen causes the client to open the editor for a sign so that
            /// it can write to it. Only sent in vanilla when the player places a sign.
            packet SignEditorOpen {
                field location: Position =,
            }
            packet SignEditorOpen_i32 {
                field x: i32 =,
                field y: i32 =,
                field z: i32 =,
            }
            /// CraftRecipeResponse is a response to CraftRecipeRequest, notifies the UI.
            packet CraftRecipeResponse {
                field window_id: u8 =,
                field recipe: VarInt =,
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
            packet PlayerInfo_String {
                field name: String =,
                field online: bool =,
                field ping: u16 =,
            }
            packet FacePlayer {
                field feet_eyes: VarInt =,
                field target_x: f64 =,
                field target_y: f64 =,
                field target_z: f64 =,
                field is_entity: bool =,
                field entity_id: Option<VarInt> = when(|p: &FacePlayer| p.is_entity),
                field entity_feet_eyes: Option<VarInt> = when(|p: &FacePlayer| p.is_entity),
            }
            /// TeleportPlayer is sent to change the player's position. The client is expected
            /// to reply to the server with the same positions as contained in this packet
            /// otherwise will reject future packets.
            packet TeleportPlayer_WithConfirm {
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: f32 =,
                field pitch: f32 =,
                field flags: u8 =,
                field teleport_id: VarInt =,
            }
            packet TeleportPlayer_NoConfirm {
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: f32 =,
                field pitch: f32 =,
                field flags: u8 =,
            }
            packet TeleportPlayer_OnGround {
                field x: f64 =,
                field eyes_y: f64 =,
                field z: f64 =,
                field yaw: f32 =,
                field pitch: f32 =,
                field on_ground: bool =,
            }
            /// EntityUsedBed is sent by the server when a player goes to bed.
            packet EntityUsedBed {
                field entity_id: VarInt =,
                field location: Position =,
            }
            packet EntityUsedBed_i32 {
                field entity_id: i32 =,
                field x: i32 =,
                field y: u8 =,
                field z: i32 =,
            }
            packet UnlockRecipes_NoSmelting {
                field action: VarInt =,
                field crafting_book_open: bool =,
                field filtering_craftable: bool =,
                field recipe_ids: LenPrefixed<VarInt, VarInt> =,
                field recipe_ids2: LenPrefixed<VarInt, VarInt> = when(|p: &UnlockRecipes_NoSmelting| p.action.0 == 0),
            }
            packet UnlockRecipes_WithSmelting {
                field action: VarInt =,
                field crafting_book_open: bool =,
                field filtering_craftable: bool =,
                field smelting_book_open: bool =,
                field filtering_smeltable: bool =,
                field recipe_ids: LenPrefixed<VarInt, String> =,
                field recipe_ids2: LenPrefixed<VarInt, String> = when(|p: &UnlockRecipes_WithSmelting| p.action.0 == 0),
            }
            /// EntityDestroy destroys the entities with the ids in the provided slice.
            packet EntityDestroy {
                field entity_ids: LenPrefixed<VarInt, VarInt> =,
            }
            packet EntityDestroy_u8 {
                field entity_ids: LenPrefixed<u8, i32> =,
            }
            /// EntityRemoveEffect removes an effect from an entity.
            packet EntityRemoveEffect {
                field entity_id: VarInt =,
                field effect_id: i8 =,
            }
            packet EntityRemoveEffect_i32 {
                field entity_id: i32 =,
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
            packet Respawn_Gamemode {
                field dimension: i32 =,
                field difficulty: u8 =,
                field gamemode: u8 =,
                field level_type: String =,
            }
            packet Respawn_HashedSeed {
                field dimension: i32 =,
                field hashed_seed: i64 =,
                field difficulty: u8 =,
                field gamemode: u8 =,
                field level_type: String =,
            }
            packet Respawn_WorldName {
                field dimension: String =,
                field world_name: String =,
                field hashed_seed: i64 =,
                field gamemode: u8 =,
                field previous_gamemode: u8 =,
                field is_debug: bool =,
                field is_flat: bool =,
                field copy_metadata: bool =,
            }
            /// EntityHeadLook rotates an entity's head to the new angle.
            packet EntityHeadLook {
                field entity_id: VarInt =,
                field head_yaw: i8 =,
            }
            packet EntityHeadLook_i32 {
                field entity_id: i32 =,
                field head_yaw: i8 =,
            }
            packet EntityStatus {
                field entity_id: i32 =,
                field entity_status: i8 =,
            }
            packet NBTQueryResponse {
                field transaction_id: VarInt =,
                field nbt: Option<nbt::NamedTag> =,
            }
            /// SelectAdvancementTab indicates the client should switch the advancement tab.
            packet SelectAdvancementTab {
                field has_id: bool =,
                field tab_id: String = when(|p: &SelectAdvancementTab| p.has_id),
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
            /// UpdateViewPosition is used to determine what chunks should be remain loaded.
            packet UpdateViewPosition {
                field chunk_x: VarInt =,
                field chunk_z: VarInt =,
            }
            /// UpdateViewDistance is sent by the integrated server when changing render distance.
            packet UpdateViewDistance {
                field view_distance: VarInt =,
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
            packet EntityMetadata_i32 {
                field entity_id: i32 =,
                field metadata: types::Metadata =,
            }
            /// EntityAttach attaches to entities together, either by mounting or leashing.
            /// -1 can be used at the EntityID to deattach.
            packet EntityAttach {
                field entity_id: i32 =,
                field vehicle: i32 =,
            }
            packet EntityAttach_leashed {
                field entity_id: i32 =,
                field vehicle: i32 =,
                field leash: bool =,
            }
            /// EntityVelocity sets the velocity of an entity in 1/8000 of a block
            /// per a tick.
            packet EntityVelocity {
                field entity_id: VarInt =,
                field velocity_x: i16 =,
                field velocity_y: i16 =,
                field velocity_z: i16 =,
            }
            packet EntityVelocity_i32 {
                field entity_id: i32 =,
                field velocity_x: i16 =,
                field velocity_y: i16 =,
                field velocity_z: i16 =,
            }
            /// EntityEquipment is sent to display an item on an entity, like a sword
            /// or armor. Slot 0 is the held item and slots 1 to 4 are boots, leggings
            /// chestplate and helmet respectively.
            packet EntityEquipment_VarInt {
                field entity_id: VarInt =,
                field slot: VarInt =,
                field item: Option<item::Stack> =,
            }
            packet EntityEquipment_u16 {
                field entity_id: VarInt =,
                field slot: u16 =,
                field item: Option<item::Stack> =,
            }
            packet EntityEquipment_u16_i32 {
                field entity_id: i32 =,
                field slot: u16 =,
                field item: Option<item::Stack> =,
            }
            /// SetExperience updates the experience bar on the client.
            packet SetExperience {
                field experience_bar: f32 =,
                field level: VarInt =,
                field total_experience: VarInt =,
            }
            packet SetExperience_i16 {
                field experience_bar: f32 =,
                field level: i16 =,
                field total_experience: i16 =,
            }
            /// UpdateHealth is sent by the server to update the player's health and food.
            packet UpdateHealth {
                field health: f32 =,
                field food: VarInt =,
                field food_saturation: f32 =,
            }
            packet UpdateHealth_u16 {
                field health: f32 =,
                field food: u16 =,
                field food_saturation: f32 =,
            }
            /// ScoreboardObjective creates/updates a scoreboard objective.
            packet ScoreboardObjective {
                field name: String =,
                field mode: u8 =,
                field value: String = when(|p: &ScoreboardObjective| p.mode == 0 || p.mode == 2),
                field ty: String = when(|p: &ScoreboardObjective| p.mode == 0 || p.mode == 2),
            }
            packet ScoreboardObjective_NoMode {
                field name: String =,
                field value: String =,
                field ty: u8 =,
            }
            /// SetPassengers mounts entities to an entity
            packet SetPassengers {
                field entity_id: VarInt =,
                field passengers: LenPrefixed<VarInt, VarInt> =,
            }
            /// Teams creates and updates teams
            packet Teams_VarInt {
                field name: String =,
                field mode: u8 =,
                field display_name: Option<String> = when(|p: &Teams_VarInt| p.mode == 0 || p.mode == 2),
                field flags: Option<u8> = when(|p: &Teams_VarInt| p.mode == 0 || p.mode == 2),
                field name_tag_visibility: Option<String> = when(|p: &Teams_VarInt| p.mode == 0 || p.mode == 2),
                field collision_rule: Option<String> = when(|p: &Teams_VarInt| p.mode == 0 || p.mode == 2),
                field formatting: Option<VarInt> = when(|p: &Teams_VarInt| p.mode == 0 || p.mode == 2),
                field prefix: Option<String> = when(|p: &Teams_VarInt| p.mode == 0 || p.mode == 2),
                field suffix: Option<String> = when(|p: &Teams_VarInt| p.mode == 0 || p.mode == 2),
                field players: Option<LenPrefixed<VarInt, String>> = when(|p: &Teams_VarInt| p.mode == 0 || p.mode == 3 || p.mode == 4),
            }
            packet Teams_u8 {
                field name: String =,
                field mode: u8 =,
                field data: Vec<u8> =,
                field display_name: Option<String> = when(|p: &Teams_u8| p.mode == 0 || p.mode == 2),
                field prefix: Option<String> = when(|p: &Teams_u8| p.mode == 0 || p.mode == 2),
                field suffix: Option<String> = when(|p: &Teams_u8| p.mode == 0 || p.mode == 2),
                field flags: Option<u8> = when(|p: &Teams_u8| p.mode == 0 || p.mode == 2),
                field name_tag_visibility: Option<String> = when(|p: &Teams_u8| p.mode == 0 || p.mode == 2),
                field collision_rule: Option<String> = when(|p: &Teams_u8| p.mode == 0 || p.mode == 2),
                field color: Option<i8> = when(|p: &Teams_u8| p.mode == 0 || p.mode == 2),
                field players: Option<LenPrefixed<VarInt, String>> = when(|p: &Teams_u8| p.mode == 0 || p.mode == 3 || p.mode == 4),
            }
            packet Teams_NoVisColor {
                field name: String =,
                field mode: u8 =,
                field display_name: Option<String> = when(|p: &Teams_NoVisColor| p.mode == 0 || p.mode == 2),
                field prefix: Option<String> = when(|p: &Teams_NoVisColor| p.mode == 0 || p.mode == 2),
                field suffix: Option<String> = when(|p: &Teams_NoVisColor| p.mode == 0 || p.mode == 2),
                field flags: Option<u8> = when(|p: &Teams_NoVisColor| p.mode == 0 || p.mode == 2),
                field players: Option<LenPrefixed<VarInt, String>> = when(|p: &Teams_NoVisColor| p.mode == 0 || p.mode == 3 || p.mode == 4),
            }
            /// UpdateScore is used to update or remove an item from a scoreboard
            /// objective.
            packet UpdateScore {
                field name: String =,
                field action: u8 =,
                field object_name: String =,
                field value: Option<VarInt> = when(|p: &UpdateScore| p.action != 1),
            }
            packet UpdateScore_i32 {
                field name: String =,
                field action: u8 =,
                field object_name: String =,
                field value: Option<i32 > = when(|p: &UpdateScore_i32| p.action != 1),
            }
            /// SpawnPosition is sent to change the player's current spawn point. Currently
            /// only used by the client for the compass.
            packet SpawnPosition {
                field location: Position =,
            }
            packet SpawnPosition_i32 {
                field x: i32 =,
                field y: i32 =,
                field z: i32 =,
            }
            /// TimeUpdate is sent to sync the world's time to the client, the client
            /// will manually tick the time itself so this doesn't need to sent repeatedly
            /// but if the server or client has issues keeping up this can fall out of sync
            /// so it is a good idea to send this now and again
            packet TimeUpdate {
                field world_age: i64 =,
                field time_of_day: i64 =,
            }
            packet StopSound {
                field flags: u8 =,
                field source: Option<VarInt> = when(|p: &StopSound| p.flags & 0x01 != 0),
                field sound: Option<String> = when(|p: &StopSound| p.flags & 0x02 != 0),
            }
            /// Title configures an on-screen title.
            packet Title {
                field action: VarInt =,
                field title: Option<format::Component> = when(|p: &Title| p.action.0 == 0),
                field sub_title: Option<format::Component> = when(|p: &Title| p.action.0 == 1),
                field action_bar_text: Option<String> = when(|p: &Title| p.action.0 == 2),
                field fade_in: Option<i32> = when(|p: &Title| p.action.0 == 3),
                field fade_stay: Option<i32> = when(|p: &Title| p.action.0 == 3),
                field fade_out: Option<i32> = when(|p: &Title| p.action.0 == 3),
            }
            packet Title_notext {
                field action: VarInt =,
                field title: Option<format::Component> = when(|p: &Title_notext| p.action.0 == 0),
                field sub_title: Option<format::Component> = when(|p: &Title_notext| p.action.0 == 1),
                field fade_in: Option<i32> = when(|p: &Title_notext| p.action.0 == 2),
                field fade_stay: Option<i32> = when(|p: &Title_notext| p.action.0 == 2),
                field fade_out: Option<i32> = when(|p: &Title_notext| p.action.0 == 2),
            }
            packet Title_notext_component {
                field action: VarInt =,
                field title: Option<format::Component> = when(|p: &Title_notext_component| p.action.0 == 0),
                field sub_title: Option<format::Component> = when(|p: &Title_notext_component| p.action.0 == 1),
                field fade_in: Option<format::Component> = when(|p: &Title_notext_component| p.action.0 == 2),
                field fade_stay: Option<format::Component> = when(|p: &Title_notext_component| p.action.0 == 2),
                field fade_out: Option<format::Component> = when(|p: &Title_notext_component| p.action.0 == 2),
            }
            /// UpdateSign sets or changes the text on a sign.
            packet UpdateSign {
                field location: Position =,
                field line1: format::Component =,
                field line2: format::Component =,
                field line3: format::Component =,
                field line4: format::Component =,
            }
            packet UpdateSign_u16 {
                field x: i32 =,
                field y: u16 =,
                field z: i32 =,
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
                field pitch: f32 =,
            }
            packet SoundEffect_u8 {
                field name: VarInt =,
                field category: VarInt =,
                field x: i32 =,
                field y: i32 =,
                field z: i32 =,
                field volume: f32 =,
                field pitch: u8 =,
            }
            /// Plays a sound effect from an entity.
            packet EntitySoundEffect {
                field sound_id: VarInt =,
                field sound_category: VarInt =,
                field entity_id: VarInt =,
                field volume: f32 =,
                field pitch: f32 =,
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
                field number_of_items: VarInt =,
            }
            packet CollectItem_nocount {
                field collected_entity_id: VarInt =,
                field collector_entity_id: VarInt =,
            }
            packet CollectItem_nocount_i32 {
                field collected_entity_id: i32 =,
                field collector_entity_id: i32 =,
            }
            /// EntityTeleport teleports the entity to the target location. This is
            /// sent if the entity moves further than EntityMove allows.
            packet EntityTeleport_f64 {
                field entity_id: VarInt =,
                field x: f64 =,
                field y: f64 =,
                field z: f64 =,
                field yaw: i8 =,
                field pitch: i8 =,
                field on_ground: bool =,
            }
            packet EntityTeleport_i32 {
                field entity_id: VarInt =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
                field yaw: i8 =,
                field pitch: i8 =,
                field on_ground: bool =,
            }
            packet EntityTeleport_i32_i32_NoGround {
                field entity_id: i32 =,
                field x: FixedPoint5<i32> =,
                field y: FixedPoint5<i32> =,
                field z: FixedPoint5<i32> =,
                field yaw: i8 =,
                field pitch: i8 =,
            }
            packet Advancements {
                field data: Vec<u8> =,
                /* TODO: fix parsing modded advancements 1.12.2 (e.g. SevTech Ages)
                 * see https://github.com/iceiix/stevenarella/issues/148
                field reset_clear: bool =,
                field mapping: LenPrefixed<VarInt, packet::Advancement> =,
                field identifiers: LenPrefixed<VarInt, String> =,
                field progress: LenPrefixed<VarInt, packet::AdvancementProgress> =,
                */
            }
            /// EntityProperties updates the properties for an entity.
            packet EntityProperties {
                field entity_id: VarInt =,
                field properties: LenPrefixed<i32, packet::EntityProperty> =,
            }
            packet EntityProperties_i32 {
                field entity_id: i32 =,
                field properties: LenPrefixed<i32, packet::EntityProperty_i16> =,
            }
            /// EntityEffect applies a status effect to an entity for a given duration.
            packet EntityEffect {
                field entity_id: VarInt =,
                field effect_id: i8 =,
                field amplifier: i8 =,
                field duration: VarInt =,
                field hide_particles: bool =,
            }
            packet EntityEffect_i32 {
                field entity_id: i32 =,
                field effect_id: i8 =,
                field amplifier: i8 =,
                field duration: i16 =,
            }
            packet DeclareRecipes {
                field recipes: LenPrefixed<VarInt, packet::Recipe> =,
            }
            packet Tags {
                field block_tags: LenPrefixed<VarInt, packet::Tags> =,
                field item_tags: LenPrefixed<VarInt, packet::Tags> =,
                field fluid_tags: LenPrefixed<VarInt, packet::Tags> =,
            }
            packet TagsWithEntities {
                field block_tags: LenPrefixed<VarInt, packet::Tags> =,
                field item_tags: LenPrefixed<VarInt, packet::Tags> =,
                field fluid_tags: LenPrefixed<VarInt, packet::Tags> =,
                field entity_tags: LenPrefixed<VarInt, packet::Tags> =,
            }
            packet AcknowledgePlayerDigging {
                field location: Position =,
                field block: VarInt =,
                field status: VarInt =,
                field successful: bool =,
            }
            packet UpdateLight_WithTrust {
                field chunk_x: VarInt =,
                field chunk_z: VarInt =,
                field trust_edges: bool =,
                field sky_light_mask: VarInt =,
                field block_light_mask: VarInt =,
                field empty_sky_light_mask: VarInt =,
                field light_arrays: Vec<u8> =,
            }
            packet UpdateLight_NoTrust {
                field chunk_x: VarInt =,
                field chunk_z: VarInt =,
                field sky_light_mask: VarInt =,
                field block_light_mask: VarInt =,
                field empty_sky_light_mask: VarInt =,
                field light_arrays: Vec<u8> =,
            }
            packet TradeList_WithoutRestock {
                field id: VarInt =,
                field trades: LenPrefixed<u8, packet::Trade> =,
                field villager_level: VarInt =,
                field experience: VarInt =,
                field is_regular_villager: bool =,
            }
            packet TradeList_WithRestock {
                field id: VarInt =,
                field trades: LenPrefixed<u8, packet::Trade> =,
                field villager_level: VarInt =,
                field experience: VarInt =,
                field is_regular_villager: bool =,
                field can_restock: bool =,
            }
            packet CoFHLib_SendUUID {
                field player_uuid: UUID =,
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
            packet EncryptionResponse_i16 {
                field shared_secret: LenPrefixedBytes<i16> =,
                field verify_token: LenPrefixedBytes<i16> =,
            }
            packet LoginPluginResponse {
                field message_id: VarInt =,
                field successful: bool =,
                field data: Vec<u8> =,
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
            packet EncryptionRequest_i16 {
                field server_id: String =,
                field public_key: LenPrefixedBytes<i16> =,
                field verify_token: LenPrefixedBytes<i16> =,
            }
            /// LoginSuccess is sent by the server if the player successfully
            /// authenicates with the session servers (online mode) or straight
            /// after LoginStart (offline mode).
            packet LoginSuccess_String {
                /// String encoding of a uuid (with hyphens)
                field uuid: String =,
                field username: String =,
            }
            packet LoginSuccess_UUID {
                field uuid: UUID =,
                field username: String =,
            }
            /// SetInitialCompression sets the compression threshold during the
            /// login state.
            packet SetInitialCompression {
                /// Threshold where a packet should be sent compressed
                field threshold: VarInt =,
            }
            packet LoginPluginRequest {
                field message_id: VarInt =,
                field channel: String =,
                field data: Vec<u8> =,
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
pub struct SpawnProperty {
    pub name: String,
    pub value: String,
    pub signature: String,
}

impl Serializable for SpawnProperty {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(SpawnProperty {
            name: Serializable::read_from(buf)?,
            value: Serializable::read_from(buf)?,
            signature: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.name.write_to(buf)?;
        self.value.write_to(buf)?;
        self.signature.write_to(buf)
    }
}

#[derive(Debug, Default)]
pub struct Statistic {
    pub name: String,
    pub value: VarInt,
}

impl Serializable for Statistic {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(Statistic {
            name: Serializable::read_from(buf)?,
            value: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.name.write_to(buf)?;
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
            xz: Serializable::read_from(buf)?,
            y: Serializable::read_from(buf)?,
            block_id: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.xz.write_to(buf)?;
        self.y.write_to(buf)?;
        self.block_id.write_to(buf)
    }
}

#[derive(Debug, Default)]
pub struct ChunkMeta {
    pub x: i32,
    pub z: i32,
    pub bitmask: u16,
}

impl Serializable for ChunkMeta {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(ChunkMeta {
            x: Serializable::read_from(buf)?,
            z: Serializable::read_from(buf)?,
            bitmask: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.x.write_to(buf)?;
        self.z.write_to(buf)?;
        self.bitmask.write_to(buf)
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
            x: Serializable::read_from(buf)?,
            y: Serializable::read_from(buf)?,
            z: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.x.write_to(buf)?;
        self.y.write_to(buf)?;
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
            direction_type: Serializable::read_from(buf)?,
            x: Serializable::read_from(buf)?,
            z: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.direction_type.write_to(buf)?;
        self.x.write_to(buf)?;
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
pub struct Advancement {
    pub id: String,
    pub parent_id: Option<String>,
    pub display_data: Option<AdvancementDisplay>,
    pub criteria: LenPrefixed<VarInt, String>,
    pub requirements: LenPrefixed<VarInt, LenPrefixed<VarInt, String>>,
}

impl Serializable for Advancement {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let id: String = Serializable::read_from(buf)?;
        let parent_id = {
            let has_parent: u8 = Serializable::read_from(buf)?;
            if has_parent != 0 {
                let parent_id: String = Serializable::read_from(buf)?;
                Some(parent_id)
            } else {
                None
            }
        };

        let has_display: u8 = Serializable::read_from(buf)?;
        let display_data = {
            if has_display != 0 {
                let display_data: AdvancementDisplay = Serializable::read_from(buf)?;
                Some(display_data)
            } else {
                None
            }
        };

        let criteria: LenPrefixed<VarInt, String> = Serializable::read_from(buf)?;
        let requirements: LenPrefixed<VarInt, LenPrefixed<VarInt, String>> =
            Serializable::read_from(buf)?;
        Ok(Advancement {
            id,
            parent_id,
            display_data,
            criteria,
            requirements,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.id.write_to(buf)?;
        self.parent_id.write_to(buf)?;
        self.display_data.write_to(buf)?;
        self.criteria.write_to(buf)?;
        self.requirements.write_to(buf)
    }
}

#[derive(Debug, Default)]
pub struct AdvancementDisplay {
    pub title: String,
    pub description: String,
    pub icon: Option<crate::item::Stack>,
    pub frame_type: VarInt,
    pub flags: i32,
    pub background_texture: Option<String>,
    pub x_coord: f32,
    pub y_coord: f32,
}

impl Serializable for AdvancementDisplay {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let title: String = Serializable::read_from(buf)?;
        let description: String = Serializable::read_from(buf)?;
        let icon: Option<crate::item::Stack> = Serializable::read_from(buf)?;
        let frame_type: VarInt = Serializable::read_from(buf)?;
        let flags: i32 = Serializable::read_from(buf)?;
        let background_texture: Option<String> = if flags & 1 != 0 {
            Serializable::read_from(buf)?
        } else {
            None
        };
        let x_coord: f32 = Serializable::read_from(buf)?;
        let y_coord: f32 = Serializable::read_from(buf)?;

        Ok(AdvancementDisplay {
            title,
            description,
            icon,
            frame_type,
            flags,
            background_texture,
            x_coord,
            y_coord,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.title.write_to(buf)?;
        self.description.write_to(buf)?;
        self.icon.write_to(buf)?;
        self.frame_type.write_to(buf)?;
        self.flags.write_to(buf)?;
        if self.flags & 1 != 0 {
            self.background_texture.write_to(buf)?;
        }
        self.x_coord.write_to(buf)?;
        self.y_coord.write_to(buf)
    }
}

#[derive(Debug, Default)]
pub struct AdvancementProgress {
    pub id: String,
    pub criteria: LenPrefixed<VarInt, CriterionProgress>,
}

impl Serializable for AdvancementProgress {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(AdvancementProgress {
            id: Serializable::read_from(buf)?,
            criteria: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.id.write_to(buf)?;
        self.criteria.write_to(buf)
    }
}

#[derive(Debug, Default)]
pub struct CriterionProgress {
    pub id: String,
    pub date_of_achieving: Option<i64>,
}

impl Serializable for CriterionProgress {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let id = Serializable::read_from(buf)?;
        let achieved: u8 = Serializable::read_from(buf)?;
        let date_of_achieving: Option<i64> = if achieved != 0 {
            Serializable::read_from(buf)?
        } else {
            None
        };

        Ok(CriterionProgress {
            id,
            date_of_achieving,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.id.write_to(buf)?;
        self.date_of_achieving.write_to(buf)
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
            key: Serializable::read_from(buf)?,
            value: Serializable::read_from(buf)?,
            modifiers: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.key.write_to(buf)?;
        self.value.write_to(buf)?;
        self.modifiers.write_to(buf)
    }
}

#[derive(Debug, Default)]
pub struct EntityProperty_i16 {
    pub key: String,
    pub value: f64,
    pub modifiers: LenPrefixed<i16, PropertyModifier>,
}

impl Serializable for EntityProperty_i16 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(EntityProperty_i16 {
            key: Serializable::read_from(buf)?,
            value: Serializable::read_from(buf)?,
            modifiers: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.key.write_to(buf)?;
        self.value.write_to(buf)?;
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
            uuid: Serializable::read_from(buf)?,
            amount: Serializable::read_from(buf)?,
            operation: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        self.uuid.write_to(buf)?;
        self.amount.write_to(buf)?;
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
            action: Serializable::read_from(buf)?,
            players: Vec::new(),
        };
        let len = VarInt::read_from(buf)?;
        for _ in 0..len.0 {
            let uuid = UUID::read_from(buf)?;
            match m.action.0 {
                0 => {
                    let name = String::read_from(buf)?;
                    let mut props = Vec::new();
                    let plen = VarInt::read_from(buf)?.0;
                    for _ in 0..plen {
                        let mut prop = PlayerProperty {
                            name: String::read_from(buf)?,
                            value: String::read_from(buf)?,
                            signature: Default::default(),
                        };
                        if bool::read_from(buf)? {
                            prop.signature = Some(String::read_from(buf)?);
                        }
                        props.push(prop);
                    }
                    let p = PlayerDetail::Add {
                        uuid,
                        name,
                        properties: props,
                        gamemode: Serializable::read_from(buf)?,
                        ping: Serializable::read_from(buf)?,
                        display: {
                            if bool::read_from(buf)? {
                                Some(Serializable::read_from(buf)?)
                            } else {
                                None
                            }
                        },
                    };
                    m.players.push(p);
                }
                1 => m.players.push(PlayerDetail::UpdateGamemode {
                    uuid,
                    gamemode: Serializable::read_from(buf)?,
                }),
                2 => m.players.push(PlayerDetail::UpdateLatency {
                    uuid,
                    ping: Serializable::read_from(buf)?,
                }),
                3 => m.players.push(PlayerDetail::UpdateDisplayName {
                    uuid,
                    display: {
                        if bool::read_from(buf)? {
                            Some(Serializable::read_from(buf)?)
                        } else {
                            None
                        }
                    },
                }),
                4 => m.players.push(PlayerDetail::Remove { uuid }),
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

use crate::item;
type RecipeIngredient = LenPrefixed<VarInt, Option<item::Stack>>;

#[derive(Debug)]
pub enum RecipeData {
    Shapeless {
        group: String,
        ingredients: LenPrefixed<VarInt, RecipeIngredient>,
        result: Option<item::Stack>,
    },
    Shaped {
        width: VarInt,
        height: VarInt,
        group: String,
        ingredients: Vec<RecipeIngredient>,
        result: Option<item::Stack>,
    },
    ArmorDye,
    BookCloning,
    MapCloning,
    MapExtending,
    FireworkRocket,
    FireworkStar,
    FireworkStarFade,
    RepairItem,
    TippedArrow,
    BannerDuplicate,
    BannerAddPattern,
    ShieldDecoration,
    ShulkerBoxColoring,
    SuspiciousStew,
    Smelting {
        group: String,
        ingredient: RecipeIngredient,
        result: Option<item::Stack>,
        experience: f32,
        cooking_time: VarInt,
    },
    Blasting {
        group: String,
        ingredient: RecipeIngredient,
        result: Option<item::Stack>,
        experience: f32,
        cooking_time: VarInt,
    },
    Smoking {
        group: String,
        ingredient: RecipeIngredient,
        result: Option<item::Stack>,
        experience: f32,
        cooking_time: VarInt,
    },
    Campfire {
        group: String,
        ingredient: RecipeIngredient,
        result: Option<item::Stack>,
        experience: f32,
        cooking_time: VarInt,
    },
    Stonecutting {
        group: String,
        ingredient: RecipeIngredient,
        result: Option<item::Stack>,
    },
    Smithing {
        base: RecipeIngredient,
        addition: RecipeIngredient,
        result: Option<item::Stack>,
    },
}

impl Default for RecipeData {
    fn default() -> Self {
        RecipeData::ArmorDye
    }
}

#[derive(Debug, Default)]
pub struct Recipe {
    pub id: String,
    pub ty: String,
    pub data: RecipeData,
}

impl Serializable for Recipe {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let (id, ty) = {
            let a = String::read_from(buf)?;
            let b = String::read_from(buf)?;

            let protocol_version = super::current_protocol_version();

            // 1.14+ swaps recipe identifier and type, and adds namespace to type
            if protocol_version >= 477 {
                let ty = a;
                let id = b;

                if ty.find(':').is_some() {
                    (id, ty)
                } else {
                    (id, format!("minecraft:{}", ty))
                }
            } else {
                let ty = b;
                let id = a;
                (id, format!("minecraft:{}", ty))
            }
        };

        let data = match ty.as_ref() {
            "minecraft:crafting_shapeless" => RecipeData::Shapeless {
                group: Serializable::read_from(buf)?,
                ingredients: Serializable::read_from(buf)?,
                result: Serializable::read_from(buf)?,
            },
            "minecraft:crafting_shaped" => {
                let width: VarInt = Serializable::read_from(buf)?;
                let height: VarInt = Serializable::read_from(buf)?;
                let group: String = Serializable::read_from(buf)?;

                let capacity = width.0 as usize * height.0 as usize;

                let mut ingredients = Vec::with_capacity(capacity);
                for _ in 0..capacity {
                    ingredients.push(Serializable::read_from(buf)?);
                }
                let result: Option<item::Stack> = Serializable::read_from(buf)?;

                RecipeData::Shaped {
                    width,
                    height,
                    group,
                    ingredients,
                    result,
                }
            }
            "minecraft:crafting_special_armordye" => RecipeData::ArmorDye,
            "minecraft:crafting_special_bookcloning" => RecipeData::BookCloning,
            "minecraft:crafting_special_mapcloning" => RecipeData::MapCloning,
            "minecraft:crafting_special_mapextending" => RecipeData::MapExtending,
            "minecraft:crafting_special_firework_rocket" => RecipeData::FireworkRocket,
            "minecraft:crafting_special_firework_star" => RecipeData::FireworkStar,
            "minecraft:crafting_special_firework_star_fade" => RecipeData::FireworkStarFade,
            "minecraft:crafting_special_repairitem" => RecipeData::RepairItem,
            "minecraft:crafting_special_tippedarrow" => RecipeData::TippedArrow,
            "minecraft:crafting_special_bannerduplicate" => RecipeData::BannerDuplicate,
            "minecraft:crafting_special_banneraddpattern" => RecipeData::BannerAddPattern,
            "minecraft:crafting_special_shielddecoration" => RecipeData::ShieldDecoration,
            "minecraft:crafting_special_shulkerboxcoloring" => RecipeData::ShulkerBoxColoring,
            "minecraft:crafting_special_suspiciousstew" => RecipeData::SuspiciousStew,
            "minecraft:smelting" => RecipeData::Smelting {
                group: Serializable::read_from(buf)?,
                ingredient: Serializable::read_from(buf)?,
                result: Serializable::read_from(buf)?,
                experience: Serializable::read_from(buf)?,
                cooking_time: Serializable::read_from(buf)?,
            },
            "minecraft:blasting" => RecipeData::Blasting {
                group: Serializable::read_from(buf)?,
                ingredient: Serializable::read_from(buf)?,
                result: Serializable::read_from(buf)?,
                experience: Serializable::read_from(buf)?,
                cooking_time: Serializable::read_from(buf)?,
            },
            "minecraft:smoking" => RecipeData::Smoking {
                group: Serializable::read_from(buf)?,
                ingredient: Serializable::read_from(buf)?,
                result: Serializable::read_from(buf)?,
                experience: Serializable::read_from(buf)?,
                cooking_time: Serializable::read_from(buf)?,
            },
            "minecraft:campfire" | "minecraft:campfire_cooking" => RecipeData::Campfire {
                group: Serializable::read_from(buf)?,
                ingredient: Serializable::read_from(buf)?,
                result: Serializable::read_from(buf)?,
                experience: Serializable::read_from(buf)?,
                cooking_time: Serializable::read_from(buf)?,
            },
            "minecraft:stonecutting" => RecipeData::Stonecutting {
                group: Serializable::read_from(buf)?,
                ingredient: Serializable::read_from(buf)?,
                result: Serializable::read_from(buf)?,
            },
            "minecraft:smithing" => RecipeData::Smithing {
                base: Serializable::read_from(buf)?,
                addition: Serializable::read_from(buf)?,
                result: Serializable::read_from(buf)?,
            },
            _ => panic!("unrecognized recipe type: {}", ty),
        };

        Ok(Recipe { id, ty, data })
    }

    fn write_to<W: io::Write>(&self, _: &mut W) -> Result<(), Error> {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
pub struct Tags {
    pub tag_name: String,
    pub entries: LenPrefixed<VarInt, VarInt>,
}

impl Serializable for Tags {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        Ok(Tags {
            tag_name: Serializable::read_from(buf)?,
            entries: Serializable::read_from(buf)?,
        })
    }

    fn write_to<W: io::Write>(&self, _: &mut W) -> Result<(), Error> {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
pub struct Trade {
    pub input_item_1: Option<nbt::NamedTag>,
    pub output_item: Option<nbt::NamedTag>,
    pub has_second_item: bool,
    pub input_item_2: Option<nbt::NamedTag>,
    pub trades_disabled: bool,
    pub tool_uses: i32,
    pub max_trade_uses: i32,
    pub xp: i32,
    pub special_price: i32,
    pub price_multiplier: f32,
    pub demand: Option<i32>,
}

impl Serializable for Trade {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let protocol_version = super::current_protocol_version();

        Ok(Trade {
            input_item_1: Serializable::read_from(buf)?,
            output_item: Serializable::read_from(buf)?,
            has_second_item: Serializable::read_from(buf)?,
            input_item_2: Serializable::read_from(buf)?,
            trades_disabled: Serializable::read_from(buf)?,
            tool_uses: Serializable::read_from(buf)?,
            max_trade_uses: Serializable::read_from(buf)?,
            xp: Serializable::read_from(buf)?,
            special_price: Serializable::read_from(buf)?,
            price_multiplier: Serializable::read_from(buf)?,
            demand: if protocol_version >= 498 {
                Some(Serializable::read_from(buf)?)
            } else {
                None
            },
        })
    }

    fn write_to<W: io::Write>(&self, _: &mut W) -> Result<(), Error> {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
pub struct CommandNode {
    pub flags: u8,
    pub children: LenPrefixed<VarInt, VarInt>,
    pub redirect_node: Option<VarInt>,
    pub name: Option<String>,
    pub parser: Option<String>,
    pub properties: Option<CommandProperty>,
    pub suggestions_type: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
enum CommandNodeType {
    Root,
    Literal,
    Argument,
}

#[derive(Debug)]
pub enum CommandProperty {
    Bool,
    Double {
        flags: u8,
        min: Option<f64>,
        max: Option<f64>,
    },
    Float {
        flags: u8,
        min: Option<f32>,
        max: Option<f32>,
    },
    Integer {
        flags: u8,
        min: Option<i32>,
        max: Option<i32>,
    },
    String {
        token_type: VarInt,
    },
    Entity {
        flags: u8,
    },
    GameProfile,
    BlockPos,
    ColumnPos,
    Time,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
    Component,
    Message,
    Nbt,
    NbtPath,
    NbtTag,
    NbtCompoundTag,
    Objective,
    ObjectiveCriteria,
    Operation,
    Particle,
    Rotation,
    ScoreboardSlot,
    ScoreHolder {
        flags: u8,
    },
    Swizzle,
    Team,
    ItemSlot,
    ResourceLocation,
    MobEffect,
    Function,
    EntityAnchor,
    Range {
        decimals: bool,
    },
    IntRange,
    FloatRange,
    ItemEnchantment,
    EntitySummon,
    Dimension,
    UUID,
}

impl Serializable for CommandNode {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let flags: u8 = Serializable::read_from(buf)?;
        let children: LenPrefixed<VarInt, VarInt> = Serializable::read_from(buf)?;

        let node_type = match flags & 0x03 {
            0 => CommandNodeType::Root,
            1 => CommandNodeType::Literal,
            2 => CommandNodeType::Argument,
            _ => panic!("unrecognized command node type {}", flags & 0x03),
        };
        let _is_executable = flags & 0x04 != 0;
        let has_redirect = flags & 0x08 != 0;
        let has_suggestions_type = flags & 0x10 != 0;

        let redirect_node: Option<VarInt> = if has_redirect {
            Some(Serializable::read_from(buf)?)
        } else {
            None
        };

        let name: Option<String> =
            if node_type == CommandNodeType::Argument || node_type == CommandNodeType::Literal {
                Serializable::read_from(buf)?
            } else {
                None
            };
        let parser: Option<String> = if node_type == CommandNodeType::Argument {
            Serializable::read_from(buf)?
        } else {
            None
        };

        let properties: Option<CommandProperty> = if let Some(ref parse) = parser {
            Some(match parse.as_ref() {
                "brigadier:bool" => CommandProperty::Bool,
                "brigadier:double" => {
                    let flags = Serializable::read_from(buf)?;
                    let min = if flags & 0x01 != 0 {
                        Some(Serializable::read_from(buf)?)
                    } else {
                        None
                    };
                    let max = if flags & 0x02 != 0 {
                        Some(Serializable::read_from(buf)?)
                    } else {
                        None
                    };
                    CommandProperty::Double { flags, min, max }
                }
                "brigadier:float" => {
                    let flags = Serializable::read_from(buf)?;
                    let min = if flags & 0x01 != 0 {
                        Some(Serializable::read_from(buf)?)
                    } else {
                        None
                    };
                    let max = if flags & 0x02 != 0 {
                        Some(Serializable::read_from(buf)?)
                    } else {
                        None
                    };
                    CommandProperty::Float { flags, min, max }
                }
                "brigadier:integer" => {
                    let flags = Serializable::read_from(buf)?;
                    let min = if flags & 0x01 != 0 {
                        Some(Serializable::read_from(buf)?)
                    } else {
                        None
                    };
                    let max = if flags & 0x02 != 0 {
                        Some(Serializable::read_from(buf)?)
                    } else {
                        None
                    };
                    CommandProperty::Integer { flags, min, max }
                }
                "brigadier:string" => CommandProperty::String {
                    token_type: Serializable::read_from(buf)?,
                },
                "minecraft:entity" => CommandProperty::Entity {
                    flags: Serializable::read_from(buf)?,
                },
                "minecraft:game_profile" => CommandProperty::GameProfile,
                "minecraft:block_pos" => CommandProperty::BlockPos,
                "minecraft:column_pos" => CommandProperty::ColumnPos,
                "minecraft:time" => CommandProperty::Time,
                "minecraft:vec3" => CommandProperty::Vec3,
                "minecraft:vec2" => CommandProperty::Vec2,
                "minecraft:block_state" => CommandProperty::BlockState,
                "minecraft:block_predicate" => CommandProperty::BlockPredicate,
                "minecraft:item_stack" => CommandProperty::ItemStack,
                "minecraft:item_predicate" => CommandProperty::ItemPredicate,
                "minecraft:color" => CommandProperty::Color,
                "minecraft:component" => CommandProperty::Component,
                "minecraft:message" => CommandProperty::Message,
                "minecraft:nbt" => CommandProperty::Nbt,
                "minecraft:nbt_path" => CommandProperty::NbtPath,
                "minecraft:nbt_tag" => CommandProperty::NbtTag,
                "minecraft:nbt_compound_tag" => CommandProperty::NbtCompoundTag,
                "minecraft:objective" => CommandProperty::Objective,
                "minecraft:objective_criteria" => CommandProperty::ObjectiveCriteria,
                "minecraft:operation" => CommandProperty::Operation,
                "minecraft:particle" => CommandProperty::Particle,
                "minecraft:rotation" => CommandProperty::Rotation,
                "minecraft:scoreboard_slot" => CommandProperty::ScoreboardSlot,
                "minecraft:score_holder" => CommandProperty::ScoreHolder {
                    flags: Serializable::read_from(buf)?,
                },
                "minecraft:swizzle" => CommandProperty::Swizzle,
                "minecraft:team" => CommandProperty::Team,
                "minecraft:item_slot" => CommandProperty::ItemSlot,
                "minecraft:resource_location" => CommandProperty::ResourceLocation,
                "minecraft:mob_effect" => CommandProperty::MobEffect,
                "minecraft:function" => CommandProperty::Function,
                "minecraft:entity_anchor" => CommandProperty::EntityAnchor,
                "minecraft:range" => CommandProperty::Range {
                    decimals: Serializable::read_from(buf)?,
                },
                "minecraft:int_range" => CommandProperty::IntRange,
                "minecraft:float_range" => CommandProperty::FloatRange,
                "minecraft:item_enchantment" => CommandProperty::ItemEnchantment,
                "minecraft:entity_summon" => CommandProperty::EntitySummon,
                "minecraft:dimension" => CommandProperty::Dimension,
                "minecraft:uuid" => CommandProperty::UUID,
                _ => panic!("unsupported command node parser {}", parse),
            })
        } else {
            None
        };

        let suggestions_type: Option<String> = if has_suggestions_type {
            Serializable::read_from(buf)?
        } else {
            None
        };

        Ok(CommandNode {
            flags,
            children,
            redirect_node,
            name,
            parser,
            properties,
            suggestions_type,
        })
    }

    fn write_to<W: io::Write>(&self, _: &mut W) -> Result<(), Error> {
        unimplemented!()
    }
}
