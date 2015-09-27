use ui;
use render;

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

		// Login
		let (mut login, mut txt) = super::new_button_text(renderer, "Login", 0.0, 100.0, 400.0, 40.0);
		login.set_v_attach(ui::VAttach::Middle);
		login.set_h_attach(ui::HAttach::Center);
		let re = ui_container.add(login);
		txt.set_parent(&re);
		let tre = ui_container.add(txt);
		super::button_action(ui_container, re.clone(), Some(tre.clone()), None);
		elements.add(re);
		elements.add(tre);

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