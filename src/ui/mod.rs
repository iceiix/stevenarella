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

use std::collections::HashMap;
use std::marker::PhantomData;
use rand;
use render;

const SCALED_WIDTH: f64 = 854.0;
const SCALED_HEIGHT: f64 = 480.0;

pub enum Element {
	Image(Image),
	None,
}

impl Element {
	fn should_draw(&self) -> bool {
		match self {
			&Element::Image(ref img) => img.should_draw,
			_ => unimplemented!(),
		}
	}

	fn get_parent(&self) -> Option<ElementRefInner> {
		match self {
			&Element::Image(ref img) => img.parent,
			_ => unimplemented!(),
		}
	}

	fn get_attachment(&self) -> (VAttach, HAttach) {
		match self {
			&Element::Image(ref img) => (img.v_attach, img.h_attach),
			_ => unimplemented!(),
		}		
	}

	fn get_offset(&self) -> (f64, f64) {
		match self {
			&Element::Image(ref img) => (img.x, img.y),
			_ => unimplemented!(),
		}		
	}

	fn get_size(&self) -> (f64, f64) {
		match self {
			&Element::Image(ref img) => (img.width, img.height),
			_ => unimplemented!(),
		}		
	}

	fn is_dirty(&self) -> bool {
		match self {
			&Element::Image(ref img) => img.dirty,
			_ => unimplemented!(),
		}				
	}

	fn set_dirty(&mut self, val: bool) {
		match self {
			&mut Element::Image(ref mut img) => img.dirty = val,
			_ => unimplemented!(),
		}				
	}

	fn draw(&mut self, renderer: &mut render::Renderer, r: &Region, width: f64, height: f64, delta: f64) {
		match self {
			&mut Element::Image(ref mut img) => img.draw(renderer, r, width, height, delta),
			_ => unimplemented!(),
		}		
	}
}

pub enum Mode {
	Scaled,
	Unscaled(f64)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VAttach {
	Top,
	Middle,
	Bottom,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HAttach {
	Left,
	Center,
	Right,
}

#[derive(Clone)]
struct Region {
	x: f64,
	y: f64,
	w: f64,
	h: f64,
}

impl Region {
	fn intersects(&self, o: &Region) -> bool {
		!(self.x+self.w < o.x ||
			self.x > o.x+o.w ||
			self.y+self.h < o.y ||
			self.y > o.y+o.h)
	}
}

/// Reference to an element currently attached to a
/// container.
#[derive(Clone, Copy)]
pub struct ElementRef<T> {
	inner: ElementRefInner,
	ty: PhantomData<T>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct ElementRefInner {
	index: usize,
}

const SCREEN: Region = Region{x: 0.0, y: 0.0, w: SCALED_WIDTH, h: SCALED_HEIGHT};

pub struct Container {
	pub mode: Mode,
	elements: HashMap<ElementRefInner, Element>,
}

impl Container {
	pub fn new() -> Container {
		Container {
			mode: Mode::Scaled,
			elements: HashMap::new(),
		}
	}

	pub fn add<T: UIElement>(&mut self, e: T) -> ElementRef<T> {
		let mut r = ElementRefInner{index: rand::random()};
		while self.elements.contains_key(&r) {
			r = ElementRefInner{index: rand::random()};
		}
		self.elements.insert(r, e.wrap());
		ElementRef{inner: r, ty: PhantomData}
	}

	pub fn get<T: UIElement>(&self, r: &ElementRef<T>) -> &T {
		T::unwrap_ref(self.elements.get(&r.inner).unwrap())
	}

	pub fn get_mut<T: UIElement>(&mut self, r: &ElementRef<T>) -> &mut T {
		T::unwrap_ref_mut(self.elements.get_mut(&r.inner).unwrap())
	}

	pub fn remove<T: UIElement>(&mut self, r: &ElementRef<T>) {
		self.elements.remove(&r.inner);
	}

	pub fn tick(&mut self, renderer: &mut render::Renderer, delta: f64, width: f64, height: f64) {
		let (sw, sh) = match self.mode {
			Mode::Scaled => (SCALED_WIDTH / width, SCALED_HEIGHT / height),
			Mode::Unscaled(scale) => (scale, scale),
		};

		// Borrow rules seem to prevent us from doing this in the first pass
		// so we split it.
		let regions = self.collect_elements(sw, sh);
		for (re, e) in &mut self.elements {
			if !e.should_draw() {
				continue;
			}
			if let Some(&(ref r, ref dirty)) = regions.get(re) {
				e.set_dirty(*dirty);
				e.draw(renderer, r, width, height, delta);
			}
		}
	}

	fn collect_elements(&self, sw: f64, sh: f64) -> HashMap<ElementRefInner, (Region, bool)> {
		let mut map = HashMap::new();
		for (re, e) in &self.elements {
			if !e.should_draw() {
				continue;
			}
			let r = self.get_draw_region(e, sw, sh);
			if r.intersects(&SCREEN) {
				// Mark this as dirty if any of its 
				// parents are dirty too.
				let mut dirty = e.is_dirty();
				let mut parent = e.get_parent();
				while !dirty && parent.is_some() {
					let p = self.elements.get(&parent.unwrap()).unwrap();
					dirty = p.is_dirty();
					parent = p.get_parent();
				}
				map.insert(*re, (r, dirty));
			}
		}		
		map
	}

	fn get_draw_region(&self, e: &Element, sw: f64, sh: f64) -> Region {		
		let super_region = match e.get_parent() {
			Some(ref p) => self.get_draw_region(self.elements.get(p).unwrap(), sw, sh),
			None => SCREEN,
		};
		let mut r = Region{x:0.0,y:0.0,w:0.0,h:0.0};
		let (w, h) = e.get_size();
		let (ox, oy) = e.get_offset();
		r.w = w * sw;
		r.h = h * sh;
		let (v_attach, h_attach) = e.get_attachment();
		match h_attach {
			HAttach::Left => r.x = ox * sw,
			HAttach::Center => r.x = (super_region.w / 2.0) - (r.w / 2.0) + ox * sw,
			HAttach::Right => r.x = super_region.w - ox * sw - r.w,
		}
		match v_attach {
			VAttach::Top => r.y = oy * sh,
			VAttach::Middle => r.y = (super_region.h / 2.0) - (r.h / 2.0) + oy * sh,
			VAttach::Bottom => r.y = super_region.h - oy * sh - r.h,
		}
		r.x += super_region.x;
		r.y += super_region.y;
		r
	}
}

pub trait UIElement {
	fn wrap(self) -> Element;
	fn unwrap_ref<'a>(&'a Element) -> &'a Self;
	fn unwrap_ref_mut<'a>(&'a mut Element) -> &'a mut Self;
}

macro_rules! lazy_field {
	($name:ident, $t:ty, $get:ident, $set:ident) => (
		pub fn $get(&self) -> $t {
			self.$name
		} 

		pub fn $set(&mut self, val: $t) {
			if self.$name != val {
				self.$name = val;
				self.dirty = true;	
			}
		}
	)
}

pub struct Image {
	dirty: bool,
	data: Vec<u8>,

	parent: Option<ElementRefInner>,
	should_draw: bool,
	texture: render::Texture,
	layer: isize,
	x: f64,
	y: f64,
	width: f64,
	height: f64,
	v_attach: VAttach,
	h_attach: HAttach,

	t_x: f64,
	t_y: f64,
	t_width: f64,
	t_height: f64,

	r: u8,
	g: u8,
	b: u8,
	a: u8,
}

impl Image {
	pub fn new(texture: render::Texture, x: f64, y: f64, w: f64, h: f64, t_x: f64, t_y: f64, t_width: f64, t_height: f64, r: u8, g: u8, b: u8) -> Image {
		Image {
			dirty: true,
			data: Vec::new(),

			parent: None,
			should_draw: true,
			texture: texture,
			layer: 0,
			x: x,
			y: y,
			width: w,
			height: h,
			v_attach: VAttach::Top,
			h_attach: HAttach::Left,

			t_x: t_x,
			t_y: t_y,
			t_width: t_width,
			t_height: t_height,

			r: r,
			g: g,
			b: b,
			a: 255,
		}
	}

	fn draw(&mut self, renderer: &mut render::Renderer, r: &Region, width: f64, height: f64, delta: f64) {
		if self.dirty {
			self.dirty = false;
			self.texture = renderer.check_texture(self.texture.clone());
			let mut e = render::ui::UIElement::new(&self.texture, r.x, r.y, r.w, r.h, self.t_x, self.t_y, self.t_width, self.t_height);
			e.r = self.r;
			e.g = self.g;
			e.b = self.b;
			e.a = self.a;
			e.layer = self.layer;
			self.data = e.bytes(width, height);
		}
		renderer.ui.add_bytes(&self.data);
	}

	pub fn set_parent<T: UIElement>(&mut self, other: ElementRef<T>) {
		self.parent = Some(other.inner);
		self.dirty = true;
	}

	pub fn get_texture(&self) -> render::Texture {
		self.texture.clone()
	}

	pub fn set_texture(&mut self, val: render::Texture) {
		self.texture = val;
		self.dirty = true;
	}

	lazy_field!(layer, isize, get_layer, set_layer);
	lazy_field!(x, f64, get_x, set_x);
	lazy_field!(y, f64, get_y, set_y);
	lazy_field!(width, f64, get_width, set_width);
	lazy_field!(height, f64, get_height, set_height);
	lazy_field!(v_attach, VAttach, get_v_attach, set_v_attach);
	lazy_field!(h_attach, HAttach, get_h_attach, set_h_attach);

	lazy_field!(t_x, f64, get_t_x, set_t_x);
	lazy_field!(t_y, f64, get_t_y, set_t_y);
	lazy_field!(t_width, f64, get_t_width, set_t_width);
	lazy_field!(t_height, f64, get_t_height, set_t_height);

	lazy_field!(r, u8, get_r, set_r);
	lazy_field!(g, u8, get_g, set_g);
	lazy_field!(b, u8, get_b, set_b);
	lazy_field!(a, u8, get_a, set_a);
}

impl UIElement for Image {
	fn wrap(self) -> Element {
		Element::Image(self)
	}

	fn unwrap_ref<'a>(e: &'a Element) -> &'a Image {
		match e {
			&Element::Image(ref val) => val,
			_ => panic!("Incorrect type"),
		}
	}

	fn unwrap_ref_mut<'a>(e: &'a mut Element) -> &'a mut Image {
		match e {
			&mut Element::Image(ref mut val) => val,
			_ => panic!("Incorrect type"),
		}
	}
}
