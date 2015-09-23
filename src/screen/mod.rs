
mod server_list;
pub use self::server_list::*;

use render;
use ui;

pub trait Screen {
	// Called once
	fn init(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container);
	fn deinit(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container);

	// May be called multiple times
	fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container);
	fn on_deactive(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container);

	// Called every frame the screen is active
	fn tick(&mut self, delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container);

	// Events
	fn on_scroll(&mut self, x: f64, y: f64) {}
}

struct ScreenInfo {
	screen: Box<Screen>,
	init: bool,
	active: bool,
}

pub struct ScreenSystem {
	screens: Vec<ScreenInfo>,
	remove_queue: Vec<ScreenInfo>,
}

impl ScreenSystem {
	pub fn new() -> ScreenSystem {
		ScreenSystem {
			screens: Vec::new(),
			remove_queue: Vec::new(),
		}
	}

	pub fn add_screen(&mut self, screen: Box<Screen>) {
		self.screens.push(ScreenInfo {
			screen: screen,
			init: false,
			active: false,
		});
	}

	pub fn pop_screen(&mut self) {
		if let Some(screen) = self.screens.pop() {
			self.remove_queue.push(screen);
		}
	}

	pub fn tick(&mut self, delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
		for screen in &mut self.remove_queue {
			if screen.active {
				screen.screen.on_deactive(renderer, ui_container);
			}
			if screen.init {
				screen.screen.deinit(renderer, ui_container);
			}
		}
		self.remove_queue.clear();
		if self.screens.is_empty() {
			return;
		}
		// Update state for screens
		let len = self.screens.len();
		for screen in &mut self.screens[..len - 1] {
			if screen.active {
				screen.active = false;
				screen.screen.on_deactive(renderer, ui_container);
			}
		}
		let current = self.screens.last_mut().unwrap();
		if !current.init {
			current.init = true;
			current.screen.init(renderer, ui_container);
		}
		if !current.active {
			current.active = true;
			current.screen.on_active(renderer, ui_container);
		}

		// Handle current
		current.screen.tick(delta, renderer, ui_container);
	}

	pub fn on_scroll(&mut self, x: f64, y: f64) {
		if self.screens.is_empty() {
			return;
		}		
		let current = self.screens.last_mut().unwrap();
		current.screen.on_scroll(x, y);
	}
}

pub fn new_button(renderer: &mut render::Renderer, x: f64, y: f64, w: f64, h: f64) -> ui::Batch {
	let mut batch = ui::Batch::new(x, y, w, h);

	let texture = render::Renderer::get_texture(renderer.get_textures_ref(), "gui/widgets").relative(
		0.0, 66.0 / 256.0, 200.0 / 256.0, 20.0 / 256.0
	);

	// Corners
	batch.add(ui::Image::new(texture.clone(), 0.0, 0.0, 4.0, 4.0, 0.0, 0.0, 2.0 / 200.0, 2.0 / 20.0, 255, 255, 255));
	batch.add(ui::Image::new(texture.clone(), w - 4.0, 0.0, 4.0, 4.0, 198.0 / 200.0, 0.0, 2.0 / 200.0, 2.0 / 20.0, 255, 255, 255));
	batch.add(ui::Image::new(texture.clone(), 0.0, h - 6.0, 4.0, 6.0, 0.0, 17.0 / 20.0, 2.0 / 200.0, 3.0 / 20.0, 255, 255, 255));
	batch.add(ui::Image::new(texture.clone(), w - 4.0, h - 6.0, 4.0, 6.0, 198.0 / 200.0, 17.0 / 20.0, 2.0 / 200.0, 3.0 / 20.0, 255, 255, 255));

	// Widths
	batch.add(ui::Image::new(texture.clone().relative(
		2.0 / 200.0, 0.0, 196.0 / 200.0, 2.0 / 20.0
	), 4.0, 0.0, w - 8.0, 4.0, 0.0, 0.0, 1.0, 1.0, 255, 255, 255));
	batch.add(ui::Image::new(texture.clone().relative(
		2.0 / 200.0, 17.0 / 20.0, 196.0 / 200.0, 3.0 / 20.0
	), 4.0, h - 6.0, w - 8.0, 6.0, 0.0, 0.0, 1.0, 1.0, 255, 255, 255));

	// Heights
	batch.add(ui::Image::new(texture.clone().relative(
		0.0, 2.0 / 20.0, 2.0 / 200.0, 15.0 / 20.0
	), 0.0, 4.0, 4.0, h - 10.0, 0.0, 0.0, 1.0, 1.0, 255, 255, 255));
	batch.add(ui::Image::new(texture.clone().relative(
		198.0 / 200.0, 2.0 / 20.0, 2.0 / 200.0, 15.0 / 20.0
	), w - 4.0, 4.0, 4.0, h - 10.0, 0.0, 0.0, 1.0, 1.0, 255, 255, 255));

	// Center
	batch.add(ui::Image::new(texture.clone().relative(
		2.0 / 200.0, 2.0 / 20.0, 196.0 / 200.0, 15.0 / 20.0
	), 4.0, 4.0, w - 8.0, h - 10.0, 0.0, 0.0, 1.0, 1.0, 255, 255, 255));

	batch
}

pub fn new_button_text(renderer: &mut render::Renderer, val: &str, x: f64, y: f64, w: f64, h: f64) -> (ui::Batch, ui::Text) {
	let batch = new_button(renderer, x, y, w, h);
	let mut text = ui::Text::new(renderer, val, 0.0, 0.0, 255, 255, 255);
	text.set_v_attach(ui::VAttach::Middle);
	text.set_h_attach(ui::HAttach::Center);
	(batch, text)
}