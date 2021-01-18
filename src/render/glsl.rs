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

use std::collections::HashMap;

#[derive(Default)]
pub struct Registry {
    shaders: HashMap<String, String>,
    shader_version: String,
}

impl Registry {
    pub fn new(shader_version: &str) -> Registry {
        Registry {
            shaders: Default::default(),
            shader_version: shader_version.to_string(),
        }
    }

    pub fn register(&mut self, name: &str, source: &str) {
        if self.shaders.contains_key(name) {
            panic!("shader {} is already defined", name);
        }
        self.shaders
            .insert(name.to_owned(), source.trim().to_owned());
    }

    fn add_version(&self, out: &mut String) {
        out.push_str(&self.shader_version);
        out.push('\n');
        if self.shader_version.ends_with(" es") {
            out.push_str(
                r#"precision mediump float;
precision mediump sampler2DArray;
#define ES
"#,
            );
        }
    }

    pub fn get(&self, name: &str) -> String {
        let mut out = String::new();
        self.add_version(&mut out);
        self.get_internal(&mut out, name);
        out
    }

    pub fn get_define(&self, name: &str, define: &str) -> String {
        let mut out = String::new();
        self.add_version(&mut out);
        out.push_str("#define ");
        out.push_str(define);
        out.push('\n');
        self.get_internal(&mut out, name);
        out
    }

    fn get_internal(&self, out: &mut String, name: &str) {
        let src = self.shaders.get(name).unwrap();
        for line in src.lines() {
            if let Some(stripped) = line.strip_prefix("#include ") {
                let inc = stripped.trim();
                self.get_internal(out, inc);
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
    }
}
