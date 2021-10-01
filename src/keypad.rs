pub trait Keypad {
    fn set_is_pressed(&mut self, code: usize, is_pressed: bool);
    fn get_is_pressed(&self, code: usize) -> bool;
}

pub struct KeyboardKeypad {
    keys: Vec<bool>
}

impl KeyboardKeypad {
    pub fn new(codes: usize) -> KeyboardKeypad {
        KeyboardKeypad {
            keys: vec![false; codes]
        }
    }
}

impl Keypad for KeyboardKeypad {
    fn set_is_pressed(&mut self, code: usize, is_pressed: bool) {
        self.keys[code] = is_pressed;
    }

    fn get_is_pressed(&self, code: usize) -> bool {
        self.keys[code]
    }
}
