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
use std::thread;
use std::sync::mpsc;
use std::rc::Rc;
use std::cell::RefCell;

use ui;
use render;
use format;
use format::{Component, TextComponent};
use protocol;

use serde_json;
use time;
use image;
use rustc_serialize::base64::FromBase64;
use rand;
use rand::Rng;

pub struct ServerList {
    elements: Option<UIElements>,
    disconnect_reason: Option<Component>,

    needs_reload: Rc<RefCell<bool>>,
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

    motd: ui::ElementRef<ui::Formatted>,
    ping: ui::ElementRef<ui::Image>,
    players: ui::ElementRef<ui::Text>,
    version: ui::ElementRef<ui::Formatted>,

    icon: ui::ElementRef<ui::Image>,
    icon_texture: Option<String>,

    done_ping: bool,
    recv: mpsc::Receiver<PingInfo>,
}

struct PingInfo {
    motd: format::Component,
    ping: time::Duration,
    exists: bool,
    online: i32,
    max: i32,
    protocol_version: i32,
    protocol_name: String,
    favicon: Option<image::DynamicImage>,
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
    pub fn new(disconnect_reason: Option<Component>) -> ServerList {
        ServerList {
            elements: None,
            disconnect_reason: disconnect_reason,
            needs_reload: Rc::new(RefCell::new(false)),
        }
    }

    fn reload_server_list(&mut self,
                          renderer: &mut render::Renderer,
                          ui_container: &mut ui::Container) {
        let elements = self.elements.as_mut().unwrap();
        *self.needs_reload.borrow_mut() = false;
        {
            // Clean up previous list entries and icons.
            let mut tex = renderer.get_textures_ref().write().unwrap();
            for server in &mut elements.servers {
                server.collection.remove_all(ui_container);
                if let Some(ref icon) = server.icon_texture {
                    tex.remove_dynamic(&icon);
                }
            }
        }
        elements.servers.clear();

        let file = match fs::File::open("servers.json") {
            Ok(val) => val,
            Err(_) => return,
        };
        let servers_info: serde_json::Value = serde_json::from_reader(file).unwrap();
        let servers = servers_info.find("servers").unwrap().as_array().unwrap();
        let mut offset = 0.0;

        // Default icon whilst we ping the servers or if the server doesn't provide one
        let default_icon = render::Renderer::get_texture(renderer.get_textures_ref(),
                                                         "misc/unknown_server");
        // General gui icons
        let icons = render::Renderer::get_texture(renderer.get_textures_ref(), "gui/icons");

        for (index, svr) in servers.iter().enumerate() {
            let name = svr.find("name").unwrap().as_string().unwrap().to_owned();
            let address = svr.find("address").unwrap().as_string().unwrap().to_owned();

            let solid = render::Renderer::get_texture(renderer.get_textures_ref(), "steven:solid");

            // Everything is attached to this
            let mut back = ui::Image::new(solid,
                                          0.0,
                                          offset * 100.0,
                                          700.0,
                                          100.0,
                                          0.0,
                                          0.0,
                                          1.0,
                                          1.0,
                                          0,
                                          0,
                                          0);
            back.set_a(100);
            back.set_v_attach(ui::VAttach::Middle);
            back.set_h_attach(ui::HAttach::Center);

            let (send, recv) = mpsc::channel::<PingInfo>();
            let mut server = Server {
                collection: ui::Collection::new(),
                back: ui_container.add(back),
                offset: offset,
                y: 0.0,
                done_ping: false,
                recv: recv,

                motd: Default::default(),
                ping: Default::default(),
                players: Default::default(),
                version: Default::default(),

                icon: Default::default(),
                icon_texture: None,
            };
            server.collection.add(server.back.clone());
            server.update_position();
            // Make whole entry interactable
            {
                let back = ui_container.get_mut(&server.back);
                let back_ref = server.back.clone();
                let address = address.clone();
                back.add_hover_func(move |over, _, ui_container| {
                    let back = ui_container.get_mut(&back_ref);
                    back.set_a(if over {
                        200
                    } else {
                        100
                    });
                });

                back.add_click_func(move |game, _| {
                    game.screen_sys.replace_screen(Box::new(super::connecting::Connecting::new(&address)));
                    game.connect_to(&address);
                });
            }

            // Server name
            let mut text = ui::Text::new(renderer, &name, 100.0, 5.0, 255, 255, 255);
            text.set_parent(&server.back);
            server.collection.add(ui_container.add(text));

            // Server icon
            let mut icon = ui::Image::new(
                default_icon.clone(),
                 5.0, 5.0, 90.0, 90.0,
                 0.0, 0.0, 1.0, 1.0,
                 255, 255, 255
             );
            icon.set_parent(&server.back);
            server.icon = server.collection.add(ui_container.add(icon));

            // Ping indicator
            let mut ping = ui::Image::new(
                icons.clone(),
                5.0, 5.0, 20.0, 16.0,
                0.0, 56.0 / 256.0, 10.0 / 256.0, 8.0 / 256.0,
                255, 255, 255
            );
            ping.set_h_attach(ui::HAttach::Right);
            ping.set_parent(&server.back);
            server.ping = server.collection.add(ui_container.add(ping));

            // Player count
            let mut players = ui::Text::new(renderer, "???", 30.0, 5.0, 255, 255, 255);
            players.set_h_attach(ui::HAttach::Right);
            players.set_parent(&server.back);
            server.players = server.collection.add(ui_container.add(players));

            // Server's message of the day
            let mut motd =
                ui::Formatted::with_width_limit(renderer,
                                                Component::Text(TextComponent::new("Connecting.\
                                                                                    ..")),
                                                100.0,
                                                23.0,
                                                700.0 - (90.0 + 10.0 + 5.0));
            motd.set_parent(&server.back);
            server.motd = server.collection.add(ui_container.add(motd));

            // Version information
            let mut version =
                ui::Formatted::with_width_limit(renderer,
                                                Component::Text(TextComponent::new("")),
                                                100.0,
                                                5.0,
                                                700.0 - (90.0 + 10.0 + 5.0));
            version.set_v_attach(ui::VAttach::Bottom);
            version.set_parent(&server.back);
            server.version = server.collection.add(ui_container.add(version));

            // Delete entry button
            let (mut del, mut txt) = super::new_button_text(renderer, "X", 0.0, 0.0, 25.0, 25.0);
            del.set_v_attach(ui::VAttach::Bottom);
            del.set_h_attach(ui::HAttach::Right);
            del.set_parent(&server.back);
            let re = ui_container.add(del);
            txt.set_parent(&re);
            let tre = ui_container.add(txt);
            super::button_action(ui_container, re.clone(), Some(tre.clone()), |_,_| {}); // TOOO: delete entry
            server.collection.add(re);
            server.collection.add(tre);

            // Edit entry button
            let (mut edit, mut txt) = super::new_button_text(renderer, "E", 25.0, 0.0, 25.0, 25.0);
            edit.set_v_attach(ui::VAttach::Bottom);
            edit.set_h_attach(ui::HAttach::Right);
            edit.set_parent(&server.back);
            let re = ui_container.add(edit);
            txt.set_parent(&re);
            let tre = ui_container.add(txt);
            let index = index;
            let sname = name.clone();
            let saddr = address.clone();
            super::button_action(ui_container, re.clone(), Some(tre.clone()), move |game,_|{
                let sname = sname.clone();
                let saddr = saddr.clone();
                game.screen_sys.replace_screen(Box::new(super::edit_server::EditServerEntry::new(
                    Some((index, sname, saddr))
                )));
            });
            server.collection.add(re);
            server.collection.add(tre);

            elements.servers.push(server);
            offset += 1.0;

            // Don't block the main thread whilst pinging the server
            thread::spawn(move || {
                match protocol::Conn::new(&address).and_then(|conn| conn.do_status()) {
                    Ok(res) => {
                        let mut desc = res.0.description;
                        format::convert_legacy(&mut desc);
                        let favicon = if let Some(icon) = res.0.favicon {
                            let data = icon["data:image/png;base64,".len()..]
                                           .from_base64()
                                           .unwrap();
                            Some(image::load_from_memory(&data).unwrap())
                        } else {
                            None
                        };
                        drop(send.send(PingInfo {
                            motd: desc,
                            ping: res.1,
                            exists: true,
                            online: res.0.players.online,
                            max: res.0.players.max,
                            protocol_version: res.0.version.protocol,
                            protocol_name: res.0.version.name,
                            favicon: favicon,
                        }));
                    }
                    Err(err) => {
                        let e = format!("{}", err);
                        let mut msg = TextComponent::new(&e);
                        msg.modifier.color = Some(format::Color::Red);
                        drop(send.send(PingInfo {
                            motd: Component::Text(msg),
                            ping: time::Duration::seconds(99999),
                            exists: false,
                            online: 0,
                            max: 0,
                            protocol_version: 0,
                            protocol_name: "".to_owned(),
                            favicon: None,
                        }));
                    }
                }
            });
        }
    }
}

impl super::Screen for ServerList {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let logo = ui::logo::Logo::new(renderer.resources.clone(), renderer, ui_container);
        let mut elements = ui::Collection::new();

        // Refresh the server list
        let (mut refresh, mut txt) = super::new_button_text(renderer,
                                                            "Refresh",
                                                            300.0,
                                                            -50.0 - 15.0,
                                                            100.0,
                                                            30.0);
        refresh.set_v_attach(ui::VAttach::Middle);
        refresh.set_h_attach(ui::HAttach::Center);
        let re = ui_container.add(refresh);
        txt.set_parent(&re);
        let tre = ui_container.add(txt);
        let nr = self.needs_reload.clone();
        super::button_action(ui_container,
                             re.clone(),
                             Some(tre.clone()),
                             move |_, _| {
                                 *nr.borrow_mut() = true;
                             });
        elements.add(re);
        elements.add(tre);

        // Add a new server to the list
        let (mut add, mut txt) = super::new_button_text(
            renderer, "Add",
            200.0, -50.0 - 15.0, 100.0, 30.0
        );
        add.set_v_attach(ui::VAttach::Middle);
        add.set_h_attach(ui::HAttach::Center);
        let re = ui_container.add(add);
        txt.set_parent(&re);
        let tre = ui_container.add(txt);
        super::button_action(ui_container, re.clone(), Some(tre.clone()), |game, _|{
            game.screen_sys.replace_screen(Box::new(super::edit_server::EditServerEntry::new(
                None
            )));
        });
        elements.add(re);
        elements.add(tre);

        // Options menu
        let mut options = ui::Button::new(5.0, 25.0, 40.0, 40.0);
        options.set_v_attach(ui::VAttach::Bottom);
        options.set_h_attach(ui::HAttach::Right);
        let re = ui_container.add(options);
        let mut cog = ui::Image::new(render::Renderer::get_texture(renderer.get_textures_ref(),
                                                                   "steven:gui/cog"),
                                     0.0,
                                     0.0,
                                     40.0,
                                     40.0,
                                     0.0,
                                     0.0,
                                     1.0,
                                     1.0,
                                     255,
                                     255,
                                     255);
        cog.set_parent(&re);
        cog.set_v_attach(ui::VAttach::Middle);
        cog.set_h_attach(ui::HAttach::Center);
        super::button_action(ui_container, re.clone(), None, | game, _ | {
            game.screen_sys.add_screen(Box::new(super::SettingsMenu::new(game.console.clone(), false)));
        });
        elements.add(re);
        elements.add(ui_container.add(cog));

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

        // If we are kicked from a server display the reason
        if let Some(ref disconnect_reason) = self.disconnect_reason {
            let mut dis_msg = ui::Text::new(renderer, "Disconnected", 0.0, 32.0, 255, 0, 0);
            dis_msg.set_h_attach(ui::HAttach::Center);
            let mut dis = ui::Formatted::with_width_limit(renderer,
                                                          disconnect_reason.clone(),
                                                          0.0,
                                                          48.0,
                                                          600.0);
            dis.set_h_attach(ui::HAttach::Center);
            let mut back =
                ui::Image::new(render::Renderer::get_texture(renderer.get_textures_ref(),
                                                             "steven:solid"),
                               0.0,
                               30.0,
                               dis.get_width().max(dis_msg.get_width()) + 4.0,
                               dis.get_height() + 4.0 + 16.0,
                               0.0,
                               0.0,
                               1.0,
                               1.0,
                               0,
                               0,
                               0);
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
        // Clean up
        {
            let elements = self.elements.as_mut().unwrap();
            elements.logo.remove(ui_container);
            elements.elements.remove_all(ui_container);
            let mut tex = renderer.get_textures_ref().write().unwrap();
            for server in &mut elements.servers {
                if let Some(ref icon) = server.icon_texture {
                    tex.remove_dynamic(&icon);
                }
                server.collection.remove_all(ui_container);
            }
            elements.servers.clear();
        }
        self.elements = None
    }

    fn tick(&mut self,
            delta: f64,
            renderer: &mut render::Renderer,
            ui_container: &mut ui::Container) -> Option<Box<super::Screen>> {
        if *self.needs_reload.borrow() {
            self.reload_server_list(renderer, ui_container);
        }
        let elements = self.elements.as_mut().unwrap();

        elements.logo.tick(renderer, ui_container);

        for s in &mut elements.servers {
            // Animate the entries
            {
                let back = ui_container.get_mut(&s.back);
                let dy = s.y - back.get_y();
                if dy * dy > 1.0 {
                    let y = back.get_y();
                    back.set_y(y + delta * dy * 0.1);
                } else {
                    back.set_y(s.y);
                }
            }

            // Keep checking to see if the server has finished being
            // pinged
            if !s.done_ping {
                match s.recv.try_recv() {
                    Ok(res) => {
                        s.done_ping = true;
                        {
                            let motd = ui_container.get_mut(&s.motd);
                            motd.set_component(renderer, res.motd);
                        }
                        {
                            let ping = ui_container.get_mut(&s.ping);
                            // Selects the icon for the given ping range
                            let y = match res.ping.num_milliseconds() {
                                _x @ 0 ... 75 => 16.0 / 256.0,
                                _x @ 76 ... 150 => 24.0 / 256.0,
                                _x @ 151 ... 225 => 32.0 / 256.0,
                                _x @ 226 ... 350 => 40.0 / 256.0,
                                _x @ 351 ... 999 => 48.0 / 256.0,
                                _ => 56.0 / 256.0,
                            };
                            ping.set_t_y(y);
                        }
                        if res.exists {
                            {
                                let players = ui_container.get_mut(&s.players);
                                let txt = if res.protocol_version == protocol::SUPPORTED_PROTOCOL {
                                    players.set_g(255);
                                    players.set_b(255);
                                    format!("{}/{}", res.online, res.max)
                                } else {
                                    players.set_g(85);
                                    players.set_b(85);
                                    format!("Out of date {}/{}", res.online, res.max)
                                };
                                players.set_text(renderer, &txt);
                            }
                            {
                                let version = ui_container.get_mut(&s.version);
                                let mut txt = TextComponent::new(&res.protocol_name);
                                txt.modifier.color = Some(format::Color::Yellow);
                                let mut msg = Component::Text(txt);
                                format::convert_legacy(&mut msg);
                                version.set_component(renderer, msg);
                            }
                        }
                        if let Some(favicon) = res.favicon {
                            let name: String = rand::thread_rng()
                                                   .gen_ascii_chars()
                                                   .take(30)
                                                   .collect();
                            let tex = renderer.get_textures_ref();
                            s.icon_texture = Some(name.clone());
                            let icon_tex = tex.write()
                                              .unwrap()
                                              .put_dynamic(&name, favicon);
                            let icon = ui_container.get_mut(&s.icon);
                            icon.set_texture(icon_tex);
                        }
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        s.done_ping = true;
                        let motd = ui_container.get_mut(&s.motd);
                        let mut txt = TextComponent::new("Channel dropped");
                        txt.modifier.color = Some(format::Color::Red);
                        motd.set_component(renderer, Component::Text(txt));
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn on_scroll(&mut self, _: f64, y: f64) {
        let elements = self.elements.as_mut().unwrap();
        if elements.servers.is_empty() {
            return;
        }
        let mut diff = y / 1.0;
        {
            let last = elements.servers.last().unwrap();
            if last.offset + diff <= 2.0 {
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
