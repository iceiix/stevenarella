// Copyright 2015 Matthew Collins
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

use openssl;
use serde_json;
use hyper;

pub struct Profile {
    pub username: String,
    pub id: String,
    pub access_token: String,
}

const JOIN_URL: &'static str = "https://sessionserver.mojang.com/session/minecraft/join";

impl Profile {
    pub fn join_server(&self, server_id: &String, shared_key: &Vec<u8>, public_key: &Vec<u8>) {
        let mut sha1 = openssl::SHA1::new();
        sha1.update(server_id.as_bytes());
        sha1.update(&shared_key[..]);
        sha1.update(&public_key[..]);
        let mut hash = sha1.bytes();

        // Mojang uses a hex method which allows for
        // negatives so we have to account for that.
        let negative = hash[0] & 0x80 == 0x80;
        if negative {
            twos_compliment(&mut hash);
        }
        let hash_str = hash.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join("");
        let hash_val = hash_str.trim_matches('0');
        let hash_str = if negative {
            "-".to_owned() + &hash_val[..]
        } else {
            hash_val.to_owned()
        };

        let join_msg = serde_json::builder::ObjectBuilder::new()
                           .insert("accessToken", &self.access_token)
                           .insert("selectedProfile", &self.id)
                           .insert("serverId", hash_str)
                           .unwrap();
        let join = serde_json::to_string(&join_msg).unwrap();

        let client = hyper::Client::new();
        let res = client.post(JOIN_URL)
                        .body(&join)
                        .header(hyper::header::ContentType("application/json".parse().unwrap()))
                        .send()
                        .unwrap();

        let ret: serde_json::Value = match serde_json::from_reader(res) {
            Result::Ok(val) => val,
            Result::Err(_) => return,
        };
        panic!("{:?}", ret);
    }
}

fn twos_compliment(data: &mut Vec<u8>) {
    let mut carry = true;
    for i in (0..data.len()).rev() {
        data[i] = !data[i];
        if carry {
            carry = data[i] == 0xFF;
            data[i] += 1;
        }
    }
}
