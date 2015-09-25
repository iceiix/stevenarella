// Copyright 2015 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod atlas;
pub mod glsl;
pub mod ui;
mod shaders;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use resources;
use gl;
use image;
use image::{GenericImage};
use byteorder::{WriteBytesExt, NativeEndian};
use serde_json;

const ATLAS_SIZE: usize = 1024;

pub struct Renderer {
	resource_version: usize,
	pub resources: Arc<RwLock<resources::Manager>>,
	textures: Arc<RwLock<TextureManager>>,
	glsl: glsl::Registry,
	pub ui: ui::UIState,

	gl_texture: gl::Texture,
	texture_layers: usize,

	last_width: u32,
	last_height: u32,
}

impl Renderer {
	pub fn new(res: Arc<RwLock<resources::Manager>>) -> Renderer {
		let version = { res.read().unwrap().version() };
		let tex = gl::Texture::new();
		tex.bind(gl::TEXTURE_2D_ARRAY);
		tex.image_3d(gl::TEXTURE_2D_ARRAY, 0, ATLAS_SIZE as u32, ATLAS_SIZE as u32, 1, gl::RGBA, gl::UNSIGNED_BYTE, &[0; ATLAS_SIZE*ATLAS_SIZE*1*4]);
		tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
		tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
		tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
		tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);

		let textures = Arc::new(RwLock::new(TextureManager::new(res.clone())));

		let mut greg = glsl::Registry::new();
		shaders::add_shaders(&mut greg);
		let ui = ui::UIState::new(&greg, textures.clone(), res.clone());

		Renderer {
			resource_version: version,
			textures: textures,
			glsl: greg,
			ui: ui,
			resources: res,
			gl_texture: tex,
			texture_layers: 1,
			last_width: 0,
			last_height: 0,
		}
	}

	pub fn tick(&mut self, delta: f64, width: u32, height: u32) {
		{
			let rm = self.resources.read().unwrap();
			if rm.version() != self.resource_version {
				self.resource_version = rm.version();
				println!("Updating textures to {}", self.resource_version);
				self.textures.write().unwrap().update_textures(self.resource_version);
			}
		}

		self.update_textures(delta);

		if self.last_height != height || self.last_width != width {
			self.last_width = width;
			self.last_height = height;
			gl::viewport(0, 0, width as i32, height as i32);
		}

		gl::active_texture(0);
		self.gl_texture.bind(gl::TEXTURE_2D_ARRAY);

        gl::clear_color(14.0/255.0, 48.0/255.0, 92.0/255.0, 1.0);
        gl::clear(gl::ClearFlags::Color | gl::ClearFlags::Depth);

        self.ui.tick(width, height);
	}

	fn update_textures(&mut self, delta: f64) {
		self.gl_texture.bind(gl::TEXTURE_2D_ARRAY);
		let len = {
			let tex = self.textures.read().unwrap();
			if self.texture_layers != tex.atlases.len() {
				let len = ATLAS_SIZE*ATLAS_SIZE*4*tex.atlases.len();
				let mut data = Vec::with_capacity(len);
				unsafe { data.set_len(len); }
				self.gl_texture.get_pixels(gl::TEXTURE_2D_ARRAY, 0, gl::RGBA, gl::UNSIGNED_BYTE, &mut data[..]);
				self.gl_texture.image_3d(gl::TEXTURE_2D_ARRAY, 0, ATLAS_SIZE as u32, ATLAS_SIZE as u32, tex.atlases.len() as u32, gl::RGBA, gl::UNSIGNED_BYTE, &data[..]);
				self.texture_layers = tex.atlases.len();
			}
			tex.pending_uploads.len()
		};
		if len > 0 {
			let mut tex = self.textures.write().unwrap();
			for upload in &tex.pending_uploads {
				let atlas = upload.0;
				let rect = upload.1;
				let img = &upload.2;
				self.gl_texture.sub_image_3d(gl::TEXTURE_2D_ARRAY, 0, rect.x as u32, rect.y as u32, atlas as u32, rect.width as u32, rect.height as u32, 1, gl::RGBA, gl::UNSIGNED_BYTE,  &img[..]);
			}	
			tex.pending_uploads.clear();		
		}

		for ani in self.textures.write().unwrap().animated_textures.iter_mut() {
			if ani.remaining_time <= 0.0 {
				ani.current_frame = (ani.current_frame + 1) % ani.frames.len();
				ani.remaining_time += ani.frames[ani.current_frame].time as f64;
				let offset = ani.texture.width * ani.texture.width * ani.frames[ani.current_frame].index * 4;
				let offset2 = offset + ani.texture.width * ani.texture.width * 4;
				self.gl_texture.sub_image_3d(gl::TEXTURE_2D_ARRAY, 
					0,
					ani.texture.get_x() as u32, ani.texture.get_y() as u32, ani.texture.atlas as u32, 
					ani.texture.get_width() as u32, ani.texture.get_height() as u32, 1, 
					gl::RGBA, gl::UNSIGNED_BYTE, 
					&ani.data[offset .. offset2]
				);
			} else {
				ani.remaining_time -= delta / 3.0;
			}
		}

	}

	pub fn get_textures(&self) -> Arc<RwLock<TextureManager>> {
		self.textures.clone()
	}

	pub fn get_textures_ref(&self) -> &RwLock<TextureManager> {
		&self.textures
	}

	pub fn check_texture(&self, tex: Texture) -> Texture {
		if tex.version == self.resource_version {
			tex
		} else {
			let mut new = Renderer::get_texture(&self.textures, &tex.name);
			new.rel_x = tex.rel_x;
			new.rel_y = tex.rel_y;
			new.rel_width = tex.rel_width;
			new.rel_height = tex.rel_height;
			new.is_rel = tex.is_rel;
			new
		}
	}

	pub fn get_texture(textures: &RwLock<TextureManager>, name: &str) -> Texture {		
		let tex = { textures.read().unwrap().get_texture(name) };
		match tex {
			Some(val) => val,
			None => {
				let mut t = textures.write().unwrap();
				// Make sure it hasn't already been loaded since we switched 
				// locks.
				if let Some(val) = t.get_texture(name) {
					val
				} else {
					t.load_texture(name);
					t.get_texture(name).unwrap()
				}
			}
		}
	}
}

pub struct TextureManager {
	textures: HashMap<String, Texture>,
	version: usize,
	resources: Arc<RwLock<resources::Manager>>,
	atlases: Vec<atlas::Atlas>,

	animated_textures: Vec<AnimatedTexture>,
	pending_uploads: Vec<(i32, atlas::Rect, Vec<u8>)>,

	dynamic_textures: HashMap<String, (i32, atlas::Rect)>,
	free_dynamics: Vec<(i32, atlas::Rect)>,
}

impl TextureManager {
	fn new(res: Arc<RwLock<resources::Manager>>) -> TextureManager {
		let mut tm = TextureManager {
			textures: HashMap::new(),
			version: 0xFFFF,
			resources: res,
			atlases: Vec::new(),
			animated_textures: Vec::new(),
			pending_uploads: Vec::new(),

			dynamic_textures: HashMap::new(),
			free_dynamics: Vec::new(),
		};
		tm.add_defaults();
		tm
	}

	fn add_defaults(&mut self) {
		self.put_texture("steven", "missing_texture", 2, 2, vec![
			0, 0, 0, 255,
			255, 0, 255, 255,
			255, 0, 255, 255,
			0, 0, 0, 255,
		]);
		self.put_texture("steven", "solid", 1, 1, vec![
			255, 255, 255, 255,
		]);
	}

	fn update_textures(&mut self, version: usize) {
		self.dynamic_textures.clear();
		self.free_dynamics.clear();
		self.pending_uploads.clear();
		self.atlases.clear();
		self.animated_textures.clear();
		self.version = version;
		let map = self.textures.clone();
		self.textures.clear();

		self.add_defaults();

		for name in map.keys() {
			self.load_texture(name);
		}
	}

	fn get_texture(&self, name: &str) -> Option<Texture> {
		self.textures.get(name).map(|v| v.clone())
	}

	fn load_texture(&mut self, name: &str) {
		let (plugin, name) = if let Some(pos) = name.find(':') {
			(&name[..pos], &name[pos+1..])
		} else {
			("minecraft", name)
		};
		let path = format!("textures/{}.png", name);
		let res = self.resources.clone();
		if let Some(mut val) = res.read().unwrap().open(plugin, &path) {
			let mut data = Vec::new();
			val.read_to_end(&mut data).unwrap();
			if let Ok(img) = image::load_from_memory(&data) {
				let (width, height) = img.dimensions();
				// Might be animated
				if (name.starts_with("blocks/") || name.starts_with("items/")) && width != height {
					let id = img.to_rgba().into_vec();
					let frame = id[
						.. (width*width*4) as usize
					].to_owned();
					if let Some(mut ani) = self.load_animation(plugin, name, &img, id) {						
						ani.texture = self.put_texture(plugin, name, width, width, frame);
						self.animated_textures.push(ani);
						return;
					}
				}
				self.put_texture(plugin, name, width, height, img.to_rgba().into_vec());
				return;
			}
		}
		self.insert_texture_dummy(plugin, name);
	}

	fn load_animation(&mut self, plugin: &str, name: &str, img: &image::DynamicImage, data: Vec<u8>) -> Option<AnimatedTexture> {
		let path = format!("textures/{}.png.mcmeta", name);
		let res = self.resources.clone();
		if let Some(val) = res.read().unwrap().open(plugin, &path) {
			let meta: serde_json::Value = serde_json::from_reader(val).unwrap();
			let animation = meta.find("animation").unwrap();
			let frame_time = animation.find("frameTime").and_then(|v| v.as_i64()).unwrap_or(1);
			let interpolate = animation.find("interpolate").and_then(|v| v.as_boolean()).unwrap_or(false);
			let frames = if let Some(frames) = animation.find("frames").and_then(|v| v.as_array()) {
				let mut out = Vec::with_capacity(frames.len());
				for frame in frames {
					if let Some(index) = frame.as_i64() {
						out.push(AnimationFrame{
							index: index as usize,
							time: frame_time,
						})
					} else {
						out.push(AnimationFrame{
							index: frame.find("index").unwrap().as_i64().unwrap() as usize,
							time: frame_time * frame.find("frameTime").unwrap().as_i64().unwrap(),
						})
					}
				}
				out
			} else {
				let (width, height) = img.dimensions();
				let count = height / width;
				let mut frames = Vec::with_capacity(count as usize);
				for i in 0 .. count {
					frames.push(AnimationFrame{
						index: i as usize,
						time: frame_time,
					})
				}
				frames
			};

			return Some(AnimatedTexture{
				frames: frames,
				data: data,
				interpolate: interpolate,
				current_frame: 0,
				remaining_time: 0.0,	
				texture: self.get_texture("steven:missing_texture").unwrap(),			
			});
		}
		None
	}

	fn put_texture(&mut self, plugin: &str, name: &str, width: u32, height: u32, data: Vec<u8>) -> Texture {
		let (atlas, rect) = self.find_free(width as usize, height as usize);
		self.pending_uploads.push((atlas, rect, data));

		let mut full_name = String::new();
		if plugin != "minecraft" {
			full_name.push_str(plugin);
			full_name.push_str(":");
		}
		full_name.push_str(name);

		let tex = Texture { 
			name: full_name.clone(), 
			version: self.version, 
			atlas: atlas,
			x: rect.x, 
			y: rect.y, 
			width: rect.width, 
			height: rect.height,
			rel_x: 0.0, 
			rel_y: 0.0,
			rel_width: 1.0,
			rel_height: 1.0,
			is_rel: false,
		};
		self.textures.insert(full_name, tex.clone());
		tex
	}

	fn find_free(&mut self, width: usize, height: usize) -> (i32, atlas::Rect) {
		let mut index = 0;
		for atlas in self.atlases.iter_mut() {
			if let Some(rect) = atlas.add(width, height) {
				return (index, rect);
			}
			index += 1;
		}
		let mut atlas = atlas::Atlas::new(ATLAS_SIZE, ATLAS_SIZE);
		let rect = atlas.add(width, height);
		self.atlases.push(atlas);
		(index, rect.unwrap())
	}

	fn insert_texture_dummy(&mut self, plugin: &str, name: &str) -> Texture {
		let missing = self.get_texture("steven:missing_texture").unwrap();

		let mut full_name = String::new();
		if plugin != "minecraft" {
			full_name.push_str(plugin);
			full_name.push_str(":");
		}
		full_name.push_str(name);

		let t = Texture { 
			name: full_name.to_owned(), 
			version: self.version, 
			atlas: missing.atlas,
			x: missing.x, 
			y: missing.y, 
			width: missing.width, 
			height: missing.height,
			rel_x: 0.0, 
			rel_y: 0.0,
			rel_width: 1.0,
			rel_height: 1.0,
			is_rel: false,
		};
		self.textures.insert(full_name.to_owned(), t.clone());
		t
	}

	pub fn put_dynamic(&mut self, plugin: &str, name: &str, img: image::DynamicImage) -> Texture {
		let (width, height) = img.dimensions();
		let (width, height) = (width as usize, height as usize);
		let mut rect = None;
		let mut rect_pos = 0;
		for (i, r) in self.free_dynamics.iter().enumerate() {
			let (atlas, r) = *r;
			if r.width == width && r.height == height {
				rect_pos = i;
				rect = Some((atlas, r));
				break;
			} else if r.width >= width && r.height >= height {
				rect_pos = i;
				rect = Some((atlas, r));
			}
		}
		let data = img.to_rgba().into_vec();
		let mut new = false;
		let (atlas, rect) = if let Some(r) = rect {
			self.free_dynamics.remove(rect_pos);
			r
		} else {
			new = true;
			self.find_free(width as usize, height as usize)			
		};

		let mut full_name = String::new();
		if plugin != "minecraft" {
			full_name.push_str(plugin);
			full_name.push_str(":");
		}
		full_name.push_str(name);

		self.dynamic_textures.insert(full_name.clone(), (atlas, rect));
		if new {
			self.put_texture(plugin, name, width as u32, height as u32, data)
		} else {
			let t = Texture { 
				name: full_name.clone(), 
				version: self.version, 
				atlas: atlas,
				x: rect.x, 
				y: rect.y, 
				width: rect.width, 
				height: rect.height,
				rel_x: 0.0, 
				rel_y: 0.0,
				rel_width: 1.0,
				rel_height: 1.0,
				is_rel: false,
			};
			self.textures.insert(full_name.to_owned(), t.clone());
			t
		}
	}

	pub fn remove_dynamic(&mut self, plugin: &str, name: &str) {
		let mut full_name = String::new();
		if plugin != "minecraft" {
			full_name.push_str(plugin);
			full_name.push_str(":");
		}
		full_name.push_str(name);

		let desc = self.dynamic_textures.remove(&full_name).unwrap();
		self.free_dynamics.push(desc);
	}
}

struct AnimatedTexture {
	frames: Vec<AnimationFrame>,
	data: Vec<u8>,
	interpolate: bool,
	current_frame: usize,
	remaining_time: f64,
	texture: Texture,
}

struct AnimationFrame {
	index: usize,
	time: i64,
}

#[derive(Clone, Debug)]
pub struct Texture {
	name: String,
	version: usize,
	pub atlas: i32,
	x: usize,
	y: usize,
	width: usize,
	height: usize,
	is_rel: bool, // Save some cycles for none relative textures
	rel_x: f32,
	rel_y: f32,
	rel_width: f32,
	rel_height: f32,
}

impl Texture {
	pub fn get_x(&self) -> usize {
		if self.is_rel {
			self.x + ((self.width as f32) * self.rel_x) as usize
		} else {
			self.x
		}
	}

	pub fn get_y(&self) -> usize {
		if self.is_rel {
			self.y + ((self.height as f32) * self.rel_y) as usize
		} else {
			self.y
		}
	}

	pub fn get_width(&self) -> usize {
		if self.is_rel {
			((self.width as f32) * self.rel_width) as usize
		} else {
			self.width
		}
	}

	pub fn get_height(&self) -> usize {
		if self.is_rel {
			((self.height as f32) * self.rel_height) as usize
		} else {
			self.height
		}
	}

	pub fn relative(&self, x: f32, y: f32, width: f32, height: f32) -> Texture {
		Texture {
			name: self.name.clone(),
			version: self.version,
			x: self.x,
			y: self.y,
			atlas: self.atlas,
			width: self.width,
			height: self.height,
			is_rel: true,
			rel_x: self.rel_x + x * self.rel_width,
			rel_y: self.rel_y + y * self.rel_height,
			rel_width: width * self.rel_width,
			rel_height: height * self.rel_height,
		}
	}
}

pub fn create_program(vertex: &str, fragment: &str) -> gl::Program {
	let program = gl::Program::new();

	let v = gl::Shader::new(gl::VERTEX_SHADER);
	v.set_source(vertex);
	v.compile();

	if v.get_parameter(gl::COMPILE_STATUS) == 0 {
		println!("Src: {}", vertex);
		panic!("Shader error: {}", v.get_info_log());
	} else {
		let log = v.get_info_log();
		if log.len() > 0 {
			println!("{}", log);
		}
	}

	let f = gl::Shader::new(gl::FRAGMENT_SHADER);
	f.set_source(fragment);
	f.compile();

	if f.get_parameter(gl::COMPILE_STATUS) == 0 {
		println!("Src: {}", fragment);
		panic!("Shader error: {}", f.get_info_log());
	} else {
		let log = f.get_info_log();
		if log.len() > 0 {
			println!("{}", log);
		}
	}

	program.attach_shader(v);
	program.attach_shader(f);
	program.link();
	program.use_program();
	program
}

#[allow(unused_must_use)]
pub fn generate_element_buffer(size: usize) -> (Vec<u8>, gl::Type) {
	let mut ty = gl::UNSIGNED_SHORT;
	let mut data = if (size/6)*4*3 >= u16::max_value() as usize {
		ty = gl::UNSIGNED_INT;
		let data = Vec::with_capacity(size*4);
		data
	} else {
		let data = Vec::with_capacity(size*2);
		data
	};
	for i in 0 .. size/6 {
		for val in &[0, 1, 2, 2, 1, 3] {
			if ty == gl::UNSIGNED_INT {
				data.write_u32::<NativeEndian>((i as u32) * 4 + val);
			} else {
				data.write_u16::<NativeEndian>((i as u16) * 4 + (*val as u16));
			}
		}
	}

	(data, ty)
}