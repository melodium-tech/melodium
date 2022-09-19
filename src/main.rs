
use std::env;
use std::process::*;

extern crate clap;
use clap::{Arg, Command, crate_version};

use melodium::*;

fn main() {

    let matches = Command::new("Mélodium")
        .version(crate_version!())
        .author("Quentin Vignaud")
        .about("Mélodium script engine")
        .arg(Arg::new("stdlib")
            .long("stdlib")
            .value_name("PATH")
            .help("Sets standard library location")
            .takes_value(true))
        .arg(Arg::new("main")
            .short('m')
            .long("main")
            .value_name("SEQUENCE")
            .help("Sets the main entry point (default to 'Main')")
            .takes_value(true))
        .arg(Arg::new("parseonly")
            .short('p')
            .long("parseonly")
            .help("Parse, and make semantic analysis only"))
        .arg(Arg::new("nolaunch")
            .short('L')
            .long("nolaunch")
            .help("Parse, make semantic analysis, design world and check it, but don't make it live"))
        .arg(Arg::new("doc")
            .long("doc")
            .help("Generates documentation in the given output folder")
            .takes_value(true))
        .arg(Arg::new("draw")
            .long("draw")
            .help("Generates drawing of sequence in given output folder, in SVG format")
            .takes_value(true))
        .arg(Arg::new("script")
            .value_name("SCRIPT")
            .help("Script to run")
            .required(true)
            .index(1))
        .get_matches();
    
    let std_path =
    if let Some(path) = matches.value_of("stdlib") {
        Some(path.to_owned())
    }
    else if let Ok(path) = env::var("MELODIUM_STDLIB") {
        Some(path)
    }
    else {
        None
    };

    let main_entry;
    if let Some(main) = matches.value_of("main") {
        main_entry = main.to_owned();
    }
    else {
        main_entry = "Main".to_owned();
    }

    let file_path;
    if let Some(path) = matches.value_of("script") {
        file_path = path.to_owned();
    }
    else {
        eprintln!("No script path specified.");
        exit(1);
    }

    if let Some(path) = matches.value_of("doc") {

        let std_path = if let Some(std_path) = std_path {
            std_path
        } else {
            eprintln!("No stdlib path specified.");
            exit(1);
        };

        make_documentation(&std_path, &file_path, &path.to_string());

        exit(0);
    }

    if let Some(draw) = matches.value_of("draw") {

        let split: Vec<String> = draw.split(",").map(|s| s.to_string()).collect();
        let output = &split[0];
        let ids = Vec::from(&split[1..]);

        make_svg(std_path.as_ref(), &file_path, output, &ids);

        exit(0);
    }

    let parse_only = matches.is_present("parseonly");
    let no_launch = matches.is_present("nolaunch");

    if parse_only && no_launch {
        eprintln!("'parseonly' and 'nolaunch' options are uncompatible.");
        exit(1);
    }

    // Effective run

    if !parse_only && !no_launch {
        match execute(std_path.as_ref(), &file_path, &main_entry) {
            Ok(_) => exit(0),
            Err(_) => exit(1),
        }
    }
    else if parse_only {
        let instance = build(std_path.as_ref(), &file_path);
        print_instance_errors(&instance);
    }
    else if no_launch {
        let (instance, possible_world) = genesis(std_path.as_ref(), &file_path, &main_entry);

        print_instance_errors(&instance);

        if let Some((world, ready)) = possible_world {
            print_world_errors(&world);

            if !ready {
                exit(1);
            }
        }
        else {
            exit(1);
        }
    }
}


