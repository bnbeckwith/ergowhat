use std::fmt;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Key {
    Fx(u32),
    Key(String)
}

#[derive(Debug, PartialEq)]
pub enum Action{
    // Run a specific function
    Function(Key),
    // Tappable function
    FunctionTap(Key),
    // Sets a layer that is always valid
    DefaultLayerSet(u32),
    // Turn on/off layer only
    LayerSet(u32, String),
    // Turn on layer only and clear all layers on release
    LayerSetClear(u32),
    // Momentary layer setting
    LayerMomentary(u32),
    // Turns on momentary `layer` while holding, but key if tapping
    LayerTapKey(u32, Key),
    // Turn on layer momentarily and toggles it on taps
    LayerTapToggle(u32),
    // Toggle setting of layer
    LayerToggle(u32),
    // // Usually of the form (KEY | KEY)
    // // to press multiple
    // Mods(Key),
    // Run these two keys together
    // The usual way is (modifier, key)
    ModsKey(Key, Key),
    // Modifier while holding, key if tapping
    // (Mod, Key)
    ModsTapKey(Key, Key)
}

pub type KeyMap    = Vec<Key>;
pub type KeyMapVec = Vec<KeyMap>;
pub type ActionMap = HashMap<u32, Action>;

impl fmt::Display for Key {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let key = match self {
            &Key::Fx(ref n) => format!("FN{}",n),
            &Key::Key(ref k) => format!("{}", k)
        };
        try!(fmt.write_str(&key));
        Ok(())
    }
}
