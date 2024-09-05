use clap::{Arg, ArgAction, Command, Parser, Subcommand};
use colored::Colorize;
use core::convert::TryFrom;
use melodium::*;
use melodium_common::{
    descriptor::{Collection, Entry, Identifier, LoadingResult, Status, Treatment},
    executive::Value,
};
use melodium_lang::{
    semantic::{NoneDeclarativeElement, Value as SemanticValue},
    text::{get_words, Value as TextValue},
};
use std::path::PathBuf;
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(value_parser)]
    /// Program file to run (see `melodium run --help`)
    file: Option<String>,

    #[clap(value_parser, allow_hyphen_values(true), value_name = "ARGUMENTS")]
    /// Arguments to pass to program (see `melodium run --help`)
    file_args: Vec<String>,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Args)]
/// Run given program, with optionnal arguments
struct Run {
    #[clap(long)]
    /// Path to use for packages.
    path: Vec<String>,
    #[clap(long, value_name = "IDENTIFIER")]
    /// Force identifier to be used as entrypoint.
    force_entry: Option<String>,
    #[clap(value_parser)]
    /// Program file to run, can be either `.mel` or `.jeu` file.
    file: Option<String>,
    #[clap(
        value_parser,
        allow_hyphen_values(true),
        value_name = "COMMAND ARGUMENTS"
    )]
    /// Arguments to pass to program, if COMMAND is not set it defaults to `main`.
    prog_args: Vec<String>,
}

#[derive(clap::Args)]
/// Check given program
struct Check {
    #[clap(long)]
    /// Path to look for packages.
    path: Vec<String>,
    #[clap(long, value_name = "IDENTIFIER")]
    /// Force identifier to be used as entrypoint.
    force_entry: Option<String>,
    #[clap(value_parser)]
    /// Program file to check, can be either `.mel` or `.jeu` file.
    file: Option<String>,
    #[clap(value_parser, value_name = "COMMAND")]
    /// Entrypoint command to check (default to `main`).
    prog_cmd: Option<String>,
}

#[derive(clap::Args)]
/// Give information about program or package
struct Info {
    #[clap(long)]
    /// Path to look for packages.
    path: Vec<String>,
    #[clap(value_parser)]
    /// Program file, can be either `.mel` or `.jeu` file.
    name: String,
}

#[cfg(feature = "distributed")]
#[derive(clap::Args)]
/// Makes engine available for distribution
struct Dist {
    #[clap(short, long)]
    /// IP to listen on.
    ip: String,
    #[clap(short, long)]
    /// Port to listen on.
    port: u16,
}

#[cfg(not(feature = "distributed"))]
#[derive(clap::Args)]
/// [Not available in this release] Makes engine available for distribution
struct Dist {}

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
/// Generates documentation
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
    /// Output location to write documentation as mdBook, directory is created if it does not exists.
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
    Info(Info),
    Dist(Dist),
    #[clap(subcommand)]
    Jeu(Jeu),
    Doc(Doc),
}

pub fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let cli = Cli::parse();

    if let Some(file) = cli.file {
        let args = Run {
            path: Vec::new(),
            file: Some(file),
            prog_args: cli.file_args,
            force_entry: None,
        };

        run(args);
    } else if let Some(command) = cli.command {
        match command {
            Commands::Run(args) => run(args),
            Commands::Check(args) => check(args),
            Commands::Info(args) => info(args),
            #[cfg(feature = "distributed")]
            Commands::Dist(args) => dist(args),
            #[cfg(not(feature = "distributed"))]
            Commands::Dist(_) => {}
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
        force_entry: args.force_entry.clone(),
        prog_cmd: args.prog_args.first().cloned(),
    }) {
        let (entry_name, arguments) = if args
            .prog_args
            .first()
            .map(|arg| !arg.starts_with('-'))
            .unwrap_or(false)
        {
            let mut args = args.prog_args.clone();
            if args.first().is_some() {
                (Some(args.remove(0)), args)
            } else {
                (None, args)
            }
        } else {
            if args.force_entry.is_some() {
                (None, args.prog_args.clone())
            } else {
                (Some("main".to_string()), args.prog_args.clone())
            }
        };

        let treatment = if let Some(Entry::Treatment(tr)) = collection.get(&(&identifier).into()) {
            tr
        } else {
            eprintln!("{}: entrypoint must be a treatment", "failure".bold().red());
            std::process::exit(1);
        };

        let params = parse_args(entry_name, treatment, arguments);

        let launch = async_std::task::block_on(launch(collection, &identifier, params));
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

    let result = match (
        args.prog_cmd.as_ref().filter(|arg| !arg.starts_with('-')),
        &args.force_entry,
        file,
    ) {
        (None, None, Some(file)) => load_file(file, "main", config),
        (Some(entrypoint), None, Some(file)) => load_file(file, entrypoint, config),
        (None, Some(identifier), Some(file)) => {
            let identifier = match Identifier::try_from(identifier) {
                Ok(id) => id,
                Err(str) => {
                    eprintln!(
                        "{}: '{str}' is not a valid identifier",
                        "error".bold().red()
                    );
                    return Err(());
                }
            };
            load_file_force_entrypoint(file, &identifier, config)
        }
        (_, _, None) => {
            eprintln!("{}: file must be given", "error".bold().red());
            return Err(());
        }
        (Some(_), Some(_), _) => {
            eprintln!(
                "{}: entrypoint cannot be specified and forced at same time",
                "error".bold().red()
            );
            return Err(());
        }
    };

    print_result(&result);

    result
        .into_result()
        .map(|(pkg, collection)| {
            (
                pkg.entrypoints()
                    .get(
                        args.prog_cmd
                            .as_ref()
                            .filter(|arg| !arg.starts_with('-'))
                            .unwrap_or(&"main".to_string()),
                    )
                    .unwrap()
                    .clone(),
                collection,
            )
        })
        .map_err(|_| ())
}

fn info(args: Info) {
    let mut loading_config = core_config();
    loading_config
        .search_locations
        .extend(args.path.into_iter().map(|p| p.into()));
    let result = load_file_all_entrypoints(args.name.into(), loading_config);

    print_result(&result);

    if let Some((pkg, collection)) = result.success() {
        let mut cmd = Command::new(pkg.name().to_string())
            .no_binary_name(true)
            .disable_help_subcommand(true)
            .before_long_help(format!("{}\nVersion {}", pkg.name(), pkg.version()))
            .version(pkg.version().to_string())
            .disable_version_flag(true);
        for (name, id) in pkg.entrypoints() {
            if let Some(Entry::Treatment(treatment)) = collection.get(&id.into()) {
                let sub_cmd = build_cmd(Some(name.clone()), treatment);
                cmd = cmd.subcommand(sub_cmd);
            }
        }
        let _ = cmd.print_long_help();
    } else {
        std::process::exit(1);
    }
}

#[cfg(feature = "distributed")]
fn dist(args: Dist) {
    use core::str::FromStr;
    use std::net::{IpAddr, SocketAddr};

    use melodium_common::descriptor::Version;

    let loader = melodium_loader::Loader::new(core_config());

    let bind = SocketAddr::new(IpAddr::from_str(&args.ip).unwrap(), args.port);

    async_std::task::block_on(melodium_distributed::launch_listen(
        bind,
        &Version::parse(melodium::VERSION).unwrap(),
        loader,
    ));
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

fn build_cmd(displayed_name: Option<String>, treatment: &Arc<dyn Treatment>) -> Command {
    let mut cmd =
        Command::new(displayed_name.unwrap_or_else(|| treatment.identifier().to_string()))
            .no_binary_name(true)
            .about(treatment.documentation().to_string());

    for (name, param) in treatment.parameters() {
        let mut arg = Arg::new(name)
            .long(name)
            .action(ArgAction::Set)
            .help(param.described_type().to_string());
        if let Some(default) = param.default() {
            arg = arg.default_value(default.to_string());
        }
        cmd = cmd.arg(arg);
    }

    cmd
}

fn parse_args(
    displayed_name: Option<String>,
    treatment: &Arc<dyn Treatment>,
    arguments: Vec<String>,
) -> HashMap<String, Value> {
    let cmd = build_cmd(displayed_name, treatment);

    let matches = cmd.get_matches_from(arguments);

    let mut parsed = HashMap::new();
    for (name, param) in treatment.parameters() {
        if let Some(raw_value) = matches.get_one::<String>(name.as_str()) {
            if matches.value_source(name.as_str()) != Some(clap::parser::ValueSource::DefaultValue)
            {
                let mut words = match get_words(raw_value) {
                    Ok(w) => w,
                    Err(_) => {
                        eprintln!(
                            "{}: argument '{name}' cannot be parsed",
                            "failure".bold().red()
                        );
                        std::process::exit(1);
                    }
                };
                words.push(melodium_lang::text::Word::default());

                let value = match TextValue::build_from_first_item(
                    &mut words.windows(2),
                    &mut HashMap::new(),
                ) {
                    Ok(v) => v,
                    Err(err) => {
                        eprintln!(
                            "{}: argument '{name}' cannot be parsed: {err}",
                            "failure".bold().red()
                        );
                        std::process::exit(1);
                    }
                };

                let decl_element = Arc::new(RwLock::new(NoneDeclarativeElement));
                let value = match SemanticValue::new(decl_element.clone(), value) {
                    Status::Success { success, errors } => {
                        if !errors.is_empty() {
                            errors
                                .iter()
                                .for_each(|err| eprintln!("{}: {err}", "error".bold().red()));
                            std::process::exit(1);
                        }
                        success
                    }
                    Status::Failure { failure, errors } => {
                        eprintln!("{}: {failure}", "failure".bold().red());
                        errors
                            .iter()
                            .for_each(|err| eprintln!("{}: {err}", "error".bold().red()));
                        std::process::exit(1);
                    }
                };

                let datatype = if let Some(dt) = param.described_type().to_datatype(&HashMap::new())
                {
                    dt
                } else {
                    eprintln!(
                        "{}: provided treatment have generics, it cannot be used as entrypoint",
                        "failure".bold().red()
                    );
                    std::process::exit(1);
                };

                let value = match value.read().unwrap().make_executive_value(&datatype) {
                    Status::Success { success, errors } => {
                        if !errors.is_empty() {
                            errors
                                .iter()
                                .for_each(|err| eprintln!("{}: {err}", "error".bold().red()));
                            std::process::exit(1);
                        }
                        success
                    }
                    Status::Failure { failure, errors } => {
                        eprintln!("{}: {failure}", "failure".bold().red());
                        errors
                            .iter()
                            .for_each(|err| eprintln!("{}: {err}", "error".bold().red()));
                        std::process::exit(1);
                    }
                };

                parsed.insert(name.clone(), value);
            }
        }
    }

    parsed
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
