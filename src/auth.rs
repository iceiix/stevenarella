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

use console;
use std::marker::PhantomData;

pub const CL_USERNAME: console::CVar<String> = console::CVar {
    ty: PhantomData,
    name: "cl_username",
    description: r#"cl_username is the username that the client will use to connect
to servers."#,
    mutable: false,
    serializable: true,
    default: &|| "".to_owned(),
};

pub const CL_UUID: console::CVar<String> = console::CVar {
    ty: PhantomData,
    name: "cl_uuid",
    description: r#"cl_uuid is the uuid of the client. This is unique to a player
unlike their username."#,
    mutable: false,
    serializable: true,
    default: &|| "".to_owned(),
};

pub const AUTH_TOKEN: console::CVar<String> = console::CVar {
    ty: PhantomData,
    name: "auth_token",
    description: r#"auth_token is the token used for this session to auth to servers
or relogin to this account."#,
    mutable: false,
    serializable: true,
    default: &|| "".to_owned(),
};

pub const AUTH_CLIENT_TOKEN: console::CVar<String> = console::CVar {
    ty: PhantomData,
    name: "auth_client_token",
    description: r#"auth_client_token is a token that stays static between sessions.
Used to identify this client vs others."#,
    mutable: false,
    serializable: true,
    default: &|| "".to_owned(),
};

pub fn register_vars(console: &mut console::Console) {
    console.register(CL_USERNAME);
    console.register(CL_UUID);
    console.register(AUTH_TOKEN);
    console.register(AUTH_CLIENT_TOKEN);
}
