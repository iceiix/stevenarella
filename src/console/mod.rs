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

use std::marker::PhantomData;
use std::collections::HashMap;
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::fs;
use std::io::{BufWriter, Write, BufRead, BufReader};
use log;

use ui;
use render;
use format::{Component, TextComponent, Color};

const FILTERED_CRATES: &'static [&'static str] = &[
    "hyper",
    "mime",
];

pub struct CVar<T: Sized + Any + 'static> {
    pub name: &'static str,
    pub ty: PhantomData<T>,
    pub description: &'static str,
    pub mutable: bool,
    pub serializable: bool,
    pub default: &'static Fn() -> T,
}

impl Var for CVar<i64> {
    fn serialize(&self, val: &Box<Any>) -> String {
        val.downcast_ref::<i64>().unwrap().to_string()
    }

    fn deserialize(&self, input: &str) -> Box<Any> {
        Box::new(input.parse::<i64>().unwrap())
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn can_serialize(&self) -> bool {
        self.serializable
    }
}

impl Var for CVar<bool> {
    fn serialize(&self, val: &Box<Any>) -> String {
        val.downcast_ref::<bool>().unwrap().to_string()
    }

    fn deserialize(&self, input: &str) -> Box<Any> {
        Box::new(input.parse::<bool>().unwrap())
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn can_serialize(&self) -> bool {
        self.serializable
    }
}

impl Var for CVar<String> {
    fn serialize(&self, val: &Box<Any>) -> String {
        format!("\"{}\"", val.downcast_ref::<String>().unwrap())
    }

    fn deserialize(&self, input: &str) -> Box<Any> {
        Box::new((&input[1..input.len() - 1]).to_owned())
    }

    fn description(&self) -> &'static str {
        self.description
    }
    fn can_serialize(&self) -> bool {
        self.serializable
    }
}

pub trait Var {
    fn serialize(&self, val: &Box<Any>) -> String;
    fn deserialize(&self, input: &str) -> Box<Any>;
    fn description(&self) -> &'static str;
    fn can_serialize(&self) -> bool;
}

pub struct Console {
    names: HashMap<String, &'static str>,
    vars: HashMap<&'static str, Box<Var>>,
    var_values: HashMap<&'static str, Box<Any>>,

    history: Vec<Component>,

    collection: ui::Collection,
    active: bool,
    position: f64,
}

unsafe impl Send for Console {}

impl Console {
    pub fn new() -> Console {
        Console {
            names: HashMap::new(),
            vars: HashMap::new(),
            var_values: HashMap::new(),

            history: vec![Component::Text(TextComponent::new("")); 200],

            collection: ui::Collection::new(),
            active: false,
            position: -220.0,
        }
    }

    pub fn register<T: Sized + Any>(&mut self, var: CVar<T>)
        where CVar<T>: Var
    {
        if self.vars.contains_key(var.name) {
            panic!("Key registered twice {}", var.name);
        }
        self.names.insert(var.name.to_owned(), var.name);
        self.var_values.insert(var.name, Box::new((var.default)()));
        self.vars.insert(var.name, Box::new(var));
    }

    pub fn get<T: Sized + Any>(&self, var: CVar<T>) -> &T
        where CVar<T>: Var
    {
        // Should never fail
        self.var_values.get(var.name).unwrap().downcast_ref::<T>().unwrap()
    }

    pub fn set<T: Sized + Any>(&mut self, var: CVar<T>, val: T)
        where CVar<T>: Var
    {
        self.var_values.insert(var.name, Box::new(val));
        self.save_config();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    pub fn tick(&mut self,
                ui_container: &mut ui::Container,
                renderer: &mut render::Renderer,
                delta: f64,
                width: f64) {
        // To make sure the console is always on top it constant removes and readds itself.
        // Its hacky but the console should never appear for normal users so its not really
        // a major issue.
        self.collection.remove_all(ui_container);
        if !self.active && self.position <= -220.0 {
            return;
        }
        if self.active {
            if self.position < 0.0 {
                self.position += delta * 4.0;
            } else {
                self.position = 0.0;
            }
        } else if self.position > -220.0 {
            self.position -= delta * 4.0;
        } else {
            self.position = -220.0;
        }
        let w = match ui_container.mode {
            ui::Mode::Scaled => width,
            ui::Mode::Unscaled(scale) => 854.0 / scale,
        };

        let mut background =
            ui::Image::new(render::Renderer::get_texture(renderer.get_textures_ref(),
                                                         "steven:solid"),
                           0.0,
                           self.position,
                           w,
                           220.0,
                           0.0,
                           0.0,
                           1.0,
                           1.0,
                           0,
                           0,
                           0);
        background.set_a(180);
        let background = self.collection.add(ui_container.add(background));

        let mut lines = Vec::new();
        let mut offset = 0.0;
        for line in self.history.iter().rev() {
            if offset >= 210.0 {
                break;
            }
            let mut fmt = ui::Formatted::with_width_limit(renderer,
                                                          line.clone(),
                                                          5.0,
                                                          5.0 + offset,
                                                          w - 1.0);
            fmt.set_parent(&background);
            fmt.set_v_attach(ui::VAttach::Bottom);
            offset += fmt.get_height();
            lines.push(ui_container.add(fmt));
        }
        for fmt in lines {
            self.collection.add(fmt);
        }
    }

    pub fn load_config(&mut self) {
        if let Ok(file) = fs::File::open("conf.cfg") {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let parts = line.splitn(2, ' ').map(|v| v.to_owned()).collect::<Vec<String>>();
                let (name, arg) = (&parts[0], &parts[1]);
                if let Some(var_name) = self.names.get(name) {
                    let var = self.vars.get(var_name).unwrap();
                    let val = var.deserialize(&arg);
                    if var.can_serialize() {
                        self.var_values.insert(var_name, val);
                    }
                } else {
                    println!("Missing prop");
                }
            }
        }
    }

    pub fn save_config(&self) {
        let mut file = BufWriter::new(fs::File::create("conf.cfg").unwrap());
        for (name, var) in &self.vars {
            if !var.can_serialize() {
                continue;
            }
            for line in var.description().lines() {
                write!(file, "# {}\n", line).unwrap();
            }
            write!(file,
                   "{} {}\n\n",
                   name,
                   var.serialize(self.var_values.get(name).unwrap()))
                .unwrap();
        }
    }

    fn log(&mut self, record: &log::LogRecord) {
        for filtered in FILTERED_CRATES {
            if record.location().module_path().starts_with(filtered) {
                return;
            }
        }

        let mut file = &record.location().file().replace("\\", "/")[..];
        if let Some(pos) = file.rfind("src/") {
            file = &file[pos + 4..];
        }

        println!("[{}:{}][{}] {}",
                 file,
                 record.location().line(),
                 record.level(),
                 record.args());
        self.history.remove(0);
        let mut msg = TextComponent::new("");
        msg.modifier.extra = Some(vec![
            Component::Text(TextComponent::new("[")),
            {
                let mut msg = TextComponent::new(file);
                msg.modifier.color = Some(Color::Green);
                Component::Text(msg)
            },
            Component::Text(TextComponent::new(":")),
            {
                let mut msg = TextComponent::new(&format!("{}", record.location().line()));
                msg.modifier.color = Some(Color::Aqua);
                Component::Text(msg)
            },
            Component::Text(TextComponent::new("]")),
            Component::Text(TextComponent::new("[")),
            {
                let mut msg = TextComponent::new(&format!("{}", record.level()));
                msg.modifier.color = Some(match record.level() {
                    log::LogLevel::Debug => Color::Green,
                    log::LogLevel::Error => Color::Red,
                    log::LogLevel::Warn => Color::Yellow,
                    log::LogLevel::Info => Color::Aqua,
                    log::LogLevel::Trace => Color::Blue,
                });
                Component::Text(msg)
            },
            Component::Text(TextComponent::new("] ")),
            Component::Text(TextComponent::new(&format!("{}", record.args())))
        ]);
        self.history.push(Component::Text(msg));
    }
}

pub struct ConsoleProxy {
    console: Arc<Mutex<Console>>,
}

impl ConsoleProxy {
    pub fn new(con: Arc<Mutex<Console>>) -> ConsoleProxy {
        ConsoleProxy { console: con }
    }
}

impl log::Log for ConsoleProxy {
    fn enabled(&self, metadata: &log::LogMetadata) -> bool {
        metadata.level() <= log::LogLevel::Trace
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
            self.console.lock().unwrap().log(record);
        }
    }
}

unsafe impl Send for ConsoleProxy {}
unsafe impl Sync for ConsoleProxy {}
