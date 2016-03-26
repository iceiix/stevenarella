
use ecs;
use super::{
    Position,
    Velocity,
    Rotation,
    Gravity,
    Bounds,
    Proxy,
    GameInfo,
};
use world;
use types::Gamemode;
use collision::{Aabb, Aabb3};
use cgmath::{self, Point3};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use types::hash::FNVHash;
use sdl2::keyboard::Keycode;

pub fn add_systems(m: &mut ecs::Manager) {
    // Not actually rendering related but the faster
    // we can handle input the better.
    let sys = MovementHandler::new(m);
    m.add_render_system(sys);
}

pub fn create_local(m: &mut ecs::Manager) -> ecs::Entity {
    let entity = m.create_entity();
    m.add_component_direct(entity, Position::new(0.5, 13.2, 0.5));
    m.add_component_direct(entity, Rotation::new(0.0, 0.0));
    m.add_component_direct(entity, Velocity::new(0.0, 0.0, 0.0));
    m.add_component_direct(entity, Gamemode::Survival);
    m.add_component_direct(entity, Gravity::new());
    m.add_component_direct(entity, PlayerMovement::new());
    m.add_component_direct(entity, Bounds::new(Aabb3::new(
        Point3::new(-0.3, 0.0, -0.3),
        Point3::new(0.3, 1.8, 0.3)
    )));
    entity
}

pub struct PlayerMovement {
    pub flying: bool,
    pub did_touch_ground: bool,
    pub pressed_keys: HashMap<Keycode, bool, BuildHasherDefault<FNVHash>>,
}

impl PlayerMovement {
    pub fn new() -> PlayerMovement {
        PlayerMovement {
            flying: false,
            did_touch_ground: false,
            pressed_keys: HashMap::with_hasher(BuildHasherDefault::default()),
        }
    }

    fn calculate_movement(&self, player_yaw: f64) -> (f64, f64) {
        use std::f64::consts::PI;
        let mut forward = 0.0f64;
        let mut yaw = player_yaw - (PI/2.0);
        if self.is_key_pressed(Keycode::W) || self.is_key_pressed(Keycode::S) {
            forward = 1.0;
            if self.is_key_pressed(Keycode::S) {
                yaw += PI;
            }
        }
        let mut change = 0.0;
        if self.is_key_pressed(Keycode::A) {
            change = (PI / 2.0) / (forward.abs() + 1.0);
        }
        if self.is_key_pressed(Keycode::D) {
            change = -(PI / 2.0) / (forward.abs() + 1.0);
        }
        if self.is_key_pressed(Keycode::A) || self.is_key_pressed(Keycode::D) {
            forward = 1.0;
        }
        if self.is_key_pressed(Keycode::S) {
            yaw -= change;
        } else {
            yaw += change;
        }

        (forward, yaw)
    }

    fn is_key_pressed(&self, key: Keycode) -> bool {
        self.pressed_keys.get(&key).map_or(false, |v| *v)
    }
}

struct MovementHandler {
    filter: ecs::Filter,
    movement: ecs::Key<PlayerMovement>,
    gravity: ecs::Key<Gravity>,
    gamemode: ecs::Key<Gamemode>,
    world: ecs::Key<Proxy<world::World>>,
    position: ecs::Key<Position>,
    velocity: ecs::Key<Velocity>,
    game_info: ecs::Key<GameInfo>,
    bounds: ecs::Key<Bounds>,
    rotation: ecs::Key<Rotation>,
}

impl MovementHandler {
    pub fn new(m: &mut ecs::Manager) -> MovementHandler {
        let movement = m.get_key();
        let position = m.get_key();
        let velocity = m.get_key();
        let bounds = m.get_key();
        let rotation = m.get_key();
        MovementHandler {
            filter: ecs::Filter::new()
                .with(movement)
                .with(position)
                .with(velocity)
                .with(bounds)
                .with(rotation),
            movement: movement,
            gravity: m.get_key(),
            gamemode: m.get_key(),
            world: m.get_key(),
            position: position,
            velocity: velocity,
            game_info: m.get_key(),
            bounds: bounds,
            rotation: rotation,
        }
    }
}

impl ecs::System for MovementHandler {
    fn update(&mut self, m: &mut ecs::Manager) {
        let world_entity = m.get_world();
        let world: &world::World = m.get_component(world_entity, self.world).unwrap();
        let delta = m.get_component(world_entity, self.game_info).unwrap().delta;
        for e in m.find(&self.filter) {
            let movement = m.get_component_mut(e, self.movement).unwrap();
            if movement.flying && m.get_component(e, self.gravity).is_some() {
                m.remove_component(e, self.gravity);
            } else if !movement.flying && m.get_component(e, self.gravity).is_none() {
                m.add_component(e, self.gravity, Gravity::new());
            }
            let gamemode = m.get_component(e, self.gamemode).unwrap();
            movement.flying |= gamemode.always_fly();

            let position = m.get_component_mut(e, self.position).unwrap();
            let rotation = m.get_component(e, self.rotation).unwrap();
            let velocity = m.get_component_mut(e, self.velocity).unwrap();
            let gravity = m.get_component_mut(e, self.gravity);

            let player_bounds = m.get_component(e, self.bounds).unwrap().bounds;

            if world.is_chunk_loaded((position.position.x as i32) >> 4, (position.position.z as i32) >> 4) {
                let (forward, yaw) = movement.calculate_movement(rotation.yaw);
                let mut speed = 4.317 / 60.0;
                if movement.is_key_pressed(Keycode::LShift) {
                    speed = 5.612 / 60.0;
                }
                if movement.flying {
                    speed *= 2.5;

                    if movement.is_key_pressed(Keycode::Space) {
                        position.position.y += speed * delta;
                    }
                    if movement.is_key_pressed(Keycode::LCtrl) {
                        position.position.y -= speed * delta;
                    }
                } else if gravity.as_ref().map_or(false, |v| v.on_ground) {
                    if movement.is_key_pressed(Keycode::Space) {
                        velocity.velocity.y = 0.15;
                    } else {
                        velocity.velocity.y = 0.0;
                    }
                } else {
                    velocity.velocity.y -= 0.01 * delta;
                    if velocity.velocity.y < -0.3 {
                        velocity.velocity.y = -0.3;
                    }
                }
                position.position.x += forward * yaw.cos() * delta * speed;
                position.position.z -= forward * yaw.sin() * delta * speed;
                position.position.y += velocity.velocity.y * delta;
            }

            if !gamemode.noclip() {
                let mut target = position.position;
                position.position.y = position.last_position.y;
                position.position.z = position.last_position.z;

                // We handle each axis separately to allow for a sliding
                // effect when pushing up against walls.

                let (bounds, xhit) = check_collisions(world, position, player_bounds);
                position.position.x = bounds.min.x + 0.3;
                position.last_position.x = position.position.x;

                position.position.z = target.z;
                let (bounds, zhit) = check_collisions(world, position, player_bounds);
                position.position.z = bounds.min.z + 0.3;
                position.last_position.z = position.position.z;

                // Half block jumps
                // Minecraft lets you 'jump' up 0.5 blocks
                // for slabs and stairs (or smaller blocks).
                // Currently we implement this as a teleport to the
                // top of the block if we could move there
                // but this isn't smooth.
                if (xhit || zhit) && gravity.as_ref().map_or(false, |v| v.on_ground) {
                    let mut ox = position.position.x;
                    let mut oz = position.position.z;
                    position.position.x = target.x;
                    position.position.z = target.z;
                    for offset in 1 .. 9 {
                        let mini = player_bounds.add_v(cgmath::Vector3::new(0.0, offset as f64 / 16.0, 0.0));
                        let (_, hit) = check_collisions(world, position, mini);
                        if !hit {
                            target.y += offset as f64 / 16.0;
                            ox = target.x;
                            oz = target.z;
                            break;
                        }
                    }
                    position.position.x = ox;
                    position.position.z = oz;
                }

                position.position.y = target.y;
                let (bounds, yhit) = check_collisions(world, position, player_bounds);
                position.position.y = bounds.min.y;
                position.last_position.y = position.position.y;
                if yhit {
                    velocity.velocity.y = 0.0;
                }

                if let Some(gravity) = gravity {
                    let ground = Aabb3::new(
                        Point3::new(-0.3, -0.05, -0.3),
                        Point3::new(0.3, 0.0, 0.3)
                    );
                    let prev = gravity.on_ground;
                    let (_, hit) = check_collisions(world, position, ground);
                    gravity.on_ground = hit;
                    if !prev && gravity.on_ground {
                        movement.did_touch_ground = true;
                    }
                }
            }
        }
    }
}


fn check_collisions(world: &world::World, position: &mut Position, bounds: Aabb3<f64>) -> (Aabb3<f64>, bool) {
    let mut bounds = bounds.add_v(position.position);

    let dir = position.position - position.last_position;

    let min_x = (bounds.min.x - 1.0) as i32;
    let min_y = (bounds.min.y - 1.0) as i32;
    let min_z = (bounds.min.z - 1.0) as i32;
    let max_x = (bounds.max.x + 1.0) as i32;
    let max_y = (bounds.max.y + 1.0) as i32;
    let max_z = (bounds.max.z + 1.0) as i32;

    let mut hit = false;
    for y in min_y .. max_y {
        for z in min_z .. max_z {
            for x in min_x .. max_x {
                let block = world.get_block(x, y, z);
                for bb in block.get_collision_boxes() {
                    let bb = bb.add_v(cgmath::Vector3::new(x as f64, y as f64, z as f64));
                    if bb.collides(&bounds) {
                        bounds = bounds.move_out_of(bb, dir);
                        hit = true;
                    }
                }
            }
        }
    }

    (bounds, hit)
}

trait Collidable<T> {
    fn collides(&self, t: &T) -> bool;
    fn move_out_of(self, other: Self, dir: cgmath::Vector3<f64>) -> Self;
}

impl Collidable<Aabb3<f64>> for Aabb3<f64> {
    fn collides(&self, t: &Aabb3<f64>) -> bool {
        !(
            t.min.x >= self.max.x ||
            t.max.x <= self.min.x ||
            t.min.y >= self.max.y ||
            t.max.y <= self.min.y ||
            t.min.z >= self.max.z ||
            t.max.z <= self.min.z
        )
    }

    fn move_out_of(mut self, other: Self, dir: cgmath::Vector3<f64>) -> Self {
        if dir.x != 0.0 {
            if dir.x > 0.0 {
                let ox = self.max.x;
                self.max.x = other.min.x - 0.0001;
                self.min.x += self.max.x - ox;
            } else {
                let ox = self.min.x;
                self.min.x = other.max.x + 0.0001;
                self.max.x += self.min.x - ox;
            }
        }
        if dir.y != 0.0 {
            if dir.y > 0.0 {
                let oy = self.max.y;
                self.max.y = other.min.y - 0.0001;
                self.min.y += self.max.y - oy;
            } else {
                let oy = self.min.y;
                self.min.y = other.max.y + 0.0001;
                self.max.y += self.min.y - oy;
            }
        }
        if dir.z != 0.0 {
            if dir.z > 0.0 {
                let oz = self.max.z;
                self.max.z = other.min.z - 0.0001;
                self.min.z += self.max.z - oz;
            } else {
                let oz = self.min.z;
                self.min.z = other.max.z + 0.0001;
                self.max.z += self.min.z - oz;
            }
        }
        self
    }
}
