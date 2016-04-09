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

use render::glsl;
use gl;

pub fn add_shaders(reg: &mut glsl::Registry) {
    reg.register("lookup_texture", include_str!("shaders/lookup_texture.glsl"));
    reg.register("get_light", include_str!("shaders/get_light.glsl"));

    reg.register("ui_vertex", include_str!("shaders/ui_vertex.glsl"));
    reg.register("ui_frag", include_str!("shaders/ui_frag.glsl"));

    reg.register("chunk_vertex", include_str!("shaders/chunk_vertex.glsl"));
    reg.register("chunk_frag", include_str!("shaders/chunk_frag.glsl"));

    reg.register("trans_vertex", include_str!("shaders/trans_vertex.glsl"));
    reg.register("trans_frag", include_str!("shaders/trans_frag.glsl"));

    reg.register("model_vertex", include_str!("shaders/model_vertex.glsl"));
    reg.register("model_frag", include_str!("shaders/model_frag.glsl"));

    reg.register("sun_vertex", include_str!("shaders/sun_vertex.glsl"));
    reg.register("sun_frag", include_str!("shaders/sun_frag.glsl"));

    reg.register("clouds_vertex", include_str!("shaders/clouds_vertex.glsl"));
    reg.register("clouds_geo", include_str!("shaders/clouds_geo.glsl"));
    reg.register("clouds_frag", include_str!("shaders/clouds_frag.glsl"));
}

macro_rules! get_shader {
    ($reg:ident, $name:expr) => (
        $reg.get($name)
    );
    ($reg:ident, $name:expr, $def:expr) => (
        $reg.get_define($name, $def)
    )
}

#[macro_export]
macro_rules! init_shader {
    (
        Program $name:ident {
            vert = $vert:expr, $(#$vdef:ident)*
            frag = $frag:expr, $(#$fdef:ident)*
            attribute = {
                $(
                    required $field:ident => $glname:expr,
                )*
                $(
                    optional $ofield:ident => $oglname:expr,
                )*
            },
            uniform = {
                $(
                    required $ufield:ident => $uglname:expr,
                )*
                $(
                    optional $oufield:ident => $ouglname:expr,
                )*
            },
        }
    ) => (
        #[allow(dead_code)]
        struct $name {
            program: gl::Program,
            $(
                $field: gl::Attribute,
            )*
            $(
                $ofield: Option<gl::Attribute>,
            )*
            $(
                $ufield: gl::Uniform,
            )*
            $(
                $oufield: Option<gl::Uniform>,
            )*
        }

        impl $name {
            #[allow(dead_code)]
            pub fn new(reg: &glsl::Registry) -> $name {
                let v = get_shader!(reg, $vert $(,stringify!($vdef))*);
                let f = get_shader!(reg, $frag $(,stringify!($fdef))*);
                $name::new_manual(&v, &f)
            }

            #[allow(dead_code)]
            pub fn new_manual(v: &str, f: &str) -> $name {
                let shader = shaders::create_program(&v, &f);
                $name {
                    $(
                        $field: shader.attribute_location($glname).unwrap(),
                    )*
                    $(
                        $ofield: shader.attribute_location($oglname),
                    )*
                    $(
                        $ufield: shader.uniform_location($uglname).unwrap(),
                    )*
                    $(
                        $oufield: shader.uniform_location($ouglname),
                    )*
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
        let log = log.trim().trim_matches('\u{0}');
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
        let log = log.trim().trim_matches('\u{0}');
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
