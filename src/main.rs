
extern crate melodium_rust;

use std::env;
use std::ffi::OsString;
use melodium_rust::script_file::ScriptFile;

fn main() {

    let args: Vec<OsString> = env::args_os().collect();

    let file_path = args.last();
    if file_path.is_none() {
        println!("Path to script must be provided.");
        return;
    }

    println!("Reading file \"{}\"…", file_path.unwrap().to_string_lossy());

    let mut script_file = ScriptFile::new(file_path.unwrap());
    let load_error = script_file.load();
    if load_error.is_ok() {
        let parse_error = script_file.parse();
        if parse_error.is_ok() {
            println!("{} annotations, {} models, {} sequences",
            script_file.script().annotations.len(),
            script_file.script().models.len(),
            script_file.script().sequences.len()
            );
        }
        else {
            println!("{}", parse_error.unwrap_err());
        }
    }
    else {
        println!("{}", load_error.unwrap_err());
    }

    

    /*let words = lang_trial::text::get_words("Machin\n\t\t\"Ceci est un chaîne de caractères.\"-780.5681# Bidule\n\"Chaîne \\ un peu \\\"spéciale\\\"");

    for word in words.unwrap_or_else(|err| err) {
        println!("{}", word.text);
    }*/
}


