
use std::process::*;

extern crate clap;
use colored::*;
use clap::{Parser, Subcommand};

use melodium::*;

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
    #[clap(short, long)]
    main: Option<String>,
    #[clap(long)]
    stdlib: Option<String>,
    #[clap(value_parser)]
    file: String,
    #[clap(value_parser, value_name = "ARGUMENTS")]
    file_args: Vec<String>,
}

#[derive(clap::Args)]
#[clap(about = "Check program logical integrity")]
struct Check {
    #[clap(short, long)]
    main: Option<String>,
    #[clap(long)]
    stdlib: Option<String>,
    #[clap(value_parser)]
    file: String,
    #[clap(value_parser, value_name = "ARGUMENTS")]
    file_args: Vec<String>,
}

#[derive(clap::Args)]
#[clap(about = "Package script into `jeu` file")]
struct Package {
    #[clap(long)]
    stdlib: Option<String>,
    #[clap(value_parser)]
    script: String,
    #[clap(value_parser)]
    file: String,
}

#[derive(clap::Args)]
#[clap(about = "Render sequences as SVG")]
struct Draw {
    #[clap(short, long)]
    output: String,
    #[clap(long)]
    stdlib: Option<String>,
    #[clap(value_parser)]
    file: String,
    #[clap(value_parser)]
    entry: Vec<String>,
}

#[derive(clap::Args)]
#[clap(about = "Make documentation of program")]
struct Doc {
    #[clap(long)]
    stdlib: String,
    #[clap(value_parser)]
    input: String,
    #[clap(value_parser)]
    output: String,
}

#[derive(Subcommand)]
enum Commands {
    Run(Run),
    Check(Check),
    Package(Package),
    Draw(Draw),
    Doc(Doc),
}

fn main() {
    
    let cli = Cli::parse();

    if let Some(file) = cli.file {

        let args = Run {
            main: None,
            stdlib: None,
            file,
            file_args: cli.file_args,
        };

        run(args);
    }
    else if let Some(command) = cli.command {
        match command {
            Commands::Run(args) => run(args),
            Commands::Check(args) => check(args),
            Commands::Package(args) => package(args),
            Commands::Doc(args) => doc(args),
            Commands::Draw(args) => draw(args),
        }
    }
    else {
        eprintln!("{}: missing arguments", "error".bold().red());
        eprintln!("{}: run `melodium --help` get commands", "info".bold().blue());
        exit(1);
    }
}

fn run(args: Run) {
    match execute(args.stdlib.as_ref(), &args.file, &args.main.unwrap_or(String::from("Main"))) {
        Ok(_) => exit(0),
        Err(_) => exit(1),
    }
}

fn check(args: Check) {

    let instance = build(args.stdlib.as_ref(), &args.file);

    if !instance.errors().is_empty() {
        eprintln!("{}: program is valid", "success".bold().green());
        exit(0);
    }
    else {
        print_instance_errors(&instance);
        exit(1);
    }
}

fn package(args: Package) {

    make_package(args.stdlib.as_ref(), &args.script, &args.file);
}

fn doc(args: Doc) {

    make_documentation(&args.stdlib, &args.input, &args.output);
}

fn draw(args: Draw) {

    make_svg(args.stdlib.as_ref(), &args.file, &args.output, &args.entry);
}
