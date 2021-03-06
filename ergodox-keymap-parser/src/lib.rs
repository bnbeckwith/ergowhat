extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate svg;
extern crate regex;

mod types;
pub mod parser;
mod image;

use image::*;
use parser::*;

pub fn to_svg(input: &str) -> String {
    let (kms,am) = parse_string(&input);

    // Some change [deleteme]
    Keyboard::new(kms,am).svg()
}
