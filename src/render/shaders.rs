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

use render::glsl;
use gl;

pub fn add_shaders(reg: &mut glsl::Registry) {
    reg.register("lookup_texture",
                 include_str!("shaders/lookup_texture.glsl"));
    reg.register("get_light", include_str!("shaders/get_light.glsl"));

    reg.register("ui_vertex", include_str!("shaders/ui_vertex.glsl"));
    reg.register("ui_frag", include_str!("shaders/ui_frag.glsl"));
}

#[macro_export]
macro_rules! init_shader {
	(
		Program $name:ident {
			vert = $vert:expr,
			frag = $frag:expr,
			attribute = {
				$(
					$field:ident => $glname:expr,
				)*
			},
			uniform = {
				$(
					$ufield:ident => $uglname:expr,
				)*
			},
		}
	) => (
		struct $name {
			program: gl::Program,
			$(
				$field: gl::Attribute,
			)+
			$(
				$ufield: gl::Uniform,
			)+
		}

		impl $name {
			pub fn new(reg: &glsl::Registry) -> $name {
				let v = reg.get($vert);
				let f = reg.get($frag);
				let shader = shaders::create_program(&v, &f);
				$name {
					$(
						$field: shader.attribute_location($glname),
					)+
					$(
						$ufield: shader.uniform_location($uglname),
					)+
					program: shader,
				}
			}
		}
	)
}

pub fn create_program(vertex: &str, fragment: &str) -> gl::Program {
    let program = gl::Program::new();

    let v = gl::Shader::new(gl::VERTEX_SHADER);
    v.set_source(vertex);
    v.compile();

    if v.get_parameter(gl::COMPILE_STATUS) == 0 {
        println!("Src: {}", vertex);
        panic!("Shader error: {}", v.get_info_log());
    } else {
        let log = v.get_info_log();
        if !log.is_empty() {
            println!("{}", log);
        }
    }

    let f = gl::Shader::new(gl::FRAGMENT_SHADER);
    f.set_source(fragment);
    f.compile();

    if f.get_parameter(gl::COMPILE_STATUS) == 0 {
        println!("Src: {}", fragment);
        panic!("Shader error: {}", f.get_info_log());
    } else {
        let log = f.get_info_log();
        if !log.is_empty() {
            println!("{}", log);
        }
    }

    program.attach_shader(v);
    program.attach_shader(f);
    program.link();
    program.use_program();
    program
}
