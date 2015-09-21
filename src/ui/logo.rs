
use std::sync::{Arc, RwLock};
use ui;
use render;
use resources;

pub struct Logo {
	resources: Arc<RwLock<resources::Manager>>,
}

impl Logo {
	pub fn new(resources: Arc<RwLock<resources::Manager>>, renderer: &mut render::Renderer, ui_container: &mut ui::Container) -> Logo {
		let mut l = Logo {
			resources: resources
		};
		l.init(renderer, ui_container);
		l
	}

	fn init(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
		let res = self.resources.read().unwrap();

		let mut logo = res.open("steven", "logo/logo.txt").unwrap();
		let mut logo_str = String::new();
		logo.read_to_string(&mut logo_str).unwrap();

		let solid = render::Renderer::get_texture(renderer.get_textures_ref(), "steven:solid");
		let stone = render::Renderer::get_texture(renderer.get_textures_ref(), "blocks/planks_oak");

		let mut shadow_batch = ui::Batch::new(0.0, 8.0, 100.0, 100.0);
		let mut layer0 = ui::Batch::new(0.0, 8.0, 100.0, 100.0);

		shadow_batch.set_h_attach(ui::HAttach::Center);
		layer0.set_h_attach(ui::HAttach::Center);

		let mut row = 0;
		for line in logo_str.lines() {
			if line.is_empty() {
				continue;
			}
			let mut i = 0;
			for c in line.chars() {
				i += 1;
				if c == ' ' {
					continue;
				}
				let x = (i - 1) * 4;
				let y = row * 8;
				let (r, g, b) = if c == ':' {
					(255, 255, 255)
				} else {
					(170, 170, 170)
				};
				let mut shadow = ui::Image::new(
					solid.clone(), 
					(x+2) as f64, (y+4) as f64, 4.0, 8.0, 
					0.0, 0.0, 1.0, 1.0, 
					0, 0, 0
				);
				shadow.set_a(100);
				shadow_batch.add(shadow);


				let img = ui::Image::new(
					stone.clone(), 
					x as f64, y as f64, 4.0, 8.0, 
					(x%16) as f64 / 16.0, (y%16) as f64 / 16.0, 4.0/16.0, 8.0/16.0, 
					r, g, b
				);
				layer0.add(img);		

				let width = (x + 4) as f64;
				if shadow_batch.get_width() < width {
					shadow_batch.set_width(width);
					layer0.set_width(width);
				}
			}
			row += 1;
		}

		ui_container.add(shadow_batch);
		ui_container.add(layer0);
	}
}