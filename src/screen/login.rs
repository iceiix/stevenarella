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

use std::sync::{Arc, Mutex};

use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

use rand::{self, Rng};

use ui;
use render;
use console;
use protocol;
use protocol::mojang;
use auth;

pub struct Login {
    elements: Option<UIElements>,
    console: Arc<Mutex<console::Console>>,
}

struct UIElements {
    logo: ui::logo::Logo,
    elements: ui::Collection,

    login_btn: ui::ElementRef<ui::Button>,
    login_btn_text: ui::ElementRef<ui::Text>,
    login_error: ui::ElementRef<ui::Text>,
    username_txt: ui::ElementRef<ui::TextBox>,
    password_txt: ui::ElementRef<ui::TextBox>,
    try_login: Rc<Cell<bool>>,
    refresh: bool,
    login_res: Option<mpsc::Receiver<Result<mojang::Profile, protocol::Error>>>,

    profile: mojang::Profile,
}


impl Login {
    pub fn new(console: Arc<Mutex<console::Console>>) -> Login {
        Login { elements: None, console: console }
    }
}

impl super::Screen for Login {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let logo = ui::logo::Logo::new(renderer.resources.clone(), renderer, ui_container);
        let mut elements = ui::Collection::new();

        let try_login = Rc::new(Cell::new(false));

        // Login
        let (mut login, mut txt) = super::new_button_text(renderer, "Login", 0.0, 100.0, 400.0, 40.0);
        login.set_v_attach(ui::VAttach::Middle);
        login.set_h_attach(ui::HAttach::Center);
        let re = ui_container.add(login);
        txt.set_parent(&re);
        let tre = ui_container.add(txt);
        let tl = try_login.clone();
        super::button_action(ui_container,
                             re.clone(),
                             Some(tre.clone()),
                             move |_, _| {
            tl.set(true);
        });
        let login_btn = elements.add(re);
        let login_btn_text = elements.add(tre);

        // Login Error
        let mut login_error = ui::Text::new(renderer, "", 0.0, 150.0, 255, 50, 50);
        login_error.set_v_attach(ui::VAttach::Middle);
        login_error.set_h_attach(ui::HAttach::Center);
        let login_error = elements.add(ui_container.add(login_error));

        // Username
        let mut username = ui::TextBox::new(renderer, "", 0.0, -20.0, 400.0, 40.0);
        username.set_v_attach(ui::VAttach::Middle);
        username.set_h_attach(ui::HAttach::Center);
        username.add_submit_func(|_, ui| {
            ui.cycle_focus();
        });
        let ure = ui_container.add(username);
        let mut username_label = ui::Text::new(renderer, "Username/Email:", 0.0, -18.0, 255, 255, 255);
        username_label.set_parent(&ure);
        let username_txt = elements.add(ure);
        elements.add(ui_container.add(username_label));

        // Password
        let mut password = ui::TextBox::new(renderer, "", 0.0, 40.0, 400.0, 40.0);
        password.set_v_attach(ui::VAttach::Middle);
        password.set_h_attach(ui::HAttach::Center);
        password.set_password(true);
        let tl = try_login.clone();
        password.add_submit_func(move |_, _| {
            tl.set(true);
        });
        let pre = ui_container.add(password);
        let mut password_label = ui::Text::new(renderer, "Password:", 0.0, -18.0, 255, 255, 255);
        password_label.set_parent(&pre);
        let password_txt = elements.add(pre);
        elements.add(ui_container.add(password_label));

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

        let console = self.console.lock().unwrap();
        let profile = mojang::Profile {
            username: console.get(auth::CL_USERNAME).clone(),
            id: console.get(auth::CL_UUID).clone(),
            access_token: console.get(auth::AUTH_TOKEN).clone(),
        };
        let refresh = profile.is_complete();
        try_login.set(refresh);

        self.elements = Some(UIElements {
            logo: logo,
            elements: elements,
            profile: profile,
            login_btn: login_btn,
            login_btn_text: login_btn_text,
            login_error: login_error,
            try_login: try_login,
            refresh: refresh,
            login_res: None,

            username_txt: username_txt,
            password_txt: password_txt,
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

        if elements.try_login.get() && elements.login_res.is_none() {
            elements.try_login.set(false);
            let (tx, rx) = mpsc::channel();
            elements.login_res = Some(rx);
            {
                let btn = ui_container.get_mut(&elements.login_btn);
                btn.set_disabled(true);
            }
            {
                let txt = ui_container.get_mut(&elements.login_btn_text);
                txt.set_text(renderer, "Logging in...");
            }
            let mut console = self.console.lock().unwrap();
            let mut client_token = console.get(auth::AUTH_CLIENT_TOKEN).clone();
            if client_token.is_empty() {
                client_token = rand::thread_rng().gen_ascii_chars().take(20).collect::<String>();
                console.set(auth::AUTH_CLIENT_TOKEN, client_token);
            }
            let client_token = console.get(auth::AUTH_CLIENT_TOKEN).clone();
            let username = {
                let txt = ui_container.get(&elements.username_txt);
                txt.get_input()
            };
            let password = {
                let txt = ui_container.get(&elements.password_txt);
                txt.get_input()
            };
            let refresh = elements.refresh;
            let profile = elements.profile.clone();
            thread::spawn(move || {
                if refresh {
                    tx.send(profile.refresh(&client_token)).unwrap();
                } else {
                    tx.send(mojang::Profile::login(&username, &password, &client_token)).unwrap();
                }
            });
        }
        let mut done = false;
        if let Some(rx) = elements.login_res.as_ref() {
            if let Ok(res) = rx.try_recv() {
                done = true;
                {
                    let btn = ui_container.get_mut(&elements.login_btn);
                    btn.set_disabled(false);
                }
                {
                    let txt = ui_container.get_mut(&elements.login_btn_text);
                    txt.set_text(renderer, "Login");
                }
                match res {
                    Ok(val) => {
                        let mut console = self.console.lock().unwrap();
                        console.set(auth::CL_USERNAME, val.username.clone());
                        console.set(auth::CL_UUID, val.id.clone());
                        console.set(auth::AUTH_TOKEN, val.access_token.clone());
                        elements.profile = val;
                        return Some(Box::new(super::ServerList::new(None)));
                    },
                    Err(err) => {
                        let login_error = ui_container.get_mut(&elements.login_error);
                        login_error.set_text(renderer, &format!("{}", err));
                    },
                }
            }
        }
        if done {
            elements.login_res = None;
        }

        elements.logo.tick(renderer, ui_container);
        None
    }
}
