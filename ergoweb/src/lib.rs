#![feature(use_extern_macros,wasm_custom_section, wasm_import_module)]
extern crate wasm_bindgen;
extern crate ergodox_keymap_parser;

use wasm_bindgen::prelude::*;
use ergodox_keymap_parser::to_svg;

#[wasm_bindgen]
extern {
    type HTMLDocument;
    static document: HTMLDocument;
    #[wasm_bindgen(method)]
    fn getElementById(this: &HTMLDocument, idName: &str) -> Element;

    type Element;
    #[wasm_bindgen(method, setter=innerHTML)]
    fn set_inner_html(this: &Element, html: &str);
}

#[wasm_bindgen]
pub fn make_svg(contents: &str, id: &str){
    let svg = to_svg(contents);

    let elem = document.getElementById(id);
    elem.set_inner_html(&svg);
}
