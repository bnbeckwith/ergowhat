use pest::prelude::*;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use std::collections::HashMap;

use types::*;

impl_rdp!{
    grammar! {
        file = { (!keymaps ~ any)+ ~ keymaps }
        keymaps = { keymap_header ~ ["="] ~ open_brace ~ keymap_definition* ~ close_brace}

        keymap_header = _{ ["keymaps[][MATRIX_ROWS][MATRIX_COLS]"] }
        keymap_definition = _{ keymap ~ separator? }
        keymap = { ["KEYMAP"] ~ ["("] ~ key_entry* ~ [")"]}
        key_entry = { (key ~ separator?) }

        fn_actions = _{ (!fn_action_header ~ any)* ~ fn_action_header ~ open_brace ~ action_definition* ~ close_brace }
        fn_action_header = _{ ["fn_actions[]"] ~ ["="] }

        open_brace = _{ ["{"] }
        close_brace = _{ ["}"]}

        action_definition = _{ action ~ separator? }
        action = { index ~ ["="] ~ action_type }
        action_type = _{
            action_function
          | action_function_tap
          | action_layer_momentary
          | action_layer_set
          | action_layer_tap_key
          | action_mods_key
          | action_mods_tap_key
        }
        action_function = {        ["ACTION_FUNCTION("] ~ key  ~ [")"] }
        action_function_tap = {    ["ACTION_FUNCTION_TAP("] ~ key ~ [")"] }
        action_layer_momentary = { ["ACTION_LAYER_MOMENTARY("] ~ integer ~ [")"] }
        action_layer_set = {       ["ACTION_LAYER_SET("] ~ integer ~ separator ~ named_key ~ [")"] }
        action_layer_tap_key = {   ["ACTION_LAYER_TAP_KEY("] ~ integer ~ separator ~ key ~ [")"] }
        action_mods_key = {        ["ACTION_MODS_KEY("] ~ key ~ separator ~ key ~ [")"] }
        action_mods_tap_key = {    ["ACTION_MODS_TAP_KEY("] ~ key ~ separator ~ key ~ [")"] }
        
        separator = _{[","]}
        
        comment = _{ block_comment_start ~ (!block_comment_end ~ any)* ~ block_comment_end}
        block_comment_start  = _{ ["/*"] }
        block_comment_end = _{ ["*/"] }

        key = _{ fn_key | named_key }
        fn_key = { ["FN"] ~ action_id }
        named_key = @{ identifier+ }

        identifier = _{ (upper|lower|digit|["_"])}
        action_id = { digit+ }

        index = _{ ["["] ~ integer ~ ["]"] }
        lower = _{ ['a'..'z'] }
        upper = _{ ['A'..'Z'] }
        digit = _{ ['0'..'9']}
        integer = { ["0"] | ['1'..'9'] ~ digit* }
        
        eol = _{ ["\r\r"] | ["\r"] | ["\n"] | &eoi }
        whitespace = _{ [" "] | ["\t"] | ["\n"] }
    }

    process! {
        actions(&self) -> ActionMap {
            (_: action, head: _action_definition(), mut tail: actions()) => {
                tail.insert(head.0, head.1);
                tail
            },
            () => { ActionMap::new() }
        }
        _keymap(&self) -> KeyMap {
            (_: keymap, mut entries: _keys()) => { entries.reverse(); entries }
        }
        _keys(&self) -> KeyMap {
            (_: key_entry, head: _key(), mut tail: _keys()) => {
                tail.push(head);
                tail
            },
            () => { KeyMap::new() }
        }
        _integer(&self) -> u64 {
            (&number: integer) => number.parse::<u64>().unwrap()
        }
        _action_definition(&self) -> (u64, Action) {
            (index: _integer(), setting: _action_type()) => (index, setting)
        }
        _action_type(&self) -> Action {
            (_: action_function, key: _key()) => Action::Function(key),
            (_: action_function_tap, key: _key()) => Action::FunctionTap(key),
            (_: action_layer_momentary, n: _integer()) => Action::LayerMomentary(n),
            (_: action_layer_set, layer: _integer(), &ident: named_key)
                => Action::LayerSet(layer,String::from(ident)),
            (_: action_layer_tap_key, layer: _integer(), key: _key())
                => Action::LayerTapKey(layer, key),
            (_: action_mods_key, modifier: _key(), key: _key())
                => Action::ModsKey(modifier, key),
            (_: action_mods_tap_key, modifier: _key(), key: _key())
                => Action::ModsTapKey(modifier, key)
        }
        _key(&self) -> Key {
            (&ident: named_key) => Key::Key(String::from(ident)),
            (_: fn_key, &id: action_id) => Key::Fx(id.parse::<u64>().unwrap())
        }
    }
}

#[test]
fn test__keymap() {
    let mut parser = Rdp::new(StringInput::new("KEYMAP(F11, TRNS, FN12)"));
    assert!(parser.keymap());
    println!("Q: {:?}",parser.queue());
    let expected = vec![ Key::Key(String::from("F11")),
                         Key::Key(String::from("TRNS")),
                         Key::Fx(12)];
    assert_eq!(parser._keymap(), expected);
}

#[test]
fn test_actions() {
    let mut parser = Rdp::new(StringInput::new("#include<foo>
fn_actions[]= {
[13] = ACTION_FUNCTION_TAP(FN11), 
[0] = ACTION_LAYER_SET(13, ON_BOTH /*comment*/),
}"));
    assert!(parser.fn_actions());
    let actions = parser.actions();
    let mut idx = 0;
    assert_eq!(actions[&idx], Action::LayerSet(13, String::from("ON_BOTH")));
    idx = 13;
    assert_eq!(actions[&idx], Action::FunctionTap(Key::Fx(11)));
}

#[test]
fn test__action() {
    let mut parser = Rdp::new(StringInput::new("[13] = ACTION_FUNCTION(TRNS)"));

    assert!(parser.action_definition());
    parser.set_queue_index(1);
    assert_eq!(parser._action_definition(), (13,Action::Function( Key::Key(String::from("TRNS")))));

    parser = Rdp::new(StringInput::new("[0] /* index */ = ACTION_MODS_TAP_KEY( RGUI /* or left? */, F11)"));
    assert!(parser.action_definition());
    parser.set_queue_index(1);
    assert_eq!(parser._action_definition(),
               (0,Action::ModsTapKey(Key::Key(String::from("RGUI")), Key::Key(String::from("F11")))));
}

#[test]
fn test__action_type() {
    let mut parser = Rdp::new(StringInput::new("ACTION_FUNCTION(TRNS)"));

    assert!(parser.action_type());
    assert_eq!(parser._action_type(), Action::Function( Key::Key(String::from("TRNS"))));

    parser = Rdp::new(StringInput::new("ACTION_FUNCTION_TAP(FN11)"));
    assert!(parser.action_type());
    assert_eq!(parser._action_type(), Action::FunctionTap( Key::Fx(11) ));

    parser = Rdp::new(StringInput::new("ACTION_LAYER_MOMENTARY( /* temp layer */ 2 )"));
    assert!(parser.action_type());
    assert_eq!(parser._action_type(), Action::LayerMomentary(2));

    parser = Rdp::new(StringInput::new("ACTION_LAYER_SET(13, ON_BOTH /*comment*/)"));
    assert!(parser.action_layer_set());
    assert_eq!(parser._action_type(), Action::LayerSet(13, String::from("ON_BOTH")));

    parser = Rdp::new(StringInput::new("ACTION_LAYER_TAP_KEY(28, SPC)"));
    assert!(parser.action_type());
    assert_eq!(parser._action_type(), Action::LayerTapKey(28, Key::Key(String::from("SPC"))));

    parser = Rdp::new(StringInput::new("ACTION_MODS_KEY(LGUI, BSLS)"));
    assert!(parser.action_type());
    assert_eq!(parser._action_type(),
               Action::ModsKey(Key::Key(String::from("LGUI")), Key::Key(String::from("BSLS"))));

    parser = Rdp::new(StringInput::new("ACTION_MODS_TAP_KEY( RGUI /* or left? */, F11)"));
    assert!(parser.action_type());
    assert_eq!(parser._action_type(),
               Action::ModsTapKey(Key::Key(String::from("RGUI")), Key::Key(String::from("F11"))));
}

#[test]
fn test__key() {
    let mut parser = Rdp::new(StringInput::new("FN11"));

    assert!(parser.key());
    assert_eq!(parser._key(), Key::Fx(11));
}

#[test]
fn test_fn_actions() {
    let mut parser = Rdp::new(StringInput::new("fn_actions[] = {
       [0] =   ACTION_FUNCTION(TEENSY_KEY),
    /* Some line comment */
    [11] =   ACTION_MODS_KEY(MOD_LSFT, KC_BSLS),
    [12] =   ACTION_MODS_KEY(MOD_LSFT, KC_MINS),             
    [13] =   ACTION_MODS_KEY(MOD_LSFT, KC_COMM),             
    [14] =   ACTION_MODS_KEY(MOD_LSFT, KC_DOT),
 }"));

    assert!(parser.fn_actions());
    assert!(parser.end());
        
}

#[test]
fn test_action() {
    let mut parser = Rdp::new(StringInput::new("[11] = ACTION_FUNCTION(KC_SPC)"));

    assert!(parser.action());
    assert!(parser.end());

    let queue = vec![
        Token::new(Rule::action, 0, 30),
        Token::new(Rule::integer, 1,3),
        Token::new(Rule::action_function, 7, 30),
        Token::new(Rule::named_key, 23, 29)
    ];

    assert_eq!(parser.queue(), &queue);
}

#[test]
fn test_file() {
    let mut parser = Rdp::new(StringInput::new("#include <foo>\n/*some comment*/\nkeymaps[][MATRIX_ROWS][MATRIX_COLS]={}"));

    assert!(parser.file());
    assert!(parser.end());

    let queue = vec![
        Token::new(Rule::file, 0, 70),
        Token::new(Rule::keymaps, 32, 70),
        
    ];

    assert_eq!(parser.queue(), &queue);
}

//#[test]
pub fn test_keymaps() {
    let mut parser = Rdp::new(StringInput::new("keymaps[][MATRIX_ROWS][MATRIX_COLS] = { KEYMAP(/* layer 0*/ TRNS, LGUI), 
KEYMAP(/*layer 1*/ BTN3, FN14), 
}"));
    parser.keymaps();
    println!("QUEUE: {:?}", parser.queue());
    println!("EXPECTED: {:?}", parser.expected());
    assert!(parser.keymaps());
    assert!(parser.end());

    let queue = vec![
        Token::new(Rule::keymaps, 0, 105),
        Token::new(Rule::keymap, 40, 71),
        Token::new(Rule::named_key, 60, 64),
        Token::new(Rule::named_key, 66, 70),
        Token::new(Rule::keymap, 73, 103),
        Token::new(Rule::named_key, 92, 96),
        Token::new(Rule::fn_key, 98, 102),
        Token::new(Rule::action_id, 100,102)
    ];
    assert_eq!(parser.queue(), &queue)
}

#[test]
fn test_keymap() {
    let mut parser = Rdp::new(StringInput::new("KEYMAP( /* layer 8*/ TRNS, NO, 7, FN14)"));
    assert!(parser.keymap());
    assert!(parser.end());
}

#[test]
fn test_comment() {
    let mut parser = Rdp::new(StringInput::new("/* block *//* comment */"));
    assert!(parser.comment());
    assert!(parser.comment());
    assert!(parser.end());
}

#[test]
fn test_key() {
    let mut parser = Rdp::new(StringInput::new("FN10"));
    println!("QUEUE: {:?}", parser.queue());
    assert!(parser.key());
    assert!(parser.end());

    parser = Rdp::new(StringInput::new("MINS"));
    assert!(parser.key());
    assert!(parser.end());

    parser = Rdp::new(StringInput::new("8"));
    assert!(parser.key());
    assert!(parser.end());
}

#[test]
fn test_eol() {
    let mut parser = Rdp::new(StringInput::new("\n"));
    assert!(parser.eol());
    assert!(parser.end());
}

#[test]
fn test_whitespace() {
    let mut parser = Rdp::new(StringInput::new(" "));
    assert!(parser.whitespace());
    assert!(parser.end());
}
