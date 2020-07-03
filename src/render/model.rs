use super::glsl;
use super::shaders;
use crate::format::{self, Component};
use crate::gl;
use crate::model::BlockVertex;
use crate::shared::Direction;
use crate::types::hash::FNVHash;
use byteorder::{NativeEndian, WriteBytesExt};
use cgmath::{Matrix4, Point3, SquareMatrix};
use collision::{self, Frustum, Sphere};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::{Arc, RwLock};

pub struct Manager {
    collections: Vec<Collection>,

    index_buffer: gl::Buffer,
    index_type: gl::Type,
    max_index: usize,
}

pub const DEFAULT: CollectionKey = CollectionKey(0);
pub const SUN: CollectionKey = CollectionKey(1);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollectionKey(usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelKey(CollectionKey, usize);

impl Manager {
    pub fn new(greg: &glsl::Registry) -> Manager {
        let mut m = Manager {
            collections: vec![],

            index_buffer: gl::Buffer::new(),
            index_type: gl::UNSIGNED_SHORT,
            max_index: 0,
        };
        m.add_collection(
            &greg.get("model_vertex"),
            &greg.get("model_frag"),
            gl::SRC_ALPHA,
            gl::ONE_MINUS_SRC_ALPHA,
        );
        m.add_collection(
            &greg.get("sun_vertex"),
            &greg.get("sun_frag"),
            gl::SRC_ALPHA,
            gl::ONE_FACTOR,
        );
        m
    }

    pub fn add_collection(
        &mut self,
        vert: &str,
        frag: &str,
        blend_s: gl::Factor,
        blend_d: gl::Factor,
    ) -> CollectionKey {
        let collection = Collection {
            shader: ModelShader::new_manual(vert, frag),
            models: HashMap::with_hasher(BuildHasherDefault::default()),
            blend_s,
            blend_d,
            next_id: 0,
        };
        self.collections.push(collection);
        CollectionKey(self.collections.len())
    }

    pub fn get_model(&mut self, key: ModelKey) -> Option<&mut Model> {
        let collection = &mut self.collections[(key.0).0];
        collection.models.get_mut(&key)
    }

    pub fn create_model(&mut self, ckey: CollectionKey, parts: Vec<Vec<Vertex>>) -> ModelKey {
        let array = gl::VertexArray::new();
        array.bind();
        self.index_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
        let buffer = gl::Buffer::new();
        buffer.bind(gl::ARRAY_BUFFER);

        let mut model = {
            let collection = &mut self.collections[ckey.0];
            collection.shader.program.use_program();
            if let Some(v) = collection.shader.position {
                v.enable()
            }
            if let Some(v) = collection.shader.texture_info {
                v.enable()
            }
            if let Some(v) = collection.shader.texture_offset {
                v.enable()
            }
            if let Some(v) = collection.shader.color {
                v.enable()
            }
            if let Some(v) = collection.shader.id {
                v.enable()
            }
            if let Some(v) = collection.shader.position {
                v.vertex_pointer(3, gl::FLOAT, false, 36, 0)
            }
            if let Some(v) = collection.shader.texture_info {
                v.vertex_pointer(4, gl::UNSIGNED_SHORT, false, 36, 12)
            }
            if let Some(v) = collection.shader.texture_offset {
                v.vertex_pointer_int(3, gl::SHORT, 36, 20)
            }
            if let Some(v) = collection.shader.color {
                v.vertex_pointer(4, gl::UNSIGNED_BYTE, true, 36, 28)
            }
            if let Some(v) = collection.shader.id {
                v.vertex_pointer_int(1, gl::UNSIGNED_BYTE, 36, 32)
            }

            let mut model = Model {
                // For culling only
                x: 0.0,
                y: 0.0,
                z: 0.0,
                radius: 0.0,
                // Per a part
                matrix: Vec::with_capacity(parts.len()),
                colors: Vec::with_capacity(parts.len()),
                block_light: 15.0,
                sky_light: 15.0,

                array,
                buffer,
                buffer_size: 0,
                count: 0,

                verts: vec![],
            };

            for (i, part) in parts.into_iter().enumerate() {
                model.matrix.push(Matrix4::identity());
                model.colors.push([1.0, 1.0, 1.0, 1.0]);
                for mut pp in part {
                    pp.id = i as u8;
                    model.verts.push(pp);
                }
            }
            model
        };

        Self::rebuild_model(&mut model);
        if self.max_index < model.count as usize {
            let (data, ty) = super::generate_element_buffer(model.count as usize);
            self.index_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
            self.index_buffer
                .set_data(gl::ELEMENT_ARRAY_BUFFER, &data, gl::DYNAMIC_DRAW);
            self.max_index = model.count as usize;
            self.index_type = ty;
        }

        let collection = &mut self.collections[ckey.0];
        let key = ModelKey(ckey, collection.next_id);
        collection.next_id += 1;
        collection.models.insert(key, model);

        key
    }

    pub fn remove_model(&mut self, key: ModelKey) {
        let collection = &mut self.collections[(key.0).0];
        collection.models.remove(&key);
    }

    fn rebuild_model(model: &mut Model) {
        model.array.bind();
        model.count = ((model.verts.len() / 4) * 6) as i32;

        let mut buffer = Vec::with_capacity(36 * model.verts.len());
        for vert in &model.verts {
            let _ = buffer.write_f32::<NativeEndian>(vert.x);
            let _ = buffer.write_f32::<NativeEndian>(vert.y);
            let _ = buffer.write_f32::<NativeEndian>(vert.z);
            let _ = buffer.write_u16::<NativeEndian>(vert.texture.get_x() as u16);
            let _ = buffer.write_u16::<NativeEndian>(vert.texture.get_y() as u16);
            let _ = buffer.write_u16::<NativeEndian>(vert.texture.get_width() as u16);
            let _ = buffer.write_u16::<NativeEndian>(vert.texture.get_height() as u16);
            let _ = buffer.write_i16::<NativeEndian>(
                ((vert.texture.get_width() as f64) * 16.0 * vert.texture_x) as i16,
            );
            let _ = buffer.write_i16::<NativeEndian>(
                ((vert.texture.get_height() as f64) * 16.0 * vert.texture_y) as i16,
            );
            let _ = buffer.write_i16::<NativeEndian>(vert.texture.atlas as i16);
            let _ = buffer.write_i16::<NativeEndian>(0);
            let _ = buffer.write_u8(vert.r);
            let _ = buffer.write_u8(vert.g);
            let _ = buffer.write_u8(vert.b);
            let _ = buffer.write_u8(vert.a);
            let _ = buffer.write_u8(vert.id);
            let _ = buffer.write_u8(0);
            let _ = buffer.write_u8(0);
            let _ = buffer.write_u8(0);
        }

        model.buffer.bind(gl::ARRAY_BUFFER);
        if buffer.len() < model.buffer_size {
            model.buffer.re_set_data(gl::ARRAY_BUFFER, &buffer);
        } else {
            model
                .buffer
                .set_data(gl::ARRAY_BUFFER, &buffer, gl::DYNAMIC_DRAW);
            model.buffer_size = buffer.len();
        }
    }

    pub fn rebuild_models(
        &mut self,
        version: usize,
        textures: &Arc<RwLock<super::TextureManager>>,
    ) {
        for collection in &mut self.collections {
            for model in collection.models.values_mut() {
                for vert in &mut model.verts {
                    vert.texture = if vert.texture.version == version {
                        vert.texture.clone()
                    } else {
                        let mut new = super::Renderer::get_texture(textures, &vert.texture.name);
                        new.rel_x = vert.texture.rel_x;
                        new.rel_y = vert.texture.rel_y;
                        new.rel_width = vert.texture.rel_width;
                        new.rel_height = vert.texture.rel_height;
                        new.is_rel = vert.texture.is_rel;
                        new
                    };
                }
                Self::rebuild_model(model);
            }
        }
    }

    pub fn draw(
        &mut self,
        frustum: &Frustum<f32>,
        perspective_matrix: &Matrix4<f32>,
        camera_matrix: &Matrix4<f32>,
        light_level: f32,
        sky_offset: f32,
    ) {
        gl::enable(gl::BLEND);
        for collection in &self.collections {
            collection.shader.program.use_program();
            if let Some(v) = collection.shader.perspective_matrix {
                v.set_matrix4(perspective_matrix)
            }
            if let Some(v) = collection.shader.camera_matrix {
                v.set_matrix4(camera_matrix)
            }
            if let Some(v) = collection.shader.texture {
                v.set_int(0)
            }
            if let Some(v) = collection.shader.sky_offset {
                v.set_float(sky_offset)
            }
            if let Some(v) = collection.shader.light_level {
                v.set_float(light_level)
            }
            gl::blend_func(collection.blend_s, collection.blend_d);

            for model in collection.models.values() {
                if model.radius > 0.0
                    && frustum.contains(&Sphere {
                        center: Point3::new(model.x, -model.y, model.z),
                        radius: model.radius,
                    }) == collision::Relation::Out
                {
                    continue;
                }
                model.array.bind();
                if let Some(v) = collection.shader.lighting {
                    v.set_float2(model.block_light, model.sky_light)
                }
                if let Some(v) = collection.shader.model_matrix {
                    v.set_matrix4_multi(&model.matrix)
                }
                if let Some(v) = collection.shader.color_mul {
                    unsafe {
                        v.set_float_multi_raw(model.colors.as_ptr() as *const _, model.colors.len())
                    }
                }
                gl::draw_elements(gl::TRIANGLES, model.count, self.index_type, 0);
            }
        }
        gl::disable(gl::BLEND);
    }
}

struct Collection {
    shader: ModelShader,

    models: HashMap<ModelKey, Model, BuildHasherDefault<FNVHash>>,
    blend_s: gl::Factor,
    blend_d: gl::Factor,

    next_id: usize,
}

pub struct Model {
    // For culling only
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    // Per a part
    pub matrix: Vec<Matrix4<f32>>,
    pub colors: Vec<[f32; 4]>,
    pub block_light: f32,
    pub sky_light: f32,

    array: gl::VertexArray,
    buffer: gl::Buffer,
    buffer_size: usize,
    count: i32,

    pub verts: Vec<Vertex>,
}

#[derive(Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub texture: super::Texture,
    pub texture_x: f64,
    pub texture_y: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub id: u8,
}

init_shader! {
    Program ModelShader {
        vert = "model_vertex",
        frag = "model_frag",
        attribute = {
            optional position => "aPosition",
            optional texture_info => "aTextureInfo",
            optional texture_offset => "aTextureOffset",
            optional color => "aColor",
            optional id => "id",
        },
        uniform = {
            optional perspective_matrix => "perspectiveMatrix",
            optional camera_matrix => "cameraMatrix",
            optional model_matrix => "modelMatrix",
            optional texture => "textures",
            optional light_level => "lightLevel",
            optional sky_offset => "skyOffset",
            optional lighting => "lighting",
            optional color_mul => "colorMul",
        },
    }
}

// Helper methods
pub fn append_box(
    verts: &mut Vec<Vertex>,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    h: f32,
    d: f32,
    textures: [Option<super::Texture>; 6],
) {
    append_box_texture_scale(
        verts,
        x,
        y,
        z,
        w,
        h,
        d,
        textures,
        [
            [1.0, 1.0],
            [1.0, 1.0],
            [1.0, 1.0],
            [1.0, 1.0],
            [1.0, 1.0],
            [1.0, 1.0],
        ],
    );
}
pub fn append_box_texture_scale(
    verts: &mut Vec<Vertex>,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    h: f32,
    d: f32,
    textures: [Option<super::Texture>; 6],
    texture_scale: [[f64; 2]; 6],
) {
    for dir in Direction::all() {
        let tex = textures[dir.index()].clone();
        if tex.is_none() {
            continue;
        }
        let tex = tex.unwrap();
        for vert in BlockVertex::face_by_direction(dir) {
            let (rr, gg, bb) = if dir == Direction::West || dir == Direction::East {
                (
                    (255.0 * 0.8) as u8,
                    (255.0 * 0.8) as u8,
                    (255.0 * 0.8) as u8,
                )
            } else {
                (255, 255, 255)
            };
            verts.push(Vertex {
                x: vert.x * w + x,
                y: vert.y * h + y,
                z: vert.z * d + z,
                texture: tex.clone(),
                texture_x: (vert.toffsetx as f64) * texture_scale[dir.index()][0],
                texture_y: (vert.toffsety as f64) * texture_scale[dir.index()][1],
                r: rr,
                g: gg,
                b: bb,
                a: 255,
                id: 0,
            });
        }
    }
}

pub struct FormatState<'a> {
    pub offset: f32,
    pub width: f32,
    pub text: Vec<Vertex>,
    pub renderer: &'a mut super::Renderer,
    pub y_scale: f32,
    pub x_scale: f32,
}

impl<'a> FormatState<'a> {
    pub fn build(&mut self, c: &Component, color: format::Color) {
        match *c {
            format::Component::Text(ref txt) => {
                let col = FormatState::get_color(&txt.modifier, color);
                self.append_text(&txt.text, col);
                let modi = &txt.modifier;
                if let Some(ref extra) = modi.extra {
                    for e in extra {
                        self.build(e, col);
                    }
                }
            }
        }
    }

    fn append_text(&mut self, txt: &str, color: format::Color) {
        let (rr, gg, bb) = color.to_rgb();
        for ch in txt.chars() {
            if ch == ' ' {
                self.offset += 6.0 * self.x_scale;
                continue;
            }
            let texture = self.renderer.ui.character_texture(ch);
            let w = self.renderer.ui.size_of_char(ch) as f32;

            for vert in crate::model::BlockVertex::face_by_direction(Direction::North) {
                self.text.push(Vertex {
                    x: vert.x * self.x_scale * w - (self.offset + w * self.x_scale),
                    y: vert.y * self.y_scale + 2.1 / 16.0,
                    z: -0.6 / 16.0,
                    texture: texture.clone(),
                    texture_x: vert.toffsetx as f64,
                    texture_y: vert.toffsety as f64,
                    r: rr,
                    g: gg,
                    b: bb,
                    a: 255,
                    id: 0,
                });
            }
            self.offset += (w + 2.0) * self.x_scale;
        }
        if self.offset > self.width {
            self.width = self.offset;
        }
    }

    fn get_color(modi: &format::Modifier, color: format::Color) -> format::Color {
        modi.color.unwrap_or(color)
    }
}
