use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use gl_generator::{Registry, Api, Profile, Fallbacks, GlobalGenerator};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir);

    let mut file = BufWriter::new(File::create(&dest.join("bindings.rs")).unwrap());
    Registry::new(Api::Gl,
                  (3, 2),
                  Profile::Core,
                  Fallbacks::All,
                  [])
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
}
