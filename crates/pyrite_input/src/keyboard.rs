use std::collections::HashSet;

pub struct Keyboard {
    pressed_keys: HashSet<Key>,
    down_keys: HashSet<Key>,
    released_keys: HashSet<Key>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            down_keys: HashSet::new(),
            released_keys: HashSet::new(),
        }
    }

    pub fn submit_input(&mut self, input: SubmitInput) {
        match input {
            SubmitInput::Pressed(key) => {
                self.pressed_keys.insert(key);
                self.down_keys.insert(key);
            }
            SubmitInput::Released(key) => {
                self.released_keys.insert(key);
                self.down_keys.remove(&key);
            }
        }
    }

    pub fn clear_inputs(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_key_pressed_with_modifiers(&self, key: Key, modifiers: &[Modifier]) -> bool {
        self.is_key_pressed(key) && self.is_modifiers_down(modifiers)
    }

    pub fn is_key_down_with_modifiers(&self, key: Key, modifiers: &[Modifier]) -> bool {
        self.is_key_down(key) && self.is_modifiers_down(modifiers)
    }

    pub fn is_key_released_with_modifiers(&self, key: Key, modifiers: &[Modifier]) -> bool {
        self.is_key_released(key) && self.is_modifiers_down(modifiers)
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.down_keys.contains(&key)
    }

    pub fn is_key_released(&self, key: Key) -> bool {
        self.released_keys.contains(&key)
    }

    pub fn is_modifiers_down(&self, modifiers: &[Modifier]) -> bool {
        for modifier in modifiers {
            if !modifier
                .get_keys()
                .iter()
                .any(|k| self.is_key_down(k.clone()))
            {
                return false;
            }
        }
        return true;
    }
}

pub enum SubmitInput {
    Pressed(Key),
    Released(Key),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    Escape,

    LControl,
    LShift,
    LAlt,
    LSystem,

    RControl,
    RShift,
    RAlt,
    RSystem,

    LBracket,
    RBracket,

    Semicolon,
    Comma,
    Period,
    Quote,
    Slash,
    Backslash,
    Tilde,
    Equal,
    Hyphen,

    Space,
    Enter,
    Backspace,
    Tab,

    PageUp,
    PageDown,
    End,
    Home,
    Insert,
    Delete,

    Left,
    Right,
    Up,
    Down,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Shift,
    Control,
    Alt,
}

impl Modifier {
    pub(crate) fn get_keys(&self) -> Vec<Key> {
        match self {
            Modifier::Shift => vec![Key::LShift, Key::RShift],
            Modifier::Control => vec![Key::LControl, Key::RControl],
            Modifier::Alt => vec![Key::LAlt, Key::RAlt],
        }
    }
}
