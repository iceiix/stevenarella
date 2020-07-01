pub mod liquid;

use crate::render;
use crate::resources;
use crate::shared::Direction;
use crate::world;
use crate::world::block::{Block, TintType};
use byteorder::{NativeEndian, WriteBytesExt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, RwLock};

use crate::types::hash::FNVHash;
use log::error;
use std::hash::BuildHasherDefault;

use image::GenericImageView;
use rand::seq::SliceRandom;
use rand::Rng;

pub struct Factory {
    resources: Arc<RwLock<resources::Manager>>,
    pub textures: Arc<RwLock<render::TextureManager>>,

    models: HashMap<Key, StateModel, BuildHasherDefault<FNVHash>>,

    grass_colors: image::DynamicImage,
    foliage_colors: image::DynamicImage,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Key(String, String);

macro_rules! try_log {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => {
                error!("Error loading model {:?}", err);
                return false;
            }
        }
    };
    (opt $e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => {
                error!("Error loading model {:?}", err);
                return None;
            }
        }
    };
}

thread_local!(
    static MULTIPART_CACHE: RefCell<HashMap<(Key, Block), Model, BuildHasherDefault<FNVHash>>> = RefCell::new(HashMap::with_hasher(BuildHasherDefault::default()))
);

impl Factory {
    pub fn new(
        resources: Arc<RwLock<resources::Manager>>,
        textures: Arc<RwLock<render::TextureManager>>,
    ) -> Factory {
        Factory {
            grass_colors: Factory::load_biome_colors(resources.clone(), "grass"),
            foliage_colors: Factory::load_biome_colors(resources.clone(), "foliage"),
            resources,
            textures,

            models: HashMap::with_hasher(BuildHasherDefault::default()),
        }
    }

    fn load_biome_colors(res: Arc<RwLock<resources::Manager>>, name: &str) -> image::DynamicImage {
        let mut val = match res
            .read()
            .unwrap()
            .open("minecraft", &format!("textures/colormap/{}.png", name))
        {
            Some(val) => val,
            None => return image::DynamicImage::new_rgb8(256, 256),
        };
        let mut data = Vec::new();
        val.read_to_end(&mut data).unwrap();
        image::load_from_memory(&data).unwrap()
    }

    pub fn version_change(&mut self) {
        self.models.clear();
        self.grass_colors = Factory::load_biome_colors(self.resources.clone(), "grass");
        self.foliage_colors = Factory::load_biome_colors(self.resources.clone(), "foliage");
    }

    fn get_model<R: Rng, W: Write>(
        &self,
        key: Key,
        block: Block,
        rng: &mut R,
        snapshot: &world::Snapshot,
        x: i32,
        y: i32,
        z: i32,
        buf: &mut W,
    ) -> Result<usize, bool> {
        use std::collections::hash_map::Entry;
        if let Some(model) = self.models.get(&key) {
            if model.multipart.is_empty() {
                let variant = block.get_model_variant();
                if let Some(var) = model.get_variants(&variant) {
                    let model = var.choose_model(rng);
                    return Ok(model.render(self, snapshot, x, y, z, buf));
                }
            } else {
                return MULTIPART_CACHE.with(|cache| {
                    let mut cache = cache.borrow_mut();
                    let entry = cache.entry((key.clone(), block));
                    match entry {
                        Entry::Occupied(e) => {
                            return Ok(e.get().render(self, snapshot, x, y, z, buf));
                        }
                        Entry::Vacant(e) => {
                            let mut res: Option<Model> = None;
                            for rule in &model.multipart {
                                let ok = Self::eval_rules(block, &rule.rules);
                                if ok {
                                    if res.is_some() {
                                        res.as_mut().unwrap().join(rule.apply.choose_model(rng));
                                    } else {
                                        res = Some(rule.apply.choose_model(rng).clone());
                                    }
                                }
                            }
                            if let Some(mdl) = res {
                                return Ok(e.insert(mdl).render(self, snapshot, x, y, z, buf));
                            }
                        }
                    };
                    Err(true)
                });
            }
            return Err(true);
        }
        Err(false)
    }

    fn eval_rules(block: Block, rules: &[Rule]) -> bool {
        for mrule in rules {
            match *mrule {
                Rule::Or(ref sub_rules) => {
                    let mut ok = false;
                    for srule in sub_rules {
                        if Self::eval_rules(block, srule) {
                            ok = true;
                            break;
                        }
                    }
                    if !ok {
                        return false;
                    }
                }
                Rule::Match(ref key, ref val) => {
                    if !block.match_multipart(key, val) {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn get_state_model<R: Rng, W: Write>(
        models: &Arc<RwLock<Factory>>,
        block: Block,
        rng: &mut R,
        snapshot: &world::Snapshot,
        x: i32,
        y: i32,
        z: i32,
        buf: &mut W,
    ) -> usize {
        let (plugin, name) = block.get_model();
        let key = Key(plugin.to_owned(), name.to_owned());
        let mut missing_variant;
        {
            let m = models.read().unwrap();
            match m.get_model(key.clone(), block, rng, snapshot, x, y, z, buf) {
                Ok(val) => return val,
                Err(val) => missing_variant = val,
            };
        }
        if !missing_variant {
            // Whole model not loaded, try and load
            let mut m = models.write().unwrap();
            if !m.models.contains_key(&key) && !m.load_model(&plugin, &name) {
                error!("Error loading model {}:{}", plugin, name);
            }
            match m.get_model(key.clone(), block, rng, snapshot, x, y, z, buf) {
                Ok(val) => return val,
                Err(val) => missing_variant = val,
            };
        }
        let ret = Factory::get_state_model(models, Block::Missing {}, rng, snapshot, x, y, z, buf);
        if !missing_variant {
            // Still no model, replace with placeholder
            let mut m = models.write().unwrap();
            let model = m
                .models
                .get(&Key("steven".to_owned(), "missing_block".to_owned()))
                .unwrap()
                .clone();
            m.models.insert(key, model);
        }
        ret
    }

    fn load_model(&mut self, plugin: &str, name: &str) -> bool {
        let file = match self
            .resources
            .read()
            .unwrap()
            .open(plugin, &format!("blockstates/{}.json", name))
        {
            Some(val) => val,
            None => {
                error!("Error missing block state for {}:{}", plugin, name);
                return false;
            }
        };
        let mdl: serde_json::Value = try_log!(serde_json::from_reader(file));

        let mut model = StateModel {
            variants: HashMap::with_hasher(BuildHasherDefault::default()),
            multipart: vec![],
        };

        if let Some(variants) = mdl.get("variants").and_then(|v| v.as_object()) {
            for (k, v) in variants {
                let vars = self.parse_model_list(plugin, v);
                if vars.models.is_empty() {
                    return false;
                }
                model.variants.insert(k.clone(), vars);
            }
        }
        if let Some(multipart) = mdl.get("multipart").and_then(|v| v.as_array()) {
            for rule in multipart {
                let apply = self.parse_model_list(plugin, rule.get("apply").unwrap());
                let mut rules = vec![];
                if let Some(when) = rule.get("when").and_then(|v| v.as_object()) {
                    Self::parse_rules(when, &mut rules);
                }
                model.multipart.push(MultipartRule { apply, rules })
            }
        }

        self.models
            .insert(Key(plugin.to_owned(), name.to_owned()), model);
        true
    }

    fn parse_rules(when: &serde_json::Map<String, serde_json::Value>, rules: &mut Vec<Rule>) {
        for (name, val) in when {
            if name == "OR" {
                let mut or_rules = vec![];
                for sub in val.as_array().unwrap() {
                    let mut sub_rules = vec![];
                    Self::parse_rules(sub.as_object().unwrap(), &mut sub_rules);
                    or_rules.push(sub_rules);
                }
                rules.push(Rule::Or(or_rules));
            } else {
                let v = match *val {
                    serde_json::Value::Bool(ref v) => v.to_string(),
                    serde_json::Value::Number(ref v) => v.to_string(),
                    serde_json::Value::String(ref v) => v.to_owned(),
                    _ => unreachable!(),
                };
                rules.push(Rule::Match(name.to_owned(), v));
            }
        }
    }

    fn parse_model_list(&self, plugin: &str, v: &serde_json::Value) -> Variants {
        let mut variants = Variants { models: vec![] };
        if let Some(list) = v.as_array() {
            for val in list {
                if let Some(mdl) = self.parse_block_state_variant(plugin, val) {
                    variants.models.push(self.process_model(mdl));
                }
            }
        } else if let Some(mdl) = self.parse_block_state_variant(plugin, v) {
            variants.models.push(self.process_model(mdl));
        }
        variants
    }

    fn parse_block_state_variant(&self, plugin: &str, v: &serde_json::Value) -> Option<RawModel> {
        let model_name = match v.get("model").and_then(|v| v.as_str()) {
            Some(val) => val,
            None => {
                error!("Couldn't find model name");
                return None;
            }
        };

        let file = match self
            .resources
            .read()
            .unwrap()
            .open(plugin, &format!("models/block/{}.json", model_name))
        {
            Some(val) => val,
            None => {
                error!(
                    "Couldn't find model {}",
                    format!("models/block/{}.json", model_name)
                );
                return None;
            }
        };
        let block_model: serde_json::Value = try_log!(opt serde_json::from_reader(file));

        let mut model = match self.parse_model(plugin, &block_model) {
            Some(val) => val,
            None => {
                error!(
                    "Failed to parse model {}",
                    format!("models/block/{}.json", model_name)
                );
                return None;
            }
        };

        model.y = v.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        model.x = v.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        model.uvlock = v.get("uvlock").and_then(|v| v.as_bool()).unwrap_or(false);
        model.weight = v.get("weight").and_then(|v| v.as_f64()).unwrap_or(1.0);
        Some(model)
    }

    fn parse_model(&self, plugin: &str, v: &serde_json::Value) -> Option<RawModel> {
        let parent = v.get("parent").and_then(|v| v.as_str()).unwrap_or("");
        let mut model = if !parent.is_empty() && !parent.starts_with("builtin/") {
            let file = match self
                .resources
                .read()
                .unwrap()
                .open(plugin, &format!("models/{}.json", parent))
            {
                Some(val) => val,
                None => {
                    error!("Couldn't find model {}", format!("models/{}.json", parent));
                    return None;
                }
            };
            let block_model: serde_json::Value = try_log!(opt serde_json::from_reader(file));
            match self.parse_model(plugin, &block_model) {
                Some(val) => val,
                None => {
                    error!(
                        "Failed to parse model {}",
                        format!("models/{}.json", parent)
                    );
                    return None;
                }
            }
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

        if let Some(textures) = v.get("textures").and_then(|v| v.as_object()) {
            for (k, v) in textures {
                model
                    .texture_vars
                    .insert(k.clone(), v.as_str().unwrap_or("").to_owned());
            }
        }

        if let Some(ao) = v.get("ambientocclusion").and_then(|v| v.as_bool()) {
            model.ambient_occlusion = ao;
            model.ao_set = true;
        } else if !model.ao_set {
            model.ambient_occlusion = true;
        }

        if let Some(elements) = v.get("elements").and_then(|v| v.as_array()) {
            for e in elements {
                model.elements.push(self.parse_block_element(e));
            }
        }

        // TODO: Display

        Some(model)
    }

    fn parse_block_element(&self, v: &serde_json::Value) -> ModelElement {
        let mut element = ModelElement {
            from: v
                .get("from")
                .and_then(|v| v.as_array())
                .map(|v| {
                    [
                        v[0].as_f64().unwrap(),
                        v[1].as_f64().unwrap(),
                        v[2].as_f64().unwrap(),
                    ]
                })
                .unwrap(),
            to: v
                .get("to")
                .and_then(|v| v.as_array())
                .map(|v| {
                    [
                        v[0].as_f64().unwrap(),
                        v[1].as_f64().unwrap(),
                        v[2].as_f64().unwrap(),
                    ]
                })
                .unwrap(),
            shade: v.get("shade").and_then(|v| v.as_bool()).unwrap_or(false),
            faces: [None, None, None, None, None, None],
            rotation: None,
        };
        if let Some(faces) = v.get("faces").and_then(|v| v.as_object()) {
            for dir in Direction::all() {
                if let Some(face) = faces.get(dir.as_string()) {
                    element.faces[dir.index()] = Some(BlockFace {
                        uv: face.get("uv").and_then(|v| v.as_array()).map_or_else(
                            || {
                                let mut uv = [0.0, 0.0, 16.0, 16.0];
                                match dir {
                                    Direction::North | Direction::South => {
                                        uv[0] = element.from[0];
                                        uv[2] = element.to[0];
                                        uv[1] = 16.0 - element.to[1];
                                        uv[3] = 16.0 - element.from[1];
                                    }
                                    Direction::West | Direction::East => {
                                        uv[0] = element.from[2];
                                        uv[2] = element.to[2];
                                        uv[1] = 16.0 - element.to[1];
                                        uv[3] = 16.0 - element.from[1];
                                    }
                                    Direction::Down | Direction::Up => {
                                        uv[0] = element.from[0];
                                        uv[2] = element.to[0];
                                        uv[1] = 16.0 - element.to[2];
                                        uv[3] = 16.0 - element.from[2];
                                    }
                                    _ => unreachable!(),
                                }
                                uv
                            },
                            |v| {
                                [
                                    v[0].as_f64().unwrap(),
                                    v[1].as_f64().unwrap(),
                                    v[2].as_f64().unwrap(),
                                    v[3].as_f64().unwrap(),
                                ]
                            },
                        ),
                        texture: face
                            .get("texture")
                            .and_then(|v| v.as_str())
                            .map(|v| {
                                if v.starts_with('#') {
                                    v.to_owned()
                                } else {
                                    "#".to_owned() + v
                                }
                            })
                            .unwrap(),
                        cull_face: Direction::from_string(
                            face.get("cullface")
                                .and_then(|v| v.as_str())
                                .unwrap_or("invalid"),
                        ),
                        rotation: face
                            .get("rotation")
                            .and_then(|v| v.as_i64())
                            .map_or(0, |v| v as i32),
                        tint_index: face
                            .get("tintindex")
                            .and_then(|v| v.as_i64())
                            .map_or(-1, |v| v as i32),
                    });
                }
            }
        }

        if let Some(rotation) = v.get("rotation") {
            element.rotation = Some(BlockRotation {
                origin: rotation.get("origin").and_then(|v| v.as_array()).map_or(
                    [8.0, 8.0, 8.0],
                    |v| {
                        [
                            v[0].as_f64().unwrap(),
                            v[1].as_f64().unwrap(),
                            v[2].as_f64().unwrap(),
                        ]
                    },
                ),
                axis: rotation
                    .get("axis")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned(),
                angle: rotation
                    .get("angle")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                rescale: rotation
                    .get("rescale")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
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
                        processed_face.cull_face = rotate_direction(
                            processed_face.cull_face,
                            o,
                            FACE_ROTATION_X,
                            &[Direction::East, Direction::West, Direction::Invalid],
                        );
                        processed_face.facing = rotate_direction(
                            processed_face.facing,
                            o,
                            FACE_ROTATION_X,
                            &[Direction::East, Direction::West, Direction::Invalid],
                        );
                    }
                    if raw.y > 0.0 {
                        let o = (raw.y as i32) / 90;
                        processed_face.cull_face = rotate_direction(
                            processed_face.cull_face,
                            o,
                            FACE_ROTATION,
                            &[Direction::Up, Direction::Down, Direction::Invalid],
                        );
                        processed_face.facing = rotate_direction(
                            processed_face.facing,
                            o,
                            FACE_ROTATION,
                            &[Direction::Up, Direction::Down, Direction::Invalid],
                        );
                    }

                    let mut verts = BlockVertex::face_by_direction(all_dirs[i]).to_vec();
                    let texture_name = raw.lookup_texture(&face.texture);
                    let texture = render::Renderer::get_texture(&self.textures, &texture_name);

                    let mut ux1 = (face.uv[0] * (texture.get_width() as f64)) as i16;
                    let mut ux2 = (face.uv[2] * (texture.get_width() as f64)) as i16;
                    let mut uy1 = (face.uv[1] * (texture.get_height() as f64)) as i16;
                    let mut uy2 = (face.uv[3] * (texture.get_height() as f64)) as i16;

                    let tw = texture.get_width() as i16;
                    let th = texture.get_height() as i16;
                    if face.rotation > 0 {
                        let ox1 = ux1;
                        let ox2 = ux2;
                        let oy1 = uy1;
                        let oy2 = uy2;
                        match face.rotation {
                            270 => {
                                uy1 = tw * 16 - ox2;
                                uy2 = tw * 16 - ox1;
                                ux1 = oy1;
                                ux2 = oy2;
                            }
                            180 => {
                                uy1 = th * 16 - oy2;
                                uy2 = th * 16 - oy1;
                                ux1 = tw * 16 - ox2;
                                ux2 = tw * 16 - ox1;
                            }
                            90 => {
                                uy1 = ox1;
                                uy2 = ox2;
                                ux1 = th * 16 - oy2;
                                ux2 = th * 16 - oy1;
                            }
                            _ => {}
                        }
                    }

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
                            let angle = r.angle * (::std::f64::consts::PI / 180.0);
                            let angle = if r.axis == "z" { angle } else { -angle } as f32;
                            let ci = 1.0 / angle.cos();
                            v.x -= (r.origin[0] / 16.0) as f32;
                            v.y -= (r.origin[1] / 16.0) as f32;
                            v.z -= (r.origin[2] / 16.0) as f32;
                            match &*r.axis {
                                "y" => {
                                    let c = angle.cos();
                                    let s = angle.sin();
                                    let x = v.x;
                                    let z = v.z;
                                    v.x = x * c - z * s;
                                    v.z = z * c + x * s;

                                    if r.rescale {
                                        v.x *= ci;
                                        v.z *= ci;
                                    }
                                }
                                "x" => {
                                    let c = angle.cos();
                                    let s = angle.sin();
                                    let z = v.z;
                                    let y = v.y;
                                    v.z = z * c - y * s;
                                    v.y = y * c + z * s;

                                    if r.rescale {
                                        v.z *= ci;
                                        v.y *= ci;
                                    }
                                }
                                "z" => {
                                    let c = angle.cos();
                                    let s = angle.sin();
                                    let x = v.x;
                                    let y = v.y;
                                    v.x = x * c - y * s;
                                    v.y = y * c + x * s;

                                    if r.rescale {
                                        v.x *= ci;
                                        v.y *= ci;
                                    }
                                }
                                _ => {}
                            }
                            v.x += (r.origin[0] / 16.0) as f32;
                            v.y += (r.origin[1] / 16.0) as f32;
                            v.z += (r.origin[2] / 16.0) as f32;
                        }

                        if raw.x > 0.0 {
                            let rot_x = (raw.x * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_x.cos();
                            let s = rot_x.sin();
                            let z = v.z - 0.5;
                            let y = v.y - 0.5;
                            v.z = 0.5 + (z * c - y * s);
                            v.y = 0.5 + (y * c + z * s);
                        }

                        if raw.y > 0.0 {
                            let rot_y = (raw.y * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_y.cos();
                            let s = rot_y.sin();
                            let x = v.x - 0.5;
                            let z = v.z - 0.5;
                            v.x = 0.5 + (x * c - z * s);
                            v.z = 0.5 + (z * c + x * s);
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
                            let rot_y =
                                (-face.rotation as f64 * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_y.cos() as i16;
                            let s = rot_y.sin() as i16;
                            let x = v.toffsetx - 8 * tw;
                            let y = v.toffsety - 8 * th;
                            v.toffsetx = 8 * tw + (x * c - y * s);
                            v.toffsety = 8 * th + (y * c + x * s);
                        }

                        if raw.uvlock
                            && raw.y > 0.0
                            && (processed_face.facing == Direction::Up
                                || processed_face.facing == Direction::Down)
                        {
                            let rot_y = (raw.y * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_y.cos() as i16;
                            let s = rot_y.sin() as i16;
                            let x = v.toffsetx - 8 * tw;
                            let y = v.toffsety - 8 * th;
                            v.toffsetx = 8 * tw + (x * c - y * s);
                            v.toffsety = 8 * th + (y * c + x * s);
                        }

                        if raw.uvlock
                            && raw.x > 0.0
                            && (processed_face.facing != Direction::Up
                                && processed_face.facing != Direction::Down)
                        {
                            let rot_x = (raw.x * (::std::f64::consts::PI / 180.0)) as f32;
                            let c = rot_x.cos() as i16;
                            let s = rot_x.sin() as i16;
                            let x = v.toffsetx - 8 * tw;
                            let y = v.toffsety - 8 * th;
                            v.toffsetx = 8 * tw + (x * c - y * s);
                            v.toffsety = 8 * th + (y * c + x * s);
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

const FACE_ROTATION: &[Direction] = &[
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

const FACE_ROTATION_X: &[Direction] = &[
    Direction::North,
    Direction::Down,
    Direction::South,
    Direction::Up,
];

fn rotate_direction(
    val: Direction,
    offset: i32,
    rots: &[Direction],
    invalid: &[Direction],
) -> Direction {
    for d in invalid {
        if *d == val {
            return val;
        }
    }
    let pos = rots.iter().position(|v| *v == val).unwrap_or(0) as i32;
    rots[(rots.len() as i32 + pos + offset) as usize % rots.len()]
}

#[derive(Clone)]
pub struct StateModel {
    variants: HashMap<String, Variants, BuildHasherDefault<FNVHash>>,
    multipart: Vec<MultipartRule>,
}

impl StateModel {
    pub fn get_variants(&self, name: &str) -> Option<&Variants> {
        self.variants.get(name)
    }
}

#[derive(Clone)]
struct MultipartRule {
    apply: Variants,
    rules: Vec<Rule>,
}

#[derive(Clone)]
enum Rule {
    Match(String, String),
    Or(Vec<Vec<Rule>>),
}

#[derive(Clone)]
pub struct Variants {
    models: Vec<Model>,
}

impl Variants {
    fn choose_model<R: Rng>(&self, rng: &mut R) -> &Model {
        // TODO: Weighted random
        self.models.choose(rng).unwrap()
    }
}

#[derive(Debug)]
enum BuiltinType {
    False,
    Generated,
    Entity,
    Compass,
    Clock,
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
            let tex = self
                .texture_vars
                .get(&name[1..])
                .cloned()
                .unwrap_or_else(|| "".to_owned());
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

#[derive(Clone, Debug)]
struct Model {
    faces: Vec<Face>,
    ambient_occlusion: bool,
    weight: f64,
}

#[derive(Clone, Debug)]
struct Face {
    cull_face: Direction,
    facing: Direction,
    vertices: Vec<BlockVertex>,
    vertices_texture: Vec<render::Texture>,
    indices: usize,
    shade: bool,
    tint_index: i32,
}

impl Model {
    fn join(&mut self, other: &Model) {
        self.faces.extend_from_slice(&other.faces);
    }

    fn render<W: Write>(
        &self,
        factory: &Factory,
        snapshot: &world::Snapshot,
        x: i32,
        y: i32,
        z: i32,
        buf: &mut W,
    ) -> usize {
        let this = snapshot.get_block(x, y, z);
        let this_mat = this.get_material();
        let mut indices = 0;

        let tint = this.get_tint();

        for face in &self.faces {
            if face.cull_face != Direction::Invalid && !this_mat.never_cull {
                let (ox, oy, oz) = face.cull_face.get_offset();
                let other = snapshot.get_block(x + ox, y + oy, z + oz);
                if other.get_material().should_cull_against || other == this {
                    continue;
                }
            }
            indices += face.indices;

            for vert in &face.vertices {
                let mut vert = vert.clone();

                vert.x += x as f32;
                vert.y += y as f32;
                vert.z += z as f32;

                let (mut cr, mut cg, mut cb) = if face.tint_index == 0 {
                    match tint {
                        TintType::Default => (255, 255, 255),
                        TintType::Color { r, g, b } => (r, g, b),
                        TintType::Grass => calculate_biome(
                            snapshot,
                            vert.x as i32,
                            vert.z as i32,
                            &factory.grass_colors,
                        ),
                        TintType::Foliage => calculate_biome(
                            snapshot,
                            vert.x as i32,
                            vert.z as i32,
                            &factory.foliage_colors,
                        ),
                    }
                } else {
                    (255, 255, 255)
                };
                if face.facing == Direction::West || face.facing == Direction::East {
                    cr = ((cr as f64) * 0.8) as u8;
                    cg = ((cg as f64) * 0.8) as u8;
                    cb = ((cb as f64) * 0.8) as u8;
                }

                vert.r = cr;
                vert.g = cg;
                vert.b = cb;

                let (bl, sl) = calculate_light(
                    snapshot,
                    x,
                    y,
                    z,
                    vert.x as f64,
                    vert.y as f64,
                    vert.z as f64,
                    face.facing,
                    self.ambient_occlusion,
                    this_mat.force_shade,
                );
                vert.block_light = bl;
                vert.sky_light = sl;
                vert.write(buf);
            }
        }
        indices
    }
}

fn calculate_biome(
    snapshot: &world::Snapshot,
    x: i32,
    z: i32,
    img: &image::DynamicImage,
) -> (u8, u8, u8) {
    use std::cmp::{max, min};
    let mut count = 0;
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    for xx in -1..2 {
        for zz in -1..2 {
            let bi = snapshot.get_biome(x + xx, z + zz);
            let color_index = bi.get_color_index();
            let ix = color_index & 0xFF;
            let iy = color_index >> 8;

            let ix = min(max(ix, 0), 255);
            let iy = min(max(iy, 0), 255);

            let col = img.get_pixel(ix as u32, iy as u32);
            let col = bi.process_color(col);
            r += col.0[0] as u32;
            g += col.0[1] as u32;
            b += col.0[2] as u32;
            count += 1;
        }
    }
    ((r / count) as u8, (g / count) as u8, (b / count) as u8)
}

fn calculate_light(
    snapshot: &world::Snapshot,
    orig_x: i32,
    orig_y: i32,
    orig_z: i32,
    x: f64,
    y: f64,
    z: f64,
    face: Direction,
    smooth: bool,
    force: bool,
) -> (u16, u16) {
    use crate::world::block;
    use std::cmp::max;
    let (ox, oy, oz) = face.get_offset();

    let s_block_light = snapshot.get_block_light(orig_x + ox, orig_y + oy, orig_z + oz);
    let s_sky_light = snapshot.get_sky_light(orig_x + ox, orig_y + oy, orig_z + oz);
    if !smooth {
        return ((s_block_light as u16) * 4000, (s_sky_light as u16) * 4000);
    }

    let mut block_light = 0u32;
    let mut sky_light = 0u32;
    let mut count = 0;

    let s_block_light = max((s_block_light as i8) - 8, 0) as u8;
    let s_sky_light = max((s_sky_light as i8) - 8, 0) as u8;

    let dx = (ox as f64) * 0.6;
    let dy = (oy as f64) * 0.6;
    let dz = (oz as f64) * 0.6;

    for ox in [-0.6, 0.0].iter() {
        for oy in [-0.6, 0.0].iter() {
            for oz in [-0.6, 0.0].iter() {
                let lx = (x + ox + dx).round() as i32;
                let ly = (y + oy + dy).round() as i32;
                let lz = (z + oz + dz).round() as i32;
                let mut bl = snapshot.get_block_light(lx, ly, lz);
                let mut sl = snapshot.get_sky_light(lx, ly, lz);
                if (force
                    && match snapshot.get_block(lx, ly, lz) {
                        block::Air {} => false,
                        _ => true,
                    })
                    || (sl == 0 && bl == 0)
                {
                    bl = s_block_light;
                    sl = s_sky_light;
                }
                block_light += bl as u32;
                sky_light += sl as u32;
                count += 1;
            }
        }
    }

    (
        (((block_light * 4000) / count) as u16),
        (((sky_light * 4000) / count) as u16),
    )
}

pub const PRECOMPUTED_VERTS: [&[BlockVertex; 4]; 6] = [
    &[
        // Up
        BlockVertex::base(0.0, 1.0, 0.0, 0, 0),
        BlockVertex::base(1.0, 1.0, 0.0, 1, 0),
        BlockVertex::base(0.0, 1.0, 1.0, 0, 1),
        BlockVertex::base(1.0, 1.0, 1.0, 1, 1),
    ],
    &[
        // Down
        BlockVertex::base(0.0, 0.0, 0.0, 0, 1),
        BlockVertex::base(0.0, 0.0, 1.0, 0, 0),
        BlockVertex::base(1.0, 0.0, 0.0, 1, 1),
        BlockVertex::base(1.0, 0.0, 1.0, 1, 0),
    ],
    &[
        // North
        BlockVertex::base(0.0, 0.0, 0.0, 1, 1),
        BlockVertex::base(1.0, 0.0, 0.0, 0, 1),
        BlockVertex::base(0.0, 1.0, 0.0, 1, 0),
        BlockVertex::base(1.0, 1.0, 0.0, 0, 0),
    ],
    &[
        // South
        BlockVertex::base(0.0, 0.0, 1.0, 0, 1),
        BlockVertex::base(0.0, 1.0, 1.0, 0, 0),
        BlockVertex::base(1.0, 0.0, 1.0, 1, 1),
        BlockVertex::base(1.0, 1.0, 1.0, 1, 0),
    ],
    &[
        // West
        BlockVertex::base(0.0, 0.0, 0.0, 0, 1),
        BlockVertex::base(0.0, 1.0, 0.0, 0, 0),
        BlockVertex::base(0.0, 0.0, 1.0, 1, 1),
        BlockVertex::base(0.0, 1.0, 1.0, 1, 0),
    ],
    &[
        // East
        BlockVertex::base(1.0, 0.0, 0.0, 1, 1),
        BlockVertex::base(1.0, 0.0, 1.0, 0, 1),
        BlockVertex::base(1.0, 1.0, 0.0, 1, 0),
        BlockVertex::base(1.0, 1.0, 1.0, 0, 0),
    ],
];

#[derive(Clone, Debug)]
pub struct BlockVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub tx: u16,
    pub ty: u16,
    pub tw: u16,
    pub th: u16,
    pub toffsetx: i16,
    pub toffsety: i16,
    pub tatlas: i16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub block_light: u16,
    pub sky_light: u16,
}

impl BlockVertex {
    const fn base(x: f32, y: f32, z: f32, tx: i16, ty: i16) -> BlockVertex {
        BlockVertex {
            x,
            y,
            z,
            tx: 0,
            ty: 0,
            tw: 0,
            th: 0,
            toffsetx: tx,
            toffsety: ty,
            tatlas: 0,
            r: 0,
            g: 0,
            b: 0,
            block_light: 0,
            sky_light: 0,
        }
    }
    pub fn write<W: Write>(&self, w: &mut W) {
        let _ = w.write_f32::<NativeEndian>(self.x);
        let _ = w.write_f32::<NativeEndian>(self.y);
        let _ = w.write_f32::<NativeEndian>(self.z);
        let _ = w.write_u16::<NativeEndian>(self.tx);
        let _ = w.write_u16::<NativeEndian>(self.ty);
        let _ = w.write_u16::<NativeEndian>(self.tw);
        let _ = w.write_u16::<NativeEndian>(self.th);
        let _ = w.write_i16::<NativeEndian>(self.toffsetx);
        let _ = w.write_i16::<NativeEndian>(self.toffsety);
        let _ = w.write_i16::<NativeEndian>(self.tatlas);
        let _ = w.write_i16::<NativeEndian>(0);
        let _ = w.write_u8(self.r);
        let _ = w.write_u8(self.g);
        let _ = w.write_u8(self.b);
        let _ = w.write_u8(255);
        let _ = w.write_u16::<NativeEndian>(self.block_light);
        let _ = w.write_u16::<NativeEndian>(self.sky_light);
        let _ = w.write_u16::<NativeEndian>(0);
        let _ = w.write_u16::<NativeEndian>(0);
    }

    pub fn face_by_direction(dir: Direction) -> &'static [BlockVertex; 4] {
        match dir {
            Direction::Up => PRECOMPUTED_VERTS[0],
            Direction::Down => PRECOMPUTED_VERTS[1],
            Direction::North => PRECOMPUTED_VERTS[2],
            Direction::South => PRECOMPUTED_VERTS[3],
            Direction::West => PRECOMPUTED_VERTS[4],
            Direction::East => PRECOMPUTED_VERTS[5],
            _ => unreachable!(),
        }
    }
}
