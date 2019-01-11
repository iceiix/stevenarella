use crate::protocol::*;

mod v18w50a;
mod v1_13_2;
mod v1_12_2;
mod v1_11_2;
mod v1_10_2;
mod v1_9_2;
mod v1_9;
mod v15w39c;
mod v1_8_9;
mod v1_7_10;

pub fn translate_internal_packet_id_for_version(version: i32, state: State, dir: Direction, id: i32, to_internal: bool) -> i32 {
    match version {
        // https://wiki.vg/Protocol_History
        // https://wiki.vg/Protocol_version_numbers#Versions_after_the_Netty_rewrite

        // 18w50a
        451 => v18w50a::translate_internal_packet_id(state, dir, id, to_internal),

        // 1.13.2
        404 => v1_13_2::translate_internal_packet_id(state, dir, id, to_internal),

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

        // 15w39a/b/c
        74 => v15w39c::translate_internal_packet_id(state, dir, id, to_internal),

        // 1.8.9 - 1.8
        47 => v1_8_9::translate_internal_packet_id(state, dir, id, to_internal),

        // 1.7.10 - 1.7.6
        5 => v1_7_10::translate_internal_packet_id(state, dir, id, to_internal),

        _ => panic!("unsupported protocol version"),
    }
}
