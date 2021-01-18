use crate::console;
use std::marker::PhantomData;
use winit::event::VirtualKeyCode;
// Might just rename this to settings.rs

pub const R_MAX_FPS: console::CVar<i64> = console::CVar {
    ty: PhantomData,
    name: "r_max_fps",
    description: "fps_max caps the maximum FPS for the rendering engine",
    mutable: true,
    serializable: true,
    default: &|| 60,
};

pub const R_FOV: console::CVar<i64> = console::CVar {
    ty: PhantomData,
    name: "r_fov",
    description: "Setting for controlling the client field of view",
    mutable: true,
    serializable: true,
    default: &|| 90,
};

pub const R_VSYNC: console::CVar<bool> = console::CVar {
    ty: PhantomData,
    name: "r_vsync",
    description: "Toggle to enable/disable vsync",
    mutable: true,
    serializable: true,
    default: &|| false,
};

pub const CL_MASTER_VOLUME: console::CVar<i64> = console::CVar {
    ty: PhantomData,
    name: "cl_master_volume",
    description: "Main volume control",
    mutable: true,
    serializable: true,
    default: &|| 100,
};

macro_rules! create_keybind {
    ($keycode:ident, $name:expr, $description:expr) => {
        console::CVar {
            ty: PhantomData,
            name: $name,
            description: $description,
            mutable: true,
            serializable: true,
            default: &|| VirtualKeyCode::$keycode as i64,
        }
    };
}

pub const CL_KEYBIND_FORWARD: console::CVar<i64> =
    create_keybind!(W, "cl_keybind_forward", "Keybinding for moving forward");
pub const CL_KEYBIND_BACKWARD: console::CVar<i64> =
    create_keybind!(S, "cl_keybind_backward", "Keybinding for moving backward");
pub const CL_KEYBIND_LEFT: console::CVar<i64> =
    create_keybind!(A, "cl_keybind_left", "Keybinding for moving the left");
pub const CL_KEYBIND_RIGHT: console::CVar<i64> =
    create_keybind!(D, "cl_keybind_right", "Keybinding for moving to the right");
pub const CL_KEYBIND_OPEN_INV: console::CVar<i64> = create_keybind!(
    E,
    "cl_keybind_open_inv",
    "Keybinding for opening the inventory"
);
pub const CL_KEYBIND_SNEAK: console::CVar<i64> =
    create_keybind!(LShift, "cl_keybind_sneak", "Keybinding for sneaking");
pub const CL_KEYBIND_SPRINT: console::CVar<i64> =
    create_keybind!(LControl, "cl_keybind_sprint", "Keybinding for sprinting");
pub const CL_KEYBIND_JUMP: console::CVar<i64> =
    create_keybind!(Space, "cl_keybind_jump", "Keybinding for jumping");

pub const DOUBLE_JUMP_MS: u32 = 100;

pub fn register_vars(vars: &mut console::Vars) {
    vars.register(R_MAX_FPS);
    vars.register(R_FOV);
    vars.register(R_VSYNC);
    vars.register(CL_MASTER_VOLUME);
    vars.register(CL_KEYBIND_FORWARD);
    vars.register(CL_KEYBIND_BACKWARD);
    vars.register(CL_KEYBIND_LEFT);
    vars.register(CL_KEYBIND_RIGHT);
    vars.register(CL_KEYBIND_OPEN_INV);
    vars.register(CL_KEYBIND_SNEAK);
    vars.register(CL_KEYBIND_SPRINT);
    vars.register(CL_KEYBIND_JUMP);
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum Stevenkey {
    Forward,
    Backward,
    Left,
    Right,
    OpenInv,
    Sneak,
    Sprint,
    Jump,
}

impl Stevenkey {
    pub fn values() -> Vec<Stevenkey> {
        vec![
            Stevenkey::Forward,
            Stevenkey::Backward,
            Stevenkey::Left,
            Stevenkey::Right,
            Stevenkey::OpenInv,
            Stevenkey::Sneak,
            Stevenkey::Sprint,
            Stevenkey::Jump,
        ]
    }

    pub fn get_by_keycode(keycode: VirtualKeyCode, vars: &console::Vars) -> Option<Stevenkey> {
        for steven_key in Stevenkey::values() {
            if keycode as i64 == *vars.get(steven_key.get_cvar()) {
                return Some(steven_key);
            }
        }
        None
    }

    pub fn get_cvar(&self) -> console::CVar<i64> {
        match *self {
            Stevenkey::Forward => CL_KEYBIND_FORWARD,
            Stevenkey::Backward => CL_KEYBIND_BACKWARD,
            Stevenkey::Left => CL_KEYBIND_LEFT,
            Stevenkey::Right => CL_KEYBIND_RIGHT,
            Stevenkey::OpenInv => CL_KEYBIND_OPEN_INV,
            Stevenkey::Sneak => CL_KEYBIND_SNEAK,
            Stevenkey::Sprint => CL_KEYBIND_SPRINT,
            Stevenkey::Jump => CL_KEYBIND_JUMP,
        }
    }
}
