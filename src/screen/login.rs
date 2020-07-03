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

use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

use rand::{self, Rng};

use crate::auth;
use crate::console;
use crate::protocol;
use crate::protocol::mojang;
use crate::render;
use crate::ui;

pub struct Login {
    elements: Option<UIElements>,
    vars: Rc<console::Vars>,
}

struct UIElements {
    logo: ui::logo::Logo,

    login_btn: ui::ButtonRef,
    login_btn_text: ui::TextRef,
    login_error: ui::TextRef,
    username_txt: ui::TextBoxRef,
    password_txt: ui::TextBoxRef,
    _disclaimer: ui::TextRef,
    try_login: Rc<Cell<bool>>,
    refresh: bool,
    login_res: Option<mpsc::Receiver<Result<mojang::Profile, protocol::Error>>>,

    profile: mojang::Profile,
}

impl Login {
    pub fn new(vars: Rc<console::Vars>) -> Login {
        Login {
            elements: None,
            vars,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl super::Screen for Login {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let logo = ui::logo::Logo::new(renderer.resources.clone(), ui_container);

        let try_login = Rc::new(Cell::new(false));

        // Login
        let login_btn = ui::ButtonBuilder::new()
            .position(0.0, 100.0)
            .size(400.0, 40.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);
        let login_btn_text = ui::TextBuilder::new()
            .text("Login")
            .position(0.0, 0.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .attach(&mut *login_btn.borrow_mut());
        {
            let mut btn = login_btn.borrow_mut();
            btn.add_text(login_btn_text.clone());
            let tl = try_login.clone();
            btn.add_click_func(move |_, _| {
                tl.set(true);
                true
            });
        }

        // Login Error
        let login_error = ui::TextBuilder::new()
            .text("")
            .position(0.0, 150.0)
            .colour((255, 50, 50, 255))
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);

        // Username
        let username_txt = ui::TextBoxBuilder::new()
            .position(0.0, -20.0)
            .size(400.0, 40.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .create(ui_container);
        ui::TextBox::make_focusable(&username_txt, ui_container);
        ui::TextBuilder::new()
            .text("Username/Email:")
            .position(0.0, -18.0)
            .attach(&mut *username_txt.borrow_mut());

        // Password
        let password_txt = ui::TextBoxBuilder::new()
            .position(0.0, 40.0)
            .size(400.0, 40.0)
            .alignment(ui::VAttach::Middle, ui::HAttach::Center)
            .password(true)
            .create(ui_container);
        ui::TextBox::make_focusable(&password_txt, ui_container);
        ui::TextBuilder::new()
            .text("Password:")
            .position(0.0, -18.0)
            .attach(&mut *password_txt.borrow_mut());
        let tl = try_login.clone();
        password_txt.borrow_mut().add_submit_func(move |_, _| {
            tl.set(true);
        });

        // Disclaimer
        let disclaimer = ui::TextBuilder::new()
            .text("Not affiliated with Mojang/Minecraft")
            .position(5.0, 5.0)
            .colour((255, 200, 200, 255))
            .alignment(ui::VAttach::Bottom, ui::HAttach::Right)
            .create(ui_container);

        let profile = mojang::Profile {
            username: self.vars.get(auth::CL_USERNAME).clone(),
            id: self.vars.get(auth::CL_UUID).clone(),
            access_token: self.vars.get(auth::AUTH_TOKEN).clone(),
        };
        let refresh = profile.is_complete();
        try_login.set(refresh);

        self.elements = Some(UIElements {
            logo,
            profile,
            login_btn,
            login_btn_text,
            login_error,
            try_login,
            refresh,
            login_res: None,

            _disclaimer: disclaimer,

            username_txt,
            password_txt,
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

        if elements.try_login.get() && elements.login_res.is_none() {
            elements.try_login.set(false);
            let (tx, rx) = mpsc::channel();
            elements.login_res = Some(rx);
            elements.login_btn.borrow_mut().disabled = true;
            elements.login_btn_text.borrow_mut().text = "Logging in...".into();
            let mut client_token = self.vars.get(auth::AUTH_CLIENT_TOKEN).clone();
            if client_token.is_empty() {
                client_token = std::iter::repeat(())
                    .map(|()| rand::thread_rng().sample(&rand::distributions::Alphanumeric))
                    .take(20)
                    .collect();
                self.vars.set(auth::AUTH_CLIENT_TOKEN, client_token);
            }
            let client_token = self.vars.get(auth::AUTH_CLIENT_TOKEN).clone();
            let username = elements.username_txt.borrow().input.clone();
            let password = elements.password_txt.borrow().input.clone();
            let refresh = elements.refresh;
            let profile = elements.profile.clone();
            thread::spawn(move || {
                if refresh {
                    tx.send(profile.refresh(&client_token)).unwrap();
                } else {
                    tx.send(mojang::Profile::login(&username, &password, &client_token))
                        .unwrap();
                }
            });
        }
        let mut done = false;
        if let Some(rx) = elements.login_res.as_ref() {
            if let Ok(res) = rx.try_recv() {
                done = true;
                elements.login_btn.borrow_mut().disabled = false;
                elements.login_btn_text.borrow_mut().text = "Login".into();
                match res {
                    Ok(val) => {
                        self.vars.set(auth::CL_USERNAME, val.username.clone());
                        self.vars.set(auth::CL_UUID, val.id.clone());
                        self.vars.set(auth::AUTH_TOKEN, val.access_token.clone());
                        elements.profile = val;
                        return Some(Box::new(super::ServerList::new(None)));
                    }
                    Err(err) => {
                        elements.login_error.borrow_mut().text = format!("{}", err);
                    }
                }
            }
        }
        if done {
            elements.login_res = None;
        }

        elements.logo.tick(renderer);
        None
    }
}
