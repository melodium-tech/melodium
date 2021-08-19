
//extern crate melodium_rust;

use std::env;
use std::ffi::OsString;

fn main() {

    let args: Vec<OsString> = env::args_os().collect();

    let file_path = args.last();
    if file_path.is_none() {
        println!("Path to script must be provided.");
        return;
    }

    println!("Reading file \"{}\"…", file_path.unwrap().to_string_lossy());

}


