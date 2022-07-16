
/*!
 * Mélodium is a dataflow-oriented language, focusing on treatments applied on data, allowing high scalability and massive parallelization safely.
 * 
 * ## Introduction
 * 
 * Mélodium is a tool and language for manipulation of large amount of data, using the definition of treatments that applies on data through connections, with a track approach that makes any script higly scalable and implicitly parallelizable.
 * 
 * For more exhaustive explanations, please refer to [the Mélodium Language book](https://doc.melodium.tech/book/).
 * 
 * Mélodium is _under development_ and continously being defined and improved. Released documentation is available on [docs.rs](https://docs.rs/melodium/latest/melodium/) and standard reference on [melodium.tech](https://doc.melodium.tech/latest/). Development documentation is available at <https://melodium.gitlab.io/melodium/melodium/>, and the standard reference at <https://melodium.gitlab.io/melodium/reference/>.
 * 
 * ## Download
 * 
 * Mélodium releases can be downloaded for some platforms from the [Mélodium Repository](https://repo.melodium.tech/).
 * 
 * ## Compilation
 * 
 * ### Linux prerequisites
 * 
 * On Linux systems, ALSA development files are required, provided through `libasound2-dev` on Debian-like systems and `alsa-lib-devel` on Fedora-like ones.
 * 
 * ### Compile from source
 * 
 * Mélodium is fully written in Rust, and just need usual `cargo build` and `cargo test`.
 * ```shell
 * git clone https://gitlab.com/melodium/melodium.git
 * cd melodium
 * cargo build
 * ```
 * ### Install from crates.io
 * 
 * Mélodium can also be directly installed from [crates.io](https://crates.io/crates/melodium).
 * ```shell
 * cargo install melodium
 * ```
 * 
 * ## Usage
 * 
 * Mélodium can be called through the `melodium` command.
 * - if compiled from source, look at the `target/` directory;
 * - if installed through crates.io, it should already be in your `PATH`.
 * 
 * Mélodium also need to know where its [standard library](https://doc.melodium.tech/latest/) is located. It can either be set up with the `MELODIUM_STDLIB` environment variable, or by passing explicily the option `--stdlib <PATH>` to the command line.
 * If compiled from source, standard library can be found in the `std/` folder. If installed through crates.io, it should be found within `~/.cargo/registry/src/<cargo git reference>/melodium-<version>/std`.
 * 
 * To launch a script:
 * ```shell
 * melodium <SCRIPT>
 * ```
 * 
 * Or if your script has an entry sequence that is not called `Main`:
 * ```shell
 * melodium -m <EntrySequenceName> <SCRIPT>
 * ```
 * 
 * ## Development
 * 
 * Development channels and related Mélodium stuff are available on [Discord](https://discord.gg/GQmckruKNx).
 * 
 * ## Origin
 * 
 * Mélodium were first developed during research in signal analysis and musical information retrieval, in need of a tool to manage large amount of records and easily write experimentations, without being concerned of underlying technical operations. It has been presented in [this thesis](https://www.researchgate.net/publication/344327676_Detection_et_classification_des_notes_d'une_piste_audio_musicale) (in French).
 * 
 * The first implementation was in C++ and ran well on high performance computers, such as those of Compute Canada. That tool appeared to be really useful, and the concepts used within its configuration language to deserve more attention. This first experimental design is still available at <https://gitlab.com/qvignaud/Melodium>.
 * 
 * The current project is the continuation of that work, rewritten from ground in Rust, and redesigned with a general approach of massively multithreaded data flows in mind.
 * 
 */


#[macro_use]
extern crate lazy_static;

pub mod core;
pub mod doc;
pub mod executive;
pub mod logic;
pub mod script;

use colored::*;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use crate::executive::world::World;
use crate::logic::descriptor::SequenceTreatmentDescriptor;
use crate::script::instance::Instance;
use crate::script::path::{Path, PathRoot};
use crate::doc::instance::Instance as DocInstance;
use crate::script::error::ScriptError;
use crate::logic::error::LogicError;

/**
 * Launches a Mélodium execution
 * 
 * This function is _blocking_ and returns at the end of Mélodium execution
 * wheter everything goes right or wrong at the Mélodium level.
 * 
 * - `stdlib`: path to the standard library;
 * - `main`: path to the main script file;
 * - `entry`: entrypoint to use within the main script file.
 * 
 * The common default is to set `entry` to `"Main"`.
 */
pub fn execute(stdlib: &String, main: &String, entry: &String) -> Result<(), ()> {

    let (instance, possible_world) = genesis(stdlib, main, entry);

    if let Some((world, ready)) = possible_world {

        if ready {
            world.live();
            Ok(())
        }
        else {
            print_world_errors(&world);
            Err(())
        }
    }
    else {
        print_instance_errors(&instance);
        Err(())
    }
}

/**
 * Builds a Mélodium script instance
 * 
 * This function can be used to check if a given script is buildable or not.
 * The instance itself does nothing and is an in-memory representation of Mélodium script.
 * The building phase does not interact with the system except for reaching script files,
 * nothing is launched nor checked other than syntax and semantic.
 * 
 * - `stdlib`: path to the standard library;
 * - `main`: path to the main script file.
 */
pub fn build(stdlib: &String, main: &String) -> Instance {

    let mut instance = Instance::new(main.clone(), stdlib.clone());

    instance.build();

    instance
}

/**
 * Builds a Mélodium script instance and makes a world
 * 
 * This function can be used to check if a given script is buildable or not,
 * and if the created world is logically conform and able to exchange with the system.
 * Models are initialized but the world is not made live.
 * 
 * - `stdlib`: path to the standard library;
 * - `main`: path to the main script file;
 * - `entry`: entrypoint to use within the main script file.
 */
pub fn genesis(stdlib: &String, main: &String, entry: &String) -> (Instance, Option<(Arc<World>, bool)>) {

    let instance = build(stdlib, main);

    if !instance.errors().is_empty() {
        return (instance, None)
    }

    let collection = Arc::clone(instance.collection().as_ref().unwrap());

    let main = Arc::clone(&collection.treatments.get(
        &Path::new(
            vec!["main".to_string(), StdPath::new(main).file_stem().unwrap().to_str().unwrap().to_string()]
        ).to_identifier(entry).unwrap()
    ).unwrap()).downcast_arc::<SequenceTreatmentDescriptor>().unwrap();

    let world = World::new();
    let ready = world.genesis(&*main);

    (instance, Some((world, ready)))
}

/**
 * Builds the documentation of given script
 * 
 * The generated documentation is on [mdBook](https://rust-lang.github.io/mdBook/) format.
 * To build it run afterwards:
 * ```sh
 * # Only if render of mermaid.js graphs is needed
 * mdbook-mermaid install <output>
 * mdbook build <output>
 * ```
 * 
 * - `stdlib`: path to the standard library;
 * - `main`: path to the main script file;
 * - `output`: output path where to store the prepared documentation.
 */
pub fn make_documentation(stdlib: &String, main: &String, output: &String) {

    let root_kind = if main == stdlib {
        PathRoot::Std
    }
    else {
        PathRoot::Main
    };
    
    let mut instance = DocInstance::new(root_kind, PathBuf::from(main), PathBuf::from(output));

    if let Err((io, scr)) = instance.parse_files() {

        for err in io {
            print_io_error(&err);
        }
        for (path, err) in scr {
            print_script_error(&path, &err);
        }
    }

    if let Err(io) = instance.output_doc() {
        print_io_error(&io);
    }
}

/**
 * Prints errors present in an instance
 */
pub fn print_instance_errors(instance: &Instance) {
    for (path, error) in instance.errors() {
        print_script_error(path, error);
    }
}

/**
 * Prints errors present in a world
 */
pub fn print_world_errors(world: &Arc<World>) {
    for error in world.errors().read().unwrap().iter() {
        print_logic_error(error);
    }
}

fn print_script_error(path: &PathBuf, error: &ScriptError) {
    eprintln!("{}: in file \"{}\" {}", "error".bold().red(), path.as_os_str().to_string_lossy(), error);
}

fn print_logic_error(error: &LogicError) {
    eprintln!("{}: {:?}", "error".bold().red(), error);
}

fn print_io_error(error: &std::io::Error) {
    eprintln!("{}: {:?}", "error".bold().red(), error);
}
