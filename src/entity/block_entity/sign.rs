
use ecs;
use format::{self, Component};
use shared::{Direction, Position};
use world;
use world::block::Block;
use render;
use render::model::{self, FormatState};

pub fn add_systems(m: &mut ecs::Manager) {
    let sys = SignRenderer::new(m);
    m.add_render_system(sys);
}

pub fn init_entity(m: &mut ecs::Manager, e: ecs::Entity) {
    m.add_component_direct(e, SignInfo {
        model: None,
        lines: [
            Component::Text(format::TextComponent::new("")),
            Component::Text(format::TextComponent::new("")),
            Component::Text(format::TextComponent::new("")),
            Component::Text(format::TextComponent::new("")),
        ],
        offset_x: 0.0,
        offset_y: 0.0,
        offset_z: 0.0,
        has_stand: false,
        rotation: 0.0,
        dirty: false,
    });
}

pub struct SignInfo {
    model: Option<model::ModelKey>,

    pub lines: [format::Component; 4],
    pub dirty: bool,

    offset_x: f64,
    offset_y: f64,
    offset_z: f64,
    has_stand: bool,
    rotation: f64,
}

struct SignRenderer {
    filter: ecs::Filter,
    position: ecs::Key<Position>,
    sign_info: ecs::Key<SignInfo>,
}

impl SignRenderer {
    fn new(m: &mut ecs::Manager) -> SignRenderer {
        let sign_info = m.get_key();
        let position = m.get_key();
        SignRenderer {
            filter: ecs::Filter::new()
                .with(position)
                .with(sign_info),
            position: position,
            sign_info: sign_info,
        }
    }
}

impl ecs::System for SignRenderer {

    fn filter(&self) -> &ecs::Filter {
        &self.filter
    }

    fn update(&mut self, m: &mut ecs::Manager, world: &mut world::World, renderer: &mut render::Renderer) {
        for e in m.find(&self.filter) {
            let position = *m.get_component(e, self.position).unwrap();
            let info = m.get_component_mut(e, self.sign_info).unwrap();
            if info.dirty {
                self.entity_removed(m, e, world, renderer);
                self.entity_added(m, e, world, renderer);
            }
            if let Some(model) = info.model {
                let mdl = renderer.model.get_model(model).unwrap();
                mdl.block_light = world.get_block_light(position) as f32;
                mdl.sky_light = world.get_sky_light(position) as f32;
            }
        }
    }

    fn entity_added(&mut self, m: &mut ecs::Manager, e: ecs::Entity, world: &mut world::World, renderer: &mut render::Renderer) {
        use std::f64::consts::PI;
        use cgmath::{Vector3, Matrix4, Decomposed, Rotation3, Rad, Angle, Quaternion};
        let position = *m.get_component(e, self.position).unwrap();
        let info = m.get_component_mut(e, self.sign_info).unwrap();
        info.dirty = false;
        match world.get_block(position) {
            Block::WallSign{facing} => {
                info.offset_z = 7.5 / 16.0;
                match facing {
                    Direction::North => {},
                    Direction::South => info.rotation = PI,
                    Direction::West => info.rotation = PI / 2.0,
                    Direction::East => info.rotation = -PI / 2.0,
                    _ => unreachable!(),
                }
            },
            Block::StandingSign{rotation} => {
                info.offset_y = 5.0 / 16.0;
                info.has_stand = true;
                info.rotation = -(rotation.data() as f64 / 16.0) * PI * 2.0 + PI;
            }
            _ => return,
        }
        let tex = render::Renderer::get_texture(renderer.get_textures_ref(), "entity/sign");

        macro_rules! rel {
            ($x:expr, $y:expr, $w:expr, $h:expr) => (
                Some(tex.relative(($x) / 64.0, ($y) / 32.0, ($w) / 64.0, ($h) / 32.0))
            );
        }

        let mut verts = vec![];
        // Backboard
        model::append_box(&mut verts, -0.5, -4.0/16.0, -0.5/16.0, 1.0, 8.0/16.0, 1.0/16.0, [
            rel!(26.0, 0.0, 24.0, 2.0), // Down
            rel!(2.0, 0.0, 24.0, 2.0), // Up
            rel!(2.0, 2.0, 24.0, 12.0), // North
            rel!(26.0, 2.0, 24.0, 12.0), // South
            rel!(0.0, 2.0, 2.0, 12.0), // West
            rel!(50.0, 2.0, 2.0, 12.0), // East
        ]);
        if info.has_stand {
            model::append_box(&mut verts, -0.5/16.0, -0.25-9.0/16.0, -0.5/16.0, 1.0/16.0, 9.0/16.0, 1.0/16.0, [
                rel!(4.0, 14.0, 2.0, 2.0), // Down
                rel!(2.0, 14.0, 2.0, 2.0), // Up
                rel!(2.0, 16.0, 2.0, 12.0), // North
                rel!(6.0, 16.0, 2.0, 12.0), // South
                rel!(0.0, 16.0, 2.0, 12.0), // West
                rel!(4.0, 16.0, 2.0, 12.0), // East
            ]);
        }

        for (i, line) in info.lines.iter().enumerate() {
            const Y_SCALE: f32 = (6.0 / 16.0) / 4.0;
            const X_SCALE: f32 = Y_SCALE / 16.0;
            let mut state = FormatState {
                width: 0.0,
                offset: 0.0,
                text: Vec::new(),
                renderer: renderer,
                y_scale: Y_SCALE,
                x_scale: X_SCALE,
            };
            state.build(line, format::Color::Black);
            let width = state.width;
            // Center align text
            for vert in &mut state.text {
                vert.x += width * 0.5;
                vert.y -= (Y_SCALE + 0.4/16.0) * (i as f32);
            }
            verts.extend_from_slice(&state.text);
        }

        let model = renderer.model.create_model(
            model::DEFAULT,
            vec![verts]
        );

        {
            let mdl = renderer.model.get_model(model).unwrap();
            mdl.radius = 2.0;
            mdl.x = position.x as f32 + 0.5;
            mdl.y = position.y as f32 + 0.5;
            mdl.z = position.z as f32 + 0.5;
            mdl.matrix[0] = Matrix4::from(Decomposed {
                scale: 1.0,
                rot: Quaternion::from_angle_y(Rad::new(info.rotation as f32)),
                disp: Vector3::new(position.x as f32 + 0.5, -position.y as f32 - 0.5, position.z as f32 + 0.5),
            }) * Matrix4::from_translation(Vector3::new(info.offset_x as f32, -info.offset_y as f32, info.offset_z as f32));
        }

        info.model = Some(model);
    }

    fn entity_removed(&mut self, m: &mut ecs::Manager, e: ecs::Entity, _: &mut world::World, renderer: &mut render::Renderer) {
        let info = m.get_component_mut(e, self.sign_info).unwrap();
        if let Some(model) = info.model {
            renderer.model.remove_model(model);
        }
        info.model = None;
    }
}
