use super::{
    Bounds, GameInfo, Gravity, Light, Position, Rotation, TargetPosition, TargetRotation, Velocity,
};
use crate::ecs;
use crate::format;
use crate::render;
use crate::render::model::{self, FormatState};
use crate::settings::Stevenkey;
use crate::shared::Position as BPosition;
use crate::types::hash::FNVHash;
use crate::types::Gamemode;
use crate::world;
use cgmath::{self, Decomposed, Matrix4, Point3, Quaternion, Rad, Rotation3, Vector3};
use collision::{Aabb, Aabb3};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::time::Instant;

pub fn add_systems(m: &mut ecs::Manager) {
    let sys = MovementHandler::new(m);
    m.add_system(sys);
    let sys = PlayerRenderer::new(m);
    m.add_render_system(sys);
}

pub fn create_local(m: &mut ecs::Manager) -> ecs::Entity {
    let entity = m.create_entity();
    m.add_component_direct(entity, Position::new(0.0, 0.0, 0.0));
    let mut tpos = TargetPosition::new(0.0, 0.0, 0.0);
    tpos.lerp_amount = 1.0 / 3.0;
    m.add_component_direct(entity, tpos);
    m.add_component_direct(entity, Rotation::new(0.0, 0.0));
    m.add_component_direct(entity, Velocity::new(0.0, 0.0, 0.0));
    m.add_component_direct(entity, Gamemode::Survival);
    m.add_component_direct(entity, Gravity::new());
    m.add_component_direct(entity, PlayerMovement::new());
    m.add_component_direct(
        entity,
        Bounds::new(Aabb3::new(
            Point3::new(-0.3, 0.0, -0.3),
            Point3::new(0.3, 1.8, 0.3),
        )),
    );
    m.add_component_direct(entity, PlayerModel::new("", false, false, true));
    m.add_component_direct(entity, Light::new());
    entity
}

pub fn create_remote(m: &mut ecs::Manager, name: &str) -> ecs::Entity {
    let entity = m.create_entity();
    m.add_component_direct(entity, Position::new(0.0, 0.0, 0.0));
    m.add_component_direct(entity, TargetPosition::new(0.0, 0.0, 0.0));
    m.add_component_direct(entity, Rotation::new(0.0, 0.0));
    m.add_component_direct(entity, TargetRotation::new(0.0, 0.0));
    m.add_component_direct(entity, Velocity::new(0.0, 0.0, 0.0));
    m.add_component_direct(
        entity,
        Bounds::new(Aabb3::new(
            Point3::new(-0.3, 0.0, -0.3),
            Point3::new(0.3, 1.8, 0.3),
        )),
    );
    m.add_component_direct(entity, PlayerModel::new(name, true, true, false));
    m.add_component_direct(entity, Light::new());
    entity
}

pub struct PlayerModel {
    model: Option<model::ModelKey>,
    skin_url: Option<String>,
    dirty: bool,
    name: String,

    has_head: bool,
    has_name_tag: bool,
    first_person: bool,

    dir: i32,
    time: f64,
    still_time: f64,
    idle_time: f64,
    arm_time: f64,
}

impl PlayerModel {
    pub fn new(name: &str, has_head: bool, has_name_tag: bool, first_person: bool) -> PlayerModel {
        PlayerModel {
            model: None,
            skin_url: None,
            dirty: false,
            name: name.to_owned(),

            has_head,
            has_name_tag,
            first_person,

            dir: 0,
            time: 0.0,
            still_time: 0.0,
            idle_time: 0.0,
            arm_time: 0.0,
        }
    }

    pub fn set_skin(&mut self, skin: Option<String>) {
        self.skin_url = skin;
        self.dirty = true;
    }
}

struct PlayerRenderer {
    filter: ecs::Filter,
    player_model: ecs::Key<PlayerModel>,
    position: ecs::Key<Position>,
    rotation: ecs::Key<Rotation>,
    game_info: ecs::Key<GameInfo>,
    light: ecs::Key<Light>,
}

impl PlayerRenderer {
    fn new(m: &mut ecs::Manager) -> PlayerRenderer {
        let player_model = m.get_key();
        let position = m.get_key();
        let rotation = m.get_key();
        let light = m.get_key();
        PlayerRenderer {
            filter: ecs::Filter::new()
                .with(player_model)
                .with(position)
                .with(rotation)
                .with(light),
            player_model,
            position,
            rotation,
            game_info: m.get_key(),
            light,
        }
    }
}

enum PlayerModelPart {
    Head = 0,
    Body = 1,
    LegLeft = 2,
    LegRight = 3,
    ArmLeft = 4,
    ArmRight = 5,
    NameTag = 6,
    //Cape = 7, // TODO
}

// TODO: Setup culling
impl ecs::System for PlayerRenderer {
    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(
        &mut self,
        m: &mut ecs::Manager,
        world: &mut world::World,
        renderer: &mut render::Renderer,
    ) {
        use std::f32::consts::PI;
        use std::f64::consts::PI as PI64;
        let world_entity = m.get_world();
        let delta = m
            .get_component_mut(world_entity, self.game_info)
            .unwrap()
            .delta;
        for e in m.find(&self.filter) {
            let player_model = m.get_component_mut(e, self.player_model).unwrap();
            let position = m.get_component_mut(e, self.position).unwrap();
            let rotation = m.get_component_mut(e, self.rotation).unwrap();
            let light = m.get_component(e, self.light).unwrap();

            if player_model.dirty {
                self.entity_removed(m, e, world, renderer);
                self.entity_added(m, e, world, renderer);
            }

            if let Some(pmodel) = player_model.model {
                let mdl = renderer.model.get_model(pmodel).unwrap();

                mdl.block_light = light.block_light;
                mdl.sky_light = light.sky_light;

                let offset = if player_model.first_person {
                    let ox = (rotation.yaw - PI64 / 2.0).cos() * 0.25;
                    let oz = -(rotation.yaw - PI64 / 2.0).sin() * 0.25;
                    Vector3::new(
                        position.position.x as f32 - ox as f32,
                        -position.position.y as f32,
                        position.position.z as f32 - oz as f32,
                    )
                } else {
                    Vector3::new(
                        position.position.x as f32,
                        -position.position.y as f32,
                        position.position.z as f32,
                    )
                };
                let offset_matrix = Matrix4::from(Decomposed {
                    scale: 1.0,
                    rot: Quaternion::from_angle_y(Rad(PI + rotation.yaw as f32)),
                    disp: offset,
                });

                // TODO This sucks
                if player_model.has_name_tag {
                    let ang = (position.position.x - renderer.camera.pos.x)
                        .atan2(position.position.z - renderer.camera.pos.z)
                        as f32;
                    mdl.matrix[PlayerModelPart::NameTag as usize] = Matrix4::from(Decomposed {
                        scale: 1.0,
                        rot: Quaternion::from_angle_y(Rad(ang)),
                        disp: offset + Vector3::new(0.0, (-24.0 / 16.0) - 0.6, 0.0),
                    });
                }

                mdl.matrix[PlayerModelPart::Head as usize] = offset_matrix
                    * Matrix4::from(Decomposed {
                        scale: 1.0,
                        rot: Quaternion::from_angle_x(Rad(-rotation.pitch as f32)),
                        disp: Vector3::new(0.0, -12.0 / 16.0 - 12.0 / 16.0, 0.0),
                    });
                mdl.matrix[PlayerModelPart::Body as usize] = offset_matrix
                    * Matrix4::from(Decomposed {
                        scale: 1.0,
                        rot: Quaternion::from_angle_x(Rad(0.0)),
                        disp: Vector3::new(0.0, -12.0 / 16.0 - 6.0 / 16.0, 0.0),
                    });

                let mut time = player_model.time;
                let mut dir = player_model.dir;
                if dir == 0 {
                    dir = 1;
                    time = 15.0;
                }
                let ang = ((time / 15.0) - 1.0) * (PI64 / 4.0);

                mdl.matrix[PlayerModelPart::LegRight as usize] = offset_matrix
                    * Matrix4::from(Decomposed {
                        scale: 1.0,
                        rot: Quaternion::from_angle_x(Rad(ang as f32)),
                        disp: Vector3::new(2.0 / 16.0, -12.0 / 16.0, 0.0),
                    });
                mdl.matrix[PlayerModelPart::LegLeft as usize] = offset_matrix
                    * Matrix4::from(Decomposed {
                        scale: 1.0,
                        rot: Quaternion::from_angle_x(Rad(-ang as f32)),
                        disp: Vector3::new(-2.0 / 16.0, -12.0 / 16.0, 0.0),
                    });

                let mut i_time = player_model.idle_time;
                i_time += delta * 0.02;
                if i_time > PI64 * 2.0 {
                    i_time -= PI64 * 2.0;
                }
                player_model.idle_time = i_time;

                if player_model.arm_time <= 0.0 {
                    player_model.arm_time = 0.0;
                } else {
                    player_model.arm_time -= delta;
                }

                mdl.matrix[PlayerModelPart::ArmRight as usize] = offset_matrix
                    * Matrix4::from_translation(Vector3::new(
                        6.0 / 16.0,
                        -12.0 / 16.0 - 12.0 / 16.0,
                        0.0,
                    ))
                    * Matrix4::from(Quaternion::from_angle_x(Rad(-(ang * 0.75) as f32)))
                    * Matrix4::from(Quaternion::from_angle_z(Rad(
                        (i_time.cos() * 0.06 - 0.06) as f32
                    )))
                    * Matrix4::from(Quaternion::from_angle_x(Rad((i_time.sin() * 0.06
                        - ((7.5 - (player_model.arm_time - 7.5).abs()) / 7.5))
                        as f32)));

                mdl.matrix[PlayerModelPart::ArmLeft as usize] = offset_matrix
                    * Matrix4::from_translation(Vector3::new(
                        -6.0 / 16.0,
                        -12.0 / 16.0 - 12.0 / 16.0,
                        0.0,
                    ))
                    * Matrix4::from(Quaternion::from_angle_x(Rad((ang * 0.75) as f32)))
                    * Matrix4::from(Quaternion::from_angle_z(Rad(
                        -(i_time.cos() * 0.06 - 0.06) as f32
                    )))
                    * Matrix4::from(Quaternion::from_angle_x(Rad(-(i_time.sin() * 0.06) as f32)));

                let mut update = true;
                if position.moved {
                    player_model.still_time = 0.0;
                } else if player_model.still_time > 2.0 {
                    if (time - 15.0).abs() <= 1.5 * delta {
                        time = 15.0;
                        update = false;
                    }
                    dir = (15.0 - time).signum() as i32;
                } else {
                    player_model.still_time += delta;
                }

                if update {
                    time += delta * 1.5 * (dir as f64);
                    if time > 30.0 {
                        time = 30.0;
                        dir = -1;
                    } else if time < 0.0 {
                        time = 0.0;
                        dir = 1;
                    }
                }
                player_model.time = time;
                player_model.dir = dir;
            }
        }
    }

    fn entity_added(
        &mut self,
        m: &mut ecs::Manager,
        e: ecs::Entity,
        _: &mut world::World,
        renderer: &mut render::Renderer,
    ) {
        let player_model = m.get_component_mut(e, self.player_model).unwrap();

        player_model.dirty = false;

        let skin = if let Some(url) = player_model.skin_url.as_ref() {
            renderer.get_skin(renderer.get_textures_ref(), url)
        } else {
            render::Renderer::get_texture(renderer.get_textures_ref(), "entity/steve")
        };

        macro_rules! srel {
            ($x:expr, $y:expr, $w:expr, $h:expr) => {
                Some(skin.relative(($x) / 64.0, ($y) / 64.0, ($w) / 64.0, ($h) / 64.0))
            };
        }

        let mut head_verts = vec![];
        if player_model.has_head {
            model::append_box(
                &mut head_verts,
                -4.0 / 16.0,
                0.0,
                -4.0 / 16.0,
                8.0 / 16.0,
                8.0 / 16.0,
                8.0 / 16.0,
                [
                    srel!(16.0, 0.0, 8.0, 8.0), // Down
                    srel!(8.0, 0.0, 8.0, 8.0),  // Up
                    srel!(8.0, 8.0, 8.0, 8.0),  // North
                    srel!(24.0, 8.0, 8.0, 8.0), // South
                    srel!(16.0, 8.0, 8.0, 8.0), // West
                    srel!(0.0, 8.0, 8.0, 8.0),  // East
                ],
            );
            model::append_box(
                &mut head_verts,
                -4.2 / 16.0,
                -0.2 / 16.0,
                -4.2 / 16.0,
                8.4 / 16.0,
                8.4 / 16.0,
                8.4 / 16.0,
                [
                    srel!((16.0 + 32.0), 0.0, 8.0, 8.0), // Down
                    srel!((8.0 + 32.0), 0.0, 8.0, 8.0),  // Up
                    srel!((8.0 + 32.0), 8.0, 8.0, 8.0),  // North
                    srel!((24.0 + 32.0), 8.0, 8.0, 8.0), // South
                    srel!((16.0 + 32.0), 8.0, 8.0, 8.0), // West
                    srel!((0.0 + 32.0), 8.0, 8.0, 8.0),  // East
                ],
            );
        }

        // TODO: Cape
        let mut body_verts = vec![];
        model::append_box(
            &mut body_verts,
            -4.0 / 16.0,
            -6.0 / 16.0,
            -2.0 / 16.0,
            8.0 / 16.0,
            12.0 / 16.0,
            4.0 / 16.0,
            [
                srel!(28.0, 16.0, 8.0, 4.0),  // Down
                srel!(20.0, 16.0, 8.0, 4.0),  // Up
                srel!(20.0, 20.0, 8.0, 12.0), // North
                srel!(32.0, 20.0, 8.0, 12.0), // South
                srel!(16.0, 20.0, 4.0, 12.0), // West
                srel!(28.0, 20.0, 4.0, 12.0), // East
            ],
        );
        model::append_box(
            &mut body_verts,
            -4.2 / 16.0,
            -6.2 / 16.0,
            -2.2 / 16.0,
            8.4 / 16.0,
            12.4 / 16.0,
            4.4 / 16.0,
            [
                srel!(28.0, 16.0 + 16.0, 8.0, 4.0),  // Down
                srel!(20.0, 16.0 + 16.0, 8.0, 4.0),  // Up
                srel!(20.0, 20.0 + 16.0, 8.0, 12.0), // North
                srel!(32.0, 20.0 + 16.0, 8.0, 12.0), // South
                srel!(16.0, 20.0 + 16.0, 4.0, 12.0), // West
                srel!(28.0, 20.0 + 16.0, 4.0, 12.0), // East
            ],
        );

        let mut part_verts = vec![vec![]; 4];

        for (i, offsets) in [
            [16.0, 48.0, 0.0, 48.0],  // Left left
            [0.0, 16.0, 0.0, 32.0],   // Right Leg
            [32.0, 48.0, 48.0, 48.0], // Left arm
            [40.0, 16.0, 40.0, 32.0], // Right arm
        ]
        .iter()
        .enumerate()
        {
            let (ox, oy) = (offsets[0], offsets[1]);
            model::append_box(
                &mut part_verts[i],
                -2.0 / 16.0,
                -12.0 / 16.0,
                -2.0 / 16.0,
                4.0 / 16.0,
                12.0 / 16.0,
                4.0 / 16.0,
                [
                    srel!(ox + 8.0, oy + 0.0, 4.0, 4.0),   // Down
                    srel!(ox + 4.0, oy + 0.0, 4.0, 4.0),   // Up
                    srel!(ox + 4.0, oy + 4.0, 4.0, 12.0),  // North
                    srel!(ox + 12.0, oy + 4.0, 4.0, 12.0), // South
                    srel!(ox + 8.0, oy + 4.0, 4.0, 12.0),  // West
                    srel!(ox + 0.0, oy + 4.0, 4.0, 12.0),  // East
                ],
            );
            let (ox, oy) = (offsets[2], offsets[3]);
            model::append_box(
                &mut part_verts[i],
                -2.2 / 16.0,
                -12.2 / 16.0,
                -2.2 / 16.0,
                4.4 / 16.0,
                12.4 / 16.0,
                4.4 / 16.0,
                [
                    srel!(ox + 8.0, oy + 0.0, 4.0, 4.0),   // Down
                    srel!(ox + 4.0, oy + 0.0, 4.0, 4.0),   // Up
                    srel!(ox + 4.0, oy + 4.0, 4.0, 12.0),  // North
                    srel!(ox + 12.0, oy + 4.0, 4.0, 12.0), // South
                    srel!(ox + 8.0, oy + 4.0, 4.0, 12.0),  // West
                    srel!(ox + 0.0, oy + 4.0, 4.0, 12.0),  // East
                ],
            );
        }

        let mut name_verts = vec![];
        if player_model.has_name_tag {
            let mut state = FormatState {
                width: 0.0,
                offset: 0.0,
                text: Vec::new(),
                renderer,
                y_scale: 0.16,
                x_scale: 0.01,
            };
            let mut name = format::Component::Text(format::TextComponent::new(&player_model.name));
            format::convert_legacy(&mut name);
            state.build(&name, format::Color::Black);
            let width = state.width;
            // Center align text
            for vert in &mut state.text {
                vert.x += width * 0.5;
                vert.r = 64;
                vert.g = 64;
                vert.b = 64;
            }
            name_verts.extend_from_slice(&state.text);
            for vert in &mut state.text {
                vert.x -= 0.01;
                vert.y -= 0.01;
                vert.z -= 0.05;
                vert.r = 255;
                vert.g = 255;
                vert.b = 255;
            }
            name_verts.extend_from_slice(&state.text);
        }

        player_model.model = Some(renderer.model.create_model(
            model::DEFAULT,
            vec![
                head_verts,
                body_verts,
                part_verts[0].clone(),
                part_verts[1].clone(),
                part_verts[2].clone(),
                part_verts[3].clone(),
                name_verts,
            ],
        ));
    }

    fn entity_removed(
        &mut self,
        m: &mut ecs::Manager,
        e: ecs::Entity,
        _: &mut world::World,
        renderer: &mut render::Renderer,
    ) {
        let player_model = m.get_component_mut(e, self.player_model).unwrap();
        if let Some(model) = player_model.model.take() {
            renderer.model.remove_model(model);
            if let Some(url) = player_model.skin_url.as_ref() {
                renderer
                    .get_textures_ref()
                    .read()
                    .unwrap()
                    .release_skin(url);
            }
        }
    }
}

#[derive(Default)]
pub struct PlayerMovement {
    pub flying: bool,
    pub want_to_fly: bool,
    pub when_last_jump_pressed: Option<Instant>,
    pub when_last_jump_released: Option<Instant>,
    pub did_touch_ground: bool,
    pub pressed_keys: HashMap<Stevenkey, bool, BuildHasherDefault<FNVHash>>,
}

impl PlayerMovement {
    pub fn new() -> PlayerMovement {
        Default::default()
    }

    fn calculate_movement(&self, player_yaw: f64) -> (f64, f64) {
        use std::f64::consts::PI;
        let mut forward = 0.0f64;
        let mut yaw = player_yaw - (PI / 2.0);
        if self.is_key_pressed(Stevenkey::Forward) || self.is_key_pressed(Stevenkey::Backward) {
            forward = 1.0;
            if self.is_key_pressed(Stevenkey::Backward) {
                yaw += PI;
            }
        }
        let change = if self.is_key_pressed(Stevenkey::Left) {
            (PI / 2.0) / (forward.abs() + 1.0)
        } else if self.is_key_pressed(Stevenkey::Right) {
            -(PI / 2.0) / (forward.abs() + 1.0)
        } else {
            0.0
        };
        if self.is_key_pressed(Stevenkey::Left) || self.is_key_pressed(Stevenkey::Right) {
            forward = 1.0;
        }
        if self.is_key_pressed(Stevenkey::Backward) {
            yaw -= change;
        } else {
            yaw += change;
        }

        (forward, yaw)
    }

    fn is_key_pressed(&self, key: Stevenkey) -> bool {
        self.pressed_keys.get(&key).map_or(false, |v| *v)
    }
}

struct MovementHandler {
    filter: ecs::Filter,
    movement: ecs::Key<PlayerMovement>,
    gravity: ecs::Key<Gravity>,
    gamemode: ecs::Key<Gamemode>,
    position: ecs::Key<TargetPosition>,
    velocity: ecs::Key<Velocity>,
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
            movement,
            gravity: m.get_key(),
            gamemode: m.get_key(),
            position,
            velocity,
            bounds,
            rotation,
        }
    }
}

impl ecs::System for MovementHandler {
    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(&mut self, m: &mut ecs::Manager, world: &mut world::World, _: &mut render::Renderer) {
        for e in m.find(&self.filter) {
            let movement = m.get_component_mut(e, self.movement).unwrap();
            if movement.flying && m.get_component(e, self.gravity).is_some() {
                m.remove_component(e, self.gravity);
            } else if !movement.flying && m.get_component(e, self.gravity).is_none() {
                m.add_component(e, self.gravity, Gravity::new());
            }
            let gamemode = m.get_component(e, self.gamemode).unwrap();
            movement.flying |= gamemode.always_fly();

            // Detect double-tapping jump to toggle creative flight
            if movement.is_key_pressed(Stevenkey::Jump) {
                if movement.when_last_jump_pressed.is_none() {
                    movement.when_last_jump_pressed = Some(Instant::now());
                    if movement.when_last_jump_released.is_some() {
                        let dt = movement.when_last_jump_pressed.unwrap()
                            - movement.when_last_jump_released.unwrap();
                        if dt.as_secs() == 0
                            && dt.subsec_millis() <= crate::settings::DOUBLE_JUMP_MS
                        {
                            movement.want_to_fly = !movement.want_to_fly;
                            //info!("double jump! dt={:?} toggle want_to_fly = {}", dt, movement.want_to_fly);

                            if gamemode.can_fly() && !gamemode.always_fly() {
                                movement.flying = movement.want_to_fly;
                            }
                        }
                    }
                }
            } else if movement.when_last_jump_pressed.is_some() {
                movement.when_last_jump_released = Some(Instant::now());
                movement.when_last_jump_pressed = None;
            }

            let position = m.get_component_mut(e, self.position).unwrap();
            let rotation = m.get_component(e, self.rotation).unwrap();
            let velocity = m.get_component_mut(e, self.velocity).unwrap();
            let gravity = m.get_component_mut(e, self.gravity);

            let player_bounds = m.get_component(e, self.bounds).unwrap().bounds;

            let mut last_position = position.position;

            if world.is_chunk_loaded(
                (position.position.x as i32) >> 4,
                (position.position.z as i32) >> 4,
            ) {
                let (forward, yaw) = movement.calculate_movement(rotation.yaw);
                let mut speed = if movement.is_key_pressed(Stevenkey::Sprint) {
                    0.2806
                } else {
                    0.21585
                };
                if movement.flying {
                    speed *= 2.5;

                    if movement.is_key_pressed(Stevenkey::Jump) {
                        position.position.y += speed;
                    }
                    if movement.is_key_pressed(Stevenkey::Sneak) {
                        position.position.y -= speed;
                    }
                } else if gravity.as_ref().map_or(false, |v| v.on_ground) {
                    if movement.is_key_pressed(Stevenkey::Jump) && velocity.velocity.y.abs() < 0.001
                    {
                        velocity.velocity.y = 0.42;
                    }
                } else {
                    velocity.velocity.y -= 0.08;
                    if velocity.velocity.y < -3.92 {
                        velocity.velocity.y = -3.92;
                    }
                }
                velocity.velocity.y *= 0.98;
                position.position.x += forward * yaw.cos() * speed;
                position.position.z -= forward * yaw.sin() * speed;
                position.position.y += velocity.velocity.y;

                if !gamemode.noclip() {
                    let mut target = position.position;
                    position.position.y = last_position.y;
                    position.position.z = last_position.z;

                    // We handle each axis separately to allow for a sliding
                    // effect when pushing up against walls.

                    let (bounds, xhit) =
                        check_collisions(world, position, &last_position, player_bounds);
                    position.position.x = bounds.min.x + 0.3;
                    last_position.x = position.position.x;

                    position.position.z = target.z;
                    let (bounds, zhit) =
                        check_collisions(world, position, &last_position, player_bounds);
                    position.position.z = bounds.min.z + 0.3;
                    last_position.z = position.position.z;

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
                        for offset in 1..9 {
                            let mini = player_bounds.add_v(cgmath::Vector3::new(
                                0.0,
                                offset as f64 / 16.0,
                                0.0,
                            ));
                            let (_, hit) = check_collisions(world, position, &last_position, mini);
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
                    let (bounds, yhit) =
                        check_collisions(world, position, &last_position, player_bounds);
                    position.position.y = bounds.min.y;
                    last_position.y = position.position.y;
                    if yhit {
                        velocity.velocity.y = 0.0;
                    }

                    if let Some(gravity) = gravity {
                        let ground =
                            Aabb3::new(Point3::new(-0.3, -0.005, -0.3), Point3::new(0.3, 0.0, 0.3));
                        let prev = gravity.on_ground;
                        let (_, hit) = check_collisions(world, position, &last_position, ground);
                        gravity.on_ground = hit;
                        if !prev && gravity.on_ground {
                            movement.did_touch_ground = true;
                        }
                    }
                }
            }
        }
    }
}

fn check_collisions(
    world: &world::World,
    position: &mut TargetPosition,
    last_position: &Vector3<f64>,
    bounds: Aabb3<f64>,
) -> (Aabb3<f64>, bool) {
    let mut bounds = bounds.add_v(position.position);

    let dir = position.position - last_position;

    let min_x = (bounds.min.x - 1.0) as i32;
    let min_y = (bounds.min.y - 1.0) as i32;
    let min_z = (bounds.min.z - 1.0) as i32;
    let max_x = (bounds.max.x + 1.0) as i32;
    let max_y = (bounds.max.y + 1.0) as i32;
    let max_z = (bounds.max.z + 1.0) as i32;

    let mut hit = false;
    for y in min_y..max_y {
        for z in min_z..max_z {
            for x in min_x..max_x {
                let block = world.get_block(BPosition::new(x, y, z));
                if block.get_material().collidable {
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
    }

    (bounds, hit)
}

trait Collidable<T> {
    fn collides(&self, t: &T) -> bool;
    fn move_out_of(self, other: Self, dir: cgmath::Vector3<f64>) -> Self;
}

impl Collidable<Aabb3<f64>> for Aabb3<f64> {
    fn collides(&self, t: &Aabb3<f64>) -> bool {
        !(t.min.x >= self.max.x
            || t.max.x <= self.min.x
            || t.min.y >= self.max.y
            || t.max.y <= self.min.y
            || t.min.z >= self.max.z
            || t.max.z <= self.min.z)
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
