
use std::env;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use std::process::*;

extern crate clap;
use clap::{Arg, Command, crate_version};
use colored::*;

use melodium::executive::world::World;
use melodium::logic::descriptor::SequenceTreatmentDescriptor;
use melodium::script::instance::Instance;
use melodium::script::path::{Path, PathRoot};
use melodium::doc::instance::Instance as DocInstance;

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
            .help("Print documentation of specified element, implies --parseonly")
            .takes_value(true))
        .arg(Arg::new("script")
            .value_name("SCRIPT")
            .help("Script to run")
            .required(true)
            .index(1))
        .get_matches();
    
    let std_path;
    if let Some(path) = matches.value_of("stdlib") {
        std_path = path.to_owned();
    }
    else if let Ok(path) = env::var("MELODIUM_STDLIB") {
        std_path = path;
    }
    else {
        eprintln!("No standard library path specified.");
        exit(1);
    }

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

        let root_kind = if path == std_path {
            PathRoot::Std
        }
        else {
            PathRoot::Main
        };
        
        let mut instance = DocInstance::new(root_kind, PathBuf::from(file_path), PathBuf::from(path));

        if let Err((io, scr)) = instance.parse_files() {
            eprintln!("{:?}", io);
            eprintln!("{:?}", scr);
        }

        if let Err(io) = instance.output_doc() {
            eprintln!("{:?}", io);
        }

        exit(0);
    }

    let parse_only = matches.is_present("parseonly");
    let no_launch = matches.is_present("nolaunch");

    if parse_only && no_launch {
        eprintln!("'parseonly' and 'nolaunch' options are uncompatible.");
        exit(1);
    }

    // Effective run

    let mut instance = Instance::new(file_path.clone(), std_path.clone());

    instance.build();

    for (path, error) in instance.errors() {
        eprintln!("{}: in file \"{}\" {}", "error".bold().red(), path.as_os_str().to_string_lossy(), error);
    }

    if instance.errors().len() > 0 {
        exit(10);
    }

    if parse_only {
        exit(0);
    }

    let collection = Arc::clone(instance.collection().as_ref().unwrap());

    let main = Arc::clone(&collection.treatments.get(
        &Path::new(
            vec!["main".to_string(), StdPath::new(&file_path).file_stem().unwrap().to_str().unwrap().to_string()]
        ).to_identifier(&main_entry).unwrap()
    ).unwrap()).downcast_arc::<SequenceTreatmentDescriptor>().unwrap();

    let world = World::new();
    let ready = world.genesis(&*main);

    if ready {
        if !no_launch {
            world.live();
        }
        else {
            exit(0);
        }
    }
    else {
        for error in world.errors().read().unwrap().iter() {
            eprintln!("Logic error: {:?}", error);
        }
        exit(11);
    }
}


