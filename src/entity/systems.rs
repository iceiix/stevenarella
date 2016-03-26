
use super::*;
use ecs;

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

    fn update(&mut self, m: &mut ecs::Manager) {
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

    fn update(&mut self, m: &mut ecs::Manager) {
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

    fn update(&mut self, m: &mut ecs::Manager) {
        for e in m.find(&self.filter) {
            let pos = m.get_component_mut(e, self.position).unwrap();

            pos.last_position = pos.position;
        }
    }
}
