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
pub mod settings_menu;

pub use self::login::*;
pub mod connecting;
pub mod edit_server;
pub use self::settings_menu::{SettingsMenu, VideoSettingsMenu, AudioSettingsMenu};

use render;
use ui;

#[allow(unused_variables)]
pub trait Screen {
    // Called once
    fn init(&mut self, _renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
    }
    fn deinit(&mut self, _renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
    }

    // May be called multiple times
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container);
    fn on_deactive(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container);

    // Called every frame the screen is active
    fn tick(&mut self,
            delta: f64,
            renderer: &mut render::Renderer,
            ui_container: &mut ui::Container) -> Option<Box<Screen>>;

    // Events
    fn on_scroll(&mut self, x: f64, y: f64) {
    }

    fn is_closable(&self) -> bool {
        false
    }
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

    pub fn replace_screen(&mut self, screen: Box<Screen>) {
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

    pub fn tick(&mut self,
                delta: f64,
                renderer: &mut render::Renderer,
                ui_container: &mut ui::Container) {
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

pub fn new_button_text(renderer: &mut render::Renderer,
                       val: &str,
                       x: f64,
                       y: f64,
                       w: f64,
                       h: f64)
                       -> (ui::Button, ui::Text) {
    let btn = ui::Button::new(x, y, w, h);
    let mut text = ui::Text::new(renderer, val, 0.0, 0.0, 255, 255, 255);
    text.set_v_attach(ui::VAttach::Middle);
    text.set_h_attach(ui::HAttach::Center);
    (btn, text)
}

pub fn button_action<F: Fn(&mut ::Game, &mut ui::Container) + 'static>(ui_container: &mut ui::Container,
                     btn: ui::ElementRef<ui::Button>,
                     txt: Option<ui::ElementRef<ui::Text>>,
                     click: F) {
    let button = ui_container.get_mut(&btn);
    button.add_hover_func(move |over, _, ui_container| {
        let disabled = {
            let button = ui_container.get_mut(&btn);
            button.is_disabled()
        };
        let txt = txt.clone();
        if let Some(txt) = txt {
            let text = ui_container.get_mut(&txt);
            text.set_b(if over && !disabled {
                160
            } else {
                255
            });
        }
    });
    button.add_click_func(click);
}
