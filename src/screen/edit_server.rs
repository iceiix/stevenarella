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

use std::collections::BTreeMap;
use std::fs;

use crate::render;
use crate::ui;

use serde_json::{self, Value};

pub struct EditServerEntry {
    elements: Option<UIElements>,
    entry_info: Option<(usize, String, String)>,
}

struct UIElements {
    logo: ui::logo::Logo,

    _name: ui::TextBoxRef,
    _address: ui::TextBoxRef,
    _done: ui::ButtonRef,
    _cancel: ui::ButtonRef,
}

impl EditServerEntry {
    pub fn new(entry_info: Option<(usize, String, String)>) -> EditServerEntry {
        EditServerEntry {
            elements: None,
            entry_info,
        }
    }

    fn save_servers(index: Option<usize>, name: &str, address: &str) {
        let mut servers_info = match fs::File::open("servers.json") {
            Ok(val) => serde_json::from_reader(val).unwrap(),
            Err(_) => {
                let mut info = BTreeMap::default();
                info.insert("servers".to_owned(), Value::Array(vec![]));
                Value::Object(info.into_iter().collect())
            }
        };

        let new_entry = {
            let mut entry = BTreeMap::default();
            entry.insert("name".to_owned(), Value::String(name.to_owned()));
            entry.insert("address".to_owned(), Value::String(address.to_owned()));
            Value::Object(entry.into_iter().collect())
        };

        {
            let servers = servers_info
                .as_object_mut()
                .unwrap()
                .get_mut("servers")
                .unwrap()
                .as_array_mut()
                .unwrap();
            if let Some(index) = index {
                *servers.get_mut(index).unwrap() = new_entry;
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
        let logo = ui::logo::Logo::new(renderer.resources.clone(), ui_container);

        // Name
        let server_name = ui::TextBoxBuilder::new()
            .input(self.entry_info.as_ref().map_or("", |v| &v.1))
            .position(0.0, -20.0)
            .size(400.0, 40.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);
        ui::TextBox::make_focusable(&server_name, ui_container);
        ui::TextBuilder::new()
            .text("Name:")
            .position(0.0, -18.0)
            .attach(&mut *server_name.borrow_mut());

        // Address
        let server_address = ui::TextBoxBuilder::new()
            .input(self.entry_info.as_ref().map_or("", |v| &v.2))
            .position(0.0, 40.0)
            .size(400.0, 40.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);
        ui::TextBox::make_focusable(&server_address, ui_container);
        ui::TextBuilder::new()
            .text("Address")
            .position(0.0, -18.0)
            .attach(&mut *server_address.borrow_mut());

        // Done
        let done = ui::ButtonBuilder::new()
            .position(110.0, 100.0)
            .size(200.0, 40.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);
        {
            let mut done = done.borrow_mut();
            let txt = ui::TextBuilder::new()
                .text("Done")
                .alignment(ui::VAttach::Middle, ui::HAttach::Center)
                .attach(&mut *done);
            done.add_text(txt);
            let index = self.entry_info.as_ref().map(|v| v.0);
            let server_name = server_name.clone();
            let server_address = server_address.clone();
            done.add_click_func(move |_, game| {
                Self::save_servers(
                    index,
                    &server_name.borrow().input,
                    &server_address.borrow().input,
                );
                game.screen_sys
                    .replace_screen(Box::new(super::ServerList::new(None)));
                true
            });
        }

        // Cancel
        let cancel = ui::ButtonBuilder::new()
            .position(-110.0, 100.0)
            .size(200.0, 40.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);
        {
            let mut cancel = cancel.borrow_mut();
            let txt = ui::TextBuilder::new()
                .text("Cancel")
                .alignment(ui::VAttach::Middle, ui::HAttach::Center)
                .attach(&mut *cancel);
            cancel.add_text(txt);
            cancel.add_click_func(|_, game| {
                game.screen_sys
                    .replace_screen(Box::new(super::ServerList::new(None)));
                true
            });
        }

        self.elements = Some(UIElements {
            logo,
            _name: server_name,
            _address: server_address,
            _done: done,
            _cancel: cancel,
        });
    }

    fn on_deactive(&mut self, _renderer: &mut render::Renderer, _ui_container: &mut ui::Container) {
        // Clean up
        self.elements = None
    }

    fn tick(
        &mut self,
        _delta: f64,
        renderer: &mut render::Renderer,
        _ui_container: &mut ui::Container,
    ) -> Option<Box<dyn super::Screen>> {
        let elements = self.elements.as_mut().unwrap();
        elements.logo.tick(renderer);
        None
    }

    fn is_closable(&self) -> bool {
        true
    }
}
