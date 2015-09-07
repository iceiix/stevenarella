extern crate gl_generator;
extern crate khronos_api;

use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir);

    let mut file = BufWriter::new(File::create(&dest.join("bindings.rs")).unwrap());
    gl_generator::generate_bindings(gl_generator::GlobalGenerator,
                                    gl_generator::registry::Ns::Gl,
                                    gl_generator::Fallbacks::All,
                                    khronos_api::GL_XML, vec![], "3.2", "core",
                                    &mut file).unwrap();
}
