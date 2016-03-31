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

extern crate hyper;
extern crate zip;

extern crate steven_resources as internal;

use std::thread;
use std::path;
use std::io;
use std::fs;
use std::sync::mpsc;

const RESOURCES_VERSION: &'static str = "1.9.2";

pub trait Pack: Sync + Send {
    fn open(&self, name: &str) -> Option<Box<io::Read>>;
}

pub struct Manager {
    packs: Vec<Box<Pack>>,
    version: usize,

    vanilla_chan: Option<mpsc::Receiver<bool>>,
}

unsafe impl Sync for Manager {}

impl Manager {
    pub fn new() -> Manager {
        let mut m = Manager {
            packs: Vec::new(),
            version: 0,
            vanilla_chan: None,
        };
        m.add_pack(Box::new(InternalPack));
        m.download_vanilla();
        m
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

    pub fn tick(&mut self) {
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

        info!("Vanilla assets missing, obtaining");
        thread::spawn(move || {
            let client = hyper::Client::new();
            let url = format!("https://s3.amazonaws.com/Minecraft.Download/versions/{0}/{0}.jar",
                              RESOURCES_VERSION);
            let res = client.get(&url)
                            .send()
                            .unwrap();
            let mut file = fs::File::create(format!("{}.tmp", RESOURCES_VERSION)).unwrap();

            let length = *res.headers.get::<hyper::header::ContentLength>().unwrap();
            let mut progress = ProgressRead {
                read: res,
                progress: 0,
                total: *length,
            };
            io::copy(&mut progress, &mut file).unwrap();

            // Copy the resources from the zip
            let file = fs::File::open(format!("{}.tmp", RESOURCES_VERSION)).unwrap();
            let mut zip = zip::ZipArchive::new(file).unwrap();

            let loc = format!("./resources-{}", RESOURCES_VERSION);
            let location = path::Path::new(&loc);
            let count = zip.len();
            for i in 0..count {
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
            info!("Done");
            send.send(true).unwrap();

            fs::remove_file(format!("{}.tmp", RESOURCES_VERSION)).unwrap();
        });
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

struct ProgressRead<T> {
    read: T,
    total: u64,
    progress: u64,
}

impl <T: io::Read> io::Read for ProgressRead<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = try!(self.read.read(buf));
        self.progress += size as u64;
        trace!("Progress: {:.2}",
                 (self.progress as f64) / (self.total as f64));
        Ok(size)
    }
}
