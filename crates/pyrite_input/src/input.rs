use crate::keyboard::{Key, Modifier};
use pyrite_app::resource::Resource;
use std::collections::HashSet;

#[derive(Resource)]
pub struct Input {
    pressed_keys: HashSet<Key>,
    down_keys: HashSet<Key>,
    released_keys: HashSet<Key>,
}

impl Input {
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
