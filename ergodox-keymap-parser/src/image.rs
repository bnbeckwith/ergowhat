use svg::Document;
use svg::node::element::*;
use svg::node::{Text as TextContent};

use types::*;
use std::ops::Range;

pub struct Keyboard {
    keymaps: KeyMapVec,
    actions: ActionMap
}

fn drawkey(width: f64, height: f64) -> Group {
    let background = Rectangle::new()
        .set("x", 1)
        .set("y", 1)
        .set("width", width-2.0)
        .set("height", height-2.0)
        .set("rx", 15)
        .set("ry", 15)
        .set("stroke","white")
        .set("fill","white");

    let outside = Rectangle::new()
        .set("x", 1)
        .set("y", 1)
        .set("width", width-2.0)
        .set("height", height-2.0)
        .set("rx", 15)
        .set("ry", 15)
        .set("stroke", "#A5A5A5")
        .set("fill", "url(#keyoutside)");

    let inside = Rectangle::new()
        .set("x", 10)
        .set("y", 7)
        .set("width", width-20.0)
        .set("height", height-20.0)
        .set("rx", 10)
        .set("ry", 10)
        .set("stroke", "#F9F9F9")
        .set("fill", "url(#keyinside)");

    Group::new()
        .add(background)
        .add(outside)
        .add(inside)
 }


pub enum KeyShape {
    K10u,
    K15h,
    K20v,
    K15v
}
macro_rules! makeKeyGroup {
    ($key:path, $code:expr, $x:expr, $y:expr) => {
        {
            let (w,h) = match $key {
                KeyShape::K10u => (100.0, 100.0),
                KeyShape::K15h => (150.0, 100.0),
                KeyShape::K15v => (100.0, 150.0),
                KeyShape::K20v => (100.0, 200.0)
            };

            drawkey(w,h)
                .set("transform", format!("translate({},{})",$x, $y))
        }
    }
}

fn textoutput(input: &str) -> (String,String) {
    let input = input.replace("KC_","");
    let input = input.replace("MOD_","");
    let (normal, shifted) =
        match input.as_str() {
            "NO"   => ("",""),
            "EQL"  => ("=","+"),
            "RGHT" => ("→",""),
            "LEFT" => ("←",""),
            "UP"   => ("↑",""),
            "DOWN" => ("↓",""),
            "COMM" => (",","<"),
            "DOT"  => (".", ">"),
            "QUOT" => ("'", "\""),
            "MINS" => ("-", "_"),
            "BSLS" => ("\\","|"),
            "SLSH" => ("/","?"),
            "GRV"  => ("`","~"),
            "SCLN" => (";",":"),
            "ENT" | "PENT" => ("⏎",""),
            "LBRC" => ("[","{"),
            "RBRC" => ("]","}"),
            "SPC" => ("␣",""),
            "0" => ("0",")"),
            "1" => ("1","!"),
            "2" => ("2","@"),
            "3" => ("3","#"),
            "4" => ("4","$"),
            "5" => ("5","%"),
            "6" => ("6","^"),
            "7" => ("7","&"),
            "8" => ("8","*"),
            "9" => ("9","("),
            "F1" => ("F1",""),
            "F2" => ("F2",""),
            "F3" => ("F3",""),
            "F4" => ("F4",""),
            "F5" => ("F5",""),
            "F6" => ("F6",""),
            "F7" => ("F7",""),
            "F8" => ("F8",""),
            "F9" => ("F9",""),
            "F10" => ("F10",""),
            "F11" => ("F11",""),
            "F12" => ("F12",""),
            "F13" => ("F13",""),
            "F14" => ("F14",""),
            "F15" => ("F15",""),
            "F16" => ("F16",""),
            "F17" => ("F17",""),
            "F18" => ("F18",""),
            "F19" => ("F19",""),
            "F20" => ("F20",""),
            "F21" => ("F21",""),
            "F22" => ("F22",""),
            "F23" => ("F23",""),
            "F24" => ("F24",""),
            x => (x  , "")
        };

    (String::from(normal), String::from(shifted))
}

fn cdata(input: String) -> String {
    format!("<![CDATA[{}]]>",input)
}

macro_rules! addKeyText{
    ($group:expr, $name:expr, $offset:expr) => {{
        let (normal,shifted) = textoutput($name.as_ref());
        $group = $group.add(Text::new()
                            .set("x",50.0)
                            .set("y",$offset)
                            .set("class","shifted")
                            .add(TextContent::new(cdata(shifted))));
        $group = $group.add(Text::new()
                            .set("x", 50.0)
                            .set("y", $offset+25.0)
                            .set("id", $name)
                            .set("class","normal")
                            .add(TextContent::new(cdata(normal)))
        ).set("id", $name)
    }};
    ($group:expr, $name:expr) => {
        addKeyText!($group, $name, 25.0)
    }
}

macro_rules! addLayer {
    ($group:expr, $layer:expr, $function:expr) => {
        $group = $group.set("onclick", format!("{}({})",$function,$layer))
    };
    ($group:expr, $layer:expr) => {
        addLayer!($group, $layer, "templayeron")
    }
}

macro_rules! addMomentaryLayer{
    ($group:expr, $layer:expr) => {
        $group = $group.set("onmousedown",
                            format!("templayeron({})",$layer))
            .set("onmouseup",
                 format!("templayeroff({})",$layer))
    }
}

impl Keyboard {

    pub fn new(keymaps: KeyMapVec, actions: ActionMap) -> Keyboard {
        Keyboard{ keymaps: keymaps, actions: actions}
    }

    fn keynode(&self, x: f64, y: f64, layer: usize, keyn: usize, shape: KeyShape) -> Group {
        let ref keycode = self.keymaps[layer][keyn];
        let mut keygroup = makeKeyGroup!(shape, keycode, x, y);

        match keycode {
            &Key::Fx(action) =>
                match self.actions.get(&action) {
                    Some(act) =>
                        match act {
                            &Action::LayerSet(layer,_) => {
                                addLayer!(keygroup, layer, "onlylayer");
                                addKeyText!(keygroup, format!("#{}",layer))
                            }
                            &Action::LayerMomentary(layer) => {
                                addMomentaryLayer!(keygroup,layer);
                                addKeyText!(keygroup,format!("~{}",layer))
                            }
                            &Action::LayerTapKey(layer,ref k) => {
                                addMomentaryLayer!(keygroup,layer);
                                let s = match k {
                                    &Key::Key(ref name) => name.as_str(),
                                    _ => "WHAT?"
                                };
                                addKeyText!(keygroup, s);
                                addKeyText!(keygroup, format!("~L{}",layer), 50.0)
                            }
                            &Action::ModsTapKey(ref m, ref k) => {
                                let modifier = match m {
                                    &Key::Key(ref name) => name.as_str(),
                                    _ => "HUH?"
                                };
                                let k = match k {
                                    &Key::Key(ref name) => name.as_str(),
                                    _ => "HRM.."
                                };
                                addKeyText!(keygroup,modifier,0.0);
                                addKeyText!(keygroup,k,50.0);
                            }
                            _ => ()
                        },
                    None => {
                        addKeyText!(keygroup, "BROKEN",0.0);
                    }
                }
            ,
            &Key::Key(ref name) =>
            {
                addKeyText!(keygroup, name.as_str());
            }
        }
        keygroup
    }

    fn leftthumb(self: &Keyboard, layer: usize) -> Group {
    Group::new()
        .add(self.keynode(100.0,  0.0,layer,32,KeyShape::K10u))
        .add(self.keynode(200.0,  0.0,layer,33,KeyShape::K10u))
        .add(self.keynode(200.0,100.0,layer,34,KeyShape::K10u))
        .add(self.keynode(  0.0,100.0,layer,35,KeyShape::K20v))
        .add(self.keynode(100.0,100.0,layer,36,KeyShape::K20v))
        .add(self.keynode(200.0,200.0,layer,37,KeyShape::K10u))
    }

    fn leftmain(self: &Keyboard, layer: usize) -> Group {
        Group::new()
        // First Row
            // .set("onclick",format!("layer_click(evt,{})",layer))
            .add(self.keynode(  0.0,  0.0,layer,0,KeyShape::K15h))
            .add(self.keynode(150.0,  0.0,layer,1,KeyShape::K10u))
            .add(self.keynode(250.0,  0.0,layer,2,KeyShape::K10u))
            .add(self.keynode(350.0,  0.0,layer,3,KeyShape::K10u))
            .add(self.keynode(450.0,  0.0,layer,4,KeyShape::K10u))
            .add(self.keynode(550.0,  0.0,layer,5,KeyShape::K10u))
            .add(self.keynode(650.0,  0.0,layer,6,KeyShape::K10u))
        // Second Row
            .add(self.keynode(  0.0,100.0,layer,7,KeyShape::K15h))
            .add(self.keynode(150.0,100.0,layer,8,KeyShape::K10u))
            .add(self.keynode(250.0,100.0,layer,9,KeyShape::K10u))
            .add(self.keynode(350.0,100.0,layer,10,KeyShape::K10u))
            .add(self.keynode(450.0,100.0,layer,11,KeyShape::K10u))
            .add(self.keynode(550.0,100.0,layer,12,KeyShape::K10u))
            .add(self.keynode(650.0,100.0,layer,13,KeyShape::K15v))
        // Third Row
            .add(self.keynode(  0.0,200.0,layer,14,KeyShape::K15h))
            .add(self.keynode(150.0,200.0,layer,15,KeyShape::K10u))
            .add(self.keynode(250.0,200.0,layer,16,KeyShape::K10u))
            .add(self.keynode(350.0,200.0,layer,17,KeyShape::K10u))
            .add(self.keynode(450.0,200.0,layer,18,KeyShape::K10u))
            .add(self.keynode(550.0,200.0,layer,19,KeyShape::K10u))
        // Fourth Row
            .add(self.keynode(  0.0,300.0,layer,20,KeyShape::K15h))
            .add(self.keynode(150.0,300.0,layer,21,KeyShape::K10u))
            .add(self.keynode(250.0,300.0,layer,22,KeyShape::K10u))
            .add(self.keynode(350.0,300.0,layer,23,KeyShape::K10u))
            .add(self.keynode(450.0,300.0,layer,24,KeyShape::K10u))
            .add(self.keynode(550.0,300.0,layer,25,KeyShape::K10u))
            .add(self.keynode(650.0,250.0,layer,26,KeyShape::K15v))
          // Fifth Row
            .add(self.keynode( 50.0,400.0,layer,27,KeyShape::K10u))
            .add(self.keynode(150.0,400.0,layer,28,KeyShape::K10u))
            .add(self.keynode(250.0,400.0,layer,29,KeyShape::K10u))
            .add(self.keynode(350.0,400.0,layer,30,KeyShape::K10u))
            .add(self.keynode(450.0,400.0,layer,31,KeyShape::K10u))
    }

    fn rightmain(self: &Keyboard, layer: usize) -> Group {
        Group::new()
        // first Row
            .add(self.keynode(0.0,0.0,layer,38,KeyShape::K10u))
            .add(self.keynode(100.0,0.0,layer,39,KeyShape::K10u))
            .add(self.keynode(200.0,0.0,layer,40,KeyShape::K10u))
            .add(self.keynode(300.0,0.0,layer,41,KeyShape::K10u))
            .add(self.keynode(400.0,0.0,layer,42,KeyShape::K10u))
            .add(self.keynode(500.0,0.0,layer,43,KeyShape::K10u))
            .add(self.keynode(600.0,0.0,layer,44,KeyShape::K15h))
        // Second row
            .add(self.keynode(0.0,100.0,layer,45,KeyShape::K15v))
            .add(self.keynode(100.0,100.0,layer,46,KeyShape::K10u))
            .add(self.keynode(200.0,100.0,layer,47,KeyShape::K10u))
            .add(self.keynode(300.0,100.0,layer,48,KeyShape::K10u))
            .add(self.keynode(400.0,100.0,layer,49,KeyShape::K10u))
            .add(self.keynode(500.0,100.0,layer,50,KeyShape::K10u))
            .add(self.keynode(600.0,100.0,layer,51,KeyShape::K15h))
        // Third row
            .add(self.keynode(100.0,200.0,layer,52,KeyShape::K10u))
            .add(self.keynode(200.0,200.0,layer,53,KeyShape::K10u))
            .add(self.keynode(300.0,200.0,layer,54,KeyShape::K10u))
            .add(self.keynode(400.0,200.0,layer,55,KeyShape::K10u))
            .add(self.keynode(500.0,200.0,layer,56,KeyShape::K10u))
            .add(self.keynode(600.0,200.0,layer,57,KeyShape::K15h))
        // Fourth row
            .add(self.keynode(0.0,250.0,layer,58,KeyShape::K15v))
            .add(self.keynode(100.0,300.0,layer,59,KeyShape::K10u))
            .add(self.keynode(200.0,300.0,layer,60,KeyShape::K10u))
            .add(self.keynode(300.0,300.0,layer,61,KeyShape::K10u))
            .add(self.keynode(400.0,300.0,layer,62,KeyShape::K10u))
            .add(self.keynode(500.0,300.0,layer,63,KeyShape::K10u))
            .add(self.keynode(600.0,300.0,layer,64,KeyShape::K15h))
        // Fifth row
            .add(self.keynode(200.0,400.0,layer,65,KeyShape::K10u))
            .add(self.keynode(300.0,400.0,layer,66,KeyShape::K10u))
            .add(self.keynode(400.0,400.0,layer,67,KeyShape::K10u))
            .add(self.keynode(500.0,400.0,layer,68,KeyShape::K10u))
            .add(self.keynode(600.0,400.0,layer,69,KeyShape::K10u))

    }

    fn rightthumb(self: &Keyboard, layer: usize) -> Group {
        Group::new()
            .add(self.keynode(0.0,0.0,layer,70,KeyShape::K10u))
            .add(self.keynode(100.0,0.0,layer,71,KeyShape::K10u))
            .add(self.keynode(0.0,100.0,layer,72,KeyShape::K10u))
            .add(self.keynode(0.0,200.0,layer,73,KeyShape::K10u))
            .add(self.keynode(100.0,100.0,layer,74,KeyShape::K20v))
            .add(self.keynode(200.0,100.0,layer,75,KeyShape::K20v))
    }

    fn left(self: &Keyboard, layer: usize) -> Group {
        Group::new()
            .add(self.leftmain(layer))
            .add(self.leftthumb(layer).set("transform", "translate(675,325)"))
        // bottom right corner -- 975, 625
    }

    fn right(self: &Keyboard, layer: usize) -> Group {
        Group::new()
            .add(self.rightmain(layer).set("transform","translate(250,0)"))
            .add(self.rightthumb(layer).set("transform", "translate(0,325)"))
            .set("transform","translate(1000,0)")
    }

    fn layer(self: &Keyboard, layer: usize) -> Group {
        Group::new()
            .add(self.left(layer))
            .add(self.right(layer))
            .set("id", format!("layer{}", layer))
            .set("visibility", "hidden")
    }

    fn keymap(self: &Keyboard) -> Group {
        Range{start: 0, end: self.keymaps.len()}
            .fold(Group::new(),
                  |grp, i| grp.add(self.layer(i)))
    }

    pub fn svg(self: &Keyboard) -> String {

        let cdata = |s: &str| format!("<![CDATA[{}]]>",s);

        let css = include_str!("data/keyboard.css");

        let style = Style::new(cdata(css));

        let js = include_str!("data/keyboard.js");

        let code = Script::new(cdata(js));

        let keyboard = self.keymap();

        let keyoutside = LinearGradient::new()
            .set("id", "keyoutside")
            .set("x1", "0%")
            .set("x2", "0%")
            .set("y1", "0%")
            .set("y2", "100%")
            .add(Stop::new()
                 .set("offset", "0%")
                 .set("stop-color", "#E1E1E1"))
            .add(Stop::new()
                 .set("offset", "100%")
                 .set("stop-color", "#B2B2B2"));

        let keyinside = LinearGradient::new()
            .set("id", "keyinside")
            .set("x1", "0%")
            .set("x2", "100%")
            .set("y1", "0%")
            .set("y2", "0%")
            .add(Stop::new()
                 .set("offset", "0%")
                 .set("stop-color", "#D6D6D6"))
            .add(Stop::new()
                 .set("offset", "50%")
                 .set("stop-color", "#EBEBEB"))
            .add(Stop::new()
                 .set("offset", "100%")
                 .set("stop-color", "#D6D6D6"));

        let defs = Definitions::new()
            .add(keyoutside)
            .add(keyinside);

        let doc = Document::new()
            .set("viewBox", (0,0,2000,625))
            .add(style)
            .add(code)
            .add(defs)
            .add(keyboard);

        doc.to_string()
    }
}
