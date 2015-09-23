
use std::fs;
use ui;
use render;
use format;
use serde_json;
use std::cmp::max;

pub struct ServerList {
	elements: Option<UIElements>,
	disconnect_reason: Option<format::Component>,
}

struct UIElements {
	logo: ui::logo::Logo,
	elements: ui::Collection,
	servers: Vec<Server>,
}

struct Server {
	collection: ui::Collection,
	back: ui::ElementRef<ui::Image>,
	offset: f64,
	y: f64,
}

impl Server {
	fn update_position(&mut self) {
		if self.offset < 0.0 {
			self.y = self.offset * 200.0;
		} else {
			self.y = self.offset * 100.0;
		}
	}
}

impl ServerList {
	pub fn new(disconnect_reason: Option<format::Component>) -> ServerList {
		ServerList {
			elements: None,
			disconnect_reason: disconnect_reason,
		}
	}

	fn reload_server_list(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
		let elements = self.elements.as_mut().unwrap();
		for server in &mut elements.servers {
			server.collection.remove_all(ui_container);
		}
		elements.servers.clear();

		let file = match fs::File::open("servers.json") {
			Ok(val) => val,
			Err(e) => return,
		};
		let servers_info: serde_json::Value = serde_json::from_reader(file).unwrap();
		let servers = servers_info.find("servers").unwrap().as_array().unwrap();
		let mut offset = 0.0;

		let default_icon = render::Renderer::get_texture(renderer.get_textures_ref(), "misc/unknown_server");
		let icons = render::Renderer::get_texture(renderer.get_textures_ref(), "gui/icons");

		for svr in servers {
			let name = svr.find("name").unwrap().as_string().unwrap();
			let address = svr.find("address").unwrap().as_string().unwrap();

			let solid = render::Renderer::get_texture(renderer.get_textures_ref(), "steven:solid");

			let mut back = ui::Image::new(solid, 0.0, offset * 100.0, 700.0, 100.0, 0.0, 0.0, 1.0, 1.0, 0, 0, 0);
			back.set_a(100);
			back.set_v_attach(ui::VAttach::Middle);
			back.set_h_attach(ui::HAttach::Center);

			let mut server = Server {
				collection: ui::Collection::new(),
				back: ui_container.add(back),
				offset: offset,
				y: 0.0,
			};
			server.collection.add(server.back.clone());
			server.update_position();

			let mut text = ui::Text::new(renderer, &name, 100.0, 5.0, 255, 255, 255);
			text.set_parent(&server.back);
			server.collection.add(ui_container.add(text));

			let mut icon = ui::Image::new(default_icon.clone(), 5.0, 5.0, 90.0, 90.0, 0.0, 0.0, 1.0, 1.0, 255, 255, 255);
			icon.set_parent(&server.back);
			server.collection.add(ui_container.add(icon));

			let mut ping = ui::Image::new(icons.clone(), 5.0, 5.0, 20.0, 16.0, 0.0, 56.0/256.0, 10.0/256.0, 8.0/256.0, 255, 255, 255);
			ping.set_h_attach(ui::HAttach::Right);
			ping.set_parent(&server.back);
			server.collection.add(ui_container.add(ping));

			let mut players = ui::Text::new(renderer, "???", 30.0, 5.0, 255, 255, 255);
			players.set_h_attach(ui::HAttach::Right);
			players.set_parent(&server.back);
			server.collection.add(ui_container.add(players));

			elements.servers.push(server);
			offset += 1.0;
		}
	}
}

impl super::Screen for ServerList {
	fn init(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {}
	fn deinit(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {}

	fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
		let logo = ui::logo::Logo::new(renderer.resources.clone(), renderer, ui_container);
		let mut elements = ui::Collection::new();

		let (mut refresh, mut txt) = super::new_button_text(renderer, "Refresh", 300.0, -50.0-15.0, 100.0, 30.0);
		refresh.set_v_attach(ui::VAttach::Middle);
		refresh.set_h_attach(ui::HAttach::Center);
		let re = ui_container.add(refresh);
		txt.set_parent(&re);
		elements.add(re);
		elements.add(ui_container.add(txt));

		let (mut add, mut txt) = super::new_button_text(renderer, "Add", 200.0, -50.0-15.0, 100.0, 30.0);
		add.set_v_attach(ui::VAttach::Middle);
		add.set_h_attach(ui::HAttach::Center);
		let re = ui_container.add(add);
		txt.set_parent(&re);
		elements.add(re);
		elements.add(ui_container.add(txt));

		let mut options = super::new_button(renderer, 5.0, 25.0, 40.0, 40.0);
		options.set_v_attach(ui::VAttach::Bottom);
		options.set_h_attach(ui::HAttach::Right);
		let re = ui_container.add(options);
		let mut cog = ui::Image::new(render::Renderer::get_texture(renderer.get_textures_ref(), "steven:gui/cog"), 0.0, 0.0, 40.0, 40.0, 0.0, 0.0, 1.0, 1.0, 255, 255, 255);
		cog.set_parent(&re);
		cog.set_v_attach(ui::VAttach::Middle);
		cog.set_h_attach(ui::HAttach::Center);				
		elements.add(re);
		elements.add(ui_container.add(cog));

		let mut warn = ui::Text::new(renderer, "Not affiliated with Mojang/Minecraft", 5.0, 5.0, 255, 200, 200);
		warn.set_v_attach(ui::VAttach::Bottom);
		warn.set_h_attach(ui::HAttach::Right);
		elements.add(ui_container.add(warn));

		if let Some(ref disconnect_reason) = self.disconnect_reason {
			let mut dis_msg = ui::Text::new(renderer, "Disconnected", 0.0, 32.0, 255, 0, 0);
			dis_msg.set_h_attach(ui::HAttach::Center);
			let mut dis = ui::Formatted::with_width_limit(renderer, disconnect_reason.clone(), 0.0, 48.0, 600.0);
			dis.set_h_attach(ui::HAttach::Center);
			let mut back = ui::Image::new(
				render::Renderer::get_texture(renderer.get_textures_ref(), "steven:solid"), 
				0.0, 30.0, 
				dis.get_width().max(dis_msg.get_width()) + 4.0, dis.get_height() + 4.0 + 16.0,
				0.0, 0.0, 1.0, 1.0, 
				0, 0, 0
			);
			back.set_a(100);
			back.set_h_attach(ui::HAttach::Center);
			elements.add(ui_container.add(back));
			elements.add(ui_container.add(dis));
			elements.add(ui_container.add(dis_msg));
		}

		self.elements = Some(UIElements {
			logo: logo,
			elements: elements,
			servers: Vec::new(),
		});
		self.reload_server_list(renderer, ui_container);
	}
	fn on_deactive(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
		{
			let elements = self.elements.as_mut().unwrap();
			elements.logo.remove(ui_container);
			elements.elements.remove_all(ui_container);
			for server in &mut elements.servers {
				server.collection.remove_all(ui_container);
			}
			elements.servers.clear();
		}
		self.elements = None
	}

	fn tick(&mut self, delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
		let elements = self.elements.as_mut().unwrap();
		elements.logo.tick(renderer, ui_container);

		for s in &mut elements.servers {
			let back = ui_container.get_mut(&s.back);
			let dy = s.y - back.get_y();
			if dy*dy > 1.0 {
				let y = back.get_y();
				back.set_y(y + delta * dy * 0.1);
			} else {
				back.set_y(s.y);
			}
		}
	}

	fn on_scroll(&mut self, x: f64, y: f64) {
		let elements = self.elements.as_mut().unwrap();
		if elements.servers.is_empty() {
			return;
		}
		let mut diff = y / 1.0;
		{
			let last = elements.servers.last().unwrap();
			if last.offset+diff <= 2.0 {
				diff = 2.0 - last.offset;
			}
			let first = elements.servers.first().unwrap();
			if first.offset + diff >= 0.0 {
				diff = -first.offset;
			}
		}

		for s in &mut elements.servers {
			s.offset += diff;
			s.update_position();
		}
	}
}