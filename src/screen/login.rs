
use std::fs;
use std::thread;
use std::sync::mpsc;
use std::rc::Rc;
use std::cell::RefCell;

use ui;
use render;
use format;
use format::{Component, TextComponent};
use protocol;

use serde_json;
use time;
use image;
use rustc_serialize::base64::FromBase64;
use rand;
use rand::{Rng};

pub struct Login {
	elements: Option<UIElements>,
}

struct UIElements {
	logo: ui::logo::Logo,
	elements: ui::Collection,
}


impl Login {
	pub fn new() -> Login {
		Login {
			elements: None,
		}
	}
}

impl super::Screen for Login {
	fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
		let logo = ui::logo::Logo::new(renderer.resources.clone(), renderer, ui_container);
		let mut elements = ui::Collection::new();

		// Disclaimer
		let mut warn = ui::Text::new(renderer, "Not affiliated with Mojang/Minecraft", 5.0, 5.0, 255, 200, 200);
		warn.set_v_attach(ui::VAttach::Bottom);
		warn.set_h_attach(ui::HAttach::Right);
		elements.add(ui_container.add(warn));

		self.elements = Some(UIElements {
			logo: logo,
			elements: elements,
		});
	}
	fn on_deactive(&mut self, _renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
		// Clean up
		{
			let elements = self.elements.as_mut().unwrap();
			elements.logo.remove(ui_container);
			elements.elements.remove_all(ui_container);
		}
		self.elements = None
	}

	fn tick(&mut self, delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
		let elements = self.elements.as_mut().unwrap();

		elements.logo.tick(renderer, ui_container);
	}
}