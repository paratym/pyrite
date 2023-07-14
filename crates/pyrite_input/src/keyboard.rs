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