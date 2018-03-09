#![recursion_limit="128"]
#[macro_use]
extern crate pest;
//extern crate pest_derive;
extern crate svg;
extern crate regex;

mod types;
pub mod parser;
mod image;

use image::*;
use parser::*;

pub fn to_svg(input: &str) -> String {
    let (kms,am) = parse_string(&input);

    Keyboard::new(kms,am).svg()
}
