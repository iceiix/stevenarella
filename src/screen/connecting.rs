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

use ui;
use render;
use format::{self, Component, TextComponent};

pub struct Connecting {
    elements: Option<UIElements>,
    target: String,
}

struct UIElements {
    logo: ui::logo::Logo,
    elements: ui::Collection,
}


impl Connecting {
    pub fn new(target: &str) -> Connecting {
        Connecting {
            elements: None,
            target: target.to_owned(),
        }
    }
}

impl super::Screen for Connecting {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let logo = ui::logo::Logo::new(renderer.resources.clone(), renderer, ui_container);
        let mut elements = ui::Collection::new();

        let mut connect_msg = ui::Formatted::new(
            renderer,
            Component::Text(TextComponent::new("Connecting to")),
            0.0, -16.0
        );
        connect_msg.set_v_attach(ui::VAttach::Middle);
        connect_msg.set_h_attach(ui::HAttach::Center);
        elements.add(ui_container.add(connect_msg));

        let mut msg = TextComponent::new(&self.target);
        msg.modifier.color = Some(format::Color::Yellow);
        let mut server_msg = ui::Formatted::new(
            renderer,
            Component::Text(msg),
            0.0, 16.0
        );
        server_msg.set_v_attach(ui::VAttach::Middle);
        server_msg.set_h_attach(ui::HAttach::Center);
        elements.add(ui_container.add(server_msg));

        // Disclaimer
        let mut warn = ui::Text::new(renderer,
                                     "Not affiliated with Mojang/Minecraft",
                                     5.0,
                                     5.0,
                                     255,
                                     200,
                                     200);
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

    fn tick(&mut self,
            _delta: f64,
            renderer: &mut render::Renderer,
            ui_container: &mut ui::Container) -> Option<Box<super::Screen>>{
        let elements = self.elements.as_mut().unwrap();

        elements.logo.tick(renderer, ui_container);
        None
    }
}
