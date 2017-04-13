extern crate clap;
extern crate pom;
extern crate regex;

use clap::{Arg, App};

mod parser;

use parser::*;
use std::path::Path;

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
    let output_file = matches.value_of("OUTPUT").unwrap_or("keymap.png");

    let elements = parse_file(&Path::new(keymap_file));
    println!("{:?}", elements);
}
