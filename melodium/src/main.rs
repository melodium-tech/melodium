use clap::{Parser, Subcommand};
use colored::Colorize;
use melodium::*;
use melodium_common::descriptor::{Collection, Identifier, LoadingResult, Status};
use core::convert::TryFrom;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(value_parser)]
    /// Program file to run (see `melodium run --help`)
    file: Option<String>,

    #[clap(value_parser, value_name = "ARGUMENTS")]
    /// Arguments to pass to program (see `melodium run --help`)
    file_args: Vec<String>,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Args)]
/// Run given program, with optionnal arguments
struct Run {
    #[clap(value_parser)]
    /// Program file to run, can be either `.mel` or `.jeu` file.
    file: Option<String>,
    #[clap(long)]
    /// Path to use for packages.
    path: Vec<String>,
    #[clap(long, value_name = "IDENTIFIER")]
    /// Force identifier to use as entrypoint.
    force_entry: Option<String>,
    #[clap(short, long, value_name = "ENTRYPOINT")]
    /// Entrypoint to use (default to 'main').
    entry: Option<String>,
    #[clap(value_parser, value_name = "ARGUMENTS")]
    /// Arguments to pass to program.
    file_args: Vec<String>,
}

#[derive(clap::Args)]
/// Check given program
struct Check {
    #[clap(value_parser)]
    /// Program file to check, can be either `.mel` or `.jeu` file.
    file: Option<String>,
    #[clap(long)]
    /// Path to look for packages.
    path: Vec<String>,
    #[clap(long, value_name = "IDENTIFIER")]
    /// Force identifier to use as entrypoint.
    force_entry: Option<String>,
    #[clap(short, long, value_name = "ENTRYPOINT")]
    /// Entrypoint to use (default to 'main').
    entry: Option<String>,
}

#[derive(Subcommand)]
/// Manage `.jeu` package files

enum Jeu {
    Build(JeuBuild),
    Extract(JeuExtract),
}

#[cfg(all(feature = "jeu", feature = "fs"))]
#[derive(clap::Args)]
/// Build a `.jeu` file from package on located in input directory
struct JeuBuild {
    #[clap(value_parser)]
    /// Mélodium package to build as `.jeu` data file
    package: String,
    #[clap(value_parser)]
    /// Output file, `.jeu` suffix is appended if not present
    jeu_file: String,
}

#[cfg(all(feature = "jeu", feature = "fs"))]
#[derive(clap::Args)]
/// Extract a `.jeu` file into designated directory
struct JeuExtract {
    #[clap(value_parser)]
    /// Input `.jeu` data file to extract as Mélodium package
    jeu_file: String,
    #[clap(value_parser)]
    /// Output location to extract in, directory is created if not existing
    output_location: String,
}

#[cfg(not(all(feature = "jeu", feature = "fs")))]
#[derive(clap::Args)]
/// [Not available in this release] Build a `.jeu` file from package on located in input directory
struct JeuBuild {}

#[cfg(not(all(feature = "jeu", feature = "fs")))]
#[derive(clap::Args)]
/// [Not available in this release] Extract a `.jeu` file into designated directory
struct JeuExtract {}

#[cfg(feature = "doc")]
#[derive(clap::Args)]
/// Generates documentation as mdBook.
struct Doc {
    #[clap(long)]
    /// Document every loaded package (default if none --file or --packages options are provided)
    full: bool,
    #[clap(short, long)]
    /// Packages to document
    packages: Vec<String>,
    #[clap(short, long)]
    /// Package file to document, can be either `.mel` or `.jeu` file.
    file: Option<String>,
    #[clap(long)]
    /// Path to look for packages.
    path: Vec<String>,
    #[clap(value_parser)]
    /// Output location to write documentation, directory is created if it does not exists.
    output: String,
}

#[cfg(not(feature = "doc"))]
#[derive(clap::Args)]
/// [Not available in this release] Generates documentation
struct Doc {}

#[derive(Subcommand)]
enum Commands {
    Run(Run),
    Check(Check),
    #[clap(subcommand)]
    Jeu(Jeu),
    Doc(Doc),
}

pub fn main() {
    let cli = Cli::parse();

    if let Some(file) = cli.file {
        let args = Run {
            entry: None,
            path: Vec::new(),
            file: Some(file),
            file_args: cli.file_args,
            force_entry: None,
        };

        run(args);
    } else if let Some(command) = cli.command {
        match command {
            Commands::Run(args) => run(args),
            Commands::Check(args) => check(args),
            #[cfg(feature = "doc")]
            Commands::Doc(args) => doc(args),
            #[cfg(not(feature = "doc"))]
            Commands::Doc(_) => {}
            #[cfg(all(feature = "jeu", feature = "fs"))]
            Commands::Jeu(Jeu::Build(args)) => build_jeu(args),
            #[cfg(not(all(feature = "jeu", feature = "fs")))]
            Commands::Jeu(Jeu::Build(_)) => {}
            #[cfg(all(feature = "jeu", feature = "fs"))]
            Commands::Jeu(Jeu::Extract(args)) => extract_jeu(args),
            #[cfg(not(all(feature = "jeu", feature = "fs")))]
            Commands::Jeu(Jeu::Extract(_)) => {}
        }
    } else {
        eprintln!("{}: missing arguments", "error".bold().red());
        eprintln!(
            "{}: run `melodium --help` get commands",
            "info".bold().blue()
        );
        std::process::exit(1);
    }
}

fn run(args: Run) {
    if let Ok((identifier, collection)) = check_load(Check {
        file: args.file,
        path: args.path,
        entry: args.entry,
        force_entry: args.force_entry,
    }) {
        let launch = launch(collection, &identifier);
        if let Some(failure) = launch.failure() {
            eprintln!("{}: {failure}", "failure".bold().red());
        }
        launch
            .errors()
            .iter()
            .for_each(|err| eprintln!("{}: {err}", "error".bold().red()));
    } else {
        std::process::exit(1);
    }
}

fn check(args: Check) {
    if let Ok(_) = check_load(args) {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn check_load(args: Check) -> Result<(Identifier, Arc<Collection>), ()> {
    let mut config = LoadingConfig {
        core_packages: Vec::new(),
        search_locations: args
            .path
            .iter()
            .map(|p| PathBuf::from(p))
            .collect::<Vec<_>>(),
        raw_elements: Vec::new(),
    };

    let file = if let Some(file) = args.file.as_ref().map(|f| PathBuf::from(f)) {
        if file.is_file() {
            Some(file)
        } else if file.is_dir() {
            config.search_locations.push(file);
            None
        } else {
            None
        }
    } else {
        None
    };

    let result = match (&args.entry, &args.force_entry, file) {
        (None, None, Some(file)) => load_file(file, "main", config),
        (Some(entrypoint), None, Some(file)) => load_file(file, entrypoint, config),
        (None, Some(identifier), Some(file)) => {
            let identifier = match Identifier::try_from(identifier) {
                Ok(id) => id,
                Err(str) => {
                    eprintln!("{}: '{str}' is not a valid identifier", "error".bold().red());
            return Err(());
                }
            };
            load_file_force_entrypoint(file, &identifier, config)},
        (_, _, None) => {
            eprintln!("{}: file must be given", "error".bold().red());
            return Err(());
        }
        (Some(_), Some(_), _) => {
            eprintln!("{}: entrypoint cannot be specified and forced at same time", "error".bold().red());
            return Err(());
        }
    };

    print_result(&result);

    result
        .into_result()
        .map(|(pkg, collection)| {
            (
                pkg.entrypoints()
                    .get(args.entry.as_ref().unwrap_or(&"main".to_string()))
                    .unwrap()
                    .clone(),
                collection,
            )
        })
        .map_err(|_| ())
}

#[cfg(feature = "doc")]
fn doc(args: Doc) {
    let mut loading_config = core_config();
    loading_config.extend(LoadingConfig {
        core_packages: Vec::new(),
        search_locations: args.path.into_iter().map(|p| p.into()).collect(),
        raw_elements: Vec::new(),
    });

    let loader = melodium_loader::Loader::new(loading_config);
    let mut loading_result = LoadingResult::new_success(());
    let mut doc_packages = Vec::new();
    if let Some(file) = args.file {
        let path = file.into();
        match std::fs::read(&path) {
            Ok(data) => {
                if let Some(pkg) =
                    loading_result.merge_degrade_failure(loader.load_raw(Arc::new(data)))
                {
                    doc_packages.push(pkg.name().to_string())
                }
            }
            Err(err) => {
                loading_result = loading_result.and_degrade_failure(LoadingResult::new_failure(
                    melodium_common::descriptor::LoadingError::unreachable_file(
                        242,
                        path,
                        err.to_string(),
                    ),
                ))
            }
        }
    }
    for pkg in args.packages {
        if loading_result
            .merge_degrade_failure(loader.load_package(
                &melodium_common::descriptor::PackageRequirement {
                    package: pkg.clone(),
                    version_requirement:
                        melodium_common::descriptor::VersionReq::parse(">=0.0.0").unwrap(),
                },
            ))
            .is_some()
        {
            doc_packages.push(pkg);
        }
    }

    loading_result.merge_degrade_failure(loader.load_all());

    let subject = if doc_packages.is_empty() || args.full {
        melodium_doc::DocumentationSubject::All
    } else if doc_packages.len() > 1 {
        melodium_doc::DocumentationSubject::Multiple(doc_packages)
    } else {
        melodium_doc::DocumentationSubject::One(doc_packages.pop().unwrap_or_default())
    };

    print_result(&loading_result);

    let collection = loader.collection().clone();
    let documentation =
        melodium_doc::Documentation::new(PathBuf::from(&args.output), collection, subject);
    if let Err(err) = documentation.make_documentation() {
        eprintln!("{}: {err}", "error".bold().red());
        std::process::exit(1);
    } else {
        println!(
            "{}: documentation generated, run `mdbook build` in '{path}' to build publishable book",
            "success".bold().green(),
            path = args.output
        )
    }
}

#[cfg(all(feature = "jeu", feature = "fs"))]
fn build_jeu(args: JeuBuild) {
    let input = PathBuf::from(args.package);
    let mut output = PathBuf::from(args.jeu_file);
    output.set_extension("jeu");
    let result = melodium_loader::build_jeu(&input, &output);

    print_result(&result);

    if result.is_failure() {
        std::process::exit(1);
    }
}

#[cfg(all(feature = "jeu", feature = "fs"))]
fn extract_jeu(args: JeuExtract) {
    let input = PathBuf::from(args.jeu_file);
    let output = PathBuf::from(args.output_location);

    if let Err(err) = melodium_loader::extract_jeu(&input, &output) {
        eprintln!("{}: {err}", "error".bold().red());
        std::process::exit(1);
    }
}

fn print_result<T>(result: &LoadingResult<T>) {
    match result {
        Status::Success { success: _, errors } => {
            errors
                .iter()
                .for_each(|err| eprintln!("{}: {err}", "error".bold().red()));
        }
        Status::Failure { failure, errors } => {
            eprintln!("{}: {failure}", "failure".bold().red());
            errors
                .iter()
                .for_each(|err| eprintln!("{}: {err}", "error".bold().red()));
        }
    }
}
