use pest::prelude::*;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use regex::Regex;

use types::*;

impl_rdp!{
    grammar! {
        keymaps = { keymap_start ~ keymap_definition* ~ close_brace}
        keymap_start = _{ (!keymap_header ~ any)* ~ keymap_header ~ ["="] ~ open_brace }
        keymap_header = _{ ["keymaps[][MATRIX_ROWS][MATRIX_COLS]"] }
        keymap_definition = { keymap ~ separator? }
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
          | action_layer_set_clear
          | action_layer_toggle
          | action_layer_tap_toggle
          | action_default_layer_set
          | action_layer_tap_key
          | action_mods_key
          | action_mods_tap_key
        }
        action_function = {["ACTION_FUNCTION("] ~ key  ~ [")"] }
        action_function_tap = {["ACTION_FUNCTION_TAP("] ~ key ~ [")"] }
        action_layer_momentary = {["ACTION_LAYER_MOMENTARY("] ~ integer ~ [")"] }
        action_layer_set = {["ACTION_LAYER_SET("] ~ integer ~ separator ~ named_key ~ [")"] }
        action_layer_set_clear = {["ACTION_LAYER_SET_CLEAR("] ~ integer ~ [")"] }
        action_layer_toggle = {["ACTION_LAYER_TOGGLE("] ~ integer ~ [")"] }
        action_layer_tap_toggle = {["ACTION_LAYER_TAP_TOGGLE("] ~ integer ~ [")"] }
        action_default_layer_set = {["ACTION_DEFAULT_LAYER_SET("] ~ integer ~ [")"] }
        action_layer_tap_key = {["ACTION_LAYER_TAP_KEY("] ~ integer ~ separator ~ key ~ [")"] }
        action_mods_key = {["ACTION_MODS_KEY("] ~ key ~ separator ~ key ~ [")"] }
        action_mods_tap_key = {["ACTION_MODS_TAP_KEY("] ~ key ~ separator ~ key ~ [")"] }
        
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
        keymapvec(&self) -> KeyMapVec {
            (_: keymaps, mut entries: _keymaps()) => { entries.reverse(); entries }
        }
        _keymaps(&self) -> KeyMapVec{
            (_: keymap_definition, head: _keymap(), mut tail: _keymaps()) => {
                tail.push(head);
                tail
            },
            () => { KeyMapVec::new()}
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
                => Action::ModsTapKey(modifier, key),
            (_: action_default_layer_set, layer: _integer()) => Action::DefaultLayerSet(layer),
            (_: action_layer_set_clear, layer: _integer()) => Action::LayerSetClear(layer),
            (_: action_layer_toggle, layer: _integer()) => Action::LayerToggle(layer),
            (_: action_layer_tap_toggle, layer: _integer()) => Action::LayerTapToggle(layer)
        }
        _key(&self) -> Key {
            (&ident: named_key) => Key::Key(String::from(ident)),
            (_: fn_key, &id: action_id) => Key::Fx(id.parse::<u64>().unwrap())
        }
    }
}

fn parse_actions(input: StringInput) -> ActionMap {
    let mut parser = Rdp::new(input);
    let ret = parser.fn_actions();
    if !ret {
        println!("Q: {:?}", parser.queue());
        println!("X: {:?}", parser.expected());
    }
    parser.actions()
}

fn parse_keymaps(input: StringInput) -> KeyMapVec {
    let mut parser = Rdp::new(input);
    let ret = parser.keymaps();
    if !ret {
        println!("Q: {:?}", parser.queue());
        println!("X: {:?}", parser.expected());
    }
    parser.keymapvec()
}

pub fn parse_file(filename: &Path) -> (KeyMapVec, ActionMap) {
    let mut f = File::open(filename).expect("File couldn't be opened");
    let mut input = String::new();
    f.read_to_string(&mut input).expect("Unable to read file");

    // Strip out line comments
    let line_comment_re = Regex::new(r"//(.*)\n").unwrap();

    let processed = line_comment_re.replace_all(&input, "");

    (parse_keymaps(StringInput::new(&processed)),
     parse_actions(StringInput::new(&processed)))
}

#[test]
fn test_cub() {
    let mut parser = Rdp::new(StringInput::new("#include <util/delay.h>
#include \"action_layer.h\"
#include \"action_util.h\"
#include \"bootloader.h\"
#include \"keymap_common.h\"


const uint8_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    /*
     * Keymap: Default Layer in QWERTY
     *
     * ,--------------------------------------------------.           ,--------------------------------------------------.
     * |   ~    |   1  |   2  |   3  |   4  |   5  |   \\  |           |   -  |   6  |   7  |   8  |   9  |   0  |   =    |
     * |--------+------+------+------+------+-------------|           |------+------+------+------+------+------+--------|
     * | Tab    |   Q  |   W  |   E  |   R  |   T  | ~L5  |           | ~L6  |   Y  |   U  |   I  |   O  |   P  |   [    |
     * |--------+------+------+------+------+------|      |           |      |------+------+------+------+------+--------|
     * | Tab/Shf|   A  |   S  |   D  |   F  |   G  |------|           |------|   H  |   J  |   K  |   L  |   ;  |   '    |
     * |--------+------+------+------+------+------|  L0  |           | ~L7  |------+------+------+------+------+--------|
     * | LCtrl  |   Z  |   X  |   C  |   V  |   B  |      |           |      |   N  |   M  |   ,  |   .  |   /  |   ]    |
     * `--------+------+------+------+------+-------------'           `-------------+------+------+------+------+--------'
     *   | ~L5  | ~L2  | Caps | LAlt | LGui |                                       |  Lft |  Up  |  Dn  | Rght | ~L6  |
     *   `----------------------------------'                                       `----------------------------------'
     *                                        ,-------------.       ,-------------.
     *                                        | +L2  | Home |       | PgUp | Del  |
     *                                 ,------|------|------|       |------+------+------.
     *                                 |      |      |  End |       | PgDn |      |      |
     *                                 | BkSp |  ESC |------|       |------| Enter| Space|
     *                                 |      |      |  Spc |       | Ins  |      |      |
     *                                 `--------------------'       `--------------------'
     *
     *
     *
     ****************************************************************************************************
     *
     * Under XOrg, I use my own mapping from QWERTY to \"Workman for Programmers\"
     * See XOrg files in ./addons/ subdirectory.
     *
     * I have to do so, because of two things:
     * 1) my native language is Russian, and XOrg keymap for it is based on QWERTY layout
     * 2) I want to have non-standart shifted keys, like $ (as normal) and @ (as shifted), or _ and -
     *
     * And even if (2) could be solved using FN* keys (but there is limit in firmware for only 32 such
     * keys), then (1) can't be solved at firmware level at all.
     *
     * So, I have to stick with QWERTY as my main layout + my own XOrg keyboard layout for English.
     * But sometimes I have to input something when XOrg is not active - for example, in Linux console,
     * or in firmware console (while debugging firmware), or when keyboard is connected to not my computer.
     *
     * For such cases I have Layer1 :)
     * // hint: switch to Layer1 is only at Layer6
     *
     ****************************************************************************************************
     *
     *
     *
     * Keymap: Default Layer in Workman
     *
     * ,--------------------------------------------------.           ,--------------------------------------------------.
     * |  ~     |   ;  |   !  |   #  |   {  |   }  |   '  |           |   ^  |   [  |   ]  |   *  |   (  |   )  |   =    |
     * |--------+------+------+------+------+-------------|           |------+------+------+------+------+------+--------|
     * | Tab    |   Q  |   D  |   R  |   W  |   B  |  NO  |           | ~L7  |   J  |   F  |   U  |   P  |   $  |   :    |
     * |--------+------+------+------+------+------|      |           |      |------+------+------+------+------+--------|
     * | Tab/Shf|   A  |   S  |   H  |   T  |   G  |------|           |------|   Y  |   N  |   E  |   O  |   I  |   -    |
     * |--------+------+------+------+------+------| Home |           | End  |------+------+------+------+------+--------|
     * | LCtrl  |   Z  |   X  |   M  |   C  |   V  |      |           |      |   K  |   L  |   ,  |   .  |   /  |   |    |
     * `--------+------+------+------+------+-------------'           `-------------+------+------+------+------+--------'
     *   | ~L5  | ~L2  | Caps | LAlt | LGui |                                       |  Lft |  Up  |  Dn  | Rght | ~L6  |
     *   `----------------------------------'                                       `----------------------------------'
     *                                        ,-------------.       ,-------------.
     *                                        |  L0  |  +L2 |       | PgUp | Del  |
     *                                 ,------|------|------|       |------+------+------.
     *                                 |      |      |  NO  |       | PgDn |      |      |
     *                                 | BkSp |  ESC |------|       |------| Enter| Space|
     *                                 |      |      |  Spc |       | Ins  |      |      |
     *                                 `--------------------'       `--------------------'
     *
     * Keymap: Default Layer in Workman / with Shift
     *
     * ,--------------------------------------------------.           ,--------------------------------------------------.
     * |  `     |   1  |   2  |   3  |   4  |   5  |   \"  |           |   \\  |   6  |   7  |   8  |   9  |   0  |   +    |
     * |--------+------+------+------+------+-------------|           |------+------+------+------+------+------+--------|
     * | Tab    |   Q  |   D  |   R  |   W  |   B  |  NO  |           | ~L7  |   J  |   F  |   U  |   P  |   @  |   %    |
     * |--------+------+------+------+------+------|      |           |      |------+------+------+------+------+--------|
     * | Tab/Shf|   A  |   S  |   H  |   T  |   G  |------|           |------|   Y  |   N  |   E  |   O  |   I  |   _    |
     * |--------+------+------+------+------+------| Home |           | End  |------+------+------+------+------+--------|
     * | LCtrl  |   Z  |   X  |   M  |   C  |   V  |      |           |      |   K  |   L  |   ,  |   .  |   /  |   &    |
     * `--------+------+------+------+------+-------------'           `-------------+------+------+------+------+--------'
     *   | ~L5  | ~L2  | Caps | LAlt | LGui |                                       |  Lft |  Up  |  Dn  | Rght | ~L6  |
     *   `----------------------------------'                                       `----------------------------------'
     *                                        ,-------------.       ,-------------.
     *                                        |  L0  |  +L2 |       | PgUp | Del  |
     *                                 ,------|------|------|       |------+------+------.
     *                                 |      |      |  NO  |       | PgDn |      |      |
     *                                 | BkSp |  ESC |------|       |------| Enter| Space|
     *                                 |      |      |  Spc |       | Ins  |      |      |
     *                                 `--------------------'       `--------------------'
     *
     */

    KEYMAP(  /* Layer0: default, leftled:none*/
        /* left hand*/
        GRV, 1,   2,   3,   4,   5,   BSLS,
        FN2, Q,   W,   E,   R,   T,   FN23,
        FN11,FN28,FN29,FN30,FN31,G,
        FN12,FN24,FN25,FN26,FN27,B,   HOME,
        FN21,FN20,CAPS,FN13,FN14,
                                      FN17,FN19,
                                           NO,
                                 FN5, FN6, FN7,
        /* right hand*/
             MINS,6,   7,   8,   9,   0,   EQL,
             FN23,Y,   U,   I,   O,   P,   LBRC,
                  H,   J,   K,   L,   SCLN,FN15,
             END, N,   M,   COMM,DOT, SLSH,FN16,
                       LEFT,UP,  DOWN,RGHT,FN22,
        PGUP,DEL,
        PGDN,
        FN8, FN9, FN10
    ),

    KEYMAP(  /* Layer1: Workman layout, leftled:all*/
        /* left hand*/
        TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,Q,   D,   R,   W,   B,   TRNS,
        TRNS,A,   S,   H,   T,   G,
        TRNS,Z,   X,   M,   C,   V,   TRNS,
        TRNS,TRNS,FN17,TRNS,TRNS,
                                      TRNS,TRNS,
                                           TRNS,
                                 TRNS,TRNS,TRNS,
        /* right hand */
             TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
             TRNS,J,   F,   U,   P,   4,   TRNS,
                  Y,   N,   E,   O,   I,   TRNS,
             TRNS,K,   L,   TRNS,TRNS,TRNS,TRNS,
                       TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        TRNS,TRNS,TRNS
    ),
    KEYMAP(  /* Layer2: numpad, leftled:mid/blue*/
        /* left hand*/
        TRNS,NO,  NO,  NO,  NO,  PAUS,PSCR,
        TRNS,NO,  NO,  NO,  NO,  NO,  TRNS,
        TRNS,NO,  NO,  NO,  TRNS,NO,
        TRNS,NO,  NO,  NO,  TRNS,NO,  TRNS,
        TRNS,TRNS,FN17,TRNS,TRNS,
                                      TRNS,TRNS,
                                           TRNS,
                                 TRNS,TRNS,TRNS,
        /* right hand */
             SLCK,NLCK,PSLS,PAST,PAST,PMNS,BSPC,
             TRNS,NO,  P7,  P8,  P9,  PMNS,PGUP,
                  NO,  P4,  P5,  P6,  PPLS,PGDN,
             TRNS,NO,  P1,  P2,  P3,  PPLS,PENT,
                       P0,  PDOT,SLSH,PENT,PENT,
        TRNS,TRNS,
        TRNS,
        TRNS,TRNS,TRNS
    ),

    KEYMAP(  
        /* left hand */
        TRNS,NO,  NO,  NO,  NO,  NO,  NO,
        TRNS,NO,  NO,  NO,  NO,  NO,  TRNS,
        TRNS,NO,  TRNS,NO,  NO,  NO,
        TRNS,NO,  TRNS,NO,  NO,  NO,  TRNS,
        TRNS,TRNS,TRNS,LALT,LGUI,
                                      TRNS,TRNS,
                                           TRNS,
                                 LCTL,LSFT,TRNS,
        /* right hand */
             NO,  NO,  NO,  NO,  NO,  NO,  TRNS,
             TRNS,NO,  F1,  F2,  F3,  F4,  PGUP,
                  NO,  F5,  F6,  F7,  F8,  PGDN,
             TRNS,NO,  F9,  F10, F11, F12, APP,
                       RGUI,RALT,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        TRNS,RSFT,RCTL
    ),
    KEYMAP(  /* Layer4: unconvenient keys on right hand, leftled:top/white*/
        /* left hand*/
        TRNS,NO,  NO,  NO,  NO,  NO,  NO,
        TRNS,NO,  NO,  NO,  NO,  NO,  TRNS,
        TRNS,TRNS,NO,  NO,  NO,  NO,
        TRNS,TRNS,NO,  NO,  NO,  NO,  TRNS,
        TRNS,TRNS,TRNS,LALT,LGUI,
                                      TRNS,TRNS,
                                           TRNS,
                                 TRNS,TRNS,TRNS,

        /* in Workman right hand will be:
                { } ( ) +
              ^ ! ?     =
              ' ! $ \" ; \
              # [ < > ] \
        */

        /* right hand */
             NO,  NO,  4,   5,   9,   0,   PPLS,
             TRNS,MINS,2,   FN5, 9,   0,   EQL,
                  BSLS,2,   P,   FN1, 1,   FN2,
             TRNS,3,   6,   FN3, FN4, 7,   FN2,
                       TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        TRNS,TRNS,TRNS
    ),


    KEYMAP(  
        TRNS,F1,  F2,  F3,  F4,  F5,  F6,
        TRNS,P1,  P2,  P3,  P4,  P5,  TRNS,
        TRNS,TRNS,TRNS,E,   TRNS,TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,
                                      TRNS,TRNS,
                                           TRNS,
                                 TRNS,TRNS,TRNS,
             F7,  F8,  F9,  F10, F11, F12, TRNS,
             TRNS,P6,  P7,  P8,  P9,  P0,  TRNS,
                  TRNS,U,   TRNS,TRNS,TRNS,TRNS,
             TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
                       TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        TRNS,TRNS,TRNS
    ),

    KEYMAP(  
        FN0, F1,  F2,  F3,  F4,  F5,  F6,
        TRNS,P1,  P2,  P3,  P4,  P5,  TRNS,
        TRNS,TRNS,TRNS,E,   TRNS,TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
        FN18,TRNS,TRNS,TRNS,TRNS,
                                      TRNS,TRNS,
                                           TRNS,
                                 TRNS,TRNS,TRNS,
             F7,  F8,  F9,  F10, F11, F12, FN0,
             TRNS,P6,  P7,  P8,  P9,  P0,  TRNS,
                  TRNS,U,   TRNS,TRNS,TRNS,TRNS,
             TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
                       TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        TRNS,TRNS,TRNS
    ),

    KEYMAP( 
        FN0, NO,  NO,  NO,  NO,  NO,  NO,
        FN1, F13, F14, F15, F16, NO,  TRNS,
        TRNS,F17, F18, F19, F20, NO,
        TRNS,F21, F22, F23, F24, NO,  TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,
                                      TRNS,TRNS,
                                           TRNS,
                                 TRNS,TRNS,TRNS,
        /* right hand */
             NO,  NO,  NO,  NO,  NO,  NO,  TRNS,
             TRNS,NO,  F1,  F2,  F3,  F4,  TRNS,
                  NO,  F5,  F6,  F7,  F8,  TRNS,
             TRNS,NO,  F9,  F10, F11, F12, TRNS,
                       TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        SLEP,TRNS,TRNS
    ),

    KEYMAP(  /* Layer8: mouse and navigation, leftled:mid/blue+bot/green */
        /* left hand */
        TRNS,NO,  NO,  NO,  NO,  NO,  NO,
        TRNS,NO,  NO,  NO,  ACL0,NO,  TRNS,
        TRNS,NO,  NO,  TRNS,ACL1,NO,
        TRNS,NO,  NO,  TRNS,ACL2,NO,  TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,
                                      TRNS,TRNS,
                                           TRNS,
                                 TRNS,TRNS,TRNS,

        /* right hand */
             F16, MPLY,MPRV,MNXT,VOLD,VOLU,MUTE,
             F14, BTN2,WH_L,WH_U,WH_D,WH_R,PGUP,
                  BTN1,MS_L,MS_U,MS_D,MS_R,PGDN,
             F15, BTN3,HOME,END, DEL, INS, NO,
                       TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        TRNS,TRNS,TRNS
    ),

    KEYMAP(  /* Layer9: application-specific shortcuts (mostly browser), leftled:top/white+bot/green */
        /* left hand */
        TRNS,NO,  NO,  NO,  NO,  NO,  NO,
        TRNS,NO,  NO,  NO,  NO,  NO,  TRNS,
        TRNS,NO,  NO,  NO,  NO,  NO,
        TRNS,NO,  NO,  NO,  TRNS,NO,  TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,
                                      TRNS,TRNS,
                                           TRNS,
                                 TRNS,TRNS,TRNS,
        /* right hand */
             NO,  NO,  NO,  NO,  NO,  NO,  TRNS,
             TRNS,NO,  FN12,FN13,FN14,FN15,FN10,
                  FN1, FN2, FN3, FN4, FN5, FN11,
             TRNS,TRNS,FN6, FN7, FN8, FN9, FN0,
                       TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        TRNS,TRNS,TRNS
    ),

/*
       KEYMAP(
        TRNS,NO,  NO,  NO,  NO,  NO,  NO,
        TRNS,NO,  NO,  NO,  NO,  NO,  TRNS,
        TRNS,NO,  NO,  NO,  NO,  NO,
        TRNS,NO,  NO,  NO,  NO,  NO,  TRNS,
        TRNS,TRNS,TRNS,LALT,LGUI,
                                      TRNS,TRNS,
                                           TRNS,
                                 LCTL,LSFT,TRNS,
                   NO,  NO,  NO,  NO,  NO,  NO,  TRNS,
             TRNS,NO,  NO,  NO,  NO,  NO,  TRNS,
                  NO,  NO,  NO,  NO,  NO,  TRNS,
             TRNS,NO,  NO,  NO,  NO,  NO,  TRNS,
                       RGUI,RALT,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        TRNS,RSFT,RCTL
    ),
    KEYMAP( 
        TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,TRNS,TRNS,TRNS,
                                      TRNS,TRNS,
                                           TRNS,
                                 TRNS,TRNS,TRNS,
             TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
             TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
                  TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
             TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,TRNS,
                       TRNS,TRNS,TRNS,TRNS,TRNS,
        TRNS,TRNS,
        TRNS,
        TRNS,TRNS,TRNS
    ),
*/

}"));
    let success = parser.keymaps();
    println!("Q: {:?}", parser.queue());
    println!("X: {:?}", parser.expected());
    assert!(success);
        
}

#[test]
fn test_keymapvec() {
    let mut parser = Rdp::new(StringInput::new("extra beginging junk
keymaps[][MATRIX_ROWS][MATRIX_COLS] 
= 
{

  KEYMAP(A,B,C,D),
  KEYMAP(X,Y,Z),
  KEYMAP(F11, F12, FN8, FN12)
}
"));
    assert!(parser.keymaps());
    let keymaps = parser.keymapvec();

    let km1 = vec![Key::Key(String::from("A")),
                   Key::Key(String::from("B")),
                   Key::Key(String::from("C")),
                   Key::Key(String::from("D"))];

    let km2 = vec![Key::Key(String::from("X")),
                   Key::Key(String::from("Y")),
                   Key::Key(String::from("Z"))];

    let km3 = vec![Key::Key(String::from("F11")),
                   Key::Key(String::from("F12")),
                   Key::Fx(8),
                   Key::Fx(12)];

    let expectation = vec![km1,km2,km3];

    assert_eq!(keymaps, expectation);
}

#[test]
fn test_processed_keymap() {
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
fn test_processed_action() {
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
fn test_processed_action_type() {
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
fn test_processed_key() {
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
pub fn test_keymaps() {
    let mut parser = Rdp::new(StringInput::new("keymaps[][MATRIX_ROWS][MATRIX_COLS] = { KEYMAP(/* layer 0*/ TRNS, LGUI), 
KEYMAP(/*layer 1*/ BTN3, FN14), 
}"));
    assert!(parser.keymaps());
    assert!(parser.end());

    let queue = vec![
        Token::new(Rule::keymaps, 0, 108),
        Token::new(Rule::keymap_definition, 40, 72),
        Token::new(Rule::keymap, 40, 71),
        Token::new(Rule::key_entry, 60, 65),
        Token::new(Rule::named_key, 60, 64),
        Token::new(Rule::key_entry, 66, 70),
        Token::new(Rule::named_key, 66, 70),
        Token::new(Rule::keymap_definition, 74, 105),
        Token::new(Rule::keymap, 74, 104),
        Token::new(Rule::key_entry, 93, 98),
        Token::new(Rule::named_key, 93, 97),
        Token::new(Rule::key_entry, 99, 103),
        Token::new(Rule::fn_key, 99, 103),
        Token::new(Rule::action_id, 101,103)
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
fn test_whitespace() {
    let mut parser = Rdp::new(StringInput::new(" "));
    assert!(parser.whitespace());
    assert!(parser.end());
}
