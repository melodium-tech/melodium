use clap::{Parser, Subcommand};
use colored::Colorize;
use core::convert::TryFrom;
use melodium::*;
use melodium_common::descriptor::{Collection, Identifier};
use melodium_doc::Documentation;
use melodium_loader::Loader;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(value_parser)]
    file: Option<String>,

    #[clap(value_parser, value_name = "ARGUMENTS")]
    file_args: Vec<String>,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Args)]
#[clap(about = "Run given program, with optionnal arguments")]
struct Run {
    #[clap(value_parser)]
    file: Option<String>,
    #[clap(short, long)]
    path: Vec<String>,
    #[clap(short, long)]
    main: Option<String>,
    #[clap(value_parser, value_name = "ARGUMENTS")]
    file_args: Vec<String>,
}

#[derive(clap::Args)]
#[clap(about = "Check given program")]
struct Check {
    #[clap(value_parser)]
    file: Option<String>,
    #[clap(short, long)]
    path: Vec<String>,
    #[clap(short, long)]
    main: Option<String>,
}

#[derive(clap::Args)]
#[clap(about = "Generates documentation")]
struct Doc {
    #[clap(short, long)]
    main: Option<String>,
    #[clap(short, long)]
    root: Vec<String>,
    #[clap(value_parser)]
    output: String,
}

#[derive(Subcommand)]
enum Commands {
    Run(Run),
    Check(Check),
    Doc(Doc),
}

pub fn main() {
    let cli = Cli::parse();

    if let Some(file) = cli.file {
        let args = Run {
            main: None,
            path: Vec::new(),
            file: Some(file),
            file_args: cli.file_args,
        };

        run(args);
    } else if let Some(command) = cli.command {
        match command {
            Commands::Run(args) => run(args),
            Commands::Check(args) => check(args),
            Commands::Doc(args) => doc(args),
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
        main: args.main,
    }) {
        if let Err(errs) = launch(collection, &identifier) {
            for err in errs {
                eprintln!("{}: logic: {err:?}", "error".bold().red());
            }
            std::process::exit(1);
        }
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
    let id = if let Some(main) = args.main {
        match Identifier::try_from(&main) {
            Ok(id) => Some(id),
            Err(err) => {
                eprintln!(
                    "{}: '{err}' is not a valid identifier",
                    "error".bold().red()
                );
                return Err(());
            }
        }
    } else {
        None
    };

    let mut config = LoadingConfig {
        core_packages: Vec::new(),
        search_locations: args
            .path
            .iter()
            .map(|p| PathBuf::from(p))
            .collect::<Vec<_>>(),
    };

    let file = if let Some(file) = args.file.map(|f| PathBuf::from(f)) {
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

    let success;
    let error;
    match (id, file) {
        (Some(id), None) => match load_entry(config, &id) {
            Ok(loaded_collection) => {
                success = Some((id, loaded_collection));
                error = None;
            }
            Err(errs) => {
                success = None;
                error = Some(errs);
            }
        },
        (None, Some(file)) => match load_file(file, config) {
            Ok((id, loaded_collection)) => {
                success = Some((id, loaded_collection));
                error = None;
            }
            Err(errs) => {
                success = None;
                error = Some(errs);
            }
        },
        (Some(id), Some(file)) => match load_file(file, config) {
            Ok((_, loaded_collection)) => {
                success = Some((id, loaded_collection));
                error = None;
            }
            Err(errs) => {
                success = None;
                error = Some(errs);
            }
        },
        _ => {
            eprintln!("{}: file or identifier must be given", "error".bold().red());
            return Err(());
        }
    }

    if let Some(err) = error {
        eprintln!("{}: loading: {err:?}", "error".bold().red());
        return Err(());
    } else if let Some(success) = success {
        return Ok(success);
    } else {
        return Err(());
    }
}

fn doc(args: Doc) {
    let loader = Loader::new(core_config());
    loader.load_all().unwrap();

    let collection = loader.collection().clone();
    let documentation = Documentation::new(PathBuf::from(args.output), collection);
    documentation.make_documentation().unwrap();
}
