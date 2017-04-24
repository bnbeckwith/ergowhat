use std::fmt;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Key {
    Fx(u64),
    Key(String)
}

#[derive(Debug, PartialEq)]
pub enum Action{
    Function(Key),
    FunctionTap(Key),
    LayerSet(u64, String),
    LayerMomentary(u64),
    LayerTapKey(u64, Key),
    ModsKey(Key, Key),
    ModsTapKey(Key, Key)
}

pub type KeyMap    = Vec<Key>;
pub type KeyMapVec = Vec<KeyMap>;
pub type ActionMap = HashMap<u64, Action>;

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
