
pub mod player;
pub mod block_entity;

use ecs;
use cgmath::Vector3;
use collision::Aabb3;

mod systems;

pub fn add_systems(m: &mut ecs::Manager) {
    let sys = systems::UpdateLastPosition::new(m);
    m.add_system(sys);

    player::add_systems(m);

    let sys = systems::ApplyVelocity::new(m);
    m.add_system(sys);
    let sys = systems::ApplyGravity::new(m);
    m.add_system(sys);
    let sys = systems::LerpPosition::new(m);
    m.add_render_system(sys);
    let sys = systems::LerpRotation::new(m);
    m.add_render_system(sys);
    let sys = systems::LightEntity::new(m);
    m.add_render_system(sys);

    block_entity::add_systems(m);
}

/// Location of an entity in the world.
#[derive(Debug)]
pub struct Position {
    pub position: Vector3<f64>,
    pub last_position: Vector3<f64>,
    pub moved: bool,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Position {
        Position {
            position: Vector3::new(x, y, z),
            last_position: Vector3::new(x, y, z),
            moved: false,
        }
    }

    pub fn zero() -> Position {
        Position::new(0.0, 0.0, 0.0)
    }
}

#[derive(Debug)]
pub struct TargetPosition {
    pub position: Vector3<f64>,
    pub lerp_amount: f64,
}

impl TargetPosition {
    pub fn new(x: f64, y: f64, z: f64) -> TargetPosition {
        TargetPosition {
            position: Vector3::new(x, y, z),
            lerp_amount: 0.2,
        }
    }

    pub fn zero() -> TargetPosition {
        TargetPosition::new(0.0, 0.0, 0.0)
    }
}

/// Velocity of an entity in the world.
#[derive(Debug)]
pub struct Velocity {
    pub velocity: Vector3<f64>,
}

impl Velocity {
    pub fn new(x: f64, y: f64, z: f64) -> Velocity {
        Velocity {
            velocity: Vector3::new(x, y, z),
        }
    }

    pub fn zero() -> Velocity {
        Velocity::new(0.0, 0.0, 0.0)
    }
}

/// Rotation of an entity in the world
#[derive(Debug)]
pub struct Rotation {
    pub yaw: f64,
    pub pitch: f64,
}

impl Rotation {
    pub fn new(yaw: f64, pitch: f64) -> Rotation {
        Rotation {
            yaw: yaw,
            pitch: pitch,
        }
    }

    pub fn zero() -> Rotation {
        Rotation::new(0.0, 0.0)
    }
}
#[derive(Debug)]
pub struct TargetRotation {
    pub yaw: f64,
    pub pitch: f64,
}

impl TargetRotation {
    pub fn new(yaw: f64, pitch: f64) -> TargetRotation {
        TargetRotation {
            yaw: yaw,
            pitch: pitch,
        }
    }

    pub fn zero() -> TargetRotation {
        TargetRotation::new(0.0, 0.0)
    }
}

pub struct Gravity {
    pub on_ground: bool,
}

impl Gravity {
    pub fn new() -> Gravity {
        Gravity {
            on_ground: false,
        }
    }
}

pub struct Bounds {
    pub bounds: Aabb3<f64>,
}

impl Bounds {
    pub fn new(bounds: Aabb3<f64>) -> Bounds {
        Bounds {
            bounds: bounds,
        }
    }
}

pub struct GameInfo {
    pub delta: f64,
}

impl GameInfo {
    pub fn new() -> GameInfo {
        GameInfo {
            delta: 0.0,
        }
    }
}

pub struct Light {
    pub block_light: f32,
    pub sky_light: f32,
}

impl Light {
    pub fn new() -> Light {
        Light {
            block_light: 0.0,
            sky_light: 0.0,
        }
    }
}
