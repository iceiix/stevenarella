
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::io::Write;
use resources;
use render;
use world;
use world::block::TintType;
use chunk_builder::{self, Direction};
use serde_json;

use std::hash::BuildHasherDefault;
use types::hash::FNVHash;

use rand::Rng;

pub struct Factory {
    resources: Arc<RwLock<resources::Manager>>,
    textures: Arc<RwLock<render::TextureManager>>,

    models: HashMap<Key, StateModel, BuildHasherDefault<FNVHash>>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Key(String, String);

macro_rules! try_log {
    ($e:expr) => (
        match $e {
            Ok(val) => val,
            Err(err) => {
                error!("Error loading model {:?}", err);
                return false;
            }
        }
    );
    (opt $e:expr) => (
        match $e {
            Ok(val) => val,
            Err(err) => {
                error!("Error loading model {:?}", err);
                return None;
            }
        }
    );
}

impl Factory {
    pub fn new(resources: Arc<RwLock<resources::Manager>>, textures: Arc<RwLock<render::TextureManager>>) -> Factory {
        Factory {
            resources: resources,
            textures: textures,

            models: HashMap::with_hasher(BuildHasherDefault::default()),
        }
    }

    pub fn clear_cache(&mut self) {
        self.models.clear();
    }

    pub fn get_state_model<R: Rng, W: Write>(models: &Arc<RwLock<Factory>>, plugin: &str, name: &str, variant: &str, rng: &mut R,
            snapshot: &world::Snapshot, x: i32, y: i32, z: i32, buf: &mut W) -> usize {
        let key = Key(plugin.to_owned(), name.to_owned());
        let mut missing = false;
        {
            let m = models.read().unwrap();
            if let Some(model) = m.models.get(&key) {
                if let Some(var) = model.get_variants(variant) {
                    let model = var.choose_model(rng);
                    return model.render(snapshot, x, y, z, buf);
                }
                missing = true;
            }
        }
        if !missing {
            let mut m = models.write().unwrap();
            if !m.models.contains_key(&key) {
                if !m.load_model(plugin, name) {
                    error!("Error loading model {}:{}", plugin, name);
                }
            }
            if let Some(model) = m.models.get(&key) {
                if let Some(var) = model.get_variants(variant) {
                    let model = var.choose_model(rng);
                    return model.render(snapshot, x, y, z, buf);
                }
            }
        }
        let ret = Factory::get_state_model(models, "steven", "missing_block", "normal", rng, snapshot, x, y, z, buf);
        {
            let mut m = models.write().unwrap();
            let model = m.models.get(&Key("steven".to_owned(), "missing_block".to_owned())).unwrap().clone();
            m.models.insert(key, model);
        }
        ret
    }

    fn load_model(&mut self, plugin: &str, name: &str) -> bool {
        let file = match self.resources.read().unwrap().open(plugin, &format!("blockstates/{}.json", name)) {
            Some(val) => val,
            None => return false,
        };
        let mdl: serde_json::Value = try_log!(serde_json::from_reader(file));

        let mut model = StateModel {
            variants: HashMap::with_hasher(BuildHasherDefault::default()),
        };

        if let Some(variants) = mdl.find("variants").and_then(|v| v.as_object()) {
            for (k, v) in variants {
                let vars = self.parse_model_list(plugin, v);
                if vars.models.is_empty() {
                    return false;
                }
                model.variants.insert(k.clone(), vars);
            }
        }
        if let Some(multipart) = mdl.find("multipart").and_then(|v| v.as_array()) {
            warn!("Found unhandled multipart {:?}", multipart);
        }

        self.models.insert(Key(plugin.to_owned(), name.to_owned()), model);
        true
    }

    fn parse_model_list(&self, plugin: &str, v: &serde_json::Value) -> Variants {
        let mut variants = Variants {
            models: vec![]
        };
        if let Some(list) = v.as_array() {
            for val in list {
                if let Some(mdl) = self.parse_block_state_variant(plugin, val) {
                    variants.models.push(self.process_model(mdl));
                }
            }
        } else {
            if let Some(mdl) = self.parse_block_state_variant(plugin, v) {
                variants.models.push(self.process_model(mdl));
            }
        }
        variants
    }

    fn parse_block_state_variant(&self, plugin: &str, v: &serde_json::Value) -> Option<RawModel> {
        let model_name = match v.find("model").and_then(|v| v.as_string()) {
            Some(val) => val,
            None => {
                error!("Couldn't find model name");
                return None;
            },
        };

        let file = match self.resources.read().unwrap().open(plugin, &format!("models/block/{}.json", model_name)) {
            Some(val) => val,
            None => {
                error!("Couldn't find model {}", format!("models/block/{}.json", model_name));
                return None;
            },
        };
        let block_model: serde_json::Value = try_log!(opt serde_json::from_reader(file));

        let mut model = match self.parse_model(plugin, &block_model) {
            Some(val) => val,
            None => {
                error!("Failed to parse model {}", format!("models/block/{}.json", model_name));
                return None;
            },
        };

        model.y = v.find("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        model.x = v.find("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        model.uvlock = v.find("uvlock").and_then(|v| v.as_boolean()).unwrap_or(false);
        model.weight = v.find("weight").and_then(|v| v.as_f64()).unwrap_or(1.0);
        Some(model)
    }

    fn parse_model(&self, plugin: &str, v: &serde_json::Value) -> Option<RawModel> {
        let parent = v.find("parent").and_then(|v| v.as_string()).unwrap_or("");
        let mut model = if !parent.is_empty() && !parent.starts_with("builtin/") {
            let file = match self.resources.read().unwrap().open(plugin, &format!("models/{}.json", parent)) {
                Some(val) => val,
                None => {
                    error!("Couldn't find model {}", format!("models/{}.json", parent));
                    return None;
                },
            };
            let block_model: serde_json::Value = try_log!(opt serde_json::from_reader(file));
            let model = match self.parse_model(plugin, &block_model) {
                Some(val) => val,
                None => {
                    error!("Failed to parse model {}", format!("models/{}.json", parent));
                    return None
                },
            };
            model
        } else {
            RawModel {
                texture_vars: HashMap::with_hasher(BuildHasherDefault::default()),
                elements: vec![],
                ambient_occlusion: true,
                ao_set: false,

                x: 0.0,
                y: 0.0,
                uvlock: false,
                weight: 1.0,

                display: HashMap::with_hasher(BuildHasherDefault::default()),
                builtin: match parent {
                    "builtin/generated" => BuiltinType::Generated,
                    "builtin/entity" => BuiltinType::Entity,
                    "builtin/compass" => BuiltinType::Compass,
                    "builtin/clock" => BuiltinType::Clock,
                    _ => BuiltinType::False,
                },
            }
        };

        if let Some(textures) = v.find("textures").and_then(|v| v.as_object()) {
            for (k, v) in textures {
                model.texture_vars.insert(k.clone(), v.as_string().unwrap_or("").to_owned());
            }
        }

        if let Some(ao) = v.find("ambientocclusion").and_then(|v| v.as_boolean()) {
            model.ambient_occlusion = ao;
            model.ao_set = true;
        } else if !model.ao_set {
            model.ambient_occlusion = true;
        }

        if let Some(elements) = v.find("elements").and_then(|v| v.as_array()) {
            for e in elements {
                model.elements.push(self.parse_block_element(e));
            }
        }

        // TODO: Display

        Some(model)
    }

    fn parse_block_element(&self, v: &serde_json::Value) -> ModelElement {
        let mut element = ModelElement {
            from: v.find("from").and_then(|v| v.as_array()).map(|v| [
                v[0].as_f64().unwrap(),
                v[1].as_f64().unwrap(),
                v[2].as_f64().unwrap()
            ]).unwrap(),
            to: v.find("to").and_then(|v| v.as_array()).map(|v| [
                v[0].as_f64().unwrap(),
                v[1].as_f64().unwrap(),
                v[2].as_f64().unwrap()
            ]).unwrap(),
            shade: v.find("shade").and_then(|v| v.as_boolean()).unwrap_or(false),
            faces: [None, None, None, None, None, None],
            rotation: None,
        };
        if let Some(faces) = v.find("faces").and_then(|v| v.as_object()) {
            for dir in Direction::all() {
                if let Some(face) = faces.get(dir.as_string()) {
                    element.faces[dir.index()] = Some(BlockFace {
                        uv: face.find("uv").and_then(|v| v.as_array()).map(|v| [
                            v[0].as_f64().unwrap(),
                            v[1].as_f64().unwrap(),
                            v[2].as_f64().unwrap(),
                            v[3].as_f64().unwrap()
                        ]).unwrap_or_else(|| {
                            let mut uv = [0.0, 0.0, 16.0, 16.0];
                            match dir {
                                Direction::North | Direction::South => {
                                        uv[0] = element.from[0];
                                        uv[2] = element.to[0];
                                        uv[1] = 16.0 - element.to[1];
                                        uv[3] = 16.0 - element.from[1];
                                },
                                Direction::West | Direction::East => {
                                        uv[0] = element.from[2];
                                        uv[2] = element.to[2];
                                        uv[1] = 16.0 - element.to[1];
                                        uv[3] = 16.0 - element.from[1];
                                },
                                Direction::Down | Direction::Up => {
                                        uv[0] = element.from[0];
                                        uv[2] = element.to[0];
                                        uv[1] = 16.0 - element.to[2];
                                        uv[3] = 16.0 - element.from[2];
                                },
                                _ => unreachable!(),
                            }
                            uv
                        }),
                        texture: face.find("texture")
                            .and_then(|v| v.as_string())
                            .map(|v| if v.starts_with('#') {
                                v.to_owned()
                            } else {
                                "#".to_owned() + v
                            }).unwrap(),
                        cull_face: Direction::from_string(
                                face.find("cullface")
                                    .and_then(|v| v.as_string())
                                    .unwrap_or("invalid")
                            ),
                        rotation: face.find("rotation")
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32)
                            .unwrap_or(0),
                        tint_index: face.find("tintindex")
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32)
                            .unwrap_or(-1),
                    });
                }
            }
        }

        if let Some(rotation) = v.find("rotation") {
            element.rotation = Some(BlockRotation {
                origin: rotation.find("origin").and_then(|v| v.as_array()).map(|v| [
                    v[0].as_f64().unwrap(),
                    v[1].as_f64().unwrap(),
                    v[2].as_f64().unwrap()
                ]).unwrap_or([8.0, 8.0, 8.0]),
                axis: rotation.find("axis").and_then(|v| v.as_string()).unwrap_or("").to_owned(),
                angle: rotation.find("angle").and_then(|v| v.as_f64()).unwrap_or(0.0),
                rescale: rotation.find("rescale").and_then(|v| v.as_boolean()).unwrap_or(false),
            });
        }

        element
    }

    fn process_model(&self, mut raw: RawModel) -> Model {
        let mut model = Model {
            faces: vec![],
            ambient_occlusion: raw.ambient_occlusion,
            weight: raw.weight,
        };
        let elements = ::std::mem::replace(&mut raw.elements, vec![]);
        for el in elements {
            let all_dirs = Direction::all();
            for (i, face) in el.faces.iter().enumerate() {
                if let Some(face) = face.as_ref() {
                    let face_id = all_dirs[i];
                    let mut processed_face = Face {
                        cull_face: face.cull_face,
                        facing: face_id,
                        vertices: vec![],
                        vertices_texture: vec![],
                        indices: 0,
                        shade: el.shade,
                        tint_index: face.tint_index,
                    };
                    if raw.x > 0.0 {
                        let o = (raw.x as i32) / 90;
                        processed_face.cull_face = rotate_direction(processed_face.cull_face, o, FACE_ROTATION_X, &[
                            Direction::East,
                            Direction::West,
                            Direction::Invalid,
                        ]);
                        processed_face.facing = rotate_direction(processed_face.facing, o, FACE_ROTATION_X, &[
                            Direction::East,
                            Direction::West,
                            Direction::Invalid,
                        ]);
                    }
                    if raw.y > 0.0 {
                        let o = (raw.y as i32) / 90;
                        processed_face.cull_face = rotate_direction(processed_face.cull_face, o, FACE_ROTATION, &[
                            Direction::Up,
                            Direction::Down,
                            Direction::Invalid,
                        ]);
                        processed_face.facing = rotate_direction(processed_face.facing, o, FACE_ROTATION, &[
                            Direction::Up,
                            Direction::Down,
                            Direction::Invalid,
                        ]);
                    }

                    let mut verts = all_dirs[i].get_verts().to_vec();
                    let texture_name = raw.lookup_texture(&face.texture);
                    let texture = render::Renderer::get_texture(&self.textures, &texture_name);

                    let mut ux1 = (face.uv[0] * (texture.get_width() as f64)) as i16;
                    let mut ux2 = (face.uv[2] * (texture.get_width() as f64)) as i16;
                    let mut uy1 = (face.uv[1] * (texture.get_height() as f64)) as i16;
                    let mut uy2 = (face.uv[3] * (texture.get_height() as f64)) as i16;

                    let tw = texture.get_width() as i16;
                    let th = texture.get_height() as i16;
                    if face.rotation > 0 {
                        let x = ux1;
                        let y = uy1;
                        let w = ux2 - ux1;
                        let h = uy2 - uy1;
                        match face.rotation {
                            90 => {
            					uy2 = x + w;
            					ux1 = tw*16 - (y + h);
            					ux2 = tw*16 - y;
            					uy1 = x;
                            },
                            180 => {
            					uy1 = th*16 - (y + h);
            					uy2 = th*16 - y;
            					ux1 = x + w;
            					ux2 = x;
                            },
                            270 => {
            					uy2 = x;
            					uy1 = x + w;
            					ux2 = y + h;
            					ux1 = y;
                            },
                            _ => {},
                        }
                    }

                    let mut min_x = ::std::f32::INFINITY;
                    let mut min_y = ::std::f32::INFINITY;
                    let mut min_z = ::std::f32::INFINITY;
                    let mut max_x = ::std::f32::NEG_INFINITY;
                    let mut max_y = ::std::f32::NEG_INFINITY;
                    let mut max_z = ::std::f32::NEG_INFINITY;

                    for v in &mut verts {
                        processed_face.vertices_texture.push(texture.clone());
                        v.tx = texture.get_x() as u16;
                        v.ty = texture.get_y() as u16;
                        v.tw = texture.get_width() as u16;
                        v.th = texture.get_height() as u16;
                        v.tatlas = texture.atlas as i16;

                        if v.x < 0.5 {
                            v.x = (el.from[0] / 16.0) as f32;
                        } else {
                            v.x = (el.to[0] / 16.0) as f32;
                        }

                        if v.y < 0.5 {
                            v.y = (el.from[1] / 16.0) as f32;
                        } else {
                            v.y = (el.to[1] / 16.0) as f32;
                        }

                        if v.z < 0.5 {
                            v.z = (el.from[2] / 16.0) as f32;
                        } else {
                            v.z = (el.to[2] / 16.0) as f32;
                        }

                        if let Some(r) = el.rotation.as_ref() {
                            match &*r.axis {
                                "y" => {
                                    let rot_y = (-r.angle * (::std::f64::consts::PI / 180.0)) as f32;
                                    let c = rot_y.cos();
                                    let s = rot_y.sin();
                                    let x = v.x - ((r.origin[0] as f32)/16.0);
                                    let z = v.z - ((r.origin[2] as f32)/16.0);
                                    v.x = ((r.origin[0] as f32)/16.0) + (x*c - z*s);
                                    v.z = ((r.origin[2] as f32)/16.0) + (z*c + x*s);
                                },
                                "x" => {
                                    let rot_x = (-r.angle * (::std::f64::consts::PI / 180.0)) as f32;
                                    let c = rot_x.cos();
                                    let s = rot_x.sin();
                                    let z = v.z - ((r.origin[2] as f32)/16.0);
                                    let y = v.y - ((r.origin[1] as f32)/16.0);
                                    v.z = ((r.origin[2] as f32)/16.0) + (z*c - y*s);
                                    v.y = ((r.origin[1] as f32)/16.0) + (y*c + z*s);
                                },
                                "z" => {
                                    let rot_z = (r.angle * (::std::f64::consts::PI / 180.0)) as f32;
                                    let c = rot_z.cos();
                                    let s = rot_z.sin();
                                    let x = v.x - ((r.origin[0] as f32)/16.0);
                                    let y = v.y - ((r.origin[1] as f32)/16.0);
                                    v.x = ((r.origin[0] as f32)/16.0) + (x*c - y*s);
                                    v.y = ((r.origin[1] as f32)/16.0) + (y*c + x*s);
                                },
                                _ => {}
                            }
                        }

                        if raw.x > 0.0 {
                            let rot_x = (raw.x * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_x.cos();
                            let s = rot_x.sin();
                            let z = v.z - 0.5;
                            let y = v.y - 0.5;
                            v.z = 0.5 + (z*c - y*s);
                            v.y = 0.5 + (y*c + z*s);
                        }

                        if raw.y > 0.0 {
                            let rot_y = (raw.y * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_y.cos();
                            let s = rot_y.sin();
                            let x = v.x - 0.5;
                            let z = v.z - 0.5;
                            v.x = 0.5 + (x*c - z*s);
                            v.z = 0.5 + (z*c + x*s);
                        }

                        if v.toffsetx == 0 {
                            v.toffsetx = ux1;
                        } else {
                            v.toffsetx = ux2;
                        }

                        if v.toffsety == 0 {
                            v.toffsety = uy1;
                        } else {
                            v.toffsety = uy2;
                        }

                        if face.rotation > 0 {
                            let rot_y = (face.rotation as f64 * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_y.cos() as i16;
                            let s = rot_y.sin() as i16;
                            let x = v.toffsetx - 8*tw;
                            let y = v.toffsety - 8*th;
                            v.toffsetx = 8*tw + (x*c - y*s);
                            v.toffsety = 8*th + (y*c + x*s);
                        }

                        if raw.uvlock && raw.y > 0.0
                            && (processed_face.facing == Direction::Up || processed_face.facing == Direction::Down) {
                            let rot_y = (raw.y * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_y.cos() as i16;
                            let s = rot_y.sin() as i16;
                            let x = v.toffsetx - 8*16;
                            let y = v.toffsety - 8*16;
                            v.toffsetx = 8*16 + (x*c - y*s);
                            v.toffsety = 8*16 + (y*c + x*s);
                        }

                        if raw.uvlock && raw.x > 0.0
                            && (processed_face.facing != Direction::Up && processed_face.facing != Direction::Down) {
                            let rot_x = (raw.x * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_x.cos() as i16;
                            let s = rot_x.sin() as i16;
                            let x = v.toffsetx - 8*16;
                            let y = v.toffsety - 8*16;
                            v.toffsetx = 8*16 + (x*c - y*s);
                            v.toffsety = 8*16 + (y*c + x*s);
                        }

                        if let Some(r) = el.rotation.as_ref() {
                            if r.rescale {
                                min_x = min_x.min(v.x);
                                min_y = min_y.min(v.y);
                                min_z = min_z.min(v.z);
                                max_x = max_x.max(v.x);
                                max_y = max_y.max(v.y);
                                max_z = max_z.max(v.z);
                            }
                        }
                    }

                    if let Some(r) = el.rotation.as_ref() {
                        if r.rescale {
                            let dx = max_x - min_x;
                            let dy = max_y - min_y;
                            let dz = max_z - min_z;
                            for v in &mut verts {
                                v.x = (v.x - min_x) / dx;
                                v.y = (v.y - min_y) / dy;
                                v.z = (v.z - min_z) / dz;
                            }
                        }
                    }

                    processed_face.vertices = verts;
                    processed_face.indices = 6;
                    model.faces.push(processed_face);
                }
            }
        }
        model
    }
}

const FACE_ROTATION: &'static [Direction] = &[
	Direction::North,
	Direction::East,
	Direction::South,
	Direction::West,
];

const FACE_ROTATION_X: &'static [Direction] = &[
	Direction::North,
	Direction::Down,
	Direction::South,
	Direction::Up,
];

fn rotate_direction(val: Direction, offset: i32, rots: &[Direction], invalid: &[Direction]) -> Direction {
    for d in invalid {
        if *d == val {
            return val;
        }
    }
    let pos = rots.iter()
        .position(|v| *v == val)
        .unwrap_or(0);
    rots[(pos + offset as usize) % rots.len()]
}

#[derive(Clone)]
pub struct StateModel {
    variants: HashMap<String, Variants, BuildHasherDefault<FNVHash>>,
}

impl StateModel {
    pub fn get_variants(&self, name: &str) -> Option<&Variants> {
        self.variants.get(name)
    }
}

#[derive(Clone)]
pub struct Variants {
    models: Vec<Model>,
}

impl Variants {
    fn choose_model<R: Rng>(&self, rng: &mut R) -> &Model {
        // TODO: Weighted random
        rng.choose(&self.models).unwrap()
    }
}

#[derive(Debug)]
enum BuiltinType {
    False,
    Generated,
    Entity,
    Compass,
    Clock
}

#[derive(Debug)]
struct RawModel {
    texture_vars: HashMap<String, String, BuildHasherDefault<FNVHash>>,
    elements: Vec<ModelElement>,
    ambient_occlusion: bool,
    ao_set: bool,

    x: f64,
    y: f64,
    uvlock: bool,
    weight: f64,

    display: HashMap<String, ModelDisplay, BuildHasherDefault<FNVHash>>,
    builtin: BuiltinType,
}

impl RawModel {
    fn lookup_texture(&self, name: &str) -> String {
        if !name.is_empty() && name.starts_with('#') {
            let tex = self.texture_vars.get(&name[1..]).map(|v| v.clone()).unwrap_or("".to_owned());
            return self.lookup_texture(&tex);
        }
        name.to_owned()
    }
}

#[derive(Debug)]
struct ModelDisplay {
    rotation: [f64; 3],
    translation: [f64; 3],
    scale: [f64; 3],
}

#[derive(Debug)]
struct ModelElement {
    from: [f64; 3],
    to: [f64; 3],
    shade: bool,
    rotation: Option<BlockRotation>,
    faces: [Option<BlockFace>; 6],
}

#[derive(Debug)]
struct BlockRotation {
    origin: [f64; 3],
    axis: String,
    angle: f64,
    rescale: bool,
}

#[derive(Debug)]
struct BlockFace {
    uv: [f64; 4],
    texture: String,
    cull_face: Direction,
    rotation: i32,
    tint_index: i32,
}

#[derive(Clone)]
struct Model {
    faces: Vec<Face>,
    ambient_occlusion: bool,
    weight: f64,
}

#[derive(Clone)]
struct Face {
    cull_face: Direction,
    facing: Direction,
    vertices: Vec<chunk_builder::BlockVertex>,
    vertices_texture: Vec<render::Texture>,
    indices: usize,
    shade: bool,
    tint_index: i32,
}

impl Model {
    fn render<W: Write>(&self, snapshot: &world::Snapshot, x: i32, y: i32, z: i32, buf: &mut W) -> usize {
        let this = snapshot.get_block(x, y, z);
        let this_mat = this.get_material();
        let mut indices = 0;

        let tint = this.get_tint();

        for face in &self.faces {
            if face.cull_face != Direction::Invalid {
                let (ox, oy, oz) = face.cull_face.get_offset();
                let other = snapshot.get_block(x + ox, y + oy, z + oz);
                if other.get_material().should_cull_against || other == this {
                    continue;
                }
            }


            let (mut cr, mut cg, mut cb) = (255, 255, 255);
            match face.tint_index {
                0 => {
                    match tint {
                        TintType::Default => {},
                        TintType::Color{r, g, b} => {
                            cr = r;
                            cg = g;
                            cb = b;
                        },
                        TintType::Grass => {}, // TODO
                        TintType::Foliage => {}, // TODO
                    }
                },
                _ => {},
            }
            if face.facing == Direction::West || face.facing == Direction::East {
                cr = ((cr as f64) * 0.8) as u8;
                cg = ((cg as f64) * 0.8) as u8;
                cb = ((cb as f64) * 0.8) as u8;
            }
            indices += face.indices;

            for vert in &face.vertices {
                let mut vert = vert.clone();
                vert.r = cr;
                vert.g = cg;
                vert.b = cb;

                vert.x += x as f32;
                vert.y += y as f32;
                vert.z += z as f32;

                let (bl, sl) = calculate_light(
                    &snapshot,
                    x, y, z,
                    vert.x as f64,
                    vert.y as f64,
                    vert.z as f64,
                    face.facing,
                    self.ambient_occlusion,
                    this_mat.force_shade
                );
                vert.block_light = bl;
                vert.sky_light = sl;
                vert.write(buf);
            }
        }
        indices
    }
}



fn calculate_light(snapshot: &world::Snapshot, orig_x: i32, orig_y: i32, orig_z: i32,
                    x: f64, y: f64, z: f64, face: Direction, smooth: bool, force: bool) -> (u16, u16) {
    use std::cmp::max;
    use world::block;
    let (ox, oy, oz) = if !snapshot.get_block(orig_x, orig_y, orig_z).get_material().should_cull_against {
        (0, 0, 0)
    } else {
        face.get_offset()
    };

    let s_block_light = snapshot.get_block_light(orig_x + ox, orig_y + oy, orig_z + oz);
    let s_sky_light = snapshot.get_sky_light(orig_x + ox, orig_y + oy, orig_z + oz);
    if !smooth {
        return ((s_block_light as u16) * 4000, (s_sky_light as u16) * 4000);
    }

    let mut block_light = 0u32;
    let mut sky_light = 0u32;
    let mut count = 0;

    let s_block_light = max(((s_block_light as i8) - 8), 0) as u8;
    let s_sky_light = max(((s_sky_light as i8) - 8), 0) as u8;

    let dx = (ox as f64) * 0.6;
    let dy = (oy as f64) * 0.6;
    let dz = (oz as f64) * 0.6;

    for ox in [-0.6, 0.0].into_iter() {
        for oy in [-0.6, 0.0].into_iter() {
            for oz in [-0.6, 0.0].into_iter() {
                let lx = (x + ox + dx).round() as i32;
                let ly = (y + oy + dy).round() as i32;
                let lz = (z + oz + dz).round() as i32;
                let mut bl = snapshot.get_block_light(lx, ly, lz);
                let mut sl = snapshot.get_sky_light(lx, ly, lz);
                if (force && match snapshot.get_block(lx, ly, lz) { block::Air{} => false, _ => true })
                    || (sl == 0 && bl == 0){
                    bl = s_block_light;
                    sl = s_sky_light;
                }
                block_light += bl as u32;
                sky_light += sl as u32;
                count += 1;
            }
        }
    }

    ((((block_light * 4000) / count) as u16), (((sky_light * 4000) / count) as u16))
}
