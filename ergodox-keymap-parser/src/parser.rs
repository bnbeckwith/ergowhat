use regex::Regex;
use std::str::FromStr;

use types::*;

use pest::*;

#[derive(Parser)]
#[grammar = "keymap.pest"]
struct KeymapParser;

fn parse_actions(input: &str) -> ActionMap {
    let actions = KeymapParser::parse(Rule::fn_actions, input).unwrap_or_else(|e| panic!("{}", e));

    let mut amap = ActionMap::new();

    for action in actions {
        let mut pairs = action.clone().into_inner();
        let idx = u32::from_str(pairs.next().unwrap().into_span().as_str()).unwrap();
        let a = pairs.next().unwrap();
        let rule = a.as_rule();
        let inner: Vec<_> = a.into_inner().collect();

        let val = match rule {
            Rule::action_function => {
                let rule = inner[0].as_rule();
                let mut xs = inner[0].clone().into_inner();
                let k = match inner[0].as_rule() {
                    Rule::fn_key => Key::Fx(u32::from_str( xs.next().unwrap().as_str() ).unwrap()),
                    Rule::named_key => Key::Key(String::from( inner[0].as_str())),
                    _ => panic!("What? {:?}", rule)
                };
                Action::Function( k )
            }
            Rule::action_function_tap => {
                let rule = inner[0].as_rule();
                let mut xs = inner[0].clone().into_inner();
                let k = match inner[0].as_rule() {
                    Rule::fn_key => Key::Fx(u32::from_str( xs.next().unwrap().as_str() ).unwrap()),
                    Rule::named_key => Key::Key(String::from( inner[0].as_str())),
                    _ => panic!("What? {:?}", rule)
                };
                Action::FunctionTap( k )
            }
            Rule::action_layer_momentary => {
                let l = u32::from_str(inner[0].as_str()).unwrap();
                Action::LayerMomentary( l )
            }
            Rule::action_layer_set => {
                let l = u32::from_str(inner[0].as_str()).unwrap();
                let s = String::from(inner[1].as_str());
                Action::LayerSet( l, s )   
            }
            Rule::action_layer_set_clear => {
                let l = u32::from_str(inner[0].as_str()).unwrap();
                Action::LayerSetClear( l )
            }
            Rule::action_layer_toggle => {
                let l = u32::from_str(inner[0].as_str()).unwrap();
                Action::LayerToggle( l )
            } 
            Rule::action_layer_tap_toggle => {
                let l = u32::from_str(inner[0].as_str()).unwrap();
                Action::LayerTapToggle( l )
            } 
            Rule::action_default_layer_set => {
                let l = u32::from_str(inner[0].as_str()).unwrap();
                Action::DefaultLayerSet( l )
            }
            Rule::action_layer_tap_key => {
                let l = u32::from_str(inner[0].as_str()).unwrap();
                let rule = inner[1].as_rule();
                let mut xs = inner[1].clone().into_inner();
                let k = match inner[1].as_rule() {
                    Rule::fn_key => Key::Fx(u32::from_str( xs.next().unwrap().as_str() ).unwrap()),
                    Rule::named_key => Key::Key(String::from( inner[1].as_str())),
                    _ => panic!("What? {:?}", rule)
                };
                Action::LayerTapKey( l, k)
            }
            Rule::action_mods_key => {
                let rule0 = inner[0].as_rule();
                let mut xs0 = inner[0].clone().into_inner();
                let k0 = match rule0 {
                    Rule::fn_key => Key::Fx(u32::from_str( xs0.next().unwrap().as_str() ).unwrap()),
                    Rule::named_key => Key::Key(String::from( inner[0].as_str())),
                    _ => panic!("What? {:?}", rule0)
                };
                let rule1 = inner[1].as_rule();
                let mut xs1 = inner[1].clone().into_inner();
                let k1 = match inner[1].as_rule() {
                    Rule::fn_key => Key::Fx(u32::from_str( xs1.next().unwrap().as_str() ).unwrap()),
                    Rule::named_key => Key::Key(String::from( inner[0].as_str())),
                    _ => panic!("What? {:?}", rule1)
                };
                
                Action::ModsKey( k0, k1 )
            }
            Rule::action_mods_tap_key => {
                let rule0 = inner[0].as_rule();
                let mut xs0 = inner[0].clone().into_inner();
                let k0 = match inner[0].as_rule() {
                    Rule::fn_key => Key::Fx(u32::from_str( xs0.next().unwrap().as_str() ).unwrap()),
                    Rule::named_key => Key::Key(String::from( inner[0].as_str())),
                    _ => panic!("What? {:?}", rule0)
                };
                let rule1 = inner[1].as_rule();
                let mut xs1 = inner[1].clone().into_inner();
                let k1 = match inner[1].as_rule() {
                    Rule::fn_key => Key::Fx(u32::from_str( xs1.next().unwrap().as_str() ).unwrap()),
                    Rule::named_key => Key::Key(String::from( inner[1].as_str())),
                    _ => panic!("What? {:?}", rule1)
                };

                Action::ModsTapKey( k0, k1)
            }
            _ => panic!("{:?}", action)
        };

        amap.insert(idx, val);
    }

    amap
}

fn parse_keymaps(input: &str) -> KeyMapVec {
    let keymaps = KeymapParser::parse(Rule::keymaps, input).unwrap_or_else(|e| panic!("{}", e))
        .flatten()
        .next()
        .unwrap();

    let mut kmv = KeyMapVec::new();
    
    for map in keymaps.into_inner() {
        let mut km = KeyMap::new();
        for entry in map.into_inner() {
            let key = match entry.as_rule() {
                Rule::fn_key => Key::Fx( u32::from_str( entry.into_inner().next().unwrap().as_str()).unwrap() ),
                Rule::named_key => Key::Key( String::from( entry.as_str() )),
                _ => panic!("What did you do? {:?}", entry)
            };
            km.push(key)
        }
        kmv.push(km)
    }
    kmv
}

pub fn parse_string(input: &str) -> (KeyMapVec, ActionMap) {

    // Strip out line comments
    let line_comment_re = Regex::new(r"//(.*)\n").unwrap();

    let processed = line_comment_re.replace_all(&input, "");

    (parse_keymaps(&processed),
     parse_actions(&processed))
}

#[test]
fn test_keymap() {
    parses_to! {
        parser: KeymapParser,
        input: "KEYMAP( /* layer 8*/ TRNS, NO, 7, FN14)",
        rule: Rule::keymap,
        tokens: [
            keymap(0,39, [
                key_entry(21,26, [named_key(21,25)]),
                key_entry(27,30, [named_key(27,29)]),
                key_entry(31,33, [named_key(31,32)]),
                key_entry(34,38, [fn_key(34,38, [action_id(36,38)])]) 
            ])
        ]
    }
}

#[test]
fn test_integer() {
    parses_to! {
        parser: KeymapParser,
        input: "12345",
        rule: Rule::integer,
        tokens: [
            integer(0, 5)
        ]
    }
}

#[test]
fn test_action_id() {
    parses_to! {
        parser: KeymapParser,
        input: "01",
        rule: Rule::action_id,
        tokens: [
            action_id(0,2)
        ]
    }
}

#[test]
fn test_named_key() {
    parses_to! {
        parser: KeymapParser,
        input: "SPC",
        rule: Rule::named_key,
        tokens: [
            named_key(0,3)
        ]
    }
}

#[test]
fn test_fn_key() {
    parses_to! {
        parser: KeymapParser,
        input: "FN10",
        rule: Rule::fn_key,
        tokens: [
            fn_key(0,4, [
                action_id(2,4)
            ])
        ]
    }
}
