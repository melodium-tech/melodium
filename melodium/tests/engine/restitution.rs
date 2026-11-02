//! Restitution of the programs of the repository: restituted code gives back the designs it
//! comes from, and restituting it again gives the same code.

use melodium::{load_file_all_entrypoints, load_raw_all_entrypoints, LoadingConfig};
use melodium_common::descriptor::{Collection, Identified, Identifier};
use melodium_engine::design::{Treatment as TreatmentDesign, IO};
use melodium_engine::designer::Value;
use melodium_lang::{restitution::Area, Path};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

fn config() -> LoadingConfig {
    LoadingConfig {
        core_packages: Vec::new(),
        search_locations: Vec::new(),
        raw_elements: Vec::new(),
    }
}

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Comparable form of a value, as descriptors are not comparable.
fn value(value: &Value) -> String {
    match value {
        // Secrets get a new identifier each time they are built.
        Value::Raw(raw) => format!("{raw:?}")
            .split("Secret { id: ")
            .enumerate()
            .map(|(i, part)| {
                if i == 0 {
                    part
                } else {
                    part.trim_start_matches(|c: char| c.is_ascii_digit())
                }
            })
            .collect::<Vec<_>>()
            .join("Secret { id: _"),
        Value::Array(values) => format!(
            "[{}]",
            values.iter().map(self::value).collect::<Vec<_>>().join(",")
        ),
        Value::Variable(name) => name.clone(),
        Value::Context(context, entry) => format!("{}[{entry}]", context.identifier()),
        Value::Function(function, generics, params) => format!(
            "{}{:?}({})",
            function.identifier(),
            generics.iter().collect::<BTreeMap<_, _>>(),
            params.iter().map(self::value).collect::<Vec<_>>().join(",")
        ),
    }
}

fn io(io: &IO) -> &str {
    match io {
        IO::Sequence() => "Self",
        IO::Treatment(name) => name,
    }
}

/// Comparable form of a treatment design: its instanciations and the set of its connections.
fn design(design: &TreatmentDesign) -> String {
    let models = design
        .model_instanciations
        .iter()
        .map(|(name, model)| {
            (
                name,
                (
                    model.descriptor.upgrade().unwrap().identifier().clone(),
                    model
                        .parameters
                        .iter()
                        .map(|(name, param)| (name, value(&param.value)))
                        .collect::<BTreeMap<_, _>>(),
                    model.attributes.iter().collect::<BTreeMap<_, _>>(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let treatments = design
        .treatments
        .iter()
        .map(|(name, treatment)| {
            (
                name,
                (
                    treatment.descriptor.upgrade().unwrap().identifier().clone(),
                    format!(
                        "{:?}",
                        treatment.generics.iter().collect::<BTreeMap<_, _>>()
                    ),
                    treatment.models.iter().collect::<BTreeMap<_, _>>(),
                    treatment
                        .parameters
                        .iter()
                        .map(|(name, param)| (name, value(&param.value)))
                        .collect::<BTreeMap<_, _>>(),
                    treatment.attributes.iter().collect::<BTreeMap<_, _>>(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut connections = design
        .connections
        .iter()
        .map(|connection| {
            format!(
                "{}.{} -> {}.{} {:?}",
                io(&connection.output_treatment),
                connection.output_name,
                io(&connection.input_treatment),
                connection.input_name,
                connection.attributes.iter().collect::<BTreeMap<_, _>>()
            )
        })
        .collect::<Vec<_>>();
    connections.sort();
    format!("{models:#?}\n{treatments:#?}\n{connections:#?}")
}

/// Areas of `root` holding designed elements, with the restitution of each.
fn restitute(root: &str, collection: &Arc<Collection>) -> BTreeMap<Vec<String>, Area> {
    let mut areas = BTreeMap::new();
    for id in collection.identifiers() {
        if id.root() == root && !areas.contains_key(id.path()) {
            let area = Area::new(
                Path::new(id.version().cloned().unwrap(), id.path().clone()),
                collection.clone(),
            );
            if !area.treatments().is_empty() || !area.models().is_empty() {
                areas.insert(id.path().clone(), area);
            }
        }
    }
    areas
}

fn designs(areas: &BTreeMap<Vec<String>, Area>) -> BTreeMap<Identifier, String> {
    areas
        .values()
        .flat_map(|area| area.treatments())
        .map(|treatment| {
            (
                treatment
                    .design()
                    .descriptor
                    .upgrade()
                    .unwrap()
                    .identifier()
                    .clone(),
                design(treatment.design()),
            )
        })
        .collect()
}

fn codes(areas: &BTreeMap<Vec<String>, Area>) -> BTreeMap<Vec<String>, String> {
    areas
        .iter()
        .map(|(path, area)| (path.clone(), area.implementation()))
        .collect()
}

/// Loads a program, single script or project, from `codes` of its areas.
fn reload(
    original: &std::path::Path,
    root: &str,
    codes: &BTreeMap<Vec<String>, String>,
) -> Arc<Collection> {
    if original.file_name().unwrap() == "Compo.toml" {
        let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("restitution")
            .join(root);
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::copy(original, directory.join("Compo.toml")).unwrap();
        for (path, code) in codes {
            let mut file = directory.clone();
            if path.len() == 1 {
                file.push(melodium_loader::LIB_ROOT_FILENAME);
            } else {
                path.iter().skip(1).for_each(|step| file.push(step));
                file.set_extension("mel");
            }
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, code).unwrap();
        }
        load_file_all_entrypoints(directory.join("Compo.toml"), config())
            .into_result()
            .unwrap_or_else(|err| panic!("{:?}\n{:#?}", err, codes))
            .1
    } else {
        let header = std::fs::read_to_string(original)
            .unwrap()
            .lines()
            .take_while(|line| line.starts_with("#!"))
            .map(|line| format!("{line}\n"))
            .collect::<String>();
        let code = format!("{header}{}", codes.values().next().unwrap());
        load_raw_all_entrypoints(Arc::new(code.clone().into_bytes()), config())
            .into_result()
            .unwrap_or_else(|err| panic!("{:?}\n{}", err, code))
            .1
    }
}

fn programs() -> Vec<PathBuf> {
    let mut programs = Vec::new();
    for entry in std::fs::read_dir(repository().join("tests")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e == "mel").unwrap_or(false) {
            programs.push(path);
        }
    }
    for track in ["tutorial", "showcase"] {
        for entry in std::fs::read_dir(repository().join("examples").join(track)).unwrap() {
            let path = entry.unwrap().path().join("Compo.toml");
            if path.exists() {
                programs.push(path);
            }
        }
    }
    programs.sort();
    programs
}

#[test]
fn programs_are_restituted_identically() {
    let mut restituted = 0;
    for program in programs() {
        let Some((package, collection)) = load_file_all_entrypoints(program.clone(), config())
            .into_result()
            .ok()
        else {
            continue;
        };
        let root = package.name().to_string();

        let areas = restitute(&root, &collection);
        let codes = codes(&areas);
        let collection = reload(&program, &root, &codes);
        let reloaded = restitute(&root, &collection);

        assert_eq!(
            designs(&areas),
            designs(&reloaded),
            "designs of {} differ once restituted",
            program.display()
        );
        assert_eq!(
            codes,
            self::codes(&reloaded),
            "restitution of {} is not stable",
            program.display()
        );
        restituted += 1;
    }
    assert!(restituted > 30, "only {} programs restituted", restituted);
}

/// Packages written in Mélodium are restituted into code the parser reads back with the
/// same connections.
#[test]
fn packages_are_restituted_with_their_connections() {
    let (_, collection) = melodium::load_all(config()).into_result().unwrap();

    let mut roots = collection
        .identifiers()
        .into_iter()
        .map(|id| id.root().to_string())
        .collect::<Vec<_>>();
    roots.sort();
    roots.dedup();

    let mut restituted = 0;
    for root in roots {
        for (path, area) in restitute(&root, &collection) {
            let code = area.implementation();
            let script = melodium_lang::text::Script::build(&code).unwrap_or_else(|err| {
                panic!(
                    "{} gives unreadable code: {:?}\n{}",
                    path.join("/"),
                    err,
                    code
                )
            });
            for (text, treatment) in script.treatments.iter().zip(area.treatments()) {
                let mut read = text
                    .connections
                    .iter()
                    .map(|connection| {
                        format!(
                            "{}.{} -> {}.{}",
                            connection.name_start_point.string,
                            connection.name_data_out.as_ref().unwrap().string,
                            connection.name_end_point.string,
                            connection.name_data_in.as_ref().unwrap().string,
                        )
                    })
                    .collect::<Vec<_>>();
                read.sort();
                let mut designed = treatment
                    .design()
                    .connections
                    .iter()
                    .map(|connection| {
                        format!(
                            "{}.{} -> {}.{}",
                            io(&connection.output_treatment),
                            connection.output_name,
                            io(&connection.input_treatment),
                            connection.input_name,
                        )
                    })
                    .collect::<Vec<_>>();
                designed.sort();
                assert_eq!(read, designed, "connections of {}", path.join("/"));
                restituted += 1;
            }
        }
    }
    assert!(
        restituted > 100,
        "only {} treatments restituted",
        restituted
    );
}
