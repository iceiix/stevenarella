// Copyright 2016 Matthew Collins
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

use std::fs;
use std::collections::BTreeMap;

use ui;
use render;

use serde_json::{self, Value};

pub struct EditServerEntry {
    elements: Option<UIElements>,
    entry_info: Option<(usize, String, String)>,
}

struct UIElements {
    logo: ui::logo::Logo,
    elements: ui::Collection,
}

impl EditServerEntry {
    pub fn new(entry_info: Option<(usize, String, String)>) -> EditServerEntry {
        EditServerEntry {
            elements: None,
            entry_info: entry_info,
        }
    }

    fn save_servers(index: Option<usize>, name: &str, address: &str) {
        let mut servers_info = match fs::File::open("servers.json") {
            Ok(val) => serde_json::from_reader(val).unwrap(),
            Err(_) => {
                let mut info = BTreeMap::default();
                info.insert("servers".to_owned(), Value::Array(vec![]));
                Value::Object(info)
            }
        };

        let new_entry = {
            let mut entry = BTreeMap::default();
            entry.insert("name".to_owned(), Value::String(name.to_owned()));
            entry.insert("address".to_owned(), Value::String(address.to_owned()));
            Value::Object(entry)
        };

        {
            let servers = servers_info.as_object_mut()
                .unwrap()
                .get_mut("servers")
                .unwrap()
                .as_array_mut()
                .unwrap();
            if let Some(index) = index {
                *servers.iter_mut().nth(index).unwrap() = new_entry;
            } else {
                servers.push(new_entry);
            }
        }

        let mut out = fs::File::create("servers.json").unwrap();
        serde_json::to_writer_pretty(&mut out, &servers_info).unwrap();
    }

}

impl super::Screen for EditServerEntry {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let logo = ui::logo::Logo::new(renderer.resources.clone(), renderer, ui_container);
        let mut elements = ui::Collection::new();

        // Name
        let mut server_name = ui::TextBox::new(
            renderer, self.entry_info.as_ref().map_or("", |v| &v.1),
            0.0, -20.0, 400.0, 40.0
        );
        server_name.set_v_attach(ui::VAttach::Middle);
        server_name.set_h_attach(ui::HAttach::Center);
        server_name.add_submit_func(|_, ui| {
            ui.cycle_focus();
        });
        let ure = ui_container.add(server_name);
        let mut server_name_label = ui::Text::new(renderer, "Name:", 0.0, -18.0, 255, 255, 255);
        server_name_label.set_parent(&ure);
        let server_name_txt = elements.add(ure);
        elements.add(ui_container.add(server_name_label));

        // Name
        let mut server_address = ui::TextBox::new(
            renderer, self.entry_info.as_ref().map_or("", |v| &v.2),
            0.0, 40.0, 400.0, 40.0
        );
        server_address.set_v_attach(ui::VAttach::Middle);
        server_address.set_h_attach(ui::HAttach::Center);
        server_address.add_submit_func(|_, ui| {
            ui.cycle_focus();
        });
        let ure = ui_container.add(server_address);
        let mut server_address_label = ui::Text::new(renderer, "Address:", 0.0, -18.0, 255, 255, 255);
        server_address_label.set_parent(&ure);
        let server_address_txt = elements.add(ure);
        elements.add(ui_container.add(server_address_label));

        // Done
        let (mut done, mut txt) = super::new_button_text(
            renderer, "Done",
            110.0, 100.0, 200.0, 40.0
        );
        done.set_v_attach(ui::VAttach::Middle);
        done.set_h_attach(ui::HAttach::Center);
        let re = ui_container.add(done);
        txt.set_parent(&re);
        let tre = ui_container.add(txt);

        let index = self.entry_info.as_ref().map(|v| v.0);
        super::button_action(ui_container, re.clone(), Some(tre.clone()), move |game, uic| {
            Self::save_servers(
                index,
                &uic.get(&server_name_txt).get_input(),
                &uic.get(&server_address_txt).get_input()
            );
            game.screen_sys.replace_screen(Box::new(super::ServerList::new(None)));
        });
        elements.add(re);
        elements.add(tre);

        // Cancel
        let (mut cancel, mut txt) = super::new_button_text(
            renderer, "Cancel",
            -110.0, 100.0, 200.0, 40.0
        );
        cancel.set_v_attach(ui::VAttach::Middle);
        cancel.set_h_attach(ui::HAttach::Center);
        let re = ui_container.add(cancel);
        txt.set_parent(&re);
        let tre = ui_container.add(txt);
        super::button_action(ui_container, re.clone(), Some(tre.clone()), |game, _| {
            game.screen_sys.replace_screen(Box::new(super::ServerList::new(None)));
        });
        elements.add(re);
        elements.add(tre);


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

    fn tick(&mut self,
            _delta: f64,
            renderer: &mut render::Renderer,
            ui_container: &mut ui::Container) -> Option<Box<super::Screen>> {

        let elements = self.elements.as_mut().unwrap();
        elements.logo.tick(renderer, ui_container);
        None
    }

    fn is_closable(&self) -> bool {
        true
    }
}
