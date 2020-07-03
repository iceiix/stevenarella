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

mod atlas;
pub mod glsl;
#[macro_use]
pub mod shaders;
pub mod clouds;
pub mod model;
pub mod ui;

use crate::gl;
use crate::resources;
use crate::world;
use byteorder::{NativeEndian, WriteBytesExt};
use cgmath::prelude::*;
use image::{GenericImage, GenericImageView};
use log::{error, trace};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, RwLock};

use crate::types::hash::FNVHash;
use std::hash::BuildHasherDefault;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc;
use std::thread;

#[cfg(not(target_arch = "wasm32"))]

const ATLAS_SIZE: usize = 1024;

// TEMP
const NUM_SAMPLES: i32 = 2;

pub struct Camera {
    pub pos: cgmath::Point3<f64>,
    pub yaw: f64,
    pub pitch: f64,
}

pub struct Renderer {
    resource_version: usize,
    pub resources: Arc<RwLock<resources::Manager>>,
    textures: Arc<RwLock<TextureManager>>,
    pub ui: ui::UIState,
    pub model: model::Manager,
    pub clouds: clouds::Clouds,

    gl_texture: gl::Texture,
    texture_layers: usize,

    chunk_shader: ChunkShader,
    chunk_shader_alpha: ChunkShaderAlpha,
    trans_shader: TransShader,

    element_buffer: gl::Buffer,
    element_buffer_size: usize,
    element_buffer_type: gl::Type,

    pub camera: Camera,
    perspective_matrix: cgmath::Matrix4<f32>,
    camera_matrix: cgmath::Matrix4<f32>,
    pub frustum: collision::Frustum<f32>,
    pub view_vector: cgmath::Vector3<f32>,

    pub frame_id: u32,

    trans: Option<TransInfo>,

    pub width: u32,
    pub height: u32,

    // Light renderering
    pub light_level: f32,
    pub sky_offset: f32,
    skin_request: mpsc::Sender<String>,
    skin_reply: mpsc::Receiver<(String, Option<image::DynamicImage>)>,
}

#[derive(Default)]
pub struct ChunkBuffer {
    solid: Option<ChunkRenderInfo>,
    trans: Option<ChunkRenderInfo>,
}

impl ChunkBuffer {
    pub fn new() -> ChunkBuffer {
        Default::default()
    }
}

struct ChunkRenderInfo {
    array: gl::VertexArray,
    buffer: gl::Buffer,
    buffer_size: usize,
    count: usize,
}

init_shader! {
    Program ChunkShader {
        vert = "chunk_vertex",
        frag = "chunk_frag",
        attribute = {
            required position => "aPosition",
            required texture_info => "aTextureInfo",
            required texture_offset => "aTextureOffset",
            required color => "aColor",
            required lighting => "aLighting",
        },
        uniform = {
            required perspective_matrix => "perspectiveMatrix",
            required camera_matrix => "cameraMatrix",
            required offset => "offset",
            required texture => "textures",
            required light_level => "lightLevel",
            required sky_offset => "skyOffset",
        },
    }
}

init_shader! {
    Program ChunkShaderAlpha {
        vert = "chunk_vertex",
        frag = "chunk_frag", #alpha
        attribute = {
            required position => "aPosition",
            required texture_info => "aTextureInfo",
            required texture_offset => "aTextureOffset",
            required color => "aColor",
            required lighting => "aLighting",
        },
        uniform = {
            required perspective_matrix => "perspectiveMatrix",
            required camera_matrix => "cameraMatrix",
            required offset => "offset",
            required texture => "textures",
            required light_level => "lightLevel",
            required sky_offset => "skyOffset",
        },
    }
}

impl Renderer {
    pub fn new(res: Arc<RwLock<resources::Manager>>) -> Renderer {
        let version = { res.read().unwrap().version() };
        let tex = gl::Texture::new();
        tex.bind(gl::TEXTURE_2D_ARRAY);
        tex.image_3d(
            gl::TEXTURE_2D_ARRAY,
            0,
            ATLAS_SIZE as u32,
            ATLAS_SIZE as u32,
            1,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &[0; ATLAS_SIZE * ATLAS_SIZE * 4],
        );
        tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
        tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
        tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
        tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);

        let (textures, skin_req, skin_reply) = TextureManager::new(res.clone());
        let textures = Arc::new(RwLock::new(textures));

        let mut greg = glsl::Registry::new();
        shaders::add_shaders(&mut greg);
        let ui = ui::UIState::new(&greg, textures.clone(), res.clone());

        gl::enable(gl::DEPTH_TEST);
        gl::enable(gl::CULL_FACE_FLAG);
        gl::cull_face(gl::BACK);
        gl::front_face(gl::CLOCK_WISE);

        // Shaders
        let chunk_shader = ChunkShader::new(&greg);
        let chunk_shader_alpha = ChunkShaderAlpha::new(&greg);
        let trans_shader = TransShader::new(&greg);

        // UI
        // Line Drawer
        // Clouds

        gl::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::depth_func(gl::LESS_OR_EQUAL);

        Renderer {
            resource_version: version,
            model: model::Manager::new(&greg),
            clouds: clouds::Clouds::new(&greg, textures.clone()),
            textures,
            ui,
            resources: res,
            gl_texture: tex,
            texture_layers: 1,

            chunk_shader,
            chunk_shader_alpha,
            trans_shader,

            element_buffer: gl::Buffer::new(),
            element_buffer_size: 0,
            element_buffer_type: gl::UNSIGNED_BYTE,

            width: 0,
            height: 0,

            camera: Camera {
                pos: cgmath::Point3::new(0.0, 0.0, 0.0),
                yaw: 0.0,
                pitch: ::std::f64::consts::PI,
            },
            perspective_matrix: cgmath::Matrix4::identity(),
            camera_matrix: cgmath::Matrix4::identity(),
            frustum: collision::Frustum::from_matrix4(cgmath::Matrix4::identity()).unwrap(),
            view_vector: cgmath::Vector3::zero(),

            frame_id: 1,

            trans: None,

            light_level: 0.8,
            sky_offset: 1.0,
            skin_request: skin_req,
            skin_reply,
        }
    }

    pub fn update_camera(&mut self, width: u32, height: u32) {
        use std::f64::consts::PI as PI64;
        // Not a sane place to put this but it works
        {
            let rm = self.resources.read().unwrap();
            if rm.version() != self.resource_version {
                self.resource_version = rm.version();
                trace!("Updating textures to {}", self.resource_version);
                self.textures
                    .write()
                    .unwrap()
                    .update_textures(self.resource_version);

                self.model
                    .rebuild_models(self.resource_version, &self.textures);
            }
        }

        if self.height != height || self.width != width {
            self.width = width;
            self.height = height;
            gl::viewport(0, 0, width as i32, height as i32);

            let fovy = cgmath::Rad::from(cgmath::Deg(90.0_f32));
            let aspect = (width as f32 / height as f32).max(1.0);

            self.perspective_matrix = cgmath::Matrix4::from(cgmath::PerspectiveFov {
                fovy,
                aspect,
                near: 0.1f32,
                far: 500.0f32,
            });

            self.init_trans(width, height);
        }

        self.view_vector = cgmath::Vector3::new(
            ((self.camera.yaw - PI64 / 2.0).cos() * -self.camera.pitch.cos()) as f32,
            (-self.camera.pitch.sin()) as f32,
            (-(self.camera.yaw - PI64 / 2.0).sin() * -self.camera.pitch.cos()) as f32,
        );
        let camera = cgmath::Point3::new(
            -self.camera.pos.x as f32,
            -self.camera.pos.y as f32,
            self.camera.pos.z as f32,
        );
        let camera_matrix = cgmath::Matrix4::look_at(
            camera,
            camera
                + cgmath::Point3::new(-self.view_vector.x, -self.view_vector.y, self.view_vector.z)
                    .to_vec(),
            cgmath::Vector3::new(0.0, -1.0, 0.0),
        );
        self.camera_matrix = camera_matrix * cgmath::Matrix4::from_nonuniform_scale(-1.0, 1.0, 1.0);
        self.frustum =
            collision::Frustum::from_matrix4(self.perspective_matrix * self.camera_matrix).unwrap();
    }

    pub fn tick(
        &mut self,
        world: &mut world::World,
        delta: f64,
        width: u32,
        height: u32,
        physical_width: u32,
        physical_height: u32,
    ) {
        self.update_textures(delta);

        let trans = self.trans.as_mut().unwrap();
        trans.main.bind();

        gl::active_texture(0);
        self.gl_texture.bind(gl::TEXTURE_2D_ARRAY);

        gl::enable(gl::MULTISAMPLE);

        let time_offset = self.sky_offset * 0.9;
        gl::clear_color(
            (122.0 / 255.0) * time_offset,
            (165.0 / 255.0) * time_offset,
            (247.0 / 255.0) * time_offset,
            1.0,
        );
        gl::clear(gl::ClearFlags::Color | gl::ClearFlags::Depth);

        // Chunk rendering
        self.chunk_shader.program.use_program();

        self.chunk_shader
            .perspective_matrix
            .set_matrix4(&self.perspective_matrix);
        self.chunk_shader
            .camera_matrix
            .set_matrix4(&self.camera_matrix);
        self.chunk_shader.texture.set_int(0);
        self.chunk_shader.light_level.set_float(self.light_level);
        self.chunk_shader.sky_offset.set_float(self.sky_offset);

        for (pos, info) in world.get_render_list() {
            if let Some(solid) = info.solid.as_ref() {
                if solid.count > 0 {
                    self.chunk_shader
                        .offset
                        .set_int3(pos.0, pos.1 * 4096, pos.2);
                    solid.array.bind();
                    gl::draw_elements(
                        gl::TRIANGLES,
                        solid.count as i32,
                        self.element_buffer_type,
                        0,
                    );
                }
            }
        }

        // Line rendering
        // Model rendering
        self.model.draw(
            &self.frustum,
            &self.perspective_matrix,
            &self.camera_matrix,
            self.light_level,
            self.sky_offset,
        );
        if world.copy_cloud_heightmap(&mut self.clouds.heightmap_data) {
            self.clouds.dirty = true;
        }
        self.clouds.draw(
            &self.camera.pos,
            &self.perspective_matrix,
            &self.camera_matrix,
            self.light_level,
            self.sky_offset,
            delta,
        );

        // Trans chunk rendering
        self.chunk_shader_alpha.program.use_program();
        self.chunk_shader_alpha
            .perspective_matrix
            .set_matrix4(&self.perspective_matrix);
        self.chunk_shader_alpha
            .camera_matrix
            .set_matrix4(&self.camera_matrix);
        self.chunk_shader_alpha.texture.set_int(0);
        self.chunk_shader_alpha
            .light_level
            .set_float(self.light_level);
        self.chunk_shader_alpha
            .sky_offset
            .set_float(self.sky_offset);

        // Copy the depth buffer
        trans.main.bind_read();
        trans.trans.bind_draw();
        gl::blit_framebuffer(
            0,
            0,
            physical_width as i32,
            physical_height as i32,
            0,
            0,
            physical_width as i32,
            physical_height as i32,
            gl::ClearFlags::Depth,
            gl::NEAREST,
        );

        gl::enable(gl::BLEND);
        gl::depth_mask(false);
        trans.trans.bind();
        gl::clear_color(0.0, 0.0, 0.0, 1.0);
        gl::clear(gl::ClearFlags::Color);
        gl::clear_buffer(gl::COLOR, 0, &[0.0, 0.0, 0.0, 1.0]);
        gl::clear_buffer(gl::COLOR, 1, &[0.0, 0.0, 0.0, 0.0]);
        gl::blend_func_separate(
            gl::ONE_FACTOR,
            gl::ONE_FACTOR,
            gl::ZERO_FACTOR,
            gl::ONE_MINUS_SRC_ALPHA,
        );

        for (pos, info) in world.get_render_list().iter().rev() {
            if let Some(trans) = info.trans.as_ref() {
                if trans.count > 0 {
                    self.chunk_shader_alpha
                        .offset
                        .set_int3(pos.0, pos.1 * 4096, pos.2);
                    trans.array.bind();
                    gl::draw_elements(
                        gl::TRIANGLES,
                        trans.count as i32,
                        self.element_buffer_type,
                        0,
                    );
                }
            }
        }

        gl::check_framebuffer_status();
        gl::unbind_framebuffer();
        gl::disable(gl::DEPTH_TEST);
        gl::clear(gl::ClearFlags::Color);
        gl::disable(gl::BLEND);
        trans.draw(&self.trans_shader);

        gl::enable(gl::DEPTH_TEST);
        gl::depth_mask(true);
        gl::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::disable(gl::MULTISAMPLE);

        self.ui.tick(width, height);

        gl::check_gl_error();

        self.frame_id = self.frame_id.wrapping_add(1);
    }

    fn ensure_element_buffer(&mut self, size: usize) {
        if self.element_buffer_size < size {
            let (data, ty) = self::generate_element_buffer(size);
            self.element_buffer_type = ty;
            self.element_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
            self.element_buffer
                .set_data(gl::ELEMENT_ARRAY_BUFFER, &data, gl::DYNAMIC_DRAW);
            self.element_buffer_size = size;
        }
    }

    pub fn update_chunk_solid(&mut self, buffer: &mut ChunkBuffer, data: &[u8], count: usize) {
        self.ensure_element_buffer(count);
        if count == 0 {
            if buffer.solid.is_some() {
                buffer.solid = None;
            }
            return;
        }
        let new = buffer.solid.is_none();
        if buffer.solid.is_none() {
            buffer.solid = Some(ChunkRenderInfo {
                array: gl::VertexArray::new(),
                buffer: gl::Buffer::new(),
                buffer_size: 0,
                count: 0,
            });
        }
        let info = buffer.solid.as_mut().unwrap();

        info.array.bind();
        self.chunk_shader.position.enable();
        self.chunk_shader.texture_info.enable();
        self.chunk_shader.texture_offset.enable();
        self.chunk_shader.color.enable();
        self.chunk_shader.lighting.enable();

        self.element_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);

        info.buffer.bind(gl::ARRAY_BUFFER);
        if new || info.buffer_size < data.len() {
            info.buffer_size = data.len();
            info.buffer
                .set_data(gl::ARRAY_BUFFER, data, gl::DYNAMIC_DRAW);
        } else {
            info.buffer.re_set_data(gl::ARRAY_BUFFER, data);
        }

        self.chunk_shader
            .position
            .vertex_pointer(3, gl::FLOAT, false, 40, 0);
        self.chunk_shader
            .texture_info
            .vertex_pointer(4, gl::UNSIGNED_SHORT, false, 40, 12);
        self.chunk_shader
            .texture_offset
            .vertex_pointer(3, gl::SHORT, false, 40, 20);
        self.chunk_shader
            .color
            .vertex_pointer(3, gl::UNSIGNED_BYTE, true, 40, 28);
        self.chunk_shader
            .lighting
            .vertex_pointer(2, gl::UNSIGNED_SHORT, false, 40, 32);

        info.count = count;
    }

    pub fn update_chunk_trans(&mut self, buffer: &mut ChunkBuffer, data: &[u8], count: usize) {
        self.ensure_element_buffer(count);
        if count == 0 {
            if buffer.trans.is_some() {
                buffer.trans = None;
            }
            return;
        }
        let new = buffer.trans.is_none();
        if buffer.trans.is_none() {
            buffer.trans = Some(ChunkRenderInfo {
                array: gl::VertexArray::new(),
                buffer: gl::Buffer::new(),
                buffer_size: 0,
                count: 0,
            });
        }
        let info = buffer.trans.as_mut().unwrap();

        info.array.bind();
        self.chunk_shader_alpha.position.enable();
        self.chunk_shader_alpha.texture_info.enable();
        self.chunk_shader_alpha.texture_offset.enable();
        self.chunk_shader_alpha.color.enable();
        self.chunk_shader_alpha.lighting.enable();

        self.element_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);

        info.buffer.bind(gl::ARRAY_BUFFER);
        if new || info.buffer_size < data.len() {
            info.buffer_size = data.len();
            info.buffer
                .set_data(gl::ARRAY_BUFFER, data, gl::DYNAMIC_DRAW);
        } else {
            info.buffer.re_set_data(gl::ARRAY_BUFFER, data);
        }

        self.chunk_shader_alpha
            .position
            .vertex_pointer(3, gl::FLOAT, false, 40, 0);
        self.chunk_shader_alpha
            .texture_info
            .vertex_pointer(4, gl::UNSIGNED_SHORT, false, 40, 12);
        self.chunk_shader_alpha
            .texture_offset
            .vertex_pointer(3, gl::SHORT, false, 40, 20);
        self.chunk_shader_alpha
            .color
            .vertex_pointer(3, gl::UNSIGNED_BYTE, true, 40, 28);
        self.chunk_shader_alpha
            .lighting
            .vertex_pointer(2, gl::UNSIGNED_SHORT, false, 40, 32);

        info.count = count;
    }

    fn do_pending_textures(&mut self) {
        let len = {
            let tex = self.textures.read().unwrap();
            // Rebuild the texture if it needs resizing
            if self.texture_layers != tex.atlases.len() {
                let len = ATLAS_SIZE * ATLAS_SIZE * 4 * tex.atlases.len();
                let mut data = Vec::with_capacity(len);
                unsafe {
                    data.set_len(len);
                }
                self.gl_texture.get_pixels(
                    gl::TEXTURE_2D_ARRAY,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    &mut data[..],
                );
                self.gl_texture.image_3d(
                    gl::TEXTURE_2D_ARRAY,
                    0,
                    ATLAS_SIZE as u32,
                    ATLAS_SIZE as u32,
                    tex.atlases.len() as u32,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    &data[..],
                );
                self.texture_layers = tex.atlases.len();
            }
            tex.pending_uploads.len()
        };
        if len > 0 {
            // Upload pending changes
            let mut tex = self.textures.write().unwrap();
            for upload in &tex.pending_uploads {
                let atlas = upload.0;
                let rect = upload.1;
                let img = &upload.2;
                self.gl_texture.sub_image_3d(
                    gl::TEXTURE_2D_ARRAY,
                    0,
                    rect.x as u32,
                    rect.y as u32,
                    atlas as u32,
                    rect.width as u32,
                    rect.height as u32,
                    1,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    &img[..],
                );
            }
            tex.pending_uploads.clear();
        }
    }

    fn update_textures(&mut self, delta: f64) {
        {
            let mut tex = self.textures.write().unwrap();
            while let Ok((hash, img)) = self.skin_reply.try_recv() {
                if let Some(img) = img {
                    tex.update_skin(hash, img);
                }
            }
            let mut old_skins = vec![];
            for (skin, refcount) in &tex.skins {
                if refcount.load(Ordering::Relaxed) == 0 {
                    old_skins.push(skin.clone());
                }
            }
            for skin in old_skins {
                tex.skins.remove(&skin);
                tex.remove_dynamic(&format!("skin-{}", skin));
            }
        }
        self.gl_texture.bind(gl::TEXTURE_2D_ARRAY);
        self.do_pending_textures();

        for ani in &mut self.textures.write().unwrap().animated_textures {
            if ani.remaining_time <= 0.0 {
                ani.current_frame = (ani.current_frame + 1) % ani.frames.len();
                ani.remaining_time += ani.frames[ani.current_frame].time as f64;
                let offset =
                    ani.texture.width * ani.texture.width * ani.frames[ani.current_frame].index * 4;
                let offset2 = offset + ani.texture.width * ani.texture.width * 4;
                self.gl_texture.sub_image_3d(
                    gl::TEXTURE_2D_ARRAY,
                    0,
                    ani.texture.get_x() as u32,
                    ani.texture.get_y() as u32,
                    ani.texture.atlas as u32,
                    ani.texture.get_width() as u32,
                    ani.texture.get_height() as u32,
                    1,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    &ani.data[offset..offset2],
                );
            } else {
                ani.remaining_time -= delta / 3.0;
            }
        }
    }

    fn init_trans(&mut self, width: u32, height: u32) {
        self.trans = None;
        self.trans = Some(TransInfo::new(
            width,
            height,
            &self.chunk_shader_alpha,
            &self.trans_shader,
        ));
    }

    pub fn get_textures(&self) -> Arc<RwLock<TextureManager>> {
        self.textures.clone()
    }

    pub fn get_textures_ref(&self) -> &RwLock<TextureManager> {
        &self.textures
    }

    pub fn check_texture(&self, tex: Texture) -> Texture {
        if tex.version == self.resource_version {
            tex
        } else {
            let mut new = Renderer::get_texture(&self.textures, &tex.name);
            new.rel_x = tex.rel_x;
            new.rel_y = tex.rel_y;
            new.rel_width = tex.rel_width;
            new.rel_height = tex.rel_height;
            new.is_rel = tex.is_rel;
            new
        }
    }

    pub fn get_texture(textures: &RwLock<TextureManager>, name: &str) -> Texture {
        let tex = { textures.read().unwrap().get_texture(name) };
        match tex {
            Some(val) => val,
            None => {
                let mut t = textures.write().unwrap();
                // Make sure it hasn't already been loaded since we switched
                // locks.
                if let Some(val) = t.get_texture(name) {
                    val
                } else {
                    t.load_texture(name);
                    t.get_texture(name).unwrap()
                }
            }
        }
    }

    pub fn get_skin(&self, textures: &RwLock<TextureManager>, url: &str) -> Texture {
        let tex = { textures.read().unwrap().get_skin(url) };
        match tex {
            Some(val) => val,
            None => {
                let mut t = textures.write().unwrap();
                // Make sure it hasn't already been loaded since we switched
                // locks.
                if let Some(val) = t.get_skin(url) {
                    val
                } else {
                    t.load_skin(self, url);
                    t.get_skin(url).unwrap()
                }
            }
        }
    }
}

struct TransInfo {
    main: gl::Framebuffer,
    fb_color: gl::Texture,
    _fb_depth: gl::Texture,
    trans: gl::Framebuffer,
    accum: gl::Texture,
    revealage: gl::Texture,
    _depth: gl::Texture,

    array: gl::VertexArray,
    _buffer: gl::Buffer,
}

init_shader! {
    Program TransShader {
        vert = "trans_vertex",
        frag = "trans_frag",
        attribute = {
            required position => "aPosition",
        },
        uniform = {
            required accum => "taccum",
            required revealage => "trevealage",
            required color => "tcolor",
            required samples => "samples",
        },
    }
}

impl TransInfo {
    pub fn new(
        width: u32,
        height: u32,
        chunk_shader: &ChunkShaderAlpha,
        shader: &TransShader,
    ) -> TransInfo {
        let trans = gl::Framebuffer::new();
        trans.bind();

        let accum = gl::Texture::new();
        accum.bind(gl::TEXTURE_2D);
        accum.image_2d_ex(
            gl::TEXTURE_2D,
            0,
            width,
            height,
            gl::RGBA16F,
            gl::RGBA,
            gl::FLOAT,
            None,
        );
        accum.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        accum.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, gl::LINEAR);
        trans.texture_2d(gl::COLOR_ATTACHMENT_0, gl::TEXTURE_2D, &accum, 0);

        let revealage = gl::Texture::new();
        revealage.bind(gl::TEXTURE_2D);
        revealage.image_2d_ex(
            gl::TEXTURE_2D,
            0,
            width,
            height,
            gl::R16F,
            gl::RED,
            gl::FLOAT,
            None,
        );
        revealage.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        revealage.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, gl::LINEAR);
        trans.texture_2d(gl::COLOR_ATTACHMENT_1, gl::TEXTURE_2D, &revealage, 0);

        let trans_depth = gl::Texture::new();
        trans_depth.bind(gl::TEXTURE_2D);
        trans_depth.image_2d_ex(
            gl::TEXTURE_2D,
            0,
            width,
            height,
            gl::DEPTH_COMPONENT24,
            gl::DEPTH_COMPONENT,
            gl::UNSIGNED_BYTE,
            None,
        );
        trans_depth.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        trans_depth.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, gl::LINEAR);
        trans.texture_2d(gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, &trans_depth, 0);

        chunk_shader.program.use_program();
        gl::bind_frag_data_location(&chunk_shader.program, 0, "accum");
        gl::bind_frag_data_location(&chunk_shader.program, 1, "revealage");
        gl::check_framebuffer_status();
        gl::draw_buffers(&[gl::COLOR_ATTACHMENT_0, gl::COLOR_ATTACHMENT_1]);

        let main = gl::Framebuffer::new();
        main.bind();

        let fb_color = gl::Texture::new();
        fb_color.bind(gl::TEXTURE_2D_MULTISAMPLE);
        fb_color.image_2d_sample(
            gl::TEXTURE_2D_MULTISAMPLE,
            NUM_SAMPLES,
            width,
            height,
            gl::RGBA8,
            false,
        );
        main.texture_2d(
            gl::COLOR_ATTACHMENT_0,
            gl::TEXTURE_2D_MULTISAMPLE,
            &fb_color,
            0,
        );

        let fb_depth = gl::Texture::new();
        fb_depth.bind(gl::TEXTURE_2D_MULTISAMPLE);
        fb_depth.image_2d_sample(
            gl::TEXTURE_2D_MULTISAMPLE,
            NUM_SAMPLES,
            width,
            height,
            gl::DEPTH_COMPONENT24,
            false,
        );
        main.texture_2d(
            gl::DEPTH_ATTACHMENT,
            gl::TEXTURE_2D_MULTISAMPLE,
            &fb_depth,
            0,
        );
        gl::check_framebuffer_status();

        gl::unbind_framebuffer();

        shader.program.use_program();
        let array = gl::VertexArray::new();
        array.bind();
        let buffer = gl::Buffer::new();
        buffer.bind(gl::ARRAY_BUFFER);

        let mut data = vec![];
        for f in [
            -1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0,
        ]
        .iter()
        {
            data.write_f32::<NativeEndian>(*f).unwrap();
        }
        buffer.set_data(gl::ARRAY_BUFFER, &data, gl::STATIC_DRAW);

        shader.position.enable();
        shader.position.vertex_pointer(2, gl::FLOAT, false, 8, 0);

        TransInfo {
            main,
            fb_color,
            _fb_depth: fb_depth,
            trans,
            accum,
            revealage,
            _depth: trans_depth,

            array,
            _buffer: buffer,
        }
    }

    fn draw(&mut self, shader: &TransShader) {
        gl::active_texture(0);
        self.accum.bind(gl::TEXTURE_2D);
        gl::active_texture(1);
        self.revealage.bind(gl::TEXTURE_2D);
        gl::active_texture(2);
        self.fb_color.bind(gl::TEXTURE_2D_MULTISAMPLE);

        shader.program.use_program();
        shader.accum.set_int(0);
        shader.revealage.set_int(1);
        shader.color.set_int(2);
        shader.samples.set_int(NUM_SAMPLES);
        self.array.bind();
        gl::draw_arrays(gl::TRIANGLES, 0, 6);
    }
}

pub struct TextureManager {
    textures: HashMap<String, Texture, BuildHasherDefault<FNVHash>>,
    version: usize,
    resources: Arc<RwLock<resources::Manager>>,
    atlases: Vec<atlas::Atlas>,

    animated_textures: Vec<AnimatedTexture>,
    pending_uploads: Vec<(i32, atlas::Rect, Vec<u8>)>,

    dynamic_textures: HashMap<String, (Texture, image::DynamicImage), BuildHasherDefault<FNVHash>>,
    free_dynamics: Vec<Texture>,

    skins: HashMap<String, AtomicIsize, BuildHasherDefault<FNVHash>>,

    _skin_thread: thread::JoinHandle<()>,
}

impl TextureManager {
    #[allow(clippy::let_and_return)]
    #[allow(clippy::type_complexity)]
    fn new(
        res: Arc<RwLock<resources::Manager>>,
    ) -> (
        TextureManager,
        mpsc::Sender<String>,
        mpsc::Receiver<(String, Option<image::DynamicImage>)>,
    ) {
        let (tx, rx) = mpsc::channel();
        let (stx, srx) = mpsc::channel();
        let skin_thread = thread::spawn(|| Self::process_skins(srx, tx));
        let mut tm = TextureManager {
            textures: HashMap::with_hasher(BuildHasherDefault::default()),
            version: {
                // TODO: fix borrow and remove clippy::let_and_return above
                let ver = res.read().unwrap().version();
                ver
            },
            resources: res,
            atlases: Vec::new(),
            animated_textures: Vec::new(),
            pending_uploads: Vec::new(),

            dynamic_textures: HashMap::with_hasher(BuildHasherDefault::default()),
            free_dynamics: Vec::new(),
            skins: HashMap::with_hasher(BuildHasherDefault::default()),

            _skin_thread: skin_thread,
        };
        tm.add_defaults();
        (tm, stx, rx)
    }

    fn add_defaults(&mut self) {
        self.put_texture(
            "steven",
            "missing_texture",
            2,
            2,
            vec![
                0, 0, 0, 255, 255, 0, 255, 255, 255, 0, 255, 255, 0, 0, 0, 255,
            ],
        );
        self.put_texture("steven", "solid", 1, 1, vec![255, 255, 255, 255]);
    }

    #[cfg(target_arch = "wasm32")]
    fn process_skins(
        recv: mpsc::Receiver<String>,
        reply: mpsc::Sender<(String, Option<image::DynamicImage>)>,
    ) {
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn process_skins(
        recv: mpsc::Receiver<String>,
        reply: mpsc::Sender<(String, Option<image::DynamicImage>)>,
    ) {
        let client = reqwest::blocking::Client::new();
        loop {
            let hash = match recv.recv() {
                Ok(val) => val,
                Err(_) => return, // Most likely shutting down
            };
            match Self::obtain_skin(&client, &hash) {
                Ok(img) => {
                    let _ = reply.send((hash, Some(img)));
                }
                Err(err) => {
                    error!("Failed to get skin {:?}: {}", hash, err);
                    let _ = reply.send((hash, None));
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn obtain_skin(
        client: &::reqwest::blocking::Client,
        hash: &str,
    ) -> Result<image::DynamicImage, ::std::io::Error> {
        use std::io::Read;
        use std::io::{Error, ErrorKind};
        use std::path::Path;
        use std_or_web::fs;
        let path = format!("skin-cache/{}/{}.png", &hash[..2], hash);
        let cache_path = Path::new(&path);
        fs::create_dir_all(cache_path.parent().unwrap())?;
        let mut buf = vec![];
        if fs::metadata(cache_path).is_ok() {
            // We have a cached image
            let mut file = fs::File::open(cache_path)?;
            file.read_to_end(&mut buf)?;
        } else {
            // Need to download it
            let url = &format!("http://textures.minecraft.net/texture/{}", hash);
            let mut res = match client.get(url).send() {
                Ok(val) => val,
                Err(err) => {
                    return Err(Error::new(ErrorKind::ConnectionAborted, err));
                }
            };
            let mut buf = vec![];
            match res.read_to_end(&mut buf) {
                Ok(_) => {}
                Err(err) => {
                    // TODO: different error for failure to read?
                    return Err(Error::new(ErrorKind::InvalidData, err));
                }
            }

            // Save to cache
            let mut file = fs::File::create(cache_path)?;
            file.write_all(&buf)?;
        }
        let mut img = match image::load_from_memory(&buf) {
            Ok(val) => val,
            Err(err) => {
                return Err(Error::new(ErrorKind::InvalidData, err));
            }
        };
        let (_, height) = img.dimensions();
        if height == 32 {
            // Needs changing to the new format
            let mut new = image::DynamicImage::new_rgba8(64, 64);
            new.copy_from(&img, 0, 0)
                .expect("Invalid png image in skin");
            for xx in 0..4 {
                for yy in 0..16 {
                    for section in 0..4 {
                        let os = match section {
                            0 => 2,
                            1 => 1,
                            2 => 0,
                            3 => 3,
                            _ => unreachable!(),
                        };
                        new.put_pixel(
                            16 + (3 - xx) + section * 4,
                            48 + yy,
                            img.get_pixel(xx + os * 4, 16 + yy),
                        );
                        new.put_pixel(
                            32 + (3 - xx) + section * 4,
                            48 + yy,
                            img.get_pixel(xx + 40 + os * 4, 16 + yy),
                        );
                    }
                }
            }
            img = new;
        }
        // Block transparent pixels in blacklisted areas
        let blacklist = [
            // X, Y, W, H
            (0, 0, 32, 16),
            (16, 16, 24, 16),
            (0, 16, 16, 16),
            (16, 48, 16, 16),
            (32, 48, 16, 16),
            (40, 16, 16, 16),
        ];
        for bl in blacklist.iter() {
            for x in bl.0..(bl.0 + bl.2) {
                for y in bl.1..(bl.1 + bl.3) {
                    let mut col = img.get_pixel(x, y);
                    col.0[3] = 255;
                    img.put_pixel(x, y, col);
                }
            }
        }
        Ok(img)
    }

    fn update_textures(&mut self, version: usize) {
        self.pending_uploads.clear();
        self.atlases.clear();
        self.animated_textures.clear();
        self.version = version;
        let map = self.textures.clone();
        self.textures.clear();

        self.free_dynamics.clear();

        self.add_defaults();

        for name in map.keys() {
            if name.starts_with("steven-dynamic:") {
                let n = &name["steven-dynamic:".len()..];
                let (width, height, data) = {
                    let dynamic_texture = match self.dynamic_textures.get(n) {
                        Some(val) => val,
                        None => continue,
                    };
                    let img = &dynamic_texture.1;
                    let (width, height) = img.dimensions();
                    (width, height, img.to_rgba().into_vec())
                };
                let new_tex =
                    self.put_texture("steven-dynamic", n, width as u32, height as u32, data);
                self.dynamic_textures.get_mut(n).unwrap().0 = new_tex;
            } else if !self.textures.contains_key(name) {
                self.load_texture(name);
            }
        }
    }

    fn get_skin(&self, url: &str) -> Option<Texture> {
        let hash = &url["http://textures.minecraft.net/texture/".len()..];
        if let Some(skin) = self.skins.get(hash) {
            skin.fetch_add(1, Ordering::Relaxed);
        }
        self.get_texture(&format!("steven-dynamic:skin-{}", hash))
    }

    pub fn release_skin(&self, url: &str) {
        let hash = &url["http://textures.minecraft.net/texture/".len()..];
        if let Some(skin) = self.skins.get(hash) {
            skin.fetch_sub(1, Ordering::Relaxed);
        }
    }

    fn load_skin(&mut self, renderer: &Renderer, url: &str) {
        let hash = &url["http://textures.minecraft.net/texture/".len()..];
        let res = self.resources.clone();
        // TODO: This shouldn't be hardcoded to steve but instead
        // have a way to select alex as a default.
        let img = if let Some(mut val) = res
            .read()
            .unwrap()
            .open("minecraft", "textures/entity/steve.png")
        {
            let mut data = Vec::new();
            val.read_to_end(&mut data).unwrap();
            image::load_from_memory(&data).unwrap()
        } else {
            image::DynamicImage::new_rgba8(64, 64)
        };
        self.put_dynamic(&format!("skin-{}", hash), img);
        self.skins.insert(hash.to_owned(), AtomicIsize::new(0));
        renderer.skin_request.send(hash.to_owned()).unwrap();
    }

    fn update_skin(&mut self, hash: String, img: image::DynamicImage) {
        if !self.skins.contains_key(&hash) {
            return;
        }
        let name = format!("steven-dynamic:skin-{}", hash);
        let tex = self.get_texture(&name).unwrap();
        let rect = atlas::Rect {
            x: tex.x,
            y: tex.y,
            width: tex.width,
            height: tex.height,
        };

        self.pending_uploads
            .push((tex.atlas, rect, img.to_rgba().into_vec()));
        self.dynamic_textures
            .get_mut(&format!("skin-{}", hash))
            .unwrap()
            .1 = img;
    }

    fn get_texture(&self, name: &str) -> Option<Texture> {
        if name.find(':').is_some() {
            self.textures.get(name).cloned()
        } else {
            self.textures.get(&format!("minecraft:{}", name)).cloned()
        }
    }

    fn load_texture(&mut self, name: &str) {
        let (plugin, name) = if let Some(pos) = name.find(':') {
            (&name[..pos], &name[pos + 1..])
        } else {
            ("minecraft", name)
        };
        let path = format!("textures/{}.png", name);
        let res = self.resources.clone();
        if let Some(mut val) = res.read().unwrap().open(plugin, &path) {
            let mut data = Vec::new();
            val.read_to_end(&mut data).unwrap();
            if let Ok(img) = image::load_from_memory(&data) {
                let (width, height) = img.dimensions();
                // Might be animated
                if (name.starts_with("blocks/") || name.starts_with("items/")) && width != height {
                    let id = img.to_rgba().into_vec();
                    let frame = id[..(width * width * 4) as usize].to_owned();
                    if let Some(mut ani) = self.load_animation(plugin, name, &img, id) {
                        ani.texture = self.put_texture(plugin, name, width, width, frame);
                        self.animated_textures.push(ani);
                        return;
                    }
                }
                self.put_texture(plugin, name, width, height, img.to_rgba().into_vec());
                return;
            }
        }
        self.insert_texture_dummy(plugin, name);
    }

    fn load_animation(
        &mut self,
        plugin: &str,
        name: &str,
        img: &image::DynamicImage,
        data: Vec<u8>,
    ) -> Option<AnimatedTexture> {
        let path = format!("textures/{}.png.mcmeta", name);
        let res = self.resources.clone();
        if let Some(val) = res.read().unwrap().open(plugin, &path) {
            let meta: serde_json::Value = serde_json::from_reader(val).unwrap();
            let animation = meta.get("animation").unwrap();
            let frame_time = animation
                .get("frametime")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);
            let interpolate = animation
                .get("interpolate")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let frames = if let Some(frames) = animation.get("frames").and_then(|v| v.as_array()) {
                let mut out = Vec::with_capacity(frames.len());
                for frame in frames {
                    if let Some(index) = frame.as_i64() {
                        out.push(AnimationFrame {
                            index: index as usize,
                            time: frame_time,
                        })
                    } else {
                        out.push(AnimationFrame {
                            index: frame.get("index").unwrap().as_i64().unwrap() as usize,
                            time: frame_time * frame.get("frameTime").unwrap().as_i64().unwrap(),
                        })
                    }
                }
                out
            } else {
                let (width, height) = img.dimensions();
                let count = height / width;
                let mut frames = Vec::with_capacity(count as usize);
                for i in 0..count {
                    frames.push(AnimationFrame {
                        index: i as usize,
                        time: frame_time,
                    })
                }
                frames
            };

            return Some(AnimatedTexture {
                frames,
                data,
                interpolate,
                current_frame: 0,
                remaining_time: 0.0,
                texture: self.get_texture("steven:missing_texture").unwrap(),
            });
        }
        None
    }

    fn put_texture(
        &mut self,
        plugin: &str,
        name: &str,
        width: u32,
        height: u32,
        data: Vec<u8>,
    ) -> Texture {
        let (atlas, rect) = self.find_free(width as usize, height as usize);
        self.pending_uploads.push((atlas, rect, data));

        let mut full_name = String::new();
        full_name.push_str(plugin);
        full_name.push_str(":");
        full_name.push_str(name);

        let tex = Texture {
            name: full_name.clone(),
            version: self.version,
            atlas,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            rel_x: 0.0,
            rel_y: 0.0,
            rel_width: 1.0,
            rel_height: 1.0,
            is_rel: false,
        };
        self.textures.insert(full_name, tex.clone());
        tex
    }

    fn find_free(&mut self, width: usize, height: usize) -> (i32, atlas::Rect) {
        let mut index = 0;
        for atlas in &mut self.atlases {
            if let Some(rect) = atlas.add(width, height) {
                return (index, rect);
            }
            index += 1;
        }
        let mut atlas = atlas::Atlas::new(ATLAS_SIZE, ATLAS_SIZE);
        let rect = atlas.add(width, height);
        self.atlases.push(atlas);
        (index, rect.unwrap())
    }

    fn insert_texture_dummy(&mut self, plugin: &str, name: &str) -> Texture {
        let missing = self.get_texture("steven:missing_texture").unwrap();

        let mut full_name = String::new();
        full_name.push_str(plugin);
        full_name.push_str(":");
        full_name.push_str(name);

        let t = Texture {
            name: full_name.to_owned(),
            version: self.version,
            atlas: missing.atlas,
            x: missing.x,
            y: missing.y,
            width: missing.width,
            height: missing.height,
            rel_x: 0.0,
            rel_y: 0.0,
            rel_width: 1.0,
            rel_height: 1.0,
            is_rel: false,
        };
        self.textures.insert(full_name, t.clone());
        t
    }

    pub fn put_dynamic(&mut self, name: &str, img: image::DynamicImage) -> Texture {
        use std::mem;
        let (width, height) = img.dimensions();
        let (width, height) = (width as usize, height as usize);
        let mut rect_pos = None;
        for (i, r) in self.free_dynamics.iter().enumerate() {
            if r.width == width && r.height == height {
                rect_pos = Some(i);
                break;
            } else if r.width >= width && r.height >= height {
                rect_pos = Some(i);
            }
        }
        let data = img.to_rgba().into_vec();

        if let Some(rect_pos) = rect_pos {
            let mut tex = self.free_dynamics.remove(rect_pos);
            let rect = atlas::Rect {
                x: tex.x,
                y: tex.y,
                width,
                height,
            };
            self.pending_uploads.push((tex.atlas, rect, data));
            let mut t = tex.relative(
                0.0,
                0.0,
                (width as f32) / (tex.width as f32),
                (height as f32) / (tex.height as f32),
            );
            let old_name = mem::replace(&mut tex.name, format!("steven-dynamic:{}", name));
            self.dynamic_textures.insert(name.to_owned(), (tex, img));
            // We need to rename the texture itself so that get_texture calls
            // work with the new name
            let mut old = self.textures.remove(&old_name).unwrap();
            old.name = format!("steven-dynamic:{}", name);
            t.name = old.name.clone();
            self.textures
                .insert(format!("steven-dynamic:{}", name), old);
            t
        } else {
            let tex = self.put_texture("steven-dynamic", name, width as u32, height as u32, data);
            self.dynamic_textures
                .insert(name.to_owned(), (tex.clone(), img));
            tex
        }
    }

    pub fn remove_dynamic(&mut self, name: &str) {
        let desc = self.dynamic_textures.remove(name).unwrap();
        self.free_dynamics.push(desc.0);
    }
}

#[allow(dead_code)]
struct AnimatedTexture {
    frames: Vec<AnimationFrame>,
    data: Vec<u8>,
    interpolate: bool,
    current_frame: usize,
    remaining_time: f64,
    texture: Texture,
}

struct AnimationFrame {
    index: usize,
    time: i64,
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub name: String,
    version: usize,
    pub atlas: i32,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    is_rel: bool, // Save some cycles for non-relative textures
    rel_x: f32,
    rel_y: f32,
    rel_width: f32,
    rel_height: f32,
}

impl Texture {
    pub fn get_x(&self) -> usize {
        if self.is_rel {
            self.x + ((self.width as f32) * self.rel_x) as usize
        } else {
            self.x
        }
    }

    pub fn get_y(&self) -> usize {
        if self.is_rel {
            self.y + ((self.height as f32) * self.rel_y) as usize
        } else {
            self.y
        }
    }

    pub fn get_width(&self) -> usize {
        if self.is_rel {
            ((self.width as f32) * self.rel_width) as usize
        } else {
            self.width
        }
    }

    pub fn get_height(&self) -> usize {
        if self.is_rel {
            ((self.height as f32) * self.rel_height) as usize
        } else {
            self.height
        }
    }

    pub fn relative(&self, x: f32, y: f32, width: f32, height: f32) -> Texture {
        Texture {
            name: self.name.clone(),
            version: self.version,
            x: self.x,
            y: self.y,
            atlas: self.atlas,
            width: self.width,
            height: self.height,
            is_rel: true,
            rel_x: self.rel_x + x * self.rel_width,
            rel_y: self.rel_y + y * self.rel_height,
            rel_width: width * self.rel_width,
            rel_height: height * self.rel_height,
        }
    }
}

#[allow(unused_must_use)]
pub fn generate_element_buffer(size: usize) -> (Vec<u8>, gl::Type) {
    let mut ty = gl::UNSIGNED_SHORT;
    let mut data = if (size / 6) * 4 * 3 >= u16::max_value() as usize {
        ty = gl::UNSIGNED_INT;
        Vec::with_capacity(size * 4)
    } else {
        Vec::with_capacity(size * 2)
    };
    for i in 0..size / 6 {
        for val in &[0, 1, 2, 2, 1, 3] {
            if ty == gl::UNSIGNED_INT {
                data.write_u32::<NativeEndian>((i as u32) * 4 + val);
            } else {
                data.write_u16::<NativeEndian>((i as u16) * 4 + (*val as u16));
            }
        }
    }

    (data, ty)
}
