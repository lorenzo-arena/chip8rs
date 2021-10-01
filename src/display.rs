pub trait Display {
    fn led_on(&mut self, x: usize, y: usize);
    fn led_off(&mut self, x: usize, y: usize);
    fn clear_screen(&mut self, on: bool);
    fn is_on(&self, x: usize, y: usize) -> bool;
}

pub struct LedsDisplay {
    x_len: usize,
    y_len: usize,
    leds: Vec<Vec<bool>>,
}

/* TODO : implement option for double ratio */
impl LedsDisplay {
    pub fn new(x_len: usize, y_len: usize, on: bool) -> LedsDisplay {
        LedsDisplay {
            x_len: x_len,
            y_len: y_len,
            leds: vec![vec![on; x_len]; y_len],
        }
    }
}

impl Display for LedsDisplay {
    fn led_on(&mut self, x: usize, y: usize) {
        self.leds[y][x] = true;
    }

    fn led_off(&mut self, x: usize, y: usize) {
        self.leds[y][x] = false;
    }

    fn clear_screen(&mut self, on: bool) {
        for y in 0..self.y_len {
            for x in 0..self.x_len {
                self.leds[y][x] = on;
            }
        }
    }

    fn is_on(&self, x: usize, y: usize) -> bool {
        self.leds[y][x]
    }
}
