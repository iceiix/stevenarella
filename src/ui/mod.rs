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

pub mod logo;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use rand;
use render;
use format;

const SCALED_WIDTH: f64 = 854.0;
const SCALED_HEIGHT: f64 = 480.0;

pub enum Element {
	Image(Image),
	Batch(Batch),
	Text(Text),
	Formatted(Formatted),
	None,
}

macro_rules! element_impl {
	($($name:ident),+) => (
impl Element {
	fn get_click_funcs(&self) -> Vec<Rc<Fn(&mut render::Renderer, &mut Container)>> {
		match self {
			$(
			&Element::$name(ref val) => val.click_funcs.clone(),
			)+
			_ => unimplemented!(),
		}		
	}

	fn get_hover_funcs(&self) -> Vec<Rc<Fn(bool, &mut render::Renderer, &mut Container)>> {
		match self {
			$(
			&Element::$name(ref val) => val.hover_funcs.clone(),
			)+
			_ => unimplemented!(),
		}		
	}

	fn should_call_hover(&mut self, new: bool) -> bool{
		match self {
			$(
			&mut Element::$name(ref mut val) => {
				let ret = val.hovered != new;
				val.hovered = new;
				ret
			},
			)+
			_ => unimplemented!(),
		}
	}

	fn should_draw(&self) -> bool {
		match self {
			$(
			&Element::$name(ref val) => val.should_draw,
			)+
			_ => unimplemented!(),
		}
	}

	fn get_parent(&self) -> Option<ElementRefInner> {
		match self {
			$(
			&Element::$name(ref val) => val.parent,
			)+
			_ => unimplemented!(),
		}
	}

	fn get_attachment(&self) -> (VAttach, HAttach) {
		match self {
			$(
			&Element::$name(ref val) => (val.v_attach, val.h_attach),
			)+
			_ => unimplemented!(),
		}		
	}

	fn get_offset(&self) -> (f64, f64) {
		match self {
			$(
			&Element::$name(ref val) => (val.x, val.y),
			)+
			_ => unimplemented!(),
		}		
	}

	fn get_size(&self) -> (f64, f64) {
		match self {
			$(
			&Element::$name(ref val) => val.get_size(),
			)+
			_ => unimplemented!(),
		}		
	}

	fn is_dirty(&self) -> bool {
		match self {
			$(
			&Element::$name(ref val) => val.dirty,
			)+
			_ => unimplemented!(),
		}				
	}

	fn set_dirty(&mut self, dirty: bool) {
		match self {
			$(
			&mut Element::$name(ref mut val) => val.dirty = dirty,
			)+
			_ => unimplemented!(),
		}				
	}

	fn update(&mut self, renderer: &mut render::Renderer) {
		match self {
			$(
			&mut Element::$name(ref mut val) => val.update(renderer),
			)+
			_ => unimplemented!(),
		}		
	}

	fn draw(&mut self, renderer: &mut render::Renderer, r: &Region, width: f64, height: f64, delta: f64) -> &Vec<u8>{
		match self {
			$(
			&mut Element::$name(ref mut val) => val.draw(renderer, r, width, height, delta),
			)+
			_ => unimplemented!(),
		}		
	}
}
	)
}

element_impl!(
	Image,
	Batch,
	Text,
	Formatted
);

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
#[derive(Copy)]
pub struct ElementRef<T> {
	inner: ElementRefInner,
	ty: PhantomData<T>,
}

impl <T> Clone for ElementRef<T> {
	fn clone(&self) -> Self {
		ElementRef {
			inner: self.inner,
			ty: PhantomData,
		}
	}
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct ElementRefInner {
	index: usize,
}

impl <T> Default for ElementRef<T> {
	fn default() -> Self {
		ElementRef {
			inner: ElementRefInner{ index: 0 },
			ty: PhantomData,
		}
	}
}

/// Allows for easy cleanup
pub struct Collection {
	elements: Vec<ElementRefInner>,
}

impl Collection {
	pub fn new() -> Collection {
		Collection {
			elements: Vec::new(),
		}
	}

	pub fn add<T: UIElement>(&mut self, element: ElementRef<T>) -> ElementRef<T> {
		self.elements.push(element.inner);
		element
	}

	pub fn remove_all(&mut self, container: &mut Container) {
		for e in &self.elements {
			container.remove_raw(e);
		}
	}
}

const SCREEN: Region = Region{x: 0.0, y: 0.0, w: SCALED_WIDTH, h: SCALED_HEIGHT};

pub struct Container {
	pub mode: Mode,
	elements: HashMap<ElementRefInner, Element>,
	// We need the order
	elements_list: Vec<ElementRefInner>,
	version: usize,

	last_sw: f64,
	last_sh: f64,
	last_width: f64,
	last_height: f64,
}

impl Container {
	pub fn new() -> Container {
		Container {
			mode: Mode::Scaled,
			elements: HashMap::new(),
			elements_list: Vec::new(),
			version: 0xFFFF,
			last_sw: 0.0,
			last_sh: 0.0,
			last_width: 0.0,
			last_height: 0.0,
		}
	}

	pub fn add<T: UIElement>(&mut self, e: T) -> ElementRef<T> {
		let mut r = ElementRefInner{index: rand::random()};
		while self.elements.contains_key(&r) {
			r = ElementRefInner{index: rand::random()};
		}
		self.elements.insert(r, e.wrap());
		self.elements_list.push(r);
		ElementRef{inner: r, ty: PhantomData}
	}

	pub fn get<T: UIElement>(&self, r: &ElementRef<T>) -> &T {
		T::unwrap_ref(self.elements.get(&r.inner).unwrap())
	}

	pub fn get_mut<T: UIElement>(&mut self, r: &ElementRef<T>) -> &mut T {
		T::unwrap_ref_mut(self.elements.get_mut(&r.inner).unwrap())
	}

	pub fn remove<T: UIElement>(&mut self, r: &ElementRef<T>) {
		self.remove_raw(&r.inner);
	}
	
	fn remove_raw(&mut self, r: &ElementRefInner) {
		self.elements.remove(&r);
		self.elements_list.iter()
			.position(|&e| e.index == r.index)
			.map(|e| self.elements_list.remove(e))
			.unwrap();
	}

	pub fn tick(&mut self, renderer: &mut render::Renderer, delta: f64, width: f64, height: f64) {
		let (sw, sh) = match self.mode {
			Mode::Scaled => (SCALED_WIDTH / width, SCALED_HEIGHT / height),
			Mode::Unscaled(scale) => (scale, scale),
		};

		if self.last_sw != sw || self.last_sh != sh 
			|| self.last_width != width || self.last_height != height 
			|| self.version != renderer.ui.version {
			self.last_sw = sw;
			self.last_sh = sh;
			self.last_width = width;
			self.last_height = height;
			for (_, e) in &mut self.elements {
				e.set_dirty(true);
				if self.version != renderer.ui.version {
					e.update(renderer);
				}
			}
			self.version = renderer.ui.version;
		}

		// Borrow rules seems to prevent us from doing this in the first pass
		// so we split it.
		let regions = self.collect_elements(sw, sh);
		for re in &self.elements_list {
			let mut e = self.elements.get_mut(re).unwrap();
			if !e.should_draw() {
				continue;
			}
			if let Some(&(ref r, ref dirty)) = regions.get(re) {
				e.set_dirty(*dirty);
				let data = e.draw(renderer, r, width, height, delta);
				renderer.ui.add_bytes(data);
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

	pub fn click_at(&mut self, renderer: &mut render::Renderer, x: f64, y: f64, width: f64, height: f64) {
		let (sw, sh) = match self.mode {
			Mode::Scaled => (SCALED_WIDTH / width, SCALED_HEIGHT / height),
			Mode::Unscaled(scale) => (scale, scale),
		};
		let mx = (x / width) * SCALED_WIDTH;
		let my = (y / height) * SCALED_HEIGHT;
		let mut click = None;
		for re in self.elements_list.iter().rev() {			
			let e = self.elements.get(re).unwrap();
			let funcs =  e.get_click_funcs();
			if !funcs.is_empty() {
				let r = self.get_draw_region(e, sw, sh);
				if mx >= r.x && mx <= r.x + r.w && my >= r.y && my <= r.y + r.h {
					click = Some(funcs);
					break;
				}
			}
		}
		if let Some(click) = click {
			for c in &click {
				c(renderer, self);
			}
		}
	}

	pub fn hover_at(&mut self, renderer: &mut render::Renderer, x: f64, y: f64, width: f64, height: f64) {
		let (sw, sh) = match self.mode {
			Mode::Scaled => (SCALED_WIDTH / width, SCALED_HEIGHT / height),
			Mode::Unscaled(scale) => (scale, scale),
		};
		let mx = (x / width) * SCALED_WIDTH;
		let my = (y / height) * SCALED_HEIGHT;
		let mut hovers = Vec::new();
		for re in self.elements_list.iter().rev() {			
			let e = self.elements.get(re).unwrap();
			let funcs =  e.get_hover_funcs();
			if !funcs.is_empty() {
				let r = self.get_draw_region(e, sw, sh);
				hovers.push((*re, funcs, mx >= r.x && mx <= r.x + r.w && my >= r.y && my <= r.y + r.h));
			}
		}
		for hover in &hovers {
			let call = {
				let e = self.elements.get_mut(&hover.0).unwrap();
				e.should_call_hover(hover.2)
			};
			if call {
				for f in &hover.1 {
					f(hover.2, renderer, self);
				}
			}
		}
	}

	fn get_draw_region(&self, e: &Element, sw: f64, sh: f64) -> Region {		
		let super_region = match e.get_parent() {
			Some(ref p) => self.get_draw_region(self.elements.get(p).unwrap(), sw, sh),
			None => SCREEN,
		};
		Container::get_draw_region_raw(e, sw, sh, &super_region)
	}

	fn get_draw_region_raw(e: &Element, sw: f64, sh: f64, super_region: &Region) -> Region {
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

macro_rules! ui_element {
	(
	$name:ident {
		$(
			$field:ident : $field_ty:ty
		),+
	}
	) => (
	pub struct $name {		
		dirty: bool,
		data: Vec<u8>,		
		parent: Option<ElementRefInner>,
		should_draw: bool,
		layer: isize,
		x: f64,
		y: f64,
		v_attach: VAttach,
		h_attach: HAttach,	
		click_funcs: Vec<Rc<Fn(&mut render::Renderer, &mut Container)>>,
		hover_funcs: Vec<Rc<Fn(bool, &mut render::Renderer, &mut Container)>>,
		hovered: bool,
		$(
			$field: $field_ty
		),+
	}
	)
}

macro_rules! base_impl {
	() => (
		pub fn set_parent<T: UIElement>(&mut self, other: &ElementRef<T>) {
			self.parent = Some(other.inner);
			self.dirty = true;
		}

		pub fn add_click_func(&mut self, f: Rc<Fn(&mut render::Renderer, &mut Container)>) {
			self.click_funcs.push(f);
		}

		pub fn add_hover_func(&mut self, f: Rc<Fn(bool, &mut render::Renderer, &mut Container)>) {
			self.hover_funcs.push(f);
		}

		lazy_field!(layer, isize, get_layer, set_layer);
		lazy_field!(x, f64, get_x, set_x);
		lazy_field!(y, f64, get_y, set_y);
		lazy_field!(v_attach, VAttach, get_v_attach, set_v_attach);
		lazy_field!(h_attach, HAttach, get_h_attach, set_h_attach);
	)
}

macro_rules! ui_create {
	($name:ident {
		$($field:ident: $e:expr),+
	}) => (
		$name {
			dirty: true,
			data: Vec::new(),

			parent: None,
			should_draw: true,
			layer: 0,
			v_attach: VAttach::Top,
			h_attach: HAttach::Left,
			click_funcs: Vec::new(),
			hover_funcs: Vec::new(),
			hovered: false,
			$($field: $e),+
		}
	)
}

ui_element!(Image {
	texture: render::Texture,
	width: f64,
	height: f64,

	t_x: f64,
	t_y: f64,
	t_width: f64,
	t_height: f64,

	r: u8,
	g: u8,
	b: u8,
	a: u8
});

impl Image {
	base_impl!();

	pub fn new(texture: render::Texture, x: f64, y: f64, w: f64, h: f64, t_x: f64, t_y: f64, t_width: f64, t_height: f64, r: u8, g: u8, b: u8) -> Image {
		ui_create!(Image {
			texture: texture,
			x: x,
			y: y,
			width: w,
			height: h,

			t_x: t_x,
			t_y: t_y,
			t_width: t_width,
			t_height: t_height,

			r: r,
			g: g,
			b: b,
			a: 255
		})
	}

	fn update(&mut self, renderer: &mut render::Renderer) {}

	fn draw(&mut self, renderer: &mut render::Renderer, r: &Region, width: f64, height: f64, delta: f64) -> &Vec<u8> {
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
		&self.data
	}

	pub fn get_size(&self) -> (f64, f64) {
		(self.width, self.height)
	}

	pub fn get_texture(&self) -> render::Texture {
		self.texture.clone()
	}

	pub fn set_texture(&mut self, val: render::Texture) {
		self.texture = val;
		self.dirty = true;
	}

	lazy_field!(width, f64, get_width, set_width);
	lazy_field!(height, f64, get_height, set_height);

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

#[derive(Clone, Copy)]
pub struct BatchRef<T: UIElement> {
	index: usize,
	ty: PhantomData<T>,
}

ui_element!(Batch {
	width: f64,
	height: f64,

	elements: Vec<Element>
});

impl Batch {
	base_impl!();

	pub fn new(x: f64, y: f64, w: f64, h: f64) -> Batch {
		ui_create!(Batch {
			x: x,
			y: y,
			width: w,
			height: h,

			elements: Vec::new()
		})
	}

	fn update(&mut self, renderer: &mut render::Renderer) {}

	fn draw(&mut self, renderer: &mut render::Renderer, r: &Region, width: f64, height: f64, delta: f64) -> &Vec<u8> {
		if self.dirty {
			self.dirty = false;
			self.data.clear();

			let sx = r.w / self.width;
			let sy = r.h / self.height;

			for e in &mut self.elements {
				let reg = Container::get_draw_region_raw(e, sx, sy, r);
				e.set_dirty(true);
				self.data.extend(e.draw(renderer, &reg, width, height, delta));
			}
		}
		&self.data
	}

	pub fn get_size(&self) -> (f64, f64) {
		(self.width, self.height)
	}

	pub fn add<T: UIElement>(&mut self, e: T) -> BatchRef<T> {
		self.elements.push(e.wrap());
		BatchRef { index: self.elements.len() - 1, ty: PhantomData }
	}

	pub fn get<T: UIElement>(&self, r: BatchRef<T>) -> &T {
		T::unwrap_ref(&self.elements[r.index])
	}

	pub fn get_mut<T: UIElement>(&mut self, r: BatchRef<T>) -> &mut T {
		self.dirty = true;
		T::unwrap_ref_mut(&mut self.elements[r.index])
	}

	pub fn get_mut_at<T: UIElement>(&mut self, index: usize) -> &mut T {
		self.dirty = true;
		T::unwrap_ref_mut(&mut self.elements[index])
	}

	pub fn len(&self) -> usize {
		self.elements.len()
	}

	lazy_field!(width, f64, get_width, set_width);
	lazy_field!(height, f64, get_height, set_height);
}

impl UIElement for Batch {
	fn wrap(self) -> Element {
		Element::Batch(self)
	}

	fn unwrap_ref<'a>(e: &'a Element) -> &'a Batch {
		match e {
			&Element::Batch(ref val) => val,
			_ => panic!("Incorrect type"),
		}
	}

	fn unwrap_ref_mut<'a>(e: &'a mut Element) -> &'a mut Batch {
		match e {
			&mut Element::Batch(ref mut val) => val,
			_ => panic!("Incorrect type"),
		}
	}
}

ui_element!(Text {
	val: String,
	width: f64,
	height: f64,
	scale_x: f64,
	scale_y: f64,
	rotation: f64,
	r: u8,
	g: u8,
	b: u8,
	a: u8
});

impl Text {
	base_impl!();

	pub fn new(renderer: &render::Renderer, val: &str, x: f64, y: f64, r: u8, g: u8, b: u8) -> Text {
		ui_create!(Text {
			val: val.to_owned(),
			x: x,
			y: y,
			width: renderer.ui.size_of_string(val),
			height: 18.0,
			scale_x: 1.0,
			scale_y: 1.0,
			rotation: 0.0,
			r: r,
			g: g,
			b: b,
			a: 255
		})
	}

	fn update(&mut self, renderer: &mut render::Renderer) {
		self.width = renderer.ui.size_of_string(&self.val);
	}

	fn draw(&mut self, renderer: &mut render::Renderer, r: &Region, width: f64, height: f64, delta: f64) -> &Vec<u8> {
		if self.dirty {
			self.dirty = false;
			let sx = r.w / self.width;
			let sy = r.h / self.height;
			let mut text = if self.rotation == 0.0 {
				renderer.ui.new_text_scaled(&self.val, r.x, r.y, sx*self.scale_x, sy*self.scale_y, self.r, self.g, self.b)
			} else {
				let c = self.rotation.cos();
				let s = self.rotation.sin();
				let tmpx = r.w / 2.0;
				let tmpy = r.h / 2.0;
				let w = (tmpx*c - tmpy*s).abs();
				let h = (tmpy*c + tmpx*s).abs();
				renderer.ui.new_text_rotated(&self.val, r.x+w-(r.w / 2.0), r.y+h-(r.h / 2.0), sx*self.scale_x, sy*self.scale_y, self.rotation, self.r, self.g, self.b)
			};
			for e in &mut text.elements {
				e.a = self.a;
				e.layer = self.layer;
			}
			self.data = text.bytes(width, height);
		}
		&self.data
	}

	pub fn get_size(&self) -> (f64, f64) {
		((self.width + 2.0) * self.scale_x, self.height * self.scale_y)
	}

	pub fn get_text(&self) -> &str {
		&self.val
	}

	pub fn set_text(&mut self, renderer: &render::Renderer, val: &str) {
		self.dirty = true;
		self.val = val.to_owned();
		self.width = renderer.ui.size_of_string(val);
	}

	lazy_field!(width, f64, get_width, set_width);
	lazy_field!(height, f64, get_height, set_height);
	lazy_field!(scale_x, f64, get_scale_x, set_scale_x);
	lazy_field!(scale_y, f64, get_scale_y, set_scale_y);
	lazy_field!(rotation, f64, get_rotation, set_rotation);
	lazy_field!(r, u8, get_r, set_r);
	lazy_field!(g, u8, get_g, set_g);
	lazy_field!(b, u8, get_b, set_b);

}

impl UIElement for Text {
	fn wrap(self) -> Element {
		Element::Text(self)
	}

	fn unwrap_ref<'a>(e: &'a Element) -> &'a Text {
		match e {
			&Element::Text(ref val) => val,
			_ => panic!("Incorrect type"),
		}
	}

	fn unwrap_ref_mut<'a>(e: &'a mut Element) -> &'a mut Text {
		match e {
			&mut Element::Text(ref mut val) => val,
			_ => panic!("Incorrect type"),
		}
	}
}

// Include instead of mod so we can access private parts
include!("formatted.rs");