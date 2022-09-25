
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
#[clap(about, long_about = None)]
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
#[clap(about, long_about = None)]
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
#[clap(about, long_about = None)]
struct Package {
    #[clap(long)]
    stdlib: Option<String>,
    #[clap(value_parser)]
    script: String,
    #[clap(value_parser)]
    file: String,
}

#[derive(clap::Args)]
#[clap(about, long_about = None)]
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
#[clap(about, long_about = None)]
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

    /* 
    let matches = Command::new("Mélodium")
        .version(crate_version!())
        .author("Quentin Vignaud")
        .about("Mélodium script engine")
        .arg(Arg::new("stdlib")
            .long("stdlib")
            .value_name("PATH")
            .help("Sets standard library location")
            .takes_value(true))
        .arg(Arg::new("main")
            .short('m')
            .long("main")
            .value_name("SEQUENCE")
            .help("Sets the main entry point (default to 'Main')")
            .takes_value(true))
        .arg(Arg::new("parseonly")
            .short('p')
            .long("parseonly")
            .help("Parse, and make semantic analysis only"))
        .arg(Arg::new("nolaunch")
            .short('L')
            .long("nolaunch")
            .help("Parse, make semantic analysis, design world and check it, but don't make it live"))
        .arg(Arg::new("doc")
            .long("doc")
            .help("Generates documentation in the given output folder")
            .takes_value(true))
        .arg(Arg::new("draw")
            .long("draw")
            .help("Generates drawing of sequence in given output folder, in SVG format")
            .takes_value(true))
        .arg(Arg::new("script")
            .value_name("SCRIPT")
            .help("Script to run")
            .required(true)
            .index(1))
        .get_matches();*/
    
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
    
    /* 
    let std_path =
    if let Some(path) = matches.value_of("stdlib") {
        Some(path.to_owned())
    }
    else if let Ok(path) = env::var("MELODIUM_STDLIB") {
        Some(path)
    }
    else {
        None
    };

    let main_entry;
    if let Some(main) = matches.value_of("main") {
        main_entry = main.to_owned();
    }
    else {
        main_entry = "Main".to_owned();
    }

    let file_path;
    if let Some(path) = matches.value_of("script") {
        file_path = path.to_owned();
    }
    else {
        eprintln!("No script path specified.");
        exit(1);
    }

    if let Some(path) = matches.value_of("doc") {

        let std_path = if let Some(std_path) = std_path {
            std_path
        } else {
            eprintln!("No stdlib path specified.");
            exit(1);
        };

        make_documentation(&std_path, &file_path, &path.to_string());

        exit(0);
    }

    if let Some(draw) = matches.value_of("draw") {

        let split: Vec<String> = draw.split(",").map(|s| s.to_string()).collect();
        let output = &split[0];
        let ids = Vec::from(&split[1..]);

        make_svg(std_path.as_ref(), &file_path, output, &ids);

        exit(0);
    }

    let parse_only = matches.is_present("parseonly");
    let no_launch = matches.is_present("nolaunch");

    if parse_only && no_launch {
        eprintln!("'parseonly' and 'nolaunch' options are uncompatible.");
        exit(1);
    }

    // Effective run

    if !parse_only && !no_launch {
        match execute(std_path.as_ref(), &file_path, &main_entry) {
            Ok(_) => exit(0),
            Err(_) => exit(1),
        }
    }
    else if parse_only {
        let instance = build(std_path.as_ref(), &file_path);
        print_instance_errors(&instance);
    }
    else if no_launch {
        let (instance, possible_world) = genesis(std_path.as_ref(), &file_path, &main_entry);

        print_instance_errors(&instance);

        if let Some((world, ready)) = possible_world {
            print_world_errors(&world);

            if !ready {
                exit(1);
            }
        }
        else {
            exit(1);
        }
    }
    */
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
