use std::collections::HashMap;
use std::env;
use steven_blocks::VanillaIDMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!(
            "usage: {} protocol_version id\nrun with DEBUG_BLOCKS=1 to dump all",
            args[0]
        );
        return;
    }
    let protocol_version = str::parse::<i32>(&args[1]).unwrap();
    let id = str::parse::<usize>(&args[2]).unwrap();

    let id_map = VanillaIDMap::new(protocol_version);
    let block = id_map.by_vanilla_id(id, &HashMap::new());

    println!("{:?}", block);
}
