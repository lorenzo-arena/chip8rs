/* TODO : convert display to a trait? */
use crate::display::Display;
use crate::display::NCursesDisplay;
use crate::fonts::Fonts;
use crate::logger::FileLogger;
use crate::logger::Logger;

use rand::Rng;
use std::{fs, thread, time};

const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 100;
const REGISTERS_SIZE: usize = 16;
const FONT_START: u16 = 0x50;
const ROM_START: u16 = 0x200;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

const LOG_FILE: &str = "chip8rs.log";

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
    fonts: Fonts,
    logger: FileLogger,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            display: NCursesDisplay::new(DISPLAY_WIDTH, DISPLAY_HEIGHT, false),
            memory: [0; MEMORY_SIZE],
            pc: 0,
            i: 0,
            stack: vec![0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            regs: [0; REGISTERS_SIZE],
            fonts: Fonts::new(),
            logger: FileLogger::new(LOG_FILE.to_string()),
        }
    }

    fn load_fonts(&mut self) {
        let mut dest = FONT_START as usize;

        for font in self.fonts.fonts {
            self.memory[dest..(dest + font.len())].copy_from_slice(&font);
            dest += font.len();
        }
    }

    fn load_rom(&mut self, rom_path: &str) {
        let file_content = fs::read(rom_path).unwrap();

        if file_content.len() > (MEMORY_SIZE - ROM_START as usize) {
            panic!("ROM content is too large");
        }

        let dest = ROM_START as usize;
        self.memory[dest..(dest + file_content.len())].copy_from_slice(&file_content);
    }

    fn fetch(&mut self) -> u16 {
        let first = self.memory[self.pc as usize] as u16;
        let second = self.memory[(self.pc + 1) as usize] as u16;

        self.pc += 2;

        (first << 8) | second
    }

    fn execute(&mut self, instr: u16) {
        match instr & 0xF000 {
            0x0000 => {
                if instr == 0x00E0 {
                    /* 00E0: clear screen instruction, turn all pixels off */
                    self.display.clear_screen(false);
                    self.display.refresh();
                } else if instr == 0x00EE {
                    /* 00EE: return from subroutine, pop the PC */
                    self.pc = self.stack.pop().unwrap();
                } else {
                    panic!("Unknown instruction found: {:X?}", instr);
                }
            },
            0x1000 => {
                /* 1NNN: jump, set the PC to NNN */
                self.pc = instr & 0x0FFF;
            },
            0x2000 => {
                /* 2NNN: call subroutine, push the PC and set the PC to NNN */
                self.stack.push(self.pc);
                self.pc = instr & 0x0FFF;
            },
            0x3000 => {
                /* 3XNN: skip one instruction if VX content is equal to NN */
                let reg = (instr & 0x0F00) >> 8;
                let value = (instr & 0x00FF) as u8;
                let reg_value = self.regs[reg as usize];

                if value == reg_value {
                    self.pc += 2;
                }
            },
            0x4000 => {
                /* 4XNN: skip one instruction if VX content is NOT equal to NN */
                let reg = (instr & 0x0F00) >> 8;
                let value = (instr & 0x00FF) as u8;
                let reg_value = self.regs[reg as usize];

                if value != reg_value {
                    self.pc += 2;
                }
            }
            0x5000 => {
                /* 5XY0: skip one instruction if VX and VY values are equal */
                if (instr & 0xF00F) == 0x5000 {
                    let reg_x = (instr & 0x0F00) >> 8;
                    let reg_y = (instr & 0x00F0) >> 4;

                    if self.regs[reg_x as usize] == self.regs[reg_y as usize] {
                        self.pc += 2;
                    }
                } else {
                    panic!("Unknown instruction found: {:X?}", instr);
                }
            }
            0x6000 => {
                /* 6XNN: set register X to value NN */
                let reg = (instr & 0x0F00) >> 8;
                self.regs[reg as usize] = (instr & 0x00FF) as u8;
            },
            0x7000 => {
                /* 7XNN: add value to register X; this can overflow, so a helper variable is used */
                let reg = (instr & 0x0F00) >> 8;
                let mut add_value = self.regs[reg as usize] as u16;
                add_value += (instr & 0x00FF) as u16;
                self.regs[reg as usize] = (add_value & 0x00FF) as u8;
            },
            0x8000 => {
                /* Process logical instruction */
                self.logical_instruction(instr);
            },
            0x9000 => {
                /* 9XY0: skip one instruction if VX and VY values are NOT equal */
                if (instr & 0xF00F) == 0x9000 {
                    let reg_x = (instr & 0x0F00) >> 8;
                    let reg_y = (instr & 0x00F0) >> 4;

                    if self.regs[reg_x as usize] != self.regs[reg_y as usize] {
                        self.pc += 2;
                    }
                } else {
                    panic!("Unknown skip instruction found: {:X?}", instr);
                }
            }
            0xA000 => {
                /* ANNN: set index to value NNN */
                self.i = instr & 0x0FFF;
            },
            0xB000 => {
                /* TODO : this should be made configurable, as some implementations interpret this like a "BXNN" */
                /* BNNN: JUMP, set PC to NNN plus the value of V0 */
                self.pc = (instr & 0x0FFF) + (self.regs[0x00 as usize] as u16);
            },
            0xC000 => {
                /* CXNN: RANDOM, generate a random number, binary AND with NN and set the result in VX */
                let mut rng = rand::thread_rng();
                let random: u8 = rng.gen();
                let reg = (instr & 0x0F00) >> 8;

                self.regs[reg as usize] = random & ((instr & 0x00FF) as u8);
            },
            0xD000 => {
                /* DXYN: display */
                self.draw_sprite(instr & 0x0FFF);
            },
            0xE000 => {
                if (instr & 0xF0FF) == 0xE09E {
                    panic!("Unknown keypad skip instruction found: {:X?}", instr);
                } else if (instr & 0xF0FF) == 0xE0A1 {
                    panic!("Unknown keypad skip instruction found: {:X?}", instr);
                } else {
                    panic!("Unknown keypad skip instruction found: {:X?}", instr);
                }
            }
            _ => {
                panic!("Unknown instruction found: {:X?}", instr);
            }
        }
    }

    fn logical_instruction(&mut self, instr: u16) {
        let reg_x = (instr & 0x0F00) >> 8;
        let reg_y = (instr & 0x00F0) >> 4;

        match instr & 0xF00F {
            0x8000 => {
                /* 8XY0: set instruction; copy VY to VX */
                self.regs[reg_x as usize] = self.regs[reg_y as usize];
            },
            0x8001 => {
                /* 8XY1: binary OR, set VX to the OR of VX and VY */
                self.regs[reg_x as usize] = self.regs[reg_x as usize] | self.regs[reg_y as usize];
            },
            0x8002 => {
                /* 8XY2: binary AND, set VX to the AND of VX and VY */
                self.regs[reg_x as usize] = self.regs[reg_x as usize] & self.regs[reg_y as usize];
            },
            0x8003 => {
                /* 8XY3: binary XOR, set VX to the XOR of VX and VY */
                self.regs[reg_x as usize] = self.regs[reg_x as usize] ^ self.regs[reg_y as usize];
            },
            0x8004 => {
                /* 8XY4: ADD, VX is set to the value of VX plus VY; if overflow occurs, set the flag register */
                let mut add_value = (self.regs[reg_x as usize] as u16) + self.regs[reg_y as usize] as u16;

                if add_value > 0xFF {
                    /* Overflow occurred, set the flag register */
                    self.regs[0x0F as usize] = 1;
                } else {
                    self.regs[0x0F as usize] = 0;
                }

                self.regs[reg_x as usize] = (add_value & 0x00FF) as u8;
            },
            0x8005 => {
                /* 8XY5: SUBTRACT, VX is set to the value of VX minus VY;
                 * in this case, the flag register is set if the first operand is larger than the second operand */

                if self.regs[reg_x as usize] >= self.regs[reg_y as usize] {
                    self.regs[0x0F as usize] = 1;
                    self.regs[reg_x as usize] = self.regs[reg_x as usize] - self.regs[reg_y as usize];
                } else {
                    self.regs[0x0F as usize] = 0;
                    /* Since the operation would underflow, let's multiply by -1 by swapping the operands */
                    self.regs[reg_x as usize] = self.regs[reg_y as usize] - self.regs[reg_x as usize];
                }
            },
            0x8006 => {
                /* TODO : is this a circular shift or not? */
                /* 8XY6: SHIFT; shift VX one bit to the right */

                /* TODO: this should be made optional, since some implementation (like CHIP-48 or SUPER-CHIP)
                 * did not apply this instruction */
                self.regs[reg_x as usize] = self.regs[reg_y as usize];

                /* Set the flag register to 1 if the shifted bit was 1 */
                if (self.regs[reg_x as usize] & 0x01) == 0x01 {
                    self.regs[0x0F as usize] = 1;
                } else {
                    self.regs[0x0F as usize] = 0;
                }

                self.regs[reg_x as usize] = self.regs[reg_x as usize] >> 1;
            }
            0x8007 => {
                /* 8XY7: SUBTRACT, VX is set to the value of VY minus VX;
                 * in this case, the flag register is set if the first operand is larger than the second operand */

                if self.regs[reg_y as usize] >= self.regs[reg_x as usize] {
                    self.regs[0x0F as usize] = 1;
                    self.regs[reg_x as usize] = self.regs[reg_y as usize] - self.regs[reg_x as usize];
                } else {
                    self.regs[0x0F as usize] = 0;
                    /* Since the operation would underflow, let's multiply by -1 by swapping the operands */
                    self.regs[reg_x as usize] = self.regs[reg_x as usize] - self.regs[reg_y as usize];
                }
            },
            0x800E => {
                /* TODO : is this a circular shift or not? */
                /* 8XYE: SHIFT; shift VX one bit to the left */

                /* TODO: this should be made optional, since some implementation (like CHIP-48 or SUPER-CHIP)
                 * did not apply this instruction */
                 self.regs[reg_x as usize] = self.regs[reg_y as usize];

                 /* Set the flag register to 1 if the shifted bit was 1 */
                 if (self.regs[reg_x as usize] & 0x01) == 0x01 {
                     self.regs[0x0F as usize] = 1;
                 } else {
                     self.regs[0x0F as usize] = 0;
                 }
 
                 self.regs[reg_x as usize] = self.regs[reg_x as usize] << 1;
            },
            _ => {
                panic!("Unknown logical instruction found: {:X?}", instr);
            }
        }
    }

    /* TODO : this should be moved to another entity */
    fn draw_sprite(&mut self, instr: u16) {
        /* Get X and Y coordinates from the registers */
        let x = (instr & 0x0F00) >> 8;
        let x = self.regs[x as usize] % (DISPLAY_WIDTH as u8);
        let y = (instr & 0x00F0) >> 4;
        let y = self.regs[y as usize] % (DISPLAY_HEIGHT as u8);

        /* Set VF to 0 as default; it will be set to 1 if any pixel is turned off */
        self.regs[0x0F as usize] = 0;

        let n = (instr & 0x000F) as u8;

        self.logger.log(format!(
            "Drawing sprite from coordinates {}:{}, size {}",
            x, y, n
        ));

        for sprite_row in 0..n {
            let y_pos = (y + sprite_row) as usize;
            if y_pos < DISPLAY_HEIGHT {
                let sprite_data = self.memory[(self.i + (sprite_row as u16)) as usize];
                self.logger.log(format!("Sprite data: {:X?}", sprite_data));

                for sprite_bit_i in 0..8 {
                    let x_pos = (x + sprite_bit_i) as usize;
                    if x_pos < DISPLAY_WIDTH {
                        /* The bits must be read from MIB to LIB */
                        let bit_index = 7 - sprite_bit_i;
                        let bit_value = (sprite_data & (0x1 << bit_index)) >> bit_index;
                        let led_status = self.display.is_on(x_pos, y_pos);

                        /* If current pixel is on and bit is high, flip the led */
                        if (bit_value != 0) && led_status {
                            self.display.led_off(x_pos, y_pos);
                            self.display.refresh();

                            /* Set VF to 1 since a led has been changed */
                            self.regs[0x0F as usize] = 1;
                        } else if (bit_value != 0) && !led_status {
                            self.logger
                                .log(format!("Turning ON led {}:{}", x_pos, y_pos));
                            self.display.led_on(x_pos, y_pos);
                            self.display.refresh();
                        }
                    } else {
                        self.logger.log(format!("X overflow while drawing sprite"));
                    }
                }
            } else {
                self.logger.log(format!("Y overflow while drawing sprite"));
            }
        }
    }

    pub fn run(&mut self, rom_path: &str) {
        self.load_fonts();
        self.load_rom(rom_path);

        self.pc = ROM_START;

        self.display.open();

        loop {
            let instr = self.fetch();
            self.execute(instr);

            /* TODO : timing can be implemented better; but supposing that the fetch/execution times
             * are negligible, a 2 ms sleep will make the emulator execute ~500 instruction per seconds.
             * It seems like a standard speed of 700 CHIP-8 instructions per seconds fits well enough for
             * most games. */
            let millis = time::Duration::from_millis(2);
            thread::sleep(millis);
        }

        self.display.close();
    }
}
