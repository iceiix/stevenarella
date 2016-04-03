
pub struct Material {
    pub renderable: bool,
    pub should_cull_against: bool,
    pub never_cull: bool, // Because leaves suck
    pub force_shade: bool,
    pub transparent: bool,
    pub absorbed_light: u8,
    pub emitted_light: u8,
}

pub const INVISIBLE: Material = Material {
    renderable: false,
    never_cull: false,
    should_cull_against: false,
    force_shade: false,
    transparent: false,
    absorbed_light: 0, // Special because of sky light
    emitted_light: 0,
};

pub const SOLID: Material = Material {
    renderable: true,
    never_cull: false,
    should_cull_against: true,
    force_shade: false,
    transparent: false,
    absorbed_light: 15,
    emitted_light: 0,
};

pub const NON_SOLID: Material = Material {
    renderable: true,
    never_cull: false,
    should_cull_against: false,
    force_shade: false,
    transparent: false,
    absorbed_light: 1,
    emitted_light: 0,
};

pub const TRANSPARENT: Material = Material {
    renderable: true,
    never_cull: false,
    should_cull_against: false,
    force_shade: false,
    transparent: true,
    absorbed_light: 1,
    emitted_light: 0,
};

pub const LEAVES: Material = Material {
    renderable: true,
    never_cull: true,
    should_cull_against: false,
    force_shade: true,
    transparent: false,
    absorbed_light: 1,
    emitted_light: 0,
};
