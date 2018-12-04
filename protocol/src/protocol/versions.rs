use crate::protocol::*;

mod v1_12_2;
mod v1_11_2;
mod v1_10_2;
mod v1_9_2;
mod v1_9;

pub fn translate_internal_packet_id_for_version(version: i32, state: State, dir: Direction, id: i32, to_internal: bool) -> i32 {
    match version {
        // https://wiki.vg/Protocol_History
        // https://wiki.vg/Protocol_version_numbers#Versions_after_the_Netty_rewrite
        // 1.12.2
        340 => v1_12_2::translate_internal_packet_id(state, dir, id, to_internal),

        // 1.11.2
        316 => v1_11_2::translate_internal_packet_id(state, dir, id, to_internal),

        // 1.11
        315 => v1_11_2::translate_internal_packet_id(state, dir, id, to_internal),

        // 1.10.2
        210 => v1_10_2::translate_internal_packet_id(state, dir, id, to_internal),

        // 1.9.2
        109 => v1_9_2::translate_internal_packet_id(state, dir, id, to_internal),

        // 1.9
        107 => v1_9::translate_internal_packet_id(state, dir, id, to_internal),

        _ => panic!("unsupported protocol version"),
    }
}
