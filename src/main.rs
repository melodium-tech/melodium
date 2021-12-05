
//extern crate melodium_rust;

use std::env;
use std::ffi::OsString;
use std::sync::Arc;

use melodium_rust::executive::world::World;
use melodium_rust::script::instance::Instance;

fn main() {

    let args: Vec<OsString> = env::args_os().collect();

    let std_path = args[1].to_owned();
    let file_path = args[2].to_owned();
    /*if file_path.is_none() {
        println!("Path to script must be provided.");
        return;
    }*/

    let mut instance = Instance::new(file_path, std_path);

    instance.build();

    println!("{:?}", instance.errors());

    let collection = Arc::clone(instance.collection().as_ref().unwrap());

    println!("{:?}", collection);

    //println!("Reading file \"{}\"…", file_path.unwrap().to_string_lossy());

}


