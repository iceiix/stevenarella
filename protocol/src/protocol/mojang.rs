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

#[cfg(not(target_arch = "wasm32"))]
use serde_json::json;
use sha1::{self, Digest};

#[derive(Clone, Debug)]
pub struct Profile {
    pub username: String,
    pub id: String,
    pub access_token: String,
}

const JOIN_URL: &str = "https://sessionserver.mojang.com/session/minecraft/join";
const LOGIN_URL: &str = "https://authserver.mojang.com/authenticate";
const REFRESH_URL: &str = "https://authserver.mojang.com/refresh";
const VALIDATE_URL: &str = "https://authserver.mojang.com/validate";

#[cfg(not(target_arch = "wasm32"))]
impl Profile {
    pub fn login(username: &str, password: &str, token: &str) -> Result<Profile, super::Error> {
        let req_msg = json!({
        "username": username,
        "password": password,
        "clientToken": token,
        "agent": {
            "name": "Minecraft",
            "version": 1
        }});
        let req = serde_json::to_string(&req_msg)?;

        let client = reqwest::blocking::Client::new();
        let res = client
            .post(LOGIN_URL)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(req)
            .send()?;

        let ret: serde_json::Value = serde_json::from_reader(res)?;
        if let Some(error) = ret.get("error").and_then(|v| v.as_str()) {
            return Err(super::Error::Err(format!(
                "{}: {}",
                error,
                ret.get("errorMessage").and_then(|v| v.as_str()).unwrap()
            )));
        }
        Ok(Profile {
            username: ret
                .pointer("/selectedProfile/name")
                .and_then(|v| v.as_str())
                .unwrap()
                .to_owned(),
            id: ret
                .pointer("/selectedProfile/id")
                .and_then(|v| v.as_str())
                .unwrap()
                .to_owned(),
            access_token: ret
                .get("accessToken")
                .and_then(|v| v.as_str())
                .unwrap()
                .to_owned(),
        })
    }

    pub fn refresh(self, token: &str) -> Result<Profile, super::Error> {
        let req_msg = json!({
        "accessToken": self.access_token,
        "clientToken": token
        });
        let req = serde_json::to_string(&req_msg)?;

        let client = reqwest::blocking::Client::new();
        let res = client
            .post(VALIDATE_URL)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(req)
            .send()?;

        if res.status() != reqwest::StatusCode::NO_CONTENT {
            let req = serde_json::to_string(&req_msg)?; // TODO: fix parsing twice to avoid move
                                                        // Refresh needed
            let res = client
                .post(REFRESH_URL)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(req)
                .send()?;

            let ret: serde_json::Value = serde_json::from_reader(res)?;
            if let Some(error) = ret.get("error").and_then(|v| v.as_str()) {
                return Err(super::Error::Err(format!(
                    "{}: {}",
                    error,
                    ret.get("errorMessage").and_then(|v| v.as_str()).unwrap()
                )));
            }
            return Ok(Profile {
                username: ret
                    .pointer("/selectedProfile/name")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_owned(),
                id: ret
                    .pointer("/selectedProfile/id")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_owned(),
                access_token: ret
                    .get("accessToken")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_owned(),
            });
        }
        Ok(self)
    }

    pub fn join_server(
        &self,
        server_id: &str,
        shared_key: &[u8],
        public_key: &[u8],
    ) -> Result<(), super::Error> {
        let mut hasher = sha1::Sha1::new();
        hasher.update(server_id.as_bytes());
        hasher.update(shared_key);
        hasher.update(public_key);
        let mut hash = hasher.finalize();

        // Mojang uses a hex method which allows for
        // negatives so we have to account for that.
        let negative = (hash[0] & 0x80) == 0x80;
        if negative {
            twos_compliment(&mut hash);
        }
        let hash_str = hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("");
        let hash_val = hash_str.trim_start_matches('0');
        let hash_str = if negative {
            "-".to_owned() + &hash_val[..]
        } else {
            hash_val.to_owned()
        };

        let join_msg = json!({
            "accessToken": &self.access_token,
            "selectedProfile": &self.id,
            "serverId": hash_str
        });
        let join = serde_json::to_string(&join_msg).unwrap();

        let client = reqwest::blocking::Client::new();
        let res = client
            .post(JOIN_URL)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(join)
            .send()?;

        if res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(super::Error::Err("Failed to auth with server".to_owned()))
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.username.is_empty() && !self.id.is_empty() && !self.access_token.is_empty()
    }
}

fn twos_compliment(data: &mut [u8]) {
    let mut carry = true;
    for i in (0..data.len()).rev() {
        data[i] = !data[i];
        if carry {
            carry = data[i] == 0xFF;
            data[i] = data[i].wrapping_add(1);
        }
    }
}
