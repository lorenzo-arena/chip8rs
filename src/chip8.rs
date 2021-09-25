/* TODO : convert display to a trait? */
use crate::display::Display;
use crate::display::NCursesDisplay;
use crate::fonts::Fonts;

use std::{thread, time, fs};

const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 100;
const REGISTERS_SIZE: usize = 16;
const FONT_START: usize = 0x50;
const ROM_START: usize = 0x200;

/* TODO : restore debug trait */
/* TODO : use arrays instead of vecs? */
pub struct Chip8 {
    display: NCursesDisplay,
    memory: [u8; MEMORY_SIZE],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    regs: [u8; REGISTERS_SIZE],
    fonts: Fonts
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            display: NCursesDisplay::new(64, 32),
            memory: [0; MEMORY_SIZE],
            pc: 0,
            i: 0,
            stack: vec![0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            regs: [0; REGISTERS_SIZE],
            fonts: Fonts::new()
        }
    }

    fn load_fonts(& mut self) {
        let mut dest = FONT_START;

        for font in self.fonts.fonts {
            self.memory[dest..(dest + font.len())].copy_from_slice(&font);
            dest += font.len();
        }
    }

    fn load_rom(& mut self, rom_path: &str) {
        let file_content = fs::read(rom_path).unwrap();

        if file_content.len() > (MEMORY_SIZE - ROM_START) {
            panic!("ROM content is too large");
        }

        let dest = ROM_START;
        self.memory[dest..(dest + file_content.len())].copy_from_slice(&file_content);
    }

    pub fn run(& mut self, rom_path: &str) {
        self.load_fonts();
        self.load_rom(rom_path);

        /* TODO: Used to debug memory content, to remove */
        //println!("{:x?}", self.memory);
        //loop {}

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