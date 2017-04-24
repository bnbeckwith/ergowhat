use pom::{Parser, DataInput};
use pom::char_class::*;
use pom::parser::*;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use std::collections::HashMap;

use types::*;

fn space() -> Parser<u8, ()> {
	one_of(b" \t\r\n").repeat(1..).discard()
}

fn integer() -> Parser<u8, u64> {
	let integer = one_of(b"123456789") - one_of(b"0123456789").repeat(0..) | sym(b'0');
	integer.collect().convert(String::from_utf8).convert(|s| s.parse::<u64>())
}

fn eol() -> Parser<u8, ()> {
    let eol = sym(b'\r') * sym(b'\r') | sym(b'\r') | sym(b'\n');
    eol.discard()
}

fn line_comment() -> Parser<u8, ()> {
    seq(b"//") * none_of(b"\r\n").repeat(0..) * eol().discard()
}

fn block_comment() -> Parser<u8, ()> {
    let not_end = one_of(b"*") - !one_of(b"/") | none_of(b"*");
    let comment = seq(b"/*") - not_end.repeat(0..) - seq(b"*/");
    comment.discard()
}

fn comment() -> Parser<u8, ()> {
    line_comment() | block_comment() 
}

fn whitespace() -> Parser<u8,()> {
    let ws = space() | comment();
    ws.repeat(0..).discard()
}

fn fn_key() -> Parser<u8, Key> {
    seq(b"FN") * integer().map(|i| Key::Fx(i))
}

fn id(term: u8) -> bool {
    alpha(term)
        || digit(term)
        || term == b'_'
}

fn ident() -> Parser<u8, String> {
    whitespace() * is_a(id).repeat(1..).collect().convert(String::from_utf8) - whitespace()
}

fn reg_key() -> Parser<u8, Key> {
    ident().map(|k| Key::Key(k))
}

fn key() -> Parser<u8, Key> {
    fn_key() | reg_key()
}

fn keymap() -> Parser<u8, Vec<Key>> {
    let keys = list(call(key), sym(b',') * whitespace());
    seq(b"KEYMAP(") * whitespace() * keys - sym(b')')
}

fn keymaps() -> Parser<u8, Vec<Vec<Key>>> {
    let keymaps = list(call(keymap), sym(b',') * whitespace());
    seq(b"keymaps[]") * seq(b"[MATRIX_ROWS]") * seq(b"[MATRIX_COLS]") * whitespace()
        * sym(b'=') * whitespace()
        * sym(b'{') * whitespace()
        * keymaps - whitespace() * sym(b',').opt() * whitespace() * seq(b"};")
}

fn index() -> Parser<u8, u64> {
    sym(b'[') * whitespace() * integer() - whitespace()* sym(b']')
}

fn comma_separator() -> Parser<u8,()> {
    whitespace() * sym(b',').discard() * whitespace()
}

fn action() -> Parser<u8, Action> {
    seq(b"ACTION_") *
        ((seq(b"FUNCTION(") * key() - sym(b')')).map(|k| Action::Function(k))
         | (seq(b"FUNCTION_TAP(") * key() - sym(b')')).map(|k| Action::FunctionTap(k))
         | (seq(b"LAYER_MOMENTARY(") * integer() - sym(b')')).map(|l| Action::LayerMomentary(l))
         | (seq(b"LAYER_SET(") * integer() - comma_separator() + ident() - sym(b')')).map(|(l,i)| Action::LayerSet(l,i))
         | (seq(b"LAYER_TAP_KEY(") * integer() - comma_separator() + key() - sym(b')')).map(|(l,k)| Action::LayerTapKey(l,k))
         | (seq(b"MODS_KEY(") * key() - comma_separator() + key() - sym(b')')).map(|(k1,k2)| Action::ModsKey(k1,k2))
         | (seq(b"MODS_TAP_KEY(") * key() - comma_separator() + key() - sym(b')')).map(|(k1,k2)| Action::ModsTapKey(k1,k2))
        )
}

fn action_and_index() -> Parser<u8, (u64, Action)> {
    whitespace() * index() - whitespace() * sym(b'=') * whitespace() + action()
}

fn actions() -> Parser<u8, ActionMap> {
    let line = list(call(action_and_index), sym(b',') * whitespace());
    let actions = seq(b"fn_actions[]") * whitespace()
        * sym(b'=') * whitespace()
        * sym(b'{') * whitespace()
        * line - sym(b',').opt() * whitespace() * seq(b"};");
    actions.map(|members| members.into_iter().collect::<HashMap<_,_>>())
}

pub fn parse_file(filename: &Path) -> (KeyMapVec, ActionMap) {
    let mut f = File::open(filename).expect("File couldn't be opened");
    let mut buf: Vec<u8> = vec![];
    f.read_to_end(&mut buf).expect("Couldn't read to end");
    let km = (!keymaps() * skip(1)).repeat(0..) * keymaps();
    let act = (!actions() * skip(1)).repeat(0..) * actions();
    let mut input = DataInput::new(&buf);
    
    (km.parse(&mut input).expect("Parsing keymaps failed"),
     act.parse(&mut input).expect("Parsing actions failed"))
}

#[test]
fn parse_ident() {
    let id = ident().parse(&mut DataInput::new(b"ID"));
    assert_eq!(id, Ok(String::from("ID")));
    let id = ident().parse(&mut DataInput::new(b"  ID\n"));
    assert_eq!(id, Ok(String::from("ID")));
}

#[test]
fn parse_action() {
    let act = action().parse(&mut DataInput::new(b"ACTION_FUNCTION(TEENSY)"));
    assert_eq!(act, Ok(Action::Function(Key::Key(String::from("TEENSY")))));
    let act = action().parse(&mut DataInput::new(b"ACTION_FUNCTION_TAP(TAP)"));
    assert_eq!(act, Ok(Action::FunctionTap(Key::Key(String::from("TAP")))));
    let act = action().parse(&mut DataInput::new(b"ACTION_LAYER_SET(4, ON_BOTH)"));
    assert_eq!(act, Ok(Action::LayerSet(4, String::from("ON_BOTH"))));
    let act = action().parse(&mut DataInput::new(b"ACTION_LAYER_MOMENTARY(2)"));
    assert_eq!(act, Ok(Action::LayerMomentary(2)));
    let act = action().parse(&mut DataInput::new(b"ACTION_LAYER_TAP_KEY(2,KEY)"));
    assert_eq!(act, Ok(Action::LayerTapKey(2, Key::Key(String::from("KEY")))));
    let act = action().parse(&mut DataInput::new(b"ACTION_MODS_KEY(KEY1,KEY2)"));
    assert_eq!(act, Ok(Action::ModsKey(Key::Key(String::from("KEY1")), Key::Key(String::from("KEY2")))));
    let act = action().parse(&mut DataInput::new(b"ACTION_MODS_TAP_KEY(KEYA, KEYB)"));
    assert_eq!(act, Ok(Action::ModsTapKey(Key::Key(String::from("KEYA")), Key::Key(String::from("KEYB")))));
}

#[test]
fn parse_actions() {
    let a0 = actions().parse(&mut DataInput::new(b"fn_actions[] = {};"));
    let hm: ActionMap = HashMap::new();
    assert_eq!(a0, Ok(hm));
    let mut hm: ActionMap = HashMap::new();
    let mut idx: u64 = 1;
    hm.insert(idx,Action::LayerMomentary(2));
    let a0 = actions().parse(&mut DataInput::new(b"fn_actions[] = { [1] = ACTION_LAYER_MOMENTARY(2) };"));
    assert!(a0.is_ok());
    assert_eq!(hm[&idx], a0.unwrap()[&idx]);
    let a0 = actions().parse(&mut DataInput::new(b"fn_actions[] = { 
[1] = ACTION_LAYER_MOMENTARY(2), // FN1 comment
 [3] = ACTION_LAYER_SET(88 , ON_BOTH), };")); 
    assert!(a0.is_ok());
    let h = a0.unwrap();
    assert_eq!(hm[&idx], h[&idx]);
    idx = 3;
    hm.insert(idx, Action::LayerSet(88, String::from("ON_BOTH")));
    assert_eq!(hm[&idx], h[&idx]);
}

#[test]
fn parse_keymaps() {
    let kms = keymaps().parse(&mut DataInput::new(b"keymaps[][MATRIX_ROWS][MATRIX_COLS] = {};"));
    assert_eq!(kms, Ok(vec![]));
    let kms = keymaps().parse(&mut DataInput::new(b"keymaps[][MATRIX_ROWS][MATRIX_COLS] = { KEYMAP(F11), KEYMAP(KC_11)};"));
    assert_eq!(kms, Ok(vec![vec![Key::Key(String::from("F11"))], vec![Key::Key(String::from("KC_11"))]]));
    let kms = keymaps().parse(&mut DataInput::new(b"keymaps[][MATRIX_ROWS][MATRIX_COLS] = { /*first*/ KEYMAP(F11), // second \nKEYMAP(KC_11)};"));
    assert_eq!(kms, Ok(vec![vec![Key::Key(String::from("F11"))], vec![Key::Key(String::from("KC_11"))]]));
}

#[test]
fn parse_keymap() {
    let km = keymap().parse(&mut DataInput::new(b"KEYMAP()"));
    assert_eq!(km, Ok(vec![]));
    let km = keymap().parse(&mut DataInput::new(b"KEYMAP(FN10,F10)"));
    assert_eq!(km, Ok(vec![Key::Fx(10), Key::Key(String::from("F10"))]));
}

#[test]
fn parse_fn_key() {
    let fk = fn_key().parse(&mut DataInput::new(b"FN2"));
    assert_eq!(fk, Ok(Key::Fx(2)));
}

#[test]
fn parse_reg_key() {
    let rk = reg_key().parse(&mut DataInput::new(b"SPC"));
    assert_eq!(rk, Ok(Key::Key(String::from("SPC"))));
}

#[test]
fn parse_key() {
    let k = key().parse(&mut DataInput::new(b"F12"));
    assert_eq!(k, Ok(Key::Key(String::from("F12"))));
    let k = key().parse(&mut DataInput::new(b"FN12"));
    assert_eq!(k, Ok(Key::Fx(12)));
}

#[test]
fn parse_whitespace() {
    let w0 = whitespace().parse(&mut DataInput::new(b"   "));
    assert_eq!(w0, Ok(()));
    let w0 = whitespace().parse(&mut DataInput::new(b"// comment\n"));
    assert_eq!(w0, Ok(()));
    let w0 = whitespace().parse(&mut DataInput::new(b"  /* block */  "));
    assert_eq!(w0, Ok(()));
    let w0 = whitespace().parse(&mut DataInput::new(b"\n /* block \n\t */ //comment\n  "));
    assert_eq!(w0, Ok(()));
}

#[test]
fn parse_comment() {
    let c0 = comment().parse(&mut DataInput::new(b"// some comment\n"));
    assert_eq!(c0, Ok(()));
    let c0 = comment().parse(&mut DataInput::new(b"/* block comment*/"));
    assert_eq!(c0, Ok(()));
}

#[test]
fn parse_index() {
    let i0 = index().parse(&mut DataInput::new(b"[123]"));
    assert_eq!(i0, Ok(123));
}

#[test]
fn parse_integer() {
    let n0 = integer().parse(&mut DataInput::new(b"1234567890"));
    assert_eq!(n0,Ok(1234567890));
}

#[test]
#[should_panic]
fn broken_parse_number() {
    let n0 = integer().parse(&mut DataInput::new(b"0123"));
    assert_eq!(n0, Ok(123));
}

#[test]
fn parse_space() {
    let s0 = space().parse(&mut DataInput::new(b" "));
    assert_eq!(s0, Ok(()));
    let s0 = space().parse(&mut DataInput::new(b"\t "));
    assert_eq!(s0, Ok(()));
    let s0 = space().parse(&mut DataInput::new(b"\n "));
    assert_eq!(s0, Ok(()));
    let s0 = space().parse(&mut DataInput::new(b"\r "));
    assert_eq!(s0, Ok(()));
    let s0 = space().parse(&mut DataInput::new(b"\t \n \r \n\r"));
    assert_eq!(s0, Ok(()));
}

#[test]
fn parse_eol() {
    let e0 = eol().parse(&mut DataInput::new(b"\n"));
    assert_eq!(e0, Ok(()));
    let e0 = eol().parse(&mut DataInput::new(b"\r"));
    assert_eq!(e0, Ok(()));
    let e0 = eol().parse(&mut DataInput::new(b"\r\n"));
    assert_eq!(e0, Ok(()));    
}

#[test]
fn parse_line_comment() {
    let l0 = line_comment().parse(&mut DataInput::new(b"//\n"));
    assert_eq!(l0, Ok(()));
    let l1 = line_comment().parse(&mut DataInput::new(b"// Comment\n"));
    assert_eq!(l1, Ok(()));
    let l2 = line_comment().parse(&mut DataInput::new(b"//\n"));
    assert_eq!(l2, Ok(()));
    let l2 = line_comment().parse(&mut DataInput::new(b"// //\n"));
    assert_eq!(l2, Ok(()));
}

#[test]
fn parse_block_comment() {
    let c0 = block_comment().parse(&mut DataInput::new(b"/**/"));
    assert_eq!(c0, Ok(()));
    let c1 = block_comment().parse(&mut DataInput::new(b"/* Comment */"));
    assert_eq!(c1, Ok(()));
    let c2 = block_comment().parse(&mut DataInput::new(b"/*
       Long Comment
     */"));
    assert_eq!(c2, Ok(()));
    let c3 = block_comment().parse(&mut DataInput::new(b"/***/"));
    assert_eq!(c3, Ok(()));
}

#[test]
#[should_panic]
fn broken_block_comment() {
    let c0 = block_comment().parse(&mut DataInput::new(b"*//*"));
    assert_eq!(c0, Ok(()));
}
