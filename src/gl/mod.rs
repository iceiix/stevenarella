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

extern crate steven_gl as gl;
extern crate glfw;

use std::ops::BitOr;
use std::ffi;
use std::mem;
use std::ptr;
use std::ops::{Deref, DerefMut};

/// Inits the gl library. This should be called once a context is ready.
pub fn init(window: &mut glfw::Window) {
    gl::load_with(|s| window.get_proc_address(s));
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

pub fn draw_elements(ty: DrawType, count: usize, dty: Type, offset: usize) {
    unsafe {
        gl::DrawElements(ty, count as i32, dty, offset as *const gl::types::GLvoid);
    }
}

/// Sets the size of the viewport of this context.
pub fn viewport(x: i32, y: i32, w: i32, h: i32) {
    unsafe { gl::Viewport(x, y, w, h); }
}

/// Sets the color the color buffer should be cleared to
/// when Clear is called with the color flag.
pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe { gl::ClearColor(r, g, b, a); }
}

/// ClearFlags is a set of flags to mark what should be cleared during
/// a Clear call.
pub enum ClearFlags {
    /// Marks the color buffer to be cleared
    Color,
    /// Marks the depth buffer to be cleared
    Depth,
    Internal(u32)
}

impl ClearFlags {
    fn internal(self) -> u32 {
        match self {
            ClearFlags::Color => gl::COLOR_BUFFER_BIT,
            ClearFlags::Depth => gl::DEPTH_BUFFER_BIT,
            ClearFlags::Internal(val) => val
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
    unsafe { gl::Clear(flags.internal()) }
}

/// Func is a function to be preformed on two values.
pub type Func = u32;

pub const NEVER: Func = gl::NEVER;
pub const LESS: Func = gl::LESS;
pub const LESS_OR_EQUAL: Func = gl::LEQUAL;
pub const GREATER: Func = gl::GREATER;
pub const ALWAYS: Func = gl::ALWAYS;
pub const EQUAL: Func = gl::EQUAL;

pub fn depth_func(f: Func) {
    unsafe { gl::DepthFunc(f); }
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
    unsafe { gl::Enable(f); }
}

/// Disables the passed flag.
pub fn disable(f: Flag) {
    unsafe { gl::Disable(f); }
}

/// Sets the texture slot with the passed id as the
/// currently active one.
pub fn active_texture(id: u32) {
    unsafe { gl::ActiveTexture(gl::TEXTURE0 + id); }
}

/// Factor is used in blending
pub type Factor = u32;
pub const SRC_ALPHA: Factor = gl::SRC_ALPHA;
pub const ONE_MINUS_SRC_ALPHA: Factor = gl::ONE_MINUS_SRC_ALPHA;
pub const ONE_FACTOR: Factor = gl::ONE;
pub const ZERO_FACTOR: Factor = gl::ZERO;

/// Sets the factors to be used when blending.
pub fn blend_func(s_factor: Factor, d_factor: Factor) {
    unsafe { gl::BlendFunc(s_factor, d_factor); }
}

/// Type is a type of data used by various operations.
pub type Type = u32;
pub const UNSIGNED_BYTE: Type = gl::UNSIGNED_BYTE;
pub const UNSIGNED_SHORT: Type = gl::UNSIGNED_SHORT;
pub const UNSIGNED_INT: Type = gl::UNSIGNED_INT;
pub const SHORT: Type = gl::SHORT;
pub const FLOAT: Type = gl::FLOAT;

/// TextureTarget is a target were a texture can be bound to
pub type TextureTarget = u32;

pub const TEXTURE_2D: TextureTarget = gl::TEXTURE_2D;
pub const TEXTURE_2D_MULTISAMPLE: TextureTarget = gl::TEXTURE_2D_MULTISAMPLE;
pub const TEXTURE_2D_ARRAY: TextureTarget = gl::TEXTURE_2D_ARRAY;
pub const TEXTURE_3D: TextureTarget = gl::TEXTURE_3D;

/// TextureFormat is the format of a texture either internally or
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

/// TextureParameter is a parameter that can be read or set on a texture.
pub type TextureParameter = u32;

pub const TEXTURE_MIN_FILTER: TextureParameter = gl::TEXTURE_MIN_FILTER;
pub const TEXTURE_MAG_FILTER: TextureParameter = gl::TEXTURE_MAG_FILTER;
pub const TEXTURE_WRAP_S: TextureParameter = gl::TEXTURE_WRAP_S;
pub const TEXTURE_WRAP_T: TextureParameter = gl::TEXTURE_WRAP_T;
pub const TEXTURE_MAX_LEVEL: TextureParameter = gl::TEXTURE_MAX_LEVEL;

/// TextureValue is a value that be set on a texture's parameter.
pub type TextureValue = i32;

pub const NEAREST: TextureValue = gl::NEAREST as TextureValue;
pub const LINEAR: TextureValue = gl::LINEAR as TextureValue;
pub const LINEAR_MIPMAP_LINEAR: TextureValue = gl::LINEAR_MIPMAP_LINEAR as TextureValue;
pub const LINEAR_MIPMAP_NEAREST: TextureValue = gl::LINEAR_MIPMAP_NEAREST as TextureValue;
pub const NEAREST_MIPMAP_NEAREST: TextureValue = gl::NEAREST_MIPMAP_NEAREST as TextureValue;
pub const NEAREST_MIPMAP_LINEAR: TextureValue = gl::NEAREST_MIPMAP_LINEAR as TextureValue;
pub const CLAMP_TO_EDGE: TextureValue = gl::CLAMP_TO_EDGE as TextureValue;

/// Texture is a buffer of data used by fragment shaders.
pub struct Texture {
    internal: u32,
}

impl Texture {
    // Allocates a new texture.
    pub fn new() -> Texture {
        let mut t = Texture{ internal: 0 };
        unsafe { gl::GenTextures(1, &mut t.internal); }
        t
    }

    /// Binds the texture to the passed target.
    pub fn bind(&self, target: TextureTarget) {
        unsafe {
            gl::BindTexture(target, self.internal);
        }
    }

    pub fn get_pixels(&self, target: TextureTarget, level: i32, format: TextureFormat, ty: Type, pixels: &mut [u8]) {
        unsafe {
            gl::GetTexImage(target, level, format, ty, pixels.as_mut_ptr() as *mut gl::types::GLvoid);
        }
    }

    pub fn image_3d(&self, target: TextureTarget, level: i32, width: u32, height: u32, depth: u32, format: TextureFormat, ty: Type, pix: &[u8]) {
        unsafe {
            gl::TexImage3D(target, level, format as i32, width as i32, height as i32, depth as i32, 0, format, ty, pix.as_ptr() as *const gl::types::GLvoid);
        }
    }

    pub fn sub_image_3d(&self, target: TextureTarget, level: i32, x: u32, y: u32, z: u32, width: u32, height: u32, depth: u32, format: TextureFormat, ty: Type, pix: &[u8]) {
        unsafe {
            gl::TexSubImage3D(target, level, x as i32, y as i32, z as i32, width as i32, height as i32, depth as i32, format, ty, pix.as_ptr() as *const gl::types::GLvoid);
        }
    }

    pub fn set_parameter(&self, target: TextureTarget, param: TextureParameter, value: TextureValue) {
        unsafe {
            gl::TexParameteri(target, param, value);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.internal); }
    }
}

pub type ShaderType = u32;

pub const VERTEX_SHADER: ShaderType = gl::VERTEX_SHADER;
pub const FRAGMENT_SHADER: ShaderType = gl::FRAGMENT_SHADER;
pub const GEOMETRY_SHADER: ShaderType = gl::GEOMETRY_SHADER;

pub type ShaderParameter = u32;

pub const COMPILE_STATUS: ShaderParameter = gl::COMPILE_STATUS;
pub const INFO_LOG_LENGTH: ShaderParameter = gl::INFO_LOG_LENGTH;

pub struct Program {
    internal: u32,
}

impl Program {
    pub fn new() -> Program {
        Program {
            internal: unsafe { gl::CreateProgram() }
        }
    }

    pub fn attach_shader(&self, shader: Shader) {
        unsafe {
            gl::AttachShader(self.internal, shader.internal);
        }
    }

    pub fn link(&self) {
        unsafe {
            gl::LinkProgram(self.internal);
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.internal);
        }
    }

    pub fn uniform_location(&self, name: &str) -> Uniform {
        Uniform {
            internal: unsafe { gl::GetUniformLocation(self.internal, ffi::CString::new(name).unwrap().as_ptr()) }
        }
    }

    pub fn attribute_location(&self, name: &str) -> Attribute {
        Attribute {
            internal: unsafe { gl::GetAttribLocation(self.internal, ffi::CString::new(name).unwrap().as_ptr()) }
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.internal); }
    }
}

pub struct Shader {
    internal: u32,
}

impl Shader {
    pub fn new(ty: ShaderType) -> Shader {
        Shader {
            internal: unsafe { gl::CreateShader(ty) }
        }
    }

    pub fn set_source(&self, src: &str) {
        unsafe {
            gl::ShaderSource(self.internal, 1, &ffi::CString::new(src).unwrap().as_ptr(), ptr::null());
        }
    }

    pub fn compile(&self) {
        unsafe {
            gl::CompileShader(self.internal);
        }
    }

    pub fn get_parameter(&self, param: ShaderParameter) -> i32 {
        let mut ret : i32 = 0;
        unsafe { gl::GetShaderiv(self.internal, param, &mut ret); }
        return ret;
    }

    pub fn get_info_log(&self) -> String {
        let len = self.get_parameter(INFO_LOG_LENGTH);

        let mut data = Vec::<u8>::with_capacity(len as usize);
        unsafe { 
            data.set_len(len as usize);
            gl::GetShaderInfoLog(self.internal, len, ptr::null_mut(), data.as_mut_ptr() as *mut i8);
        }
        String::from_utf8(data).unwrap()
    }
}

pub struct Uniform {
    internal: i32,
}

impl Uniform {
    pub fn set_int(&self, val: i32) {
        unsafe {
            gl::Uniform1i(self.internal, val);
        }
    }

    pub fn set_int3(&self, x: i32, y: i32, z: i32) {
        unsafe {
            gl::Uniform3i(self.internal, x, y, z);
        }
    }

    pub fn set_float(&self, val: f32) {
        unsafe {
            gl::Uniform1f(self.internal, val);
        }
    }

    pub fn set_float2(&self, x: f32, y: f32) {
        unsafe {
            gl::Uniform2f(self.internal, x, y);
        }
    }

    pub fn set_float3(&self, x: f32, y: f32, z: f32) {
        unsafe {
            gl::Uniform3f(self.internal, x, y, z);
        }
    }

    pub fn set_float4(&self, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            gl::Uniform4f(self.internal, x, y, z, w);
        }
    }
}

pub struct Attribute {
    internal: i32,
}

impl Attribute {
    pub fn enable(&self) {
        unsafe {
            gl::EnableVertexAttribArray(self.internal as u32);
        }
    }

    pub fn disable(&self) {
        unsafe {
            gl::DisableVertexAttribArray(self.internal as u32);
        }
    }

    pub fn vertex_pointer(&self, size: i32, ty: Type, normalized: bool, stride: i32, offset: i32) {
        unsafe {
            gl::VertexAttribPointer(self.internal as u32, size, ty, normalized as u8, stride, offset as *const gl::types::GLvoid);
        }
    }

    pub fn vertex_pointer_int(&self, size: i32, ty: Type, stride: i32, offset: i32) {
        unsafe {
            gl::VertexAttribIPointer(self.internal as u32, size, ty, stride, offset as *const gl::types::GLvoid);
        }
    }
}

// VertexArray is used to store state needed to render vertices.
// This includes buffers, the format of the buffers and enabled
// attributes.
pub struct VertexArray {
    internal: u32,
}

impl VertexArray {
    /// Allocates a new VertexArray.
    pub fn new() -> VertexArray {
        let mut va = VertexArray {
            internal: 0,
        };
        unsafe { gl::GenVertexArrays(1, &mut va.internal); }
        va
    }

    /// Marks the VertexArray as the currently active one, this
    /// means buffers/the format of the buffers etc will be bound to
    /// this VertexArray.
    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.internal); }
    }
}

impl Drop for VertexArray {   
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &self.internal); }
        self.internal = 0;
    }
}

/// BufferTarget is a target for a buffer to be bound to.
pub type BufferTarget = u32;

pub const ARRAY_BUFFER: BufferTarget = gl::ARRAY_BUFFER;
pub const ELEMENT_ARRAY_BUFFER: BufferTarget = gl::ELEMENT_ARRAY_BUFFER;

/// BufferUsage states how a buffer is going to be used by the program.
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

/// Buffer is a storage for vertex data.
pub struct Buffer {
    internal: u32,
}

impl Buffer {
    /// Allocates a new Buffer. 
    pub fn new() -> Buffer {
        let mut b = Buffer {
            internal: 0,
        };
        unsafe { gl::GenBuffers(1, &mut b.internal); }
        b
    }

    /// Makes the buffer the currently active one for the given target.
    /// This will allow it to be the source of operations that act on a buffer
    /// (Data, Map etc).
    pub fn bind(&self, target: BufferTarget) { 
        unsafe { gl::BindBuffer(target, self.internal); }
    }

    pub fn set_data(&self, target: BufferTarget, data: &[u8], usage: BufferUsage) {
        unsafe {
            gl::BufferData(target, data.len() as i64, data.as_ptr() as *const gl::types::GLvoid, usage);
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
            MappedBuffer{inner: Vec::from_raw_parts(gl::MapBuffer(target, access) as *mut u8, 0, length), target: target}
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.internal);
        }
    }
}

pub struct MappedBuffer {
    inner: Vec<u8>,
    target: BufferTarget,
}

impl Deref for MappedBuffer {
    type Target = Vec<u8>;

    fn deref<'a>(&'a self) -> &'a Self::Target {
        &self.inner
    }
}

impl DerefMut for MappedBuffer {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target {
        &mut self.inner
    }
}

impl Drop for MappedBuffer {
    fn drop(&mut self) {
        unsafe { gl::UnmapBuffer(self.target); }
        mem::forget(mem::replace(&mut self.inner, Vec::new()));
    }
}
