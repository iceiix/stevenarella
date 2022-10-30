use std::sync::Arc;
use std::sync::RwLock;

use super::*;
use crate::ecs;
use crate::render;
use crate::shared::Position as BPos;
use crate::world;
use cgmath::InnerSpace;
use log::debug;
use steven_protocol::protocol;
use steven_protocol::protocol::Conn;

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
            filter: ecs::Filter::new().with(position).with(velocity),
            position,
            velocity,
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
                // Player's handle their own physics
                continue;
            }
            let pos = m.get_component_mut(e, self.position).unwrap();
            let vel = m.get_component(e, self.velocity).unwrap();
            pos.position += vel.velocity;
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
            filter: ecs::Filter::new().with(gravity).with(velocity),
            velocity,
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
                // Player's handle their own physics
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
            filter: ecs::Filter::new().with(position),
            position,
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

            pos.moved = (pos.position - pos.last_position).magnitude2() > 0.01;
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
            filter: ecs::Filter::new().with(position).with(target_position),
            position,
            target_position,
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
        let delta = m
            .get_component_mut(world_entity, self.game_info)
            .unwrap()
            .delta
            .min(5.0);
        for e in m.find(&self.filter) {
            let pos = m.get_component_mut(e, self.position).unwrap();
            let target_pos = m.get_component(e, self.target_position).unwrap();

            pos.position = pos.position
                + (target_pos.position - pos.position) * delta * target_pos.lerp_amount;
            let len = (pos.position - target_pos.position).magnitude2();
            if !(0.001..=100.0 * 100.0).contains(&len) {
                pos.position = target_pos.position;
            }
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
            filter: ecs::Filter::new().with(rotation).with(target_rotation),
            rotation,
            target_rotation,
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
        let delta = m
            .get_component_mut(world_entity, self.game_info)
            .unwrap()
            .delta
            .min(5.0);
        for e in m.find(&self.filter) {
            let rot = m.get_component_mut(e, self.rotation).unwrap();
            let target_rot = m.get_component_mut(e, self.target_rotation).unwrap();
            target_rot.yaw = (PI * 2.0 + target_rot.yaw) % (PI * 2.0);
            target_rot.pitch = (PI * 2.0 + target_rot.pitch) % (PI * 2.0);

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
            rot.yaw = (PI * 2.0 + rot.yaw) % (PI * 2.0);
            rot.pitch = (PI * 2.0 + rot.pitch) % (PI * 2.0);
        }
    }
}

pub struct LightEntity {
    filter: ecs::Filter,
    position: ecs::Key<Position>,
    bounds: ecs::Key<Bounds>,
    light: ecs::Key<Light>,
}

impl LightEntity {
    pub fn new(m: &mut ecs::Manager) -> LightEntity {
        let position = m.get_key();
        let bounds = m.get_key();
        let light = m.get_key();
        LightEntity {
            filter: ecs::Filter::new().with(position).with(bounds).with(light),
            position,
            bounds,
            light,
        }
    }
}

impl ecs::System for LightEntity {
    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(&mut self, m: &mut ecs::Manager, world: &mut world::World, _: &mut render::Renderer) {
        for e in m.find(&self.filter) {
            let pos = m.get_component(e, self.position).unwrap();
            let bounds = m.get_component(e, self.bounds).unwrap();
            let light = m.get_component_mut(e, self.light).unwrap();
            let mut count = 0.0;
            let mut block_light = 0.0;
            let mut sky_light = 0.0;

            let min_x = (pos.position.x + bounds.bounds.min.x).floor() as i32;
            let min_y = (pos.position.y + bounds.bounds.min.y).floor() as i32;
            let min_z = (pos.position.z + bounds.bounds.min.z).floor() as i32;
            let max_x = (pos.position.x + bounds.bounds.max.x).ceil() as i32 + 1;
            let max_y = (pos.position.y + bounds.bounds.max.y).ceil() as i32 + 1;
            let max_z = (pos.position.z + bounds.bounds.max.z).ceil() as i32 + 1;

            let length = (bounds.bounds.max - bounds.bounds.min).magnitude() as f32;

            for y in min_y..max_y {
                for z in min_z..max_z {
                    for x in min_x..max_x {
                        let dist = length
                            - (((x as f32 + 0.5) - pos.position.x as f32).powi(2)
                                + ((y as f32 + 0.5) - pos.position.y as f32).powi(2)
                                + ((z as f32 + 0.5) - pos.position.z as f32).powi(2))
                            .sqrt()
                            .min(length);
                        let dist = dist / length;
                        count += dist;
                        block_light += world.get_block_light(BPos::new(x, y, z)) as f32 * dist;
                        sky_light += world.get_sky_light(BPos::new(x, y, z)) as f32 * dist;
                    }
                }
            }
            if count <= 0.01 {
                light.block_light = 0.0;
                light.sky_light = 0.0;
            } else {
                light.block_light = block_light / count;
                light.sky_light = sky_light / count;
            }
        }
    }
}

pub struct ApplyDigging {
    filter: ecs::Filter,
    mouse_buttons: ecs::Key<MouseButtons>,
    digging: ecs::Key<Digging>,
    conn: ecs::Key<Arc<RwLock<Option<Conn>>>>,
}

impl ApplyDigging {
    pub fn new(m: &mut ecs::Manager) -> Self {
        let mouse_buttons = m.get_key();
        let digging = m.get_key();
        Self {
            filter: ecs::Filter::new().with(mouse_buttons).with(digging),
            mouse_buttons,
            digging,
            conn: m.get_key(),
        }
    }

    fn send_packet(&self, conn: &mut Conn, target: &DiggingState, state: i32) {
        match state {
            0 => debug!("Send start dig packet {:?}", target),
            1 => debug!("Send cancel dig packet {:?}", target),
            2 => debug!("Send finish dig packet {:?}", target),
            n => panic!("Invalid dig state {}", n),
        }

        match conn.protocol_version {
            // 1.7.10
            5 => conn
                .write_packet(packet::play::serverbound::PlayerDigging_u8_u8y {
                    status: state as u8,
                    x: target.position.x,
                    y: target.position.y as u8,
                    z: target.position.z,
                    face: target.face.index() as u8,
                })
                .unwrap(),
            // 1.8.9 or v15w39c
            47 | 74 => conn
                .write_packet(packet::play::serverbound::PlayerDigging_u8 {
                    status: state as u8,
                    location: target.position,
                    face: target.face.index() as u8,
                })
                .unwrap(),
            // 1.9+
            _ => conn
                .write_packet(packet::play::serverbound::PlayerDigging {
                    status: protocol::VarInt(state),
                    location: target.position,
                    face: target.face.index() as u8,
                })
                .unwrap(),
        }
    }

    fn next_state(
        &self,
        last: &Option<DiggingState>,
        mouse_buttons: &MouseButtons,
        target: Option<(
            shared::Position,
            block::Block,
            shared::Direction,
            Vector3<f64>,
        )>,
    ) -> Option<DiggingState> {
        // Figure out the next state
        if !mouse_buttons.left {
            return None;
        }

        match (last, target) {
            // Started digging
            (None, Some((position, block, face, _))) => Some(DiggingState {
                block,
                face,
                position,
                start: std::time::Instant::now(),
                finished: false,
            }),
            (Some(current), Some((position, block, face, _))) => {
                // Continue digging
                if position == current.position {
                    return last.clone();
                }

                // Start digging a different block.
                Some(DiggingState {
                    block,
                    face,
                    position,
                    start: std::time::Instant::now(),
                    finished: false,
                })
            }
            // Not pointing at any target
            (_, None) => None,
        }
    }

    fn is_finished(&self, state: &DiggingState) -> bool {
        let mining_time = state.block.get_mining_time(&None);
        match mining_time {
            Some(mining_time) => {
                let finish_time = state.start + mining_time;
                finish_time > std::time::Instant::now()
            }
            None => false,
        }
    }
}

impl ecs::System for ApplyDigging {
    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(
        &mut self,
        m: &mut ecs::Manager,
        world: &mut world::World,
        renderer: &mut render::Renderer,
    ) {
        use crate::server::target::{test_block, trace_ray};
        use cgmath::EuclideanSpace;

        let world_entity = m.get_world();
        let mut conn = m
            .get_component(world_entity, self.conn)
            .unwrap()
            .write()
            .unwrap();
        let conn = match conn.as_mut() {
            Some(conn) => conn,
            // Don't keep processing digging operations if the connection was
            // closed.
            None => return,
        };

        let target = trace_ray(
            world,
            4.0,
            renderer.camera.pos.to_vec(),
            renderer.view_vector.cast().unwrap(),
            test_block,
        );

        for e in m.find(&self.filter) {
            let mouse_buttons = m.get_component(e, self.mouse_buttons).unwrap();
            let digging = m.get_component_mut(e, self.digging).unwrap();

            // Update last and current state
            std::mem::swap(&mut digging.last, &mut digging.current);
            digging.current = self.next_state(&digging.last, mouse_buttons, target);

            // Handle digging packets
            match (&digging.last, &mut digging.current) {
                // Start the new digging operation.
                (None, Some(current)) => self.send_packet(conn, current, 0),
                // Cancel the previous digging operation.
                (Some(last), None) if !last.finished => self.send_packet(conn, last, 1),
                // Move to digging a new block
                (Some(last), Some(current)) if last.position != current.position => {
                    // Cancel the previous digging operation.
                    if !current.finished {
                        self.send_packet(conn, last, 1);
                    }
                    // Start the new digging operation.
                    self.send_packet(conn, current, 0);
                }
                // Finish the new digging operation.
                (Some(_), Some(current)) if !self.is_finished(current) && !current.finished => {
                    self.send_packet(conn, current, 2);
                    current.finished = true;
                }
                _ => {}
            }
        }
    }
}
