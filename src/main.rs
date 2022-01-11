
//extern crate melodium_rust;

use std::env;
use std::path::Path as StdPath;
use std::sync::Arc;
use std::process::*;

extern crate clap;
use clap::{Arg, App};

use melodium_rust::executive::world::World;
use melodium_rust::logic::descriptor::SequenceTreatmentDescriptor;
use melodium_rust::script::instance::Instance;
use melodium_rust::script::path::Path;

fn main() {

    let matches = App::new("Mélodium")
        .version("0.1-dev")
        .author("Quentin Vignaud")
        .about("Mélodium script engine")
        .arg(Arg::with_name("stdlib")
            .long("stdlib")
            .value_name("PATH")
            .help("Sets standard library location")
            .takes_value(true))
        .arg(Arg::with_name("main")
            .short("m")
            .long("main")
            .value_name("SEQUENCE")
            .help("Sets the main entry point (default to 'Main')")
            .takes_value(true))
        .arg(Arg::with_name("parseonly")
            .short("p")
            .long("parseonly")
            .help("Parse, and make semantic analysis only"))
        .arg(Arg::with_name("nolaunch")
            .short("L")
            .long("nolaunch")
            .help("Parse, make semantic analysis, design world and check it, but don't make it live"))
        .arg(Arg::with_name("doc-list")
            .long("doc-list")
            .help("Print list of elements available, implies --parseonly"))
        .arg(Arg::with_name("doc")
            .long("doc")
            .help("Print documentation of specified element, implies --parseonly")
            .takes_value(true))
        .arg(Arg::with_name("script")
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

    let parse_only = matches.is_present("parseonly");
    let no_launch = matches.is_present("nolaunch");

    if parse_only && no_launch {
        eprintln!("'parseonly' and 'nolaunch' options are uncompatible.");
        exit(1);
    }

    // Effective run

    let mut instance = Instance::new(file_path.clone(), std_path.clone());

    instance.build();

    for error in instance.errors() {
        eprintln!("Error: {}", error);
    }

    if matches.is_present("doc-list") {
        if let Some(collection) = &instance.logic_collection {
            for id in collection.models.identifiers() {
                println!("(Model) {}", id.to_string());
            }

            for id in collection.treatments.identifiers() {
                println!("(Treatment) {}", id.to_string());
            }
        }
        exit(0);
    }

    if let Some(element) = matches.value_of("doc") {
        if let Some(collection) = &instance.logic_collection {
            let path_and_name: Vec<String> = element.split("::").map(|i| i.to_string()).collect();
            let path: Vec<String> = path_and_name.get(0).unwrap().split("/").map(|i| i.to_string()).collect();

            let identifier = Path::new(path).to_identifier(path_and_name.get(1).unwrap()).unwrap();

            if let Some(model) = collection.models.get(&identifier) {
                println!("{:?}", model);
            }
            else if let Some(treatment) = collection.treatments.get(&identifier) {
                println!("{:?}", treatment);
            }
            else {
                println!("No element for '{}'", element);
            }
        }
        exit(0);
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


