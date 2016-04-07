
use super::*;
use ecs;
use world;
use render;

pub struct ApplyVelocity {
    filter: ecs::Filter,
    position: ecs::Key<Position>,
    velocity: ecs::Key<Velocity>,
    movement: ecs::Key<super::player::PlayerMovement>,
}

impl ApplyVelocity {
    pub fn new(m: &mut ecs::Manager) -> ApplyVelocity {
        let position = m.get_key();
        let velocity = m.get_key();
        ApplyVelocity {
            filter: ecs::Filter::new()
                .with(position)
                .with(velocity),
            position: position,
            velocity: velocity,
            movement: m.get_key(),
        }
    }
}

impl ecs::System for ApplyVelocity {

    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(&mut self, m: &mut ecs::Manager, _: &mut world::World, _: &mut render::Renderer) {
        for e in m.find(&self.filter) {
            if m.get_component(e, self.movement).is_some() {
                // Player's handle their own phyiscs
                continue;
            }
            let pos = m.get_component_mut(e, self.position).unwrap();
            let vel = m.get_component(e, self.velocity).unwrap();
            pos.position = pos.position + vel.velocity;
        }
    }
}

pub struct ApplyGravity {
    filter: ecs::Filter,
    velocity: ecs::Key<Velocity>,
    movement: ecs::Key<super::player::PlayerMovement>,
}

impl ApplyGravity {
    pub fn new(m: &mut ecs::Manager) -> ApplyGravity {
        let gravity = m.get_key::<Gravity>();
        let velocity = m.get_key();
        ApplyGravity {
            filter: ecs::Filter::new()
                .with(gravity)
                .with(velocity),
            velocity: velocity,
            movement: m.get_key(),
        }
    }
}

impl ecs::System for ApplyGravity {

    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(&mut self, m: &mut ecs::Manager, _: &mut world::World, _: &mut render::Renderer) {
        for e in m.find(&self.filter) {
            if m.get_component(e, self.movement).is_some() {
                // Player's handle their own phyiscs
                continue;
            }
            let vel = m.get_component_mut(e, self.velocity).unwrap();

            vel.velocity.y -= 0.03;
            if vel.velocity.y < -0.3 {
                vel.velocity.y = -0.3;
            }
        }
    }
}

pub struct UpdateLastPosition {
    filter: ecs::Filter,
    position: ecs::Key<Position>,
}

impl UpdateLastPosition {
    pub fn new(m: &mut ecs::Manager) -> UpdateLastPosition {
        let position = m.get_key();
        UpdateLastPosition {
            filter: ecs::Filter::new()
                .with(position),
            position: position,
        }
    }
}

impl ecs::System for UpdateLastPosition {

    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(&mut self, m: &mut ecs::Manager, _: &mut world::World, _: &mut render::Renderer) {
        for e in m.find(&self.filter) {
            let pos = m.get_component_mut(e, self.position).unwrap();

            pos.moved = pos.position != pos.last_position;
            pos.last_position = pos.position;
        }
    }
}

pub struct LerpPosition {
    filter: ecs::Filter,
    position: ecs::Key<Position>,
    target_position: ecs::Key<TargetPosition>,
    game_info: ecs::Key<GameInfo>,
}

impl LerpPosition {
    pub fn new(m: &mut ecs::Manager) -> LerpPosition {
        let position = m.get_key();
        let target_position = m.get_key();
        LerpPosition {
            filter: ecs::Filter::new()
                .with(position)
                .with(target_position),
            position: position,
            target_position: target_position,
            game_info: m.get_key(),
        }
    }
}

impl ecs::System for LerpPosition {

    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(&mut self, m: &mut ecs::Manager, _: &mut world::World, _: &mut render::Renderer) {
        let world_entity = m.get_world();
        let delta = m.get_component_mut(world_entity, self.game_info).unwrap().delta.min(5.0);
        for e in m.find(&self.filter) {
            let pos = m.get_component_mut(e, self.position).unwrap();
            let target_pos = m.get_component(e, self.target_position).unwrap();

            pos.position = pos.position + (target_pos.position - pos.position) * delta * 0.2;
        }
    }
}

pub struct LerpRotation {
    filter: ecs::Filter,
    rotation: ecs::Key<Rotation>,
    target_rotation: ecs::Key<TargetRotation>,
    game_info: ecs::Key<GameInfo>,
}

impl LerpRotation {
    pub fn new(m: &mut ecs::Manager) -> LerpRotation {
        let rotation = m.get_key();
        let target_rotation = m.get_key();
        LerpRotation {
            filter: ecs::Filter::new()
                .with(rotation)
                .with(target_rotation),
            rotation: rotation,
            target_rotation: target_rotation,
            game_info: m.get_key(),
        }
    }
}

impl ecs::System for LerpRotation {

    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(&mut self, m: &mut ecs::Manager, _: &mut world::World, _: &mut render::Renderer) {
        use std::f64::consts::PI;
        let world_entity = m.get_world();
        let delta = m.get_component_mut(world_entity, self.game_info).unwrap().delta.min(5.0);
        for e in m.find(&self.filter) {
            let rot = m.get_component_mut(e, self.rotation).unwrap();
            let target_rot = m.get_component_mut(e, self.target_rotation).unwrap();
            target_rot.yaw = (PI*2.0 + target_rot.yaw) % (PI*2.0);
            target_rot.pitch = (PI*2.0 + target_rot.pitch) % (PI*2.0);

            let mut delta_yaw = target_rot.yaw - rot.yaw;
            let mut delta_pitch = target_rot.pitch - rot.pitch;

            if delta_yaw.abs() > PI {
                delta_yaw = (PI - delta_yaw.abs()) * delta_yaw.signum();
            }
            if delta_pitch.abs() > PI {
                delta_pitch = (PI - delta_pitch.abs()) * delta_pitch.signum();
            }

            rot.yaw += delta_yaw * 0.2 * delta;
            rot.pitch += delta_pitch * 0.2 * delta;
            rot.yaw = (PI*2.0 + rot.yaw) % (PI*2.0);
            rot.pitch = (PI*2.0 + rot.pitch) % (PI*2.0);
        }
    }
}
