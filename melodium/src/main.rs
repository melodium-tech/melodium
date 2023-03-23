
use clap::{Parser, Subcommand};
use colored::Colorize;
use melodium::*;
use melodium_doc::Documentation;
use std::path::PathBuf;

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
    #[clap(value_parser)]
    file: String,
    #[clap(value_parser, value_name = "ARGUMENTS")]
    file_args: Vec<String>,
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
    Doc(Doc),
}

pub fn main() {
    

    let cli = Cli::parse();

    if let Some(file) = cli.file {

        let args = Run {
            main: None,
            file,
            file_args: cli.file_args,
        };

        run(args);
    }
    else if let Some(command) = cli.command {
        match command {
            Commands::Run(args) => run(args),
            Commands::Doc(args) => doc(args),
        }
    }
    else {
        eprintln!("{}: missing arguments", "error".bold().red());
        eprintln!("{}: run `melodium --help` get commands", "info".bold().blue());
        std::process::exit(1);
    }
}

fn run(args: Run) {

}

fn doc(args: Doc) {
    let collection = load().unwrap();
    let documentation = Documentation::new(PathBuf::from(args.output), collection);
    documentation.make_documentation().unwrap();
}
