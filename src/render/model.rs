
use super::glsl;
use super::shaders;
use gl;
use cgmath::{Point3, Matrix4, SquareMatrix};
use collision::{self, Frustum, Sphere};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::{Arc, RwLock};
use types::hash::FNVHash;
use types::Direction;
use byteorder::{WriteBytesExt, NativeEndian};

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
            gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA
        );
        m.add_collection(
            &greg.get("sun_vertex"),
            &greg.get("sun_frag"),
            gl::SRC_ALPHA, gl::ONE_FACTOR
        );
        m
    }

    pub fn add_collection(&mut self, vert: &str, frag: &str, blend_s: gl::Factor, blend_d: gl::Factor) -> CollectionKey {
        let collection = Collection {
            shader: ModelShader::new_manual(vert, frag),
            models: HashMap::with_hasher(BuildHasherDefault::default()),
            blend_s: blend_s,
            blend_d: blend_d,
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
            collection.shader.position.enable();
            collection.shader.texture_info.enable();
            collection.shader.texture_offset.enable();
            collection.shader.color.enable();
            collection.shader.id.enable();
            collection.shader.position.vertex_pointer(3, gl::FLOAT, false, 36, 0);
            collection.shader.texture_info.vertex_pointer(4, gl::UNSIGNED_SHORT, false, 36, 12);
            collection.shader.texture_offset.vertex_pointer_int(3, gl::SHORT, 36, 20);
            collection.shader.color.vertex_pointer(4, gl::UNSIGNED_BYTE, true, 36, 28);
            collection.shader.id.vertex_pointer_int(4, gl::UNSIGNED_BYTE, 36, 32);

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

                array: array,
                buffer: buffer,
                buffer_size: 0,
                count: 0,

                counts: Vec::with_capacity(parts.len()),
                offsets: Vec::with_capacity(parts.len()),

                verts: vec![],
            };

            for (i, part) in parts.into_iter().enumerate() {
                model.matrix.push(Matrix4::identity());
                model.colors.push([1.0, 1.0, 1.0, 1.0]);
                model.counts.push(((part.len() / 4) * 6) as i32);
                model.offsets.push((model.verts.len() / 4) * 6);
                for mut pp in part {
                    pp.id = i as u8;
                    model.verts.push(pp);
                }
            }
            model
        };

        Self::rebuild_model(&mut model);
        if self.max_index < model.count {
            let (data, ty) = super::generate_element_buffer(model.count);
            self.index_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
            self.index_buffer.set_data(gl::ELEMENT_ARRAY_BUFFER, &data, gl::DYNAMIC_DRAW);
            self.max_index = model.count;
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
        model.count = (model.verts.len() / 4) * 6;

        let mut buffer = Vec::with_capacity(36 * model.verts.len());
        for vert in &model.verts {
            let _ = buffer.write_f32::<NativeEndian>(vert.x);
            let _ = buffer.write_f32::<NativeEndian>(vert.y);
            let _ = buffer.write_f32::<NativeEndian>(vert.z);
            let _ = buffer.write_u16::<NativeEndian>(vert.texture.get_x() as u16);
            let _ = buffer.write_u16::<NativeEndian>(vert.texture.get_y() as u16);
            let _ = buffer.write_u16::<NativeEndian>(vert.texture.get_width() as u16);
            let _ = buffer.write_u16::<NativeEndian>(vert.texture.get_height() as u16);
            let _ = buffer.write_i16::<NativeEndian>(((vert.texture.get_width() as f64) * 16.0 * vert.texture_x) as i16);
            let _ = buffer.write_i16::<NativeEndian>(((vert.texture.get_height() as f64) * 16.0 * vert.texture_y) as i16);
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
            model.buffer.set_data(gl::ARRAY_BUFFER, &buffer, gl::DYNAMIC_DRAW);
            model.buffer_size = buffer.len();
        }
    }

    pub fn rebuild_models(&mut self, version: usize, textures: &Arc<RwLock<super::TextureManager>>) {
        for collection in &mut self.collections {
            for (_, model) in &mut collection.models {
                for vert in &mut model.verts {
                    vert.texture = if vert.texture.version == version {
                        vert.texture.clone()
                    } else {
                        let mut new = super::Renderer::get_texture(&textures, &vert.texture.name);
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

    pub fn draw(&mut self, frustum: &Frustum<f32>, perspective_matrix: &Matrix4<f32>, camera_matrix: &Matrix4<f32>, sky_offset: f32, light_level: f32) {
        let m = if self.index_type == gl::UNSIGNED_SHORT {
            2
        } else {
            4
        };

        gl::enable(gl::BLEND);
        for collection in &self.collections {
            collection.shader.program.use_program();
            collection.shader.perspective_matrix.set_matrix4(perspective_matrix);
            collection.shader.camera_matrix.set_matrix4(camera_matrix);
            collection.shader.texture.set_int(0);
            collection.shader.sky_offset.set_float(sky_offset);
            collection.shader.light_level.set_float(light_level);
            gl::blend_func(collection.blend_s, collection.blend_d);

            for model in collection.models.values() {
                if model.radius > 0.0 && frustum.contains(Sphere {
                    center: Point3::new(model.x, -model.y, model.z),
                    radius: model.radius
                }) == collision::Relation::Out {
                    continue;
                }
                model.array.bind();
                collection.shader.lighting.set_float2(model.block_light, model.sky_light);
                if model.counts.len() > 1 {
                    let mut offsets = model.offsets.clone();
                    for offset in &mut offsets {
                        *offset *= m;
                    }
                    collection.shader.model_matrix.set_matrix4_multi(&model.matrix);
                    collection.shader.color_mul.set_float_mutli_raw(model.colors.as_ptr() as *const _, model.colors.len());
                    gl::multi_draw_elements(gl::TRIANGLES, &model.counts, self.index_type, &offsets);
                } else {
                    collection.shader.model_matrix.set_matrix4_multi(&model.matrix);
                    collection.shader.color_mul.set_float_mutli_raw(model.colors.as_ptr() as *const _, model.colors.len());
                    gl::draw_elements(gl::TRIANGLES, model.counts[0], self.index_type, model.offsets[0] * m);
                }
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
    count: usize,

    counts: Vec<i32>,
    offsets: Vec<usize>,

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
            position => "aPosition",
            texture_info => "aTextureInfo",
            texture_offset => "aTextureOffset",
            color => "aColor",
            id => "id",
        },
        uniform = {
            perspective_matrix => "perspectiveMatrix",
            camera_matrix => "cameraMatrix",
            model_matrix => "modelMatrix[]",
            texture => "textures",
            light_level => "lightLevel",
            sky_offset => "skyOffset",
            lighting => "lighting",
            color_mul => "colorMul[]",
        },
    }
}

// Helper methods
pub fn append_box(
    verts: &mut Vec<Vertex>,
    x: f32, y: f32, z: f32,
    w: f32, h: f32, d: f32, textures: [Option<super::Texture>; 6]
) {
    append_box_texture_scale(verts, x, y, z, w, h, d, textures, [
        [1.0, 1.0],
        [1.0, 1.0],
        [1.0, 1.0],
        [1.0, 1.0],
        [1.0, 1.0],
        [1.0, 1.0]
    ]);
}
pub fn append_box_texture_scale(
    verts: &mut Vec<Vertex>,
    x: f32, y: f32, z: f32,
    w: f32, h: f32, d: f32,
    textures: [Option<super::Texture>; 6], texture_scale: [[f64; 2]; 6]) {
    for dir in Direction::all() {
        let tex = textures[dir.index()].clone();
        if tex.is_none() {
            continue;
        }
        let tex = tex.unwrap();
        for vert in dir.get_verts() {
            let (rr, gg, bb) = if dir == Direction::West || dir == Direction::East {
                ((255.0 * 0.8) as u8, (255.0 * 0.8) as u8, (255.0 * 0.8) as u8)
            } else {
                (255, 255, 255)
            };
            verts.push(Vertex {
                x: vert.x * w + x,
                y: vert.y * h + y,
                z: vert.z * d + z,
                texture: tex.clone(),
                texture_x: (vert.toffsetx as f64) * texture_scale[dir.index()][0],
                texture_y: (vert.toffsety as f64) * texture_scale[dir.index()][0],
                r: rr,
                g: gg,
                b: bb,
                a: 255,
                id: 0,
            });
        }
    }
}
