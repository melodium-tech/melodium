
//extern crate melodium_rust;

use std::env;
use std::ffi::OsString;
use std::path::Path as StdPath;
use std::sync::Arc;
use std::process::*;

use melodium_rust::executive::world::World;
use melodium_rust::logic::descriptor::SequenceTreatmentDescriptor;
use melodium_rust::script::instance::Instance;
use melodium_rust::script::path::Path;

fn main() {

    let args: Vec<OsString> = env::args_os().collect();

    let std_path = args[1].to_owned();
    let file_path = args[2].to_owned();
    let main_entry = args[3].to_owned();

    let mut instance = Instance::new(file_path.clone(), std_path.clone());

    instance.build();

    for error in instance.errors() {
        eprintln!("Error: {}", error);
    }

    if instance.errors().len() > 0 {
        exit(10);
    }

    let collection = Arc::clone(instance.collection().as_ref().unwrap());

    let main = Arc::clone(&collection.treatments.get(
        &Path::new(
            vec!["main".to_string(), StdPath::new(&file_path).file_stem().unwrap().to_str().unwrap().to_string()]
        ).to_identifier(&main_entry.into_string().unwrap()).unwrap()
    ).unwrap()).downcast_arc::<SequenceTreatmentDescriptor>().unwrap();

    let world = World::new();
    let ready = world.genesis(&*main);

    if ready {
        world.live();
    }
    else {
        for error in world.errors().read().unwrap().iter() {
            eprintln!("Logic error: {:?}", error);
        }
        exit(11);
    }

}


