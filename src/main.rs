
//extern crate melodium_rust;

use std::env;
use std::ffi::OsString;
use std::sync::Arc;

use melodium_rust::executive::world::World;
use melodium_rust::logic::descriptor::SequenceTreatmentDescriptor;
use melodium_rust::script::instance::Instance;
use melodium_rust::script::path::Path;

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

    println!("Models: {:?}", collection.models.identifiers());
    println!("Treatments: {:?}", collection.treatments.identifiers());

    let main = Arc::clone(&collection.treatments.get(
        &Path::new(
            vec!["main".to_string(), "file_copy".to_string()]
        ).to_identifier("Main").unwrap()
    ).unwrap()).downcast_arc::<SequenceTreatmentDescriptor>().unwrap();

    let world = World::new();
    let ready = world.genesis(&*main);

    println!("{:?}", world);
    println!("Ready: {}", ready);

    if ready {
        world.live();
    }

    //println!("Reading file \"{}\"…", file_path.unwrap().to_string_lossy());

}


