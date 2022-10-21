
/*!
 * # Mélodium
 * 
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
 * ## Download & installation
 * 
 * Mélodium releases can be downloaded for multiple platforms from the [Mélodium Repository](https://repo.melodium.tech/).
 * 
 * ## Command-line usage
 * 
 * Launch a Mélodium file:
 * ```shell
 * melodium <FILE>
 * melodium run <FILE>
 * ```
 * 
 * Launch a Mélodium file with a different main entry than `Main`:
 * ```shell
 * melodium run --main OtherEntry <FILE>
 * ```
 * 
 * Check a Mélodium file:
 * ```shell
 * melodium check <FILE>
 * ```
 * 
 * Compile and package a script project into a `jeu` file:
 * ```shell
 * melodium package <SCRIPT> <FILE>
 * ```
 * 
 * Draw sequences as SVG:
 * ```shell
 * melodium draw <FILE> [ENTRY …] <OUTPUT>
 * ```
 * 
 * Generate documentation:
 * ```shell
 * melodium doc <FILE> <OUTPUT>
 * ```
 * 
 * ## Example
 * 
 * The following code makes a copy of the file `./input.txt` to `./output.txt`. More examples are available under [examples](examples/).
 * 
 * ```melodium
 * use std/fs/mono/read::ReadPath
 * use std/fs/mono/write::WritePath
 * use std/process/engine::Ready
 * 
 * sequence Main()
 * {
 *     Ready()
 *     ReadPath(path="input.txt")
 *     WritePath(path="output.txt")
 *     
 *     Ready.ready -> ReadPath.trigger,data -> WritePath.data
 * }
 * ```
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
 * ### Cross-compilation
 * 
 * A more detailed explanation on how to cross-compile Mélodium is written in [dedicated file](CROSS-COMPILATION.md).
 * 
 * ## Standard library
 * 
 * Mélodium comes with its [standard library](https://doc.melodium.tech/latest/).
 * If needed, the default standard library can be overrided using the `MELODIUM_STDLIB` environment variable, or by passing explicily the option `--stdlib <PATH>` to the command line.
 * 
 * If compiled from source, standard library can be found in the `std/` folder. If installed through [crates.io](https://crates.io/crates/melodium), it should be found within `~/.cargo/registry/src/<cargo git reference>/melodium-<version>/std`.
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
 * 
 * ## Licence
 * 
 * This software is free and open-source, under the EUPL licence.
 * 
 * Why this one specifically? Well, as this project have a particular relationship with cultural world, probably more than most other softwares, it is important to have a strong legal basis covering also the notion of artwork.
 * In the same way, as *no culture is more important than another*, it was important to have a licence readable and understanble by most of people. The EUPL is available and *legally valid* in 23 languages, covering a large number of people.
 * 
 * Then, the legal part:
 * > Licensed under the EUPL, Version 1.2 or - as soon they will be approved by the European Commission - subsequent versions of the EUPL (the "Licence"); You may not use this work except in compliance with the Licence. You may obtain a copy of the Licence at: https://joinup.ec.europa.eu/software/page/eupl
 * >
 * >Unless required by applicable law or agreed to in writing, software distributed under the Licence is distributed on an "AS IS" basis, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the Licence for the specific language governing permissions and limitations under the Licence.
 * 
 * And do not worry, this licence is explicitly compatible with the ones mentionned in its appendix, including most of the common open-source licences.
 */


#[macro_use]
extern crate lazy_static;

pub mod core;
pub mod doc;
pub mod executive;
pub mod graph;
pub mod jeu;
pub mod logic;
pub mod script;

use colored::*;
use std::collections::HashMap;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use executive::world::World;
use logic::descriptor::SequenceTreatmentDescriptor;
use logic::descriptor::identifier::Root;
use script::instance::Instance;
use script::base::Base;
use script::location::Location;
use script::path::Path;
use script::error::ScriptError;
use logic::error::LogicError;

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
pub fn execute(stdlib: Option<&String>, main: &String, entry: &String) -> Result<(), ()> {

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
pub fn build(stdlib: Option<&String>, main: &String) -> Instance {

    let main = PathBuf::from(main);
    let main_dir = PathBuf::from(main.parent().unwrap());
    let main_file = PathBuf::from(main.file_name().unwrap());

    let mut instance = Instance::new(Location::new(Base::FileSystem(main_dir), main_file),
        if let Some(stdlib) = stdlib {
            Base::FileSystem(stdlib.into())
        } else {
            STDLIB.clone()
        }
    );

    instance.build_by_main();

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
pub fn genesis(stdlib: Option<&String>, main: &String, entry: &String) -> (Instance, Option<(Arc<World>, bool)>) {

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
 * - `roots`: list of roots for which generate documentation;
 * - `output`: output path where to store the prepared documentation.
 */
pub fn make_documentation(stdlib: Option<&String>, main: Option<&String>, roots: Vec<String>, output: &String) {

     let main = main.map(|m| PathBuf::from(m)).unwrap_or_default();
    let main_dir = PathBuf::from(main.parent().unwrap());
    let main_file = PathBuf::from(main.file_name().unwrap());

    let mut instance = Instance::new(Location::new(Base::FileSystem(main_dir), main_file),
        if let Some(stdlib) = stdlib {
            Base::FileSystem(stdlib.into())
        } else {
            STDLIB.clone()
        }
    );
    instance.build_all_std();

    let doc = doc::documentation::Documentation::new(roots, Arc::clone(instance.collection().as_ref().unwrap()), PathBuf::from("/tmp/doc"));
    if let Err(e) = doc.make() {
        print_io_error(&e);
    }
}

pub fn make_package(stdlib: Option<&String>, main: &String, output: &String) {

    let instance = build(stdlib, main);

    if !instance.errors().is_empty() {
        print_instance_errors(&instance);
        return;
    }

    let mut tree = script::restitution::tree::Tree::new();

    for ref identifier in instance.collection().as_ref().unwrap().models.identifiers() {
        if identifier.root() == &Root::Main {
            if let Some(ref model_designer) = instance.collection().as_ref().unwrap().models.get(identifier).unwrap().designer() {
                tree.add_model(model_designer);
            }
        }
    }

    for ref identifier in instance.collection().as_ref().unwrap().treatments.identifiers() {
        if identifier.root() == &Root::Main {
            if let Some(ref sequence_designer) = instance.collection().as_ref().unwrap().treatments.get(identifier).unwrap().designer() {
                tree.add_sequence(sequence_designer);
            }
        }
    }

    fn write_file(content: HashMap<String, String>, output: &String) -> std::io::Result<()> {
        let file = std::fs::File::create(PathBuf::from(output))?;
        let mut builder = jeu::Builder::new(file)?;

        for (path, text) in content {
            builder.append(PathBuf::from(path), text.as_bytes())?;
        }

        builder.finish()?;

        Ok(())
    }

    let internal_content = tree.generate();

    if let Err(e) = write_file(internal_content, output) {
        print_io_error(&e);
    }

}

/**
 * Makes SVG describing sequences
 * 
 * Creates a SVG file in `output` directory, with as name `<sequence_name>.svg`.
 * Entries should be in `path/to::Sequence` format.
 */
pub fn make_svg(stdlib: Option<&String>, main: &String, output: &String, entries: &Vec<String>) {

    let instance = build(stdlib, main);
    let collection = Arc::clone(instance.collection().as_ref().unwrap());

    for entry in entries {
        let split = entry.split("::").collect::<Vec<_>>();
        let path = Path::new(split[0].split("/").map(|s| s.to_string()).collect());
        let name = split[1].to_string();
        let identifier = path.to_identifier(&name).unwrap();

        if let Some(descriptor) = collection.treatments.get(&identifier) {

            if let Some(designer) = descriptor.designer() {

                let svg = graph::draw::draw(designer);

                let path = PathBuf::from(format!("{output}/{name}.svg"));

                std::fs::write(path, svg).unwrap();
            }
        }
    }
}

/**
 * Prints errors present in an instance
 */
pub fn print_instance_errors(instance: &Instance) {
    for (location, error) in instance.errors() {
        print_script_error(location, error);
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

fn print_script_error(location: &Location, error: &ScriptError) {
    eprintln!("{}: in file \"{}\" {}", "error".bold().red(), location.path.as_os_str().to_string_lossy(), error);
}

fn print_logic_error(error: &LogicError) {
    eprintln!("{}: {:?}", "error".bold().red(), error);
}

fn print_io_error(error: &std::io::Error) {
    eprintln!("{}: {:?}", "error".bold().red(), error);
}

lazy_static! {
    static ref STDLIB_CONTENT: std::collections::BTreeMap<&'static str, &'static str> = {
        let mut content = std::collections::BTreeMap::new();

        include!(concat!(env!("OUT_DIR"), "/stdlib.rs"));

        content
    };
    pub static ref STDLIB: Base = Base::Internal(&STDLIB_CONTENT);
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_std_loading() {
        
        let mut instance = Instance::new(
            Location::new(
                Base::FileSystem(PathBuf::new()), 
                PathBuf::new()),
            STDLIB.clone());
        instance.build_all_std();
    }

    #[test]
    fn test_doc_generation() {

        let mut instance = Instance::new(
            Location::new(
                Base::FileSystem(PathBuf::new()), 
                PathBuf::new()),
            STDLIB.clone());
        instance.build_all_std();

        let doc = doc::documentation::Documentation::new(vec!["core".to_string(), "std".to_string()], Arc::clone(instance.collection().as_ref().unwrap()), PathBuf::from("/tmp/doc"));
        doc.make().unwrap();
    }
}
