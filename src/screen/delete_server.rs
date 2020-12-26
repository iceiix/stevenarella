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
use std_or_web::fs;

use crate::render;
use crate::ui;

use serde_json::{self, Value};

pub struct DeleteServerEntry {
    elements: Option<UIElements>,
    index: usize,
    name: String,
    address: String,
}

struct UIElements {
    logo: ui::logo::Logo,

    _prompt: ui::TextRef,
    _confirm: ui::ButtonRef,
    _cancel: ui::ButtonRef,
}

impl DeleteServerEntry {
    pub fn new(index: usize, name: &str, address: &str) -> DeleteServerEntry {
        DeleteServerEntry {
            elements: None,
            index,
            name: name.to_string(),
            address: address.to_string(),
        }
    }

    fn delete_server(index: usize) {
        let mut servers_info = match fs::File::open("servers.json") {
            Ok(val) => serde_json::from_reader(val).unwrap(),
            Err(_) => {
                let mut info = BTreeMap::default();
                info.insert("servers".to_owned(), Value::Array(vec![]));
                Value::Object(info.into_iter().collect())
            }
        };

        {
            let servers = servers_info
                .as_object_mut()
                .unwrap()
                .get_mut("servers")
                .unwrap()
                .as_array_mut()
                .unwrap();
            servers.remove(index);
        }

        let mut out = fs::File::create("servers.json").unwrap();
        serde_json::to_writer_pretty(&mut out, &servers_info).unwrap();
    }
}

impl super::Screen for DeleteServerEntry {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let logo = ui::logo::Logo::new(renderer.resources.clone(), ui_container);

        // Prompt
        let prompt = ui::TextBuilder::new()
            .text(format!(
                "Are you sure you wish to delete {} {}?",
                self.name, self.address
            ))
            .position(0.0, 40.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);

        // Confirm
        let confirm = ui::ButtonBuilder::new()
            .position(110.0, 100.0)
            .size(200.0, 40.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);
        {
            let mut confirm = confirm.borrow_mut();
            let txt = ui::TextBuilder::new()
                .text("Confirm")
                .alignment(ui::VAttach::Middle, ui::HAttach::Center)
                .attach(&mut *confirm);
            confirm.add_text(txt);
            let index = self.index;
            confirm.add_click_func(move |_, game| {
                Self::delete_server(index);
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
            _prompt: prompt,
            _confirm: confirm,
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
