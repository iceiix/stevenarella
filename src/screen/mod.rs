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

mod server_list;
pub use self::server_list::*;
mod login;
pub use self::login::*;

pub mod connecting;
pub mod delete_server;
pub mod edit_server;

pub mod settings_menu;
pub use self::settings_menu::{AudioSettingsMenu, SettingsMenu, VideoSettingsMenu};

use crate::render;
use crate::ui;

pub trait Screen {
    // Called once
    fn init(&mut self, _renderer: &mut render::Renderer, _ui_container: &mut ui::Container) {}
    fn deinit(&mut self, _renderer: &mut render::Renderer, _ui_container: &mut ui::Container) {}

    // May be called multiple times
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container);
    fn on_deactive(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container);

    // Called every frame the screen is active
    fn tick(
        &mut self,
        delta: f64,
        renderer: &mut render::Renderer,
        ui_container: &mut ui::Container,
    ) -> Option<Box<dyn Screen>>;

    // Events
    fn on_scroll(&mut self, _x: f64, _y: f64) {}

    fn is_closable(&self) -> bool {
        false
    }
}

struct ScreenInfo {
    screen: Box<dyn Screen>,
    init: bool,
    active: bool,
}

#[derive(Default)]
pub struct ScreenSystem {
    screens: Vec<ScreenInfo>,
    remove_queue: Vec<ScreenInfo>,
}

impl ScreenSystem {
    pub fn new() -> ScreenSystem {
        Default::default()
    }

    pub fn add_screen(&mut self, screen: Box<dyn Screen>) {
        self.screens.push(ScreenInfo {
            screen,
            init: false,
            active: false,
        });
    }

    pub fn pop_screen(&mut self) {
        if let Some(screen) = self.screens.pop() {
            self.remove_queue.push(screen);
        }
    }

    pub fn replace_screen(&mut self, screen: Box<dyn Screen>) {
        self.pop_screen();
        self.add_screen(screen);
    }

    pub fn is_current_closable(&self) -> bool {
        if let Some(last) = self.screens.last() {
            last.screen.is_closable()
        } else {
            true
        }
    }

    pub fn tick(
        &mut self,
        delta: f64,
        renderer: &mut render::Renderer,
        ui_container: &mut ui::Container,
    ) {
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
        let swap = {
            let current = self.screens.last_mut().unwrap();
            if !current.init {
                current.init = true;
                current.screen.init(renderer, ui_container);
            }
            if !current.active {
                current.active = true;
                current.screen.on_active(renderer, ui_container);
            }
            current.screen.tick(delta, renderer, ui_container)
        };
        // Handle current
        if let Some(swap) = swap {
            self.replace_screen(swap);
        }
    }

    pub fn on_scroll(&mut self, x: f64, y: f64) {
        if self.screens.is_empty() {
            return;
        }
        let current = self.screens.last_mut().unwrap();
        current.screen.on_scroll(x, y);
    }
}
