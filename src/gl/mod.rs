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

//extern crate steven_gl as gl;

use std::ops::BitOr;
use std::mem;
use std::ops::{Deref, DerefMut};
use log::{error, info};
use glow as gl;
use glow::HasContext;

static mut context: *mut glow::Context = 0 as *mut glow::Context;

/// Inits the gl library. This should be called once a context is ready.
pub fn init(vid: & glutin::WindowedContext<glutin::PossiblyCurrent>) {
    unsafe {
        context = &mut (gl::Context::from_loader_function(|s| {
            vid.get_proc_address(s) as *const _
        })) as *mut glow::Context;
    }
}

fn glow_context() -> &'static glow::Context {
    unsafe {
        context.as_ref().unwrap()
    }
}

/// Dsed to specify how the vertices will be handled
/// to draw.
pub type DrawType = u32;

/// Treats each set of 3 vertices as a triangle
pub const TRIANGLES: DrawType = gl::TRIANGLES;
/// Means the previous vertex connects to the next
/// one in a continuous strip.
pub const LINE_STRIP: DrawType = gl::LINE_STRIP;
/// Treats each set of 2 vertices as a line
pub const LINES: DrawType = gl::LINES;
/// Treats each vertex as a point
pub const POINTS: DrawType = gl::POINTS;

pub fn draw_arrays(ty: DrawType, offset: usize, count: usize) {
    unsafe {
        glow_context().draw_arrays(ty, offset as i32, count as i32);
    }
}

pub fn draw_elements(ty: DrawType, count: i32, dty: Type, offset: usize) {
    unsafe {
        glow_context().draw_elements(ty, count, dty, offset as i32);
    }
}

// Sets the size of the viewport of this context.
pub fn viewport(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        glow_context().viewport(x, y, w, h);
    }
}

/// Sets the color the color buffer should be cleared to
/// when Clear is called with the color flag.
pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        glow_context().clear_color(r, g, b, a);
    }
}

/// `ClearFlags` is a set of flags to mark what should be cleared during
/// a Clear call.
pub enum ClearFlags {
    /// Marks the color buffer to be cleared
    Color,
    /// Marks the depth buffer to be cleared
    Depth,
    Internal(u32),
}

impl ClearFlags {
    fn internal(self) -> u32 {
        match self {
            ClearFlags::Color => gl::COLOR_BUFFER_BIT,
            ClearFlags::Depth => gl::DEPTH_BUFFER_BIT,
            ClearFlags::Internal(val) => val,
        }
    }
}

impl BitOr for ClearFlags {
    type Output = ClearFlags;

    fn bitor(self, rhs: ClearFlags) -> ClearFlags {
        ClearFlags::Internal(self.internal() | rhs.internal())
    }
}

/// Clears the buffers specified by the passed flags.
pub fn clear(flags: ClearFlags) {
    unsafe { glow_context().clear(flags.internal()) }
}

pub fn depth_mask(f: bool) {
    unsafe { glow_context().depth_mask(f); }
}

/// `Func` is a function to be preformed on two values.
pub type Func = u32;

pub const NEVER: Func = gl::NEVER;
pub const LESS: Func = gl::LESS;
pub const LESS_OR_EQUAL: Func = gl::LEQUAL;
pub const GREATER: Func = gl::GREATER;
pub const ALWAYS: Func = gl::ALWAYS;
pub const EQUAL: Func = gl::EQUAL;

pub fn depth_func(f: Func) {
    unsafe {
        glow_context().depth_func(f);
    }
}

/// Flag is a setting that can be enabled or disabled on the context.
pub type Flag = u32;

pub const DEPTH_TEST: Flag = gl::DEPTH_TEST;
pub const CULL_FACE_FLAG: Flag = gl::CULL_FACE;
pub const STENCIL_TEST: Flag = gl::STENCIL_TEST;
pub const BLEND: Flag = gl::BLEND;
pub const MULTISAMPLE: Flag = gl::MULTISAMPLE;

/// Enables the passed flag.
pub fn enable(f: Flag) {
    unsafe {
        glow_context().enable(f);
    }
}

/// Disables the passed flag.
pub fn disable(f: Flag) {
    unsafe {
        glow_context().disable(f);
    }
}

/// Sets the texture slot with the passed id as the
/// currently active one.
pub fn active_texture(id: u32) {
    unsafe {
        glow_context().active_texture(gl::TEXTURE0 + id);
    }
}

/// `Factor` is used in blending
pub type Factor = u32;
pub const SRC_ALPHA: Factor = gl::SRC_ALPHA;
pub const ONE_MINUS_SRC_ALPHA: Factor = gl::ONE_MINUS_SRC_ALPHA;
pub const ONE_FACTOR: Factor = gl::ONE;
pub const ZERO_FACTOR: Factor = gl::ZERO;

/// Sets the factors to be used when blending.
pub fn blend_func(s_factor: Factor, d_factor: Factor) {
    unsafe {
        glow_context().blend_func(s_factor, d_factor);
    }
}

pub fn blend_func_separate(s_factor_rgb: Factor, d_factor_rgb: Factor, s_factor_a: Factor, d_factor_a: Factor) {
    unsafe {
        glow_context().blend_func_separate(s_factor_rgb, d_factor_rgb, s_factor_a, d_factor_a);
    }
}

// Face specifies a face to act on.
pub type Face = u32;
pub const BACK: Face = gl::BACK;
pub const FRONT: Face = gl::FRONT;

/// Sets the face to be culled by the gpu.
pub fn cull_face(face: Face) {
    unsafe {
        glow_context().cull_face(face);
    }
}

// FaceDirection is used to specify an order of vertices, normally
// used to set which is considered to be the front face.
pub type FaceDirection = u32;
pub const CLOCK_WISE: FaceDirection = gl::CW;
pub const COUNTER_CLOCK_WISE: FaceDirection = gl::CCW;

/// Sets the direction of vertices used to specify the
/// front face (e.g. for culling).
pub fn front_face(dir: FaceDirection) {
    unsafe { glow_context().front_face(dir) }
}

/// `Type` is a type of data used by various operations.
pub type Type = u32;
pub const UNSIGNED_BYTE: Type = gl::UNSIGNED_BYTE;
pub const UNSIGNED_SHORT: Type = gl::UNSIGNED_SHORT;
pub const UNSIGNED_INT: Type = gl::UNSIGNED_INT;
pub const SHORT: Type = gl::SHORT;
pub const FLOAT: Type = gl::FLOAT;

/// `TextureTarget` is a target were a texture can be bound to
pub type TextureTarget = u32;

pub const TEXTURE_2D: TextureTarget = gl::TEXTURE_2D;
pub const TEXTURE_2D_MULTISAMPLE: TextureTarget = gl::TEXTURE_2D_MULTISAMPLE;
pub const TEXTURE_2D_ARRAY: TextureTarget = gl::TEXTURE_2D_ARRAY;
pub const TEXTURE_3D: TextureTarget = gl::TEXTURE_3D;

/// `TextureFormat` is the format of a texture either internally or
/// to be uploaded.
pub type TextureFormat = u32;

pub const RED: TextureFormat = gl::RED;
pub const RGB: TextureFormat = gl::RGB;
pub const RGBA: TextureFormat = gl::RGBA;
pub const RGBA8: TextureFormat = gl::RGBA8;
pub const RGBA16F: TextureFormat = gl::RGBA16F;
pub const R16F: TextureFormat = gl::R16F;
pub const DEPTH_COMPONENT24: TextureFormat = gl::DEPTH_COMPONENT24;
pub const DEPTH_COMPONENT: TextureFormat = gl::DEPTH_COMPONENT;

/// `TextureParameter` is a parameter that can be read or set on a texture.
pub type TextureParameter = u32;

pub const TEXTURE_MIN_FILTER: TextureParameter = gl::TEXTURE_MIN_FILTER;
pub const TEXTURE_MAG_FILTER: TextureParameter = gl::TEXTURE_MAG_FILTER;
pub const TEXTURE_WRAP_S: TextureParameter = gl::TEXTURE_WRAP_S;
pub const TEXTURE_WRAP_T: TextureParameter = gl::TEXTURE_WRAP_T;
pub const TEXTURE_MAX_LEVEL: TextureParameter = gl::TEXTURE_MAX_LEVEL;

/// `TextureValue` is a value that be set on a texture's parameter.
pub type TextureValue = i32;

pub const NEAREST: TextureValue = gl::NEAREST as TextureValue;
pub const LINEAR: TextureValue = gl::LINEAR as TextureValue;
pub const LINEAR_MIPMAP_LINEAR: TextureValue = gl::LINEAR_MIPMAP_LINEAR as TextureValue;
pub const LINEAR_MIPMAP_NEAREST: TextureValue = gl::LINEAR_MIPMAP_NEAREST as TextureValue;
pub const NEAREST_MIPMAP_NEAREST: TextureValue = gl::NEAREST_MIPMAP_NEAREST as TextureValue;
pub const NEAREST_MIPMAP_LINEAR: TextureValue = gl::NEAREST_MIPMAP_LINEAR as TextureValue;
pub const CLAMP_TO_EDGE: TextureValue = gl::CLAMP_TO_EDGE as TextureValue;

/// `Texture` is a buffer of data used by fragment shaders.
pub struct Texture(u32);

impl Texture {
    // Allocates a new texture.
    pub fn new() -> Texture {
        Texture(unsafe { glow_context().create_texture().expect("create texture failed") })
    }

    /// Binds the texture to the passed target.
    pub fn bind(&self, target: TextureTarget) {
        unsafe {
            glow_context().bind_texture(target, Some(self.0));
        }
    }

    pub fn get_pixels(&self,
                      target: TextureTarget,
                      level: i32,
                      format: TextureFormat,
                      ty: Type,
                      pixels: &mut [u8]) {
        unsafe {
            glow_context().get_tex_image_u8_slice(target,
                            level,
                            format,
                            ty,
                            Some(pixels));
        }
    }

    pub fn image_2d(&self,
                    target: TextureTarget,
                    level: i32,
                    width: u32,
                    height: u32,
                    format: TextureFormat,
                    ty: Type,
                    pix: Option<&[u8]>) {
        unsafe {
            glow_context().tex_image_2d(target,
                           level,
                           format as i32,
                           width as i32,
                           height as i32,
                           0,
                           format,
                           ty,
                           pix
            );
        }
    }

    pub fn sub_image_2d(&self,
                    target: TextureTarget,
                    level: i32,
                    x: u32,
                    y: u32,
                    width: u32,
                    height: u32,
                    format: TextureFormat,
                    ty: Type,
                    pix: &[u8]) {
        unsafe {
            glow_context().tex_sub_image_2d_u8_slice(target,
                           level,
                           x as i32,
                           y as i32,
                           width as i32,
                           height as i32,
                           format,
                           ty,
                           Some(pix)
            );
        }
    }

    pub fn image_2d_ex(&self,
                    target: TextureTarget,
                    level: i32,
                    width: u32,
                    height: u32,
                    internal_format: TextureFormat,
                    format: TextureFormat,
                    ty: Type,
                    pix: Option<&[u8]>) {
        unsafe {
            glow_context().tex_image_2d(target,
                           level,
                           internal_format as i32,
                           width as i32,
                           height as i32,
                           0,
                           format,
                           ty,
                           pix
            );
        }
    }

    pub fn image_2d_sample(&self,
                    target: TextureTarget,
                    samples: i32,
                    width: u32,
                    height: u32,
                    format: TextureFormat,
                    fixed: bool) {
        unsafe {
            let result: i32 = glow_context().get_parameter_i32(gl::MAX_SAMPLES);
            let use_samples =
                if samples > result {
                    info!("glTexImage2DMultisample: requested {} samples but GL_MAX_SAMPLES is {}", samples, result);
                    result
                } else {
                    samples
                };

            // TODO: switch to glRenderbufferStorageMultisample?
            // from glTexImage2DMultisample which isn't in WebGL
            /*
            glow_context().tex_image_2d_multisample(target,
                           use_samples,
                           format,
                           width as i32,
                           height as i32,
                           fixed as u8
            );
            */
        }
    }

    pub fn image_3d(&self,
                    target: TextureTarget,
                    level: i32,
                    width: u32,
                    height: u32,
                    depth: u32,
                    format: TextureFormat,
                    ty: Type,
                    pix: &[u8]) {
        unsafe {
            glow_context().tex_image_3d(target,
                           level,
                           format as i32,
                           width as i32,
                           height as i32,
                           depth as i32,
                           0,
                           format,
                           ty,
                           Some(pix));
        }
    }

    pub fn sub_image_3d(&self,
                        target: TextureTarget,
                        level: i32,
                        x: u32,
                        y: u32,
                        z: u32,
                        width: u32,
                        height: u32,
                        depth: u32,
                        format: TextureFormat,
                        ty: Type,
                        pix: &[u8]) {
        unsafe {
            glow_context().tex_sub_image_3d_u8_slice(target,
                              level,
                              x as i32,
                              y as i32,
                              z as i32,
                              width as i32,
                              height as i32,
                              depth as i32,
                              format,
                              ty,
                              Some(pix));
        }
    }

    pub fn set_parameter(&self,
                         target: TextureTarget,
                         param: TextureParameter,
                         value: TextureValue) {
        unsafe {
            glow_context().tex_parameter_i32(target, param, value);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            glow_context().delete_texture(self.0);
        }
    }
}

pub type ShaderType = u32;

pub const VERTEX_SHADER: ShaderType = gl::VERTEX_SHADER;
pub const FRAGMENT_SHADER: ShaderType = gl::FRAGMENT_SHADER;
pub const GEOMETRY_SHADER: ShaderType = gl::GEOMETRY_SHADER;

pub type ShaderParameter = u32;

pub const COMPILE_STATUS: ShaderParameter = gl::COMPILE_STATUS;
pub const INFO_LOG_LENGTH: ShaderParameter = gl::INFO_LOG_LENGTH;

pub struct Program(u32);

impl Program {
    pub fn new() -> Program {
        Program(unsafe { glow_context().create_program().expect("program creation failed") })
    }

    pub fn attach_shader(&self, shader: Shader) {
        unsafe {
            glow_context().attach_shader(self.0, shader.0);
        }
    }

    pub fn link(&self) {
        unsafe {
            glow_context().link_program(self.0);
        }
    }

    pub fn use_program(&self) {
        unsafe {
            glow_context().use_program(Some(self.0));
        }
    }

    pub fn uniform_location(&self, name: &str) -> Option<Uniform> {
        let u = unsafe {
            glow_context().get_uniform_location(self.0, name)
        };
        if let Some(u) = u {
            Some(Uniform(u))
        } else {
            None
        }
    }

    pub fn attribute_location(&self, name: &str) -> Option<Attribute> {
        let a = unsafe {
            glow_context().get_attrib_location(self.0, name)
        };
        if let Some(a) = a {
            Some(Attribute(a as i32))
        } else {
            None
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            glow_context().delete_program(self.0);
        }
    }
}

pub struct Shader(u32);

impl Shader {
    pub fn new(ty: ShaderType) -> Shader {
        Shader(unsafe { glow_context().create_shader(ty).expect("failed to create shader") })
    }

    pub fn set_source(&self, src: &str) {
        unsafe {
            glow_context().shader_source(self.0, src);
        }
    }

    pub fn compile(&self) {
        unsafe {
            glow_context().compile_shader(self.0);
        }
    }

    pub fn get_shader_compile_status(&self) -> bool {
        unsafe {
            glow_context().get_shader_compile_status(self.0)
        }
    }

    pub fn get_info_log(&self) -> String {
        unsafe {
            glow_context().get_shader_info_log(self.0)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Uniform(u32);

impl Uniform {
    pub fn set_int(&self, val: i32) {
        unsafe {
            glow_context().uniform_1_i32(Some(&self.0), val);
        }
    }

    pub fn set_int3(&self, x: i32, y: i32, z: i32) {
        unsafe {
            glow_context().uniform_3_i32(Some(&self.0), x, y, z);
        }
    }

    pub fn set_float(&self, val: f32) {
        unsafe {
            glow_context().uniform_1_f32(Some(&self.0), val);
        }
    }

    pub fn set_float2(&self, x: f32, y: f32) {
        unsafe {
            glow_context().uniform_2_f32(Some(&self.0), x, y);
        }
    }

    pub fn set_float3(&self, x: f32, y: f32, z: f32) {
        unsafe {
            glow_context().uniform_3_f32(Some(&self.0), x, y, z);
        }
    }

    pub fn set_float4(&self, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            glow_context().uniform_4_f32(Some(&self.0), x, y, z, w);
        }
    }

    pub fn set_float_mutli_raw(&self, data: *const f32, len: usize) {
        unsafe {
            // TODO: takes a slice, not a raw pointer
            //TODO glow_context().uniform_4_f32_slice(Some(&self.0), len as i32, data);
        }
    }

    pub fn set_matrix4(&self, m: &::cgmath::Matrix4<f32>) {
        use cgmath::Matrix;
        unsafe {
            // TODO
            //TODO glow_context().uniform_matrix_4_f32_slice(Some(&self.0), 1, false as u8, m.as_ptr());
        }
    }

    pub fn set_matrix4_multi(&self, m: &[::cgmath::Matrix4<f32>]) {
        unsafe {
            // TODO
            //TODO glow_context().uniform_matrix_4_f32_slice(Some(&self.0), m.len() as i32, false as u8, m.as_ptr() as *const _); // TODO: Most likely isn't safe
        }
    }
}

#[derive(Clone, Copy)]
pub struct Attribute(i32);

impl Attribute {
    pub fn enable(&self) {
        unsafe {
            glow_context().enable_vertex_attrib_array(self.0 as u32);
        }
    }

    pub fn disable(&self) {
        unsafe {
            glow_context().disable_vertex_attrib_array(self.0 as u32);
        }
    }

    pub fn vertex_pointer(&self, size: i32, ty: Type, normalized: bool, stride: i32, offset: i32) {
        unsafe {
            glow_context().vertex_attrib_pointer_f32(self.0 as u32,
                                    size,
                                    ty,
                                    normalized,
                                    stride,
                                    offset);
        }
    }

    pub fn vertex_pointer_int(&self, size: i32, ty: Type, stride: i32, offset: i32) {
        unsafe {
            glow_context().vertex_attrib_pointer_i32(self.0 as u32,
                                     size,
                                     ty,
                                     stride,
                                     offset);
        }
    }
}

// VertexArray is used to store state needed to render vertices.
// This includes buffers, the format of the buffers and enabled
// attributes.
pub struct VertexArray(u32);

impl VertexArray {
    /// Allocates a new `VertexArray`.
    pub fn new() -> VertexArray {
        VertexArray(unsafe { glow_context().create_vertex_array().expect("create vertex array failed") })
    }

    /// Marks the `VertexArray` as the currently active one, this
    /// means buffers/the format of the buffers etc will be bound to
    /// this `VertexArray`.
    pub fn bind(&self) {
        unsafe {
            glow_context().bind_vertex_array(Some(self.0));
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            glow_context().delete_vertex_array(self.0);
        }
        self.0 = 0;
    }
}

/// `BufferTarget` is a target for a buffer to be bound to.
pub type BufferTarget = u32;

pub const ARRAY_BUFFER: BufferTarget = gl::ARRAY_BUFFER;
pub const ELEMENT_ARRAY_BUFFER: BufferTarget = gl::ELEMENT_ARRAY_BUFFER;

/// `BufferUsage` states how a buffer is going to be used by the program.
pub type BufferUsage = u32;

/// Marks the buffer as 'not going to change' after the
/// initial data upload to be rendered by the gpu.
pub const STATIC_DRAW: BufferUsage = gl::STATIC_DRAW;
/// Marks the buffer as 'changed frequently' during the
/// course of the program whilst being rendered by the gpu.
pub const DYNAMIC_DRAW: BufferUsage = gl::DYNAMIC_DRAW;
/// Marks the buffer as 'changed every frame' whilst being
/// rendered by the gpu.
pub const STREAM_DRAW: BufferUsage = gl::STREAM_DRAW;

/// Access states how a value will be accesed by the program.
pub type Access = u32;

/// States that the returned value will only be read.
pub const READ_ONLY: Access = gl::READ_ONLY;
/// States that the returned value will only be written
/// to.
pub const WRITE_ONLY: Access = gl::WRITE_ONLY;

/// `Buffer` is a storage for vertex data.
pub struct Buffer(u32);

impl Buffer {
    /// Allocates a new Buffer.
    pub fn new() -> Buffer {
        Buffer(unsafe { glow_context().create_buffer().expect("create buffer failed") })
    }

    /// Makes the buffer the currently active one for the given target.
    /// This will allow it to be the source of operations that act on a buffer
    /// (Data, Map etc).
    pub fn bind(&self, target: BufferTarget) {
        unsafe {
            glow_context().bind_buffer(target, Some(self.0));
        }
    }

    pub fn set_data(&self, target: BufferTarget, data: &[u8], usage: BufferUsage) {
        unsafe {
            glow_context().buffer_data_u8_slice(target,
                           data,
                           usage);
        }
    }

    pub fn re_set_data(&self, target: BufferTarget, data: &[u8]) {
        unsafe {
            glow_context().buffer_sub_data_u8_slice(target, 0, data);
        }
    }

    /// Maps the memory in the buffer on the gpu to memory which the program
    /// can access. The access flag will specify how the program plans to use the
    /// returned data. It'll unmap itself once the returned value is dropped.
    ///
    /// Warning: the passed length value is not checked in anyway so it is
    /// possible to overrun the memory. It is up to the program to ensure this
    /// length is valid.
    pub fn map(&self, target: BufferTarget, access: Access, length: usize) -> MappedBuffer {
        unsafe {
            MappedBuffer {
                inner: Vec::from_raw_parts(glow_context().map_buffer_range(target, 0, length as i32, access) as *mut u8, 0, length),
                target,
            }
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            glow_context().delete_buffer(self.0);
        }
    }
}

pub struct MappedBuffer {
    inner: Vec<u8>,
    target: BufferTarget,
}

impl Deref for MappedBuffer {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MappedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for MappedBuffer {
    fn drop(&mut self) {
        unsafe {
            glow_context().unmap_buffer(self.target);
        }
        mem::forget(mem::replace(&mut self.inner, Vec::new()));
    }
}

// Frame buffers

pub type Attachment = u32;
pub const COLOR_ATTACHMENT_0: Attachment = gl::COLOR_ATTACHMENT0;
pub const COLOR_ATTACHMENT_1: Attachment = gl::COLOR_ATTACHMENT1;
pub const COLOR_ATTACHMENT_2: Attachment = gl::COLOR_ATTACHMENT2;
pub const DEPTH_ATTACHMENT: Attachment = gl::DEPTH_ATTACHMENT;

pub struct Framebuffer(u32);

pub fn check_framebuffer_status() {
    unsafe {
        let status = glow_context().check_framebuffer_status(gl::FRAMEBUFFER);
        let s =
        match status {
            gl::FRAMEBUFFER_UNDEFINED => "GL_FRAMEBUFFER_UNDEFINED",
            gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT",
            gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => "GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT",
            gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "GL_FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER",
            gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "GL_FRAMEBUFFER_INCOMPLETE_READ_BUFFER",
            gl::FRAMEBUFFER_UNSUPPORTED => "GL_FRAMEBUFFER_UNSUPPORTED",
            gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "GL_FRAMEBUFFER_INCOMPLETE_MULTISAMPLE",
            gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => "GL_FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS",

            gl::FRAMEBUFFER_COMPLETE => "GL_FRAMEBUFFER_COMPLETE",
            //gl::FRAMEBUFFER_INCOMPLETE_DIMENSIONS => "GL_FRAMEBUFFER_INCOMPLETE_DIMENSIONS",
            _ => "unknown"
        };

        if status != gl::FRAMEBUFFER_COMPLETE {
            panic!("glBindFramebuffer failed, glCheckFrameBufferStatus(GL_FRAMEBUFFER) = {} {}", status, s);
        }
    }
}

pub fn check_gl_error() {
    unsafe {
        loop {
            let err = glow_context().get_error();
            if err == gl::NO_ERROR {
                break
            }

            error!("glGetError = {}", err);
        }
    }
}

impl Framebuffer {
    pub fn new() -> Framebuffer {
        Framebuffer(unsafe { glow_context().create_framebuffer().expect("create framebuffer failed") })
    }

    pub fn bind(&self) {
        unsafe {
            glow_context().bind_framebuffer(gl::FRAMEBUFFER, Some(self.0));
        }
    }

    pub fn bind_read(&self) {
        unsafe {
            glow_context().bind_framebuffer(gl::READ_FRAMEBUFFER, Some(self.0));
        }
    }

    pub fn bind_draw(&self) {
        unsafe {
            glow_context().bind_framebuffer(gl::DRAW_FRAMEBUFFER, Some(self.0));
        }
    }

    pub fn texture_2d(&self, attachment: Attachment, target: TextureTarget, tex: &Texture, level: i32) {
        unsafe {
            glow_context().framebuffer_texture_2d(gl::FRAMEBUFFER, attachment, target, Some(tex.0), level);
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            glow_context().delete_framebuffer(self.0);
        }
    }
}

pub fn unbind_framebuffer() {
    unsafe {
        glow_context().bind_framebuffer(gl::FRAMEBUFFER, Some(0));
    }
}

pub fn unbind_framebuffer_read() {
    unsafe {
        glow_context().bind_framebuffer(gl::READ_FRAMEBUFFER, Some(0));
    }
}

pub fn unbind_framebuffer_draw() {
    unsafe {
        glow_context().bind_framebuffer(gl::DRAW_FRAMEBUFFER, Some(0));
    }
}

pub fn draw_buffers(bufs: &[Attachment]) {
    unsafe {
        glow_context().draw_buffers(bufs);
    }
}

pub fn bind_frag_data_location(p: &Program, cn: u32, name: &str) {
    unsafe {
        glow_context().bind_frag_data_location(p.0, cn, name)
    }
}

pub fn blit_framebuffer(
    sx0: i32, sy0: i32, sx1: i32, sy1: i32,
    dx0: i32, dy0: i32, dx1: i32, dy1: i32,
    mask: ClearFlags, filter: TextureValue) {
    unsafe {
        glow_context().blit_framebuffer(
            sx0, sy0, sx1, sy1,
            dx0, dy0, dx1, dy1,
            mask.internal(), filter as u32
        );
    }
}

pub type TargetBuffer = u32;
pub const COLOR: TargetBuffer = gl::COLOR;

pub fn clear_buffer(buffer: TargetBuffer, draw_buffer: u32, values: &mut [f32]) {
    unsafe {
        // TODO: why does glow have &mut on clear buffer values, why would it change the color?
        glow_context().clear_buffer_f32_slice(buffer, draw_buffer, values);
    }
}
