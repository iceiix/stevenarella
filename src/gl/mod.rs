extern crate steven_gl as gl;
extern crate glfw;

use std::ops::BitOr;

pub fn init(window: &mut glfw::Window) {
    gl::load_with(|s| window.get_proc_address(s));
}

pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe { gl::ClearColor(r, g, b, a); }
}

pub enum ClearFlags {
    Color,
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

pub fn clear(flags: ClearFlags) {
    unsafe { gl::Clear(flags.internal()) }
}
