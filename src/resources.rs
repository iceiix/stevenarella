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

extern crate steven_resources as internal;

use std::thread;
use std::path;
use std::io;
use std::fs;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use hyper;
use zip;

use ui;

const RESOURCES_VERSION: &'static str = "1.9.2";
const VANILLA_CLIENT_URL: &'static str = "https://launcher.mojang.com/mc/game/1.9.2/client/19106fd5e222dca0f2dde9f66db8384c9a7db957/client.jar";

pub trait Pack: Sync + Send {
    fn open(&self, name: &str) -> Option<Box<io::Read>>;
}

pub struct Manager {
    packs: Vec<Box<Pack>>,
    version: usize,

    vanilla_chan: Option<mpsc::Receiver<bool>>,
    vanilla_progress: Arc<Mutex<Progress>>,
}

pub struct ManagerUI {
    progress_ui: Vec<ProgressUI>,
}

struct ProgressUI {
    task_name: String,
    task_file: String,
    position: f64,
    closing: bool,
    progress: f64,

    background: ui::ImageRef,
    progress_bar: ui::ImageRef,
}

struct Progress {
    tasks: Vec<Task>,
}

struct Task {
    task_name: String,
    task_file: String,
    total: u64,
    progress: u64,
}

unsafe impl Sync for Manager {}

impl Manager {
    pub fn new() -> (Manager, ManagerUI) {
        let mut m = Manager {
            packs: Vec::new(),
            version: 0,
            vanilla_chan: None,
            vanilla_progress: Arc::new(Mutex::new(Progress {
                tasks: vec![],
            })),
        };
        m.add_pack(Box::new(InternalPack));
        m.download_vanilla();
        (m, ManagerUI { progress_ui: vec!{} })
    }

    /// Returns the 'version' of the manager. The version is
    /// increase everytime a pack is added or removed.
    pub fn version(&self) -> usize {
        self.version
    }

    pub fn open(&self, plugin: &str, name: &str) -> Option<Box<io::Read>> {
        for pack in self.packs.iter().rev() {
            let path = format!("assets/{}/{}", plugin, name);
            if let Some(val) = pack.open(&path) {
                return Some(val);
            }
        }
        None
    }

    pub fn open_all(&self, plugin: &str, name: &str) -> Vec<Box<io::Read>> {
        let mut ret = Vec::new();
        for pack in self.packs.iter().rev() {
            let path = format!("assets/{}/{}", plugin, name);
            if let Some(val) = pack.open(&path) {
                ret.push(val);
            }
        }
        ret
    }

    pub fn tick(&mut self, mui: &mut ManagerUI, ui_container: &mut ui::Container, delta: f64) {
        // Check to see if the download of vanilla has completed
        // (if it was started)
        let mut done = false;
        if let Some(ref recv) = self.vanilla_chan {
            if let Ok(_) = recv.try_recv() {
                done = true;
            }
        }
        if done {
            self.vanilla_chan = None;
            self.load_vanilla();
        }

        const UI_HEIGHT: f64 = 32.0;

        let mut progress = self.vanilla_progress.lock().unwrap();
        progress.tasks.retain(|v| v.progress < v.total);
        // Find out what we have to work with
        for task in &progress.tasks {
            if !mui.progress_ui.iter()
                .filter(|v| v.task_file == task.task_file)
                .any(|v| v.task_name == task.task_name) {
                // Add a ui element for it
                let background = ui::ImageBuilder::new()
                    .texture("steven:solid")
                    .position(0.0, -UI_HEIGHT)
                    .size(300.0, UI_HEIGHT)
                    .colour((0, 0, 0, 100))
                    .draw_index(100)
                    .alignment(ui::VAttach::Bottom, ui::HAttach::Left)
                    .create(ui_container);

                ui::ImageBuilder::new()
                    .texture("steven:solid")
                    .position(0.0, 0.0)
                    .size(300.0, 10.0)
                    .colour((0, 0, 0, 200))
                    .attach(&mut *background.borrow_mut());
                ui::TextBuilder::new()
                    .text(&*task.task_name)
                    .position(3.0, 0.0)
                    .scale_x(0.5)
                    .scale_y(0.5)
                    .draw_index(1)
                    .attach(&mut *background.borrow_mut());
                ui::TextBuilder::new()
                    .text(&*task.task_file)
                    .position(3.0, 12.0)
                    .scale_x(0.5)
                    .scale_y(0.5)
                    .draw_index(1)
                    .attach(&mut *background.borrow_mut());

                let progress_bar = ui::ImageBuilder::new()
                    .texture("steven:solid")
                    .position(0.0, 0.0)
                    .size(0.0, 10.0)
                    .colour((0, 255, 0, 255))
                    .alignment(ui::VAttach::Bottom, ui::HAttach::Left)
                    .attach(&mut *background.borrow_mut());

                mui.progress_ui.push(ProgressUI {
                    task_name: task.task_name.clone(),
                    task_file: task.task_file.clone(),
                    position: -UI_HEIGHT,
                    closing: false,
                    progress: 0.0,
                    background: background,
                    progress_bar: progress_bar,
                });
            }
        }
        for ui in &mut mui.progress_ui {
            let mut found = false;
            let mut prog = 1.0;
            for task in progress.tasks.iter()
                .filter(|v| v.task_file == ui.task_file)
                .filter(|v| v.task_name == ui.task_name) {
                found = true;
                prog = task.progress as f64 / task.total as f64;
            }
            if !found {
                ui.closing = true;
                ui.position = -UI_HEIGHT;
            }
            ui.progress = prog;
        }
        let mut offset = 0.0;
        for ui in &mut mui.progress_ui {
            if ui.closing {
                continue;
            }
            ui.position = offset;
            offset += UI_HEIGHT;
        }
        // Move elements
        let delta = delta.min(5.0);
        for ui in &mut mui.progress_ui {
            let mut background = ui.background.borrow_mut();
            if (background.y - ui.position).abs() < 0.7 * delta {
                background.y = ui.position;
            } else {
                background.y += (ui.position - background.y).signum() * 0.7 * delta;
            }
            let mut bar = ui.progress_bar.borrow_mut();
            let target_size = (300.0 * ui.progress).min(300.0);
            if (bar.width - target_size).abs() < 1.0 * delta {
                bar.width = target_size;
            } else {
                bar.width += ((target_size - bar.width).signum() * delta).max(0.0);
            }
        }

        // Clean up dead elements
        mui.progress_ui.retain(|v| v.position >= -UI_HEIGHT && !v.closing);
    }

    fn add_pack(&mut self, pck: Box<Pack>) {
        self.packs.push(pck);
        self.version += 1;
    }

    fn load_vanilla(&mut self) {
        let loc = format!("./resources-{}", RESOURCES_VERSION);
        let location = path::Path::new(&loc);
        self.add_pack(Box::new(DirPack { root: location.to_path_buf() }))
    }

    fn download_vanilla(&mut self) {
        let loc = format!("./resources-{}", RESOURCES_VERSION);
        let location = path::Path::new(&loc);
        if fs::metadata(location.join("steven.assets")).is_ok() {
            self.load_vanilla();
            return;
        }
        let (send, recv) = mpsc::channel();
        self.vanilla_chan = Some(recv);

        let progress_info = self.vanilla_progress.clone();
        thread::spawn(move || {
            let client = hyper::Client::new();
            let res = client.get(VANILLA_CLIENT_URL)
                            .send()
                            .unwrap();
            let mut file = fs::File::create(format!("{}.tmp", RESOURCES_VERSION)).unwrap();

            let length = *res.headers.get::<hyper::header::ContentLength>().unwrap();
            let task_file = format!("./resources-{}", RESOURCES_VERSION);
            Self::add_task(&progress_info, "Downloading Core Assets", &task_file, *length);
            {
                let mut progress = ProgressRead {
                    read: res,
                    progress: &progress_info,
                    task_name: "Downloading Core Assets".into(),
                    task_file: task_file,
                };
                io::copy(&mut progress, &mut file).unwrap();
            }

            // Copy the resources from the zip
            let file = fs::File::open(format!("{}.tmp", RESOURCES_VERSION)).unwrap();
            let mut zip = zip::ZipArchive::new(file).unwrap();

            let task_file = format!("./resources-{}", RESOURCES_VERSION);
            Self::add_task(&progress_info, "Unpacking Core Assets", &task_file, zip.len() as u64);

            let loc = format!("./resources-{}", RESOURCES_VERSION);
            let location = path::Path::new(&loc);
            let count = zip.len();
            for i in 0..count {
                Self::add_task_progress(&progress_info, "Unpacking Core Assets", &task_file, 1);
                let mut file = zip.by_index(i).unwrap();
                if !file.name().starts_with("assets/") {
                    continue;
                }
                let path = location.join(file.name());
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                let mut out = fs::File::create(path).unwrap();
                io::copy(&mut file, &mut out).unwrap();
            }

            fs::File::create(location.join("steven.assets")).unwrap(); // Marker file
            send.send(true).unwrap();

            fs::remove_file(format!("{}.tmp", RESOURCES_VERSION)).unwrap();
        });
    }

    fn add_task(progress: &Arc<Mutex<Progress>>, name: &str, file: &str, length: u64) {
        let mut info = progress.lock().unwrap();
        info.tasks.push(Task {
            task_name: name.into(),
            task_file: file.into(),
            total: length,
            progress: 0,
        });
    }

    fn add_task_progress(progress: &Arc<Mutex<Progress>>, name: &str, file: &str, prog: u64) {
        let mut progress = progress.lock().unwrap();
        for task in progress.tasks.iter_mut()
            .filter(|v| v.task_file == file)
            .filter(|v| v.task_name == name) {
            task.progress += prog as u64;
        }
    }
}

struct DirPack {
    root: path::PathBuf,
}

impl Pack for DirPack {
    fn open(&self, name: &str) -> Option<Box<io::Read>> {
        match fs::File::open(self.root.join(name)) {
            Ok(val) => Some(Box::new(val)),
            Err(_) => None,
        }
    }
}

struct InternalPack;

impl Pack for InternalPack {
    fn open(&self, name: &str) -> Option<Box<io::Read>> {
        match internal::get_file(name) {
            Some(val) => Some(Box::new(io::Cursor::new(val))),
            None => None,
        }
    }
}

struct ProgressRead<'a, T> {
    read: T,
    progress: &'a Arc<Mutex<Progress>>,
    task_name: String,
    task_file: String,
}

impl <'a, T: io::Read> io::Read for ProgressRead<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = try!(self.read.read(buf));
        Manager::add_task_progress(self.progress, &self.task_name, &self.task_file, size as u64);
        Ok(size)
    }
}
