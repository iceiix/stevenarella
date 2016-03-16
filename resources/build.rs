use std::env;
use std::fs;
use std::path::Path;
use std::io::BufWriter;
use std::io::Write;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir);

    let base = Path::new("assets");
    let mut out = Vec::new();
    build_map(&mut out, &base);

    let mut file = BufWriter::new(fs::File::create(&dest.join("resources.rs")).unwrap());
    write!(file, "pub fn get_file(name: &str) -> Option<&'static [u8]> {{\n").unwrap();
    write!(file, "    match name {{\n").unwrap();
    for entry in &out {
    	let entry = entry.replace("\\", "/");
    	let short = &entry;
    	write!(file, "        {:?} => Some(include_bytes!(\"../{}\")),\n", short, entry).unwrap();
    }
    write!(file, "        _ => None\n    }}\n}}\n").unwrap();

}

fn build_map(out: &mut Vec<String>, path: &Path) {
    let files = fs::read_dir(path).unwrap();
    for entry in files {
    	let entry = entry.unwrap();
    	if fs::metadata(entry.path()).unwrap().is_dir() {
    		build_map(out, &entry.path());
    	} else {
    		out.push(entry.path().to_str().unwrap().to_owned());
    	}
    }
}
