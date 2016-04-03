
pub struct Material {
    pub renderable: bool,
    pub should_cull_against: bool,
    pub never_cull: bool, // Because leaves suck
    pub force_shade: bool,
    pub transparent: bool,
}

pub const INVISIBLE: Material = Material {
    renderable: false,
    never_cull: false,
    should_cull_against: false,
    force_shade: false,
    transparent: false,
};

pub const SOLID: Material = Material {
    renderable: true,
    never_cull: false,
    should_cull_against: true,
    force_shade: false,
    transparent: false,
};

pub const NON_SOLID: Material = Material {
    renderable: true,
    never_cull: false,
    should_cull_against: false,
    force_shade: false,
    transparent: false,
};

pub const TRANSPARENT: Material = Material {
    renderable: true,
    never_cull: false,
    should_cull_against: false,
    force_shade: false,
    transparent: true,
};

pub const LEAVES: Material = Material {
    renderable: true,
    never_cull: true,
    should_cull_against: false,
    force_shade: true,
    transparent: false,
};
