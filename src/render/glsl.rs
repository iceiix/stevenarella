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

pub struct Registry {
    shaders: HashMap<String, String>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry { shaders: HashMap::new() }
    }

    pub fn register(&mut self, name: &str, source: &str) {
        if self.shaders.contains_key(name) {
            panic!("shader {} is already defined", name);
        }
        self.shaders.insert(name.to_owned(), source.trim().to_owned());
    }

    pub fn get(&self, name: &str) -> String {
        let mut out = String::new();
        out.push_str("#version 150\n");
        self.get_internal(&mut out, name);
        out
    }

    pub fn get_define(&self, name: &str, define: &str) -> String {
        let mut out = String::new();
        out.push_str("#version 150\n");
        out.push_str("#define ");
        out.push_str(define);
        out.push_str("\n");
        self.get_internal(&mut out, name);
        out
    }

    fn get_internal(&self, out: &mut String, name: &str) {
        let src = self.shaders.get(name).unwrap();
        for line in src.lines() {
            if line.starts_with("#include ") {
                let inc = line["#include ".len()..].trim();
                self.get_internal(out, &inc);
                continue;
            }
            out.push_str(&line);
            out.push_str("\n");
        }
    }
}
