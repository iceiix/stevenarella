
use ecs;
use format::{self, Component};
use shared::{Direction, Position};
use world;
use world::block::Block;
use render;
use render::model;

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
                info.offset_z = 5.0 / 16.0;
                info.has_stand = true;
                info.rotation = -(rotation.data() as f64 / 16.0) * PI * 2.0 + PI;
            }
            _ => return,
        }
        let wood = render::Renderer::get_texture(renderer.get_textures_ref(), "blocks/planks_oak");

        macro_rules! rel {
            ($tex:expr, $x:expr, $y:expr, $w:expr, $h:expr) => (
                Some($tex.relative(($x) / 16.0, ($y) / 16.0, ($w) / 16.0, ($h) / 16.0))
            );
        }

        let mut verts = vec![];
        // Backboard
        model::append_box_texture_scale(&mut verts, -0.5, -4.0/16.0, -0.5/16.0, 1.0, 8.0/16.0, 1.0/16.0, [
            rel!(wood, 0.0, 0.0, 16.0, 2.0), // Up
            rel!(wood, 0.0, 0.0, 16.0, 2.0), // Down
            rel!(wood, 0.0, 4.0, 16.0, 12.0), // North
            rel!(wood, 0.0, 4.0, 16.0, 12.0), // South
            rel!(wood, 0.0, 0.0, 2.0, 12.0), // West
            rel!(wood, 0.0, 0.0, 2.0, 12.0), // East
        ], [
            [1.5, 1.0], // Up
            [1.5, 1.0], // Down
            [1.5, 1.0], // North
            [1.5, 1.0], // South
            [1.0, 1.0], // West
            [1.0, 1.0], // East
        ]);
        for vert in &mut verts[8..12] {
            vert.r = 183;
            vert.g = 183;
            vert.b = 196;
        }
        if info.has_stand {
            let log = render::Renderer::get_texture(renderer.get_textures_ref(), "blocks/log_oak");
            model::append_box(&mut verts, -0.5/16.0, -0.25-9.0/16.0, -0.5/16.0, 1.0/16.0, 9.0/16.0, 1.0/16.0, [
                rel!(log, 0.0, 0.0, 2.0, 2.0), // Up
                rel!(log, 0.0, 0.0, 2.0, 2.0), // Down
                rel!(log, 0.0, 4.0, 2.0, 12.0), // North
                rel!(log, 0.0, 4.0, 2.0, 12.0), // South
                rel!(log, 0.0, 0.0, 2.0, 12.0), // West
                rel!(log, 0.0, 0.0, 2.0, 12.0), // East
            ]);
        }

        for (i, line) in info.lines.iter().enumerate() {
            let mut state = FormatState {
                line: i as f32,
                width: 0.0,
                offset: 0.0,
                text: Vec::new(),
                renderer: renderer,
            };
            state.build(line, format::Color::Black);
            let width = state.width;
            // Center align text
            for vert in &mut state.text {
                vert.x += width * 0.5;
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

struct FormatState<'a> {
    line: f32,
    offset: f32,
    width: f32,
    text: Vec<model::Vertex>,
    renderer: &'a mut render::Renderer,
}

impl <'a> FormatState<'a> {
    fn build(&mut self, c: &Component, color: format::Color) {
        match c {
            &format::Component::Text(ref txt) => {
                let col = FormatState::get_color(&txt.modifier, color);
                self.append_text(&txt.text, col);
                let modi = &txt.modifier;
                if let Some(ref extra) = modi.extra {
                    for e in extra {
                        self.build(e, col);
                    }
                }
            }
        }
    }

    fn append_text(&mut self, txt: &str, color: format::Color) {
        const Y_SCALE: f32 = (6.0 / 16.0) / 4.0;
        const X_SCALE: f32 = Y_SCALE / 16.0;

        let (rr, gg, bb) = color.to_rgb();
        for ch in txt.chars() {
            if ch == ' ' {
                self.offset += 6.0 * X_SCALE;
                continue;
            }
            let texture = self.renderer.ui.character_texture(ch);
            let w = self.renderer.ui.size_of_char(ch) as f32;

            for vert in ::model::BlockVertex::face_by_direction(Direction::North) {
                self.text.push(model::Vertex {
                    x: vert.x * X_SCALE * w - (self.offset + w * X_SCALE),
                    y: vert.y * Y_SCALE - (Y_SCALE + 0.4/16.0) * self.line + 2.1 / 16.0,
                    z: -0.6 / 16.0,
                    texture: texture.clone(),
                    texture_x: vert.toffsetx as f64,
                    texture_y: vert.toffsety as f64,
                    r: rr,
                    g: gg,
                    b: bb,
                    a: 255,
                    id: 0,
                });
            }
            self.offset += (w + 2.0) * X_SCALE;
        }
        if self.offset > self.width {
            self.width = self.offset;
        }
    }

    fn get_color(modi: &format::Modifier, color: format::Color) -> format::Color {
        modi.color.unwrap_or(color)
    }
}
