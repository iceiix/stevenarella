use super::*;

mod v15w39c;
mod v18w50a;
mod v19w02a;
mod v1_10_2;
mod v1_11_2;
mod v1_12_2;
mod v1_13_2;
mod v1_14;
mod v1_14_1;
mod v1_14_2;
mod v1_14_3;
mod v1_14_4;
mod v1_15;
mod v1_16_1;
mod v1_7_10;
mod v1_8_9;
mod v1_9;
mod v1_9_2;

// https://wiki.vg/Protocol_History
// https://wiki.vg/Protocol_version_numbers#Versions_after_the_Netty_rewrite

pub fn protocol_name_to_protocol_version(s: String) -> i32 {
    match s.as_ref() {
        "" => SUPPORTED_PROTOCOLS[0],
        "1.16.1" => 736,
        "1.16" => 735,
        "1.15.2" => 578,
        "1.15.1" => 575,
        "1.14.4" => 498,
        "1.14.3" => 490,
        "1.14.2" => 485,
        "1.14.1" => 480,
        "1.14" => 477,
        "19w02a" => 452,
        "18w50a" => 451,
        "1.13.2" => 404,
        "1.12.2" => 340,
        "1.11.2" => 316,
        "1.11" => 315,
        "1.10.2" => 210,
        "1.9.2" => 109,
        "1.9" => 107,
        "15w39c" => 74,
        "1.8.9" => 47,
        "1.7.10" => 5,
        _ => {
            if let Ok(n) = s.parse::<i32>() {
                n
            } else {
                panic!("Unrecognized protocol name: {}", s)
            }
        }
    }
}

pub fn translate_internal_packet_id_for_version(
    version: i32,
    state: State,
    dir: Direction,
    id: i32,
    to_internal: bool,
) -> i32 {
    match version {
        736 => v1_16_1::translate_internal_packet_id(state, dir, id, to_internal),
        735 => v1_16_1::translate_internal_packet_id(state, dir, id, to_internal),
        578 => v1_15::translate_internal_packet_id(state, dir, id, to_internal),
        575 => v1_15::translate_internal_packet_id(state, dir, id, to_internal),
        498 => v1_14_4::translate_internal_packet_id(state, dir, id, to_internal),
        490 => v1_14_3::translate_internal_packet_id(state, dir, id, to_internal),
        485 => v1_14_2::translate_internal_packet_id(state, dir, id, to_internal),
        480 => v1_14_1::translate_internal_packet_id(state, dir, id, to_internal),
        477 => v1_14::translate_internal_packet_id(state, dir, id, to_internal),
        452 => v19w02a::translate_internal_packet_id(state, dir, id, to_internal),
        451 => v18w50a::translate_internal_packet_id(state, dir, id, to_internal),
        404 => v1_13_2::translate_internal_packet_id(state, dir, id, to_internal),
        340 => v1_12_2::translate_internal_packet_id(state, dir, id, to_internal),
        316 => v1_11_2::translate_internal_packet_id(state, dir, id, to_internal),
        315 => v1_11_2::translate_internal_packet_id(state, dir, id, to_internal),
        210 => v1_10_2::translate_internal_packet_id(state, dir, id, to_internal),
        109 => v1_9_2::translate_internal_packet_id(state, dir, id, to_internal),
        107 => v1_9::translate_internal_packet_id(state, dir, id, to_internal),
        74 => v15w39c::translate_internal_packet_id(state, dir, id, to_internal),
        47 => v1_8_9::translate_internal_packet_id(state, dir, id, to_internal),
        5 => v1_7_10::translate_internal_packet_id(state, dir, id, to_internal),
        _ => panic!("unsupported protocol version: {}", version),
    }
}
