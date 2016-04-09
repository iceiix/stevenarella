
use std::sync::{Arc, RwLock};

use cgmath::{Point3, Matrix4};
use byteorder::{WriteBytesExt, NativeEndian};
use gl;
use super::glsl;

pub struct Clouds {
    program: gl::Program,
    // Shader props
    _a_position: gl::Attribute,
    u_perspective_matrix: gl::Uniform,
    u_camera_matrix: gl::Uniform,
    u_light_level: gl::Uniform,
    u_sky_offset: gl::Uniform,
    u_offset: gl::Uniform,
    u_texture_info: gl::Uniform,
    u_atlas: gl::Uniform,
    u_textures: gl::Uniform,
    u_cloud_map: gl::Uniform,
    u_cloud_offset: gl::Uniform,

    array: gl::VertexArray,
    _buffer: gl::Buffer,

    textures: Arc<RwLock<super::TextureManager>>,

    texture: gl::Texture,
    pub heightmap_data: Vec<u8>,
    pub dirty: bool,

    offset: f64,
    num_points: usize,
}

impl Clouds {
    pub fn new(greg: &glsl::Registry, textures: Arc<RwLock<super::TextureManager>>) -> Clouds {
        let program = gl::Program::new();

        let vertex = greg.get("clouds_vertex");
        let geo = greg.get("clouds_geo");
        let fragment = greg.get("clouds_frag");

        let v = gl::Shader::new(gl::VERTEX_SHADER);
        v.set_source(&vertex);
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

        let g = gl::Shader::new(gl::GEOMETRY_SHADER);
        g.set_source(&geo);
        g.compile();

        if g.get_parameter(gl::COMPILE_STATUS) == 0 {
            println!("Src: {}", geo);
            panic!("Shader error: {}", g.get_info_log());
        } else {
            let log = g.get_info_log();
            let log = log.trim().trim_matches('\u{0}');
            if !log.is_empty() {
                println!("{}", log);
            }
        }

        let f = gl::Shader::new(gl::FRAGMENT_SHADER);
        f.set_source(&fragment);
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
        program.attach_shader(g);
        program.attach_shader(f);
        program.link();
        program.use_program();

        let a_position = program.attribute_location("aPosition").unwrap();
        let u_perspective_matrix = program.uniform_location("perspectiveMatrix").unwrap();
        let u_camera_matrix = program.uniform_location("cameraMatrix").unwrap();
        let u_light_level = program.uniform_location("lightLevel").unwrap();
        let u_sky_offset = program.uniform_location("skyOffset").unwrap();
        let u_offset = program.uniform_location("offset").unwrap();
        let u_texture_info = program.uniform_location("textureInfo").unwrap();
        let u_atlas = program.uniform_location("atlas").unwrap();
        let u_textures = program.uniform_location("textures").unwrap();
        let u_cloud_map = program.uniform_location("cloudMap").unwrap();
        let u_cloud_offset = program.uniform_location("cloudOffset").unwrap();

        let array = gl::VertexArray::new();
        array.bind();
        let buffer = gl::Buffer::new();
        buffer.bind(gl::ARRAY_BUFFER);
        a_position.enable();
        a_position.vertex_pointer(3, gl::FLOAT, false, 12, 0);

        let mut data = vec![];
        let mut num_points = 0;
        for x in -160 .. 160 {
            for z in -160 .. 160 {
                let _ = data.write_f32::<NativeEndian>(x as f32);
                let _ = data.write_f32::<NativeEndian>(128.0);
                let _ = data.write_f32::<NativeEndian>(z as f32);
                num_points += 1;
            }
        }

        buffer.set_data(gl::ARRAY_BUFFER, &data, gl::STATIC_DRAW);

        let heightmap_data = vec![0; 512*512];

        let texture = gl::Texture::new();
        texture.bind(gl::TEXTURE_2D);
        texture.image_2d(gl::TEXTURE_2D, 0, 512, 512, gl::RED, gl::UNSIGNED_BYTE, Some(&heightmap_data));
        texture.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
        texture.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST);

        Clouds {
            program: program,
            // Shader props
            _a_position: a_position,
            u_perspective_matrix: u_perspective_matrix,
            u_camera_matrix: u_camera_matrix,
            u_light_level: u_light_level,
            u_sky_offset: u_sky_offset,
            u_offset: u_offset,
            u_texture_info: u_texture_info,
            u_atlas: u_atlas,
            u_textures: u_textures,
            u_cloud_map: u_cloud_map,
            u_cloud_offset: u_cloud_offset,

            array: array,
            _buffer: buffer,

            textures: textures,

            texture: texture,
            heightmap_data: heightmap_data,
            dirty: false,

            offset: 0.0,
            num_points: num_points,
        }
    }

    pub fn draw(&mut self, camera_pos: &Point3<f64>, perspective_matrix: &Matrix4<f32>, camera_matrix: &Matrix4<f32>, light_level: f32, sky_offset: f32, delta: f64) {
        self.offset += delta;

        let tex = super::Renderer::get_texture(&self.textures, "steven:environment/clouds");

        self.program.use_program();
        self.u_perspective_matrix.set_matrix4(perspective_matrix);
        self.u_camera_matrix.set_matrix4(camera_matrix);
        self.u_sky_offset.set_float(sky_offset);
        self.u_light_level.set_float(light_level);
        self.u_offset.set_float3(camera_pos.x.floor() as f32, 0.0, camera_pos.z.floor() as f32);
        self.u_texture_info.set_float4(
            tex.get_x() as f32,
            tex.get_y() as f32,
            tex.get_width() as f32,
            tex.get_height() as f32
        );
        self.u_atlas.set_float(tex.atlas as f32);
        self.u_cloud_offset.set_float((self.offset / 60.0) as f32);
        self.u_textures.set_int(0);

        gl::active_texture(1);
        self.texture.bind(gl::TEXTURE_2D);
        if self.dirty {
            self.texture.sub_image_2d(gl::TEXTURE_2D, 0, 0, 0, 512, 512, gl::RED, gl::UNSIGNED_BYTE, &self.heightmap_data);
            self.dirty = false;
        }
        self.u_cloud_map.set_int(1);
        self.array.bind();
        gl::draw_arrays(gl::POINTS, 0, self.num_points);
    }
}
