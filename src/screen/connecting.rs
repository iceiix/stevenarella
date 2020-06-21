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

use crate::render;
use crate::ui;

pub struct Connecting {
    elements: Option<UIElements>,
    target: String,
}

struct UIElements {
    logo: ui::logo::Logo,
    _connect_msg: ui::TextRef,
    _msg: ui::TextRef,
    _disclaimer: ui::TextRef,
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
        let logo = ui::logo::Logo::new(renderer.resources.clone(), ui_container);

        let connect_msg = ui::TextBuilder::new()
            .text("Connecting to")
            .position(0.0, -16.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);

        let msg = ui::TextBuilder::new()
            .text(self.target.clone())
            .position(0.0, 16.0)
            .colour((255, 255, 85, 255))
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);

        // Disclaimer
        let disclaimer = ui::TextBuilder::new()
            .text("Not affiliated with Mojang/Minecraft")
            .position(5.0, 5.0)
            .colour((255, 200, 200, 255))
            .alignment(ui::VAttach::Bottom, ui::HAttach::Right)
            .create(ui_container);

        self.elements = Some(UIElements {
            logo,
            _disclaimer: disclaimer,
            _msg: msg,
            _connect_msg: connect_msg,
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
}
