extern crate clap;
extern crate ergodox_keymap_parser;

use clap::{Arg, App};

use ergodox_keymap_parser::*;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let matches = App::new("ergowhat")
        .version("0.1")
        .author("bnbeckwith <bnbeckwith@gmail.com>")
        .about("Prints out TMK/Ergodox layouts")
        .arg(Arg::with_name("FILE")
             .help("Keymap file to parse")
             .required(true)
             .index(1))
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .value_name("OUTPUT")
             .help("Sets the output filename")
             .takes_value(true))
        .get_matches();

    let keymap_file = matches.value_of("FILE").unwrap();
    let output_file = matches.value_of("OUTPUT").unwrap_or("keymap.svg");

    let mut f = File::open(Path::new(keymap_file)).expect("File couldn't be opened");
    let mut input = String::new();
    f.read_to_string(&mut input).expect("Unable to read file");
    
    let svg = to_svg(&input);
    let mut output = File::create(Path::new(output_file)).unwrap();
    output.write_all(&svg.into_bytes()).expect("Couldn't write file");
}
