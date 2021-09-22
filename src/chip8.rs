/* TODO : convert display to a trait? */
use crate::display::Display;
use crate::display::NCursesDisplay;

use std::{thread, time};

/* TODO : restore debug trait */
pub struct Chip8 {
    display: NCursesDisplay
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            display: NCursesDisplay::new(64, 32)
        }
    }

    pub fn run(& mut self) {
        self.display.open();

        let mut y = 0;

        while y < 32 {
            let mut x = 0;
            while x < 64 {
                self.display.led_off(x, y);
                self.display.refresh();

                let millis = time::Duration::from_millis(16);
                thread::sleep(millis);

                x += 1;
            }

            y += 1;
        }

        self.display.close();
    }
}