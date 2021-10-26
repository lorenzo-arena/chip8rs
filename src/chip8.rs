use crate::display::*;
use crate::fonts::Fonts;
use crate::fonts::FONT_SIZE;
use crate::keypad::*;
use crate::logger::FileLogger;
use crate::logger::Logger;

use rand::Rng;
use std::sync::{Arc, Mutex};
use std::{fs, thread, time};

use rodio::source::{SineWave, Source};
use rodio::OutputStream;

const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 100;
const REGISTERS_SIZE: usize = 16;
const FONT_START: u16 = 0x50;
const ROM_START: u16 = 0x200;

/* TODO : add getters from real display struct */
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

const LOG_FILE: &str = "chip8rs.log";

/* TODO : restore debug trait */
/* TODO : use arrays instead of vecs? */
/* TODO : set option for more verbose logs */
pub struct Chip8 {
    display: Arc<Mutex<LedsDisplay>>,
    keypad: Arc<Mutex<KeyboardKeypad>>,
    memory: [u8; MEMORY_SIZE],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: Arc<Mutex<u8>>,
    sound_timer: Arc<Mutex<u8>>,
    regs: [u8; REGISTERS_SIZE],
    fonts: Fonts,
    logger: FileLogger,
}

enum Instruction {
    ClearScreen,
    Return,
    Jump(u16),
    Call(u16),
    SkipIfEqual(u8, u8),
    SkipIfDifferent(u8, u8),
    SkipIfContentEqual(u8, u8),
    SkipIfContentDifferent(u8, u8),
    SetRegister(u8, u8),
    AddToRegister(u8, u8),
    LogicalCopy(u8, u8),
    LogicalOr(u8, u8),
    LogicalAnd(u8, u8),
    LogicalXor(u8, u8),
    LogicalAdd(u8, u8),
    LogicalSubtract(u8, u8),
    LogicalRightShift(u8),
    LogicalSubtractInverse(u8, u8),
    LogicalLeftShift(u8),
    SetIndex(u16),
    JumpWithRegister(u16),
    Random(u8, u8),
    Display(u8, u8, u8),
    SkipIfPressed(u8),
    SkipIfNotPressed(u8),
    CopyDelayTimer(u8),
    WaitForKey(u8),
    SetDelayTimer(u8),
    SetSoundTimer(u8),
    AddToIndex(u8),
    SetIndexToFont(u8),
    BinaryConversion(u8),
    Store(u8),
    Load(u8),
}

impl From<u16> for Instruction {
    fn from(instr: u16) -> Self {
        match instr & 0xF000 {
            0x0000 => {
                /* A 0NNN instruction exists to execute native 1802 machine code in the COSMAC VIP; it
                 * has not been implemented */
                if instr == 0x00E0 {
                    return Instruction::ClearScreen;
                } else if instr == 0x00EE {
                    return Instruction::Return;
                } else {
                    panic!("Unknown instruction found: {:X?}", instr);
                }
            },
            0x1000 => {
                /* 1NNN: jump, set the PC to NNN */
                return Instruction::Jump(instr & 0x0FFF);
            },
            0x2000 => {
                /* 2NNN: call subroutine, push the PC and set the PC to NNN */
                return Instruction::Call(instr & 0x0FFF);
            },
            0x3000 => {
                /* 3XNN: skip one instruction if VX content is equal to NN */
                let reg_x = (instr & 0x0F00) >> 8;
                return Instruction::SkipIfEqual(reg_x as u8, (instr & 0x00FF) as u8);
            },
            0x4000 => {
                /* 4XNN: skip one instruction if VX content is NOT equal to NN */
                let reg_x = (instr & 0x0F00) >> 8;
                return Instruction::SkipIfDifferent(reg_x as u8, (instr & 0x00FF) as u8);
            },
            0x5000 => {
                if (instr & 0xF00F) == 0x5000 {
                    /* 5XY0: skip one instruction if VX and VY values are equal */
                    let reg_x = (instr & 0x0F00) >> 8;
                    let reg_y = (instr & 0x00F0) >> 4;
                    return Instruction::SkipIfContentEqual(reg_x as u8, reg_y as u8);
                } else {
                    panic!("Unknown instruction found: {:X?}", instr);
                }
            },
            0x6000 => {
                /* 6XNN: set register X to value NN */
                let reg = (instr & 0x0F00) >> 8;
                return Instruction::SetRegister(reg as u8, (instr & 0x00FF) as u8);
            },
            0x7000 => {
                /* 7XNN: add value to register X; this can overflow, so a helper variable is used */
                let reg = (instr & 0x0F00) >> 8;
                let value = instr & 0x00FF;
                return Instruction::AddToRegister(reg as u8, value as u8);
            },
            0x8000 => {
                /* Process logical instruction */
                let reg_x = ((instr & 0x0F00) >> 8) as u8;
                let reg_y = ((instr & 0x00F0) >> 4) as u8;
                match instr & 0xF00F {
                    0x8000 => {
                        /* 8XY0: set instruction; copy VY to VX */
                        return Instruction::LogicalCopy(reg_x, reg_y);
                    },
                    0x8001 => {
                        /* 8XY1: binary OR, set VX to the OR of VX and VY */
                        return Instruction::LogicalOr(reg_x, reg_y);
                    },
                    0x8002 => {
                        /* 8XY2: binary AND, set VX to the AND of VX and VY */
                        return Instruction::LogicalAnd(reg_x, reg_y);
                    },
                    0x8003 => {
                        /* 8XY3: binary XOR, set VX to the XOR of VX and VY */
                        return Instruction::LogicalXor(reg_x, reg_y);
                    },
                    0x8004 => {
                        /* 8XY4: ADD, VX is set to the value of VX plus VY; if overflow occurs, set the flag register */
                        return Instruction::LogicalAdd(reg_x, reg_y);
                    },
                    0x8005 => {
                        /* 8XY5: SUBTRACT, VX is set to the value of VX minus VY;
                         * in this case, the flag register is set if the first operand is larger than the second operand */
                        return Instruction::LogicalSubtract(reg_x, reg_y);
                    },
                    0x8006 => {
                        /* 8XY6: SHIFT; shift VX one bit to the right */
                        return Instruction::LogicalRightShift(reg_x);
                    },
                    0x8007 => {
                        /* 8XY7: SUBTRACT, VX is set to the value of VY minus VX;
                         * in this case, the flag register is set if the first operand is larger than the second operand */
                        return Instruction::LogicalSubtractInverse(reg_x, reg_y);
                    },
                    0x800E => {
                        /* 8XYE: SHIFT; shift VX one bit to the left */
                        return Instruction::LogicalLeftShift(reg_x);
                    },
                    _ => {
                        panic!("Unknown logical instruction found: {:X?}", instr);
                    }
                }
            },
            0x9000 => {
                /* 9XY0: skip one instruction if VX and VY values are NOT equal */
                if (instr & 0xF00F) == 0x9000 {
                    let reg_x = (instr & 0x0F00) >> 8;
                    let reg_y = (instr & 0x00F0) >> 4;

                    return Instruction::SkipIfContentDifferent(reg_x as u8, reg_y as u8);
                } else {
                    panic!("Unknown skip instruction found: {:X?}", instr);
                }
            },
            0xA000 => {
                /* ANNN: set index to value NNN */
                return Instruction::SetIndex(instr & 0x0FFF);
            },
            0xB000 => {
                /* BNNN: JUMP, set PC to NNN plus the value of V0 */
                return Instruction::JumpWithRegister(instr & 0x0FFF);
            },
            0xC000 => {
                /* CXNN: RANDOM, generate a random number, binary AND with NN and set the result in VX */
                let reg = (instr & 0x0F00) >> 8;
                return Instruction::Random(reg as u8, (instr & 0x00FF) as u8);
            },
            0xD000 => {
                /* DXYN: display */
                let x = (instr & 0x0F00) >> 8;
                let y = (instr & 0x00F0) >> 4;
                let n = instr & 0x000F;
                return Instruction::Display(x as u8, y as u8, n as u8);
            },
            0xE000 => {
                if (instr & 0xF0FF) == 0xE09E {
                    /* EX9E: skip instruction if key value from VX is currenty pressed */
                    let reg = (instr & 0x0F00) >> 8;
                    return Instruction::SkipIfPressed(reg as u8);
                } else if (instr & 0xF0FF) == 0xE0A1 {
                    /* EXA1: skip instruction if key value from VX is NOT currenty pressed */
                    let reg = (instr & 0x0F00) >> 8;
                    return Instruction::SkipIfNotPressed(reg as u8);
                } else {
                    panic!("Unknown keypad skip instruction found: {:X?}", instr);
                }
            },
            0xF000 => {
                let reg = (instr & 0x0F00) >> 8;

                match instr & 0xF0FF {
                    0xF007 => {
                        /* FX07: copy timer; set VX to the current value of the delay timer */
                        return Instruction::CopyDelayTimer(reg as u8);
                    }
                    0xF00A => {
                        /* FX0A: wait for a key press and set its value to VX */
                        return Instruction::WaitForKey(reg as u8);
                    }
                    0xF015 => {
                        /* FX15: set timer; set the delay timer to the value in VX */
                        return Instruction::SetDelayTimer(reg as u8);
                    }
                    0xF018 => {
                        /* FX18: set timer; set the sound timer to the value in VX */
                        return Instruction::SetSoundTimer(reg as u8);
                    }
                    0xF01E => {
                        /* FX1E: add to index; add the content of VX to the index, checking for overflows */
                        return Instruction::AddToIndex(reg as u8);
                    }
                    0xF029 => {
                        /* FX29: font character; set I to the address of the "char" contained in VX */
                        return Instruction::SetIndexToFont(reg as u8);
                    }
                    0xF033 => {
                        /* FX33: binary-coded decimal conversion; take the value of VX and convert it in 3 decimal digits */
                        return Instruction::BinaryConversion(reg as u8);
                    }
                    0xF055 => {
                        /* FX55: store in memory; save value from V0 to VX to index from I to I * X in memory */
                        return Instruction::Store(reg as u8);
                    }
                    0xF065 => {
                        /* FX65: load from memory; save value from index I to I * X to V0 to VX  */
                        return Instruction::Load(reg as u8);
                    }
                    _ => {
                        panic!("Unknown instruction found: {:X?}", instr);
                    }
                }
            },
            _ => {
                panic!("Unknown instruction found: {:X?}", instr);
            }
        }
    }
}

impl Chip8 {
    pub fn new(display: &Arc<Mutex<LedsDisplay>>, keypad: &Arc<Mutex<KeyboardKeypad>>) -> Chip8 {
        Chip8 {
            display: Arc::clone(display),
            keypad: Arc::clone(keypad),
            memory: [0; MEMORY_SIZE],
            pc: 0,
            i: 0,
            stack: vec![0; STACK_SIZE],
            delay_timer: Arc::new(Mutex::new(0)),
            sound_timer: Arc::new(Mutex::new(0)),
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

    fn clear_screen(&mut self) {
        self.display.lock().unwrap().clear_screen(false);
    }

    fn return_subroutine(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn call_subroutine(&mut self, addr: u16) {
        self.stack.push(self.pc);
        self.pc = addr;
    }

    fn skip_if_eq(&mut self, reg: u8, value: u8) {
        let reg_value = self.regs[reg as usize];

        if value == reg_value {
            self.pc += 2;
        }
    }

    fn skip_if_diff(&mut self, reg: u8, value: u8) {
        let reg_value = self.regs[reg as usize];

        if value != reg_value {
            self.pc += 2;
        }
    }

    fn skip_if_content_eq(&mut self, reg_x: u8, reg_y: u8) {
        if self.regs[reg_x as usize] == self.regs[reg_y as usize] {
            self.pc += 2;
        }
    }

    fn set_register(&mut self, reg: u8, value: u8) {
        self.regs[reg as usize] = value;
    }

    fn add_to_reg(&mut self, reg: u8, value: u8) {
        let mut add_value = self.regs[reg as usize] as u16;
        add_value += value as u16;
        self.regs[reg as usize] = (add_value & 0x00FF) as u8;
    }

    fn logical_copy(&mut self, reg_x: u8, reg_y: u8) {
        self.regs[reg_x as usize] = self.regs[reg_y as usize];
    }

    fn logical_or(&mut self, reg_x: u8, reg_y: u8) {
        self.regs[reg_x as usize] = self.regs[reg_x as usize] | self.regs[reg_y as usize];
    }

    fn logical_and(&mut self, reg_x: u8, reg_y: u8) {
        self.regs[reg_x as usize] = self.regs[reg_x as usize] & self.regs[reg_y as usize];
    }

    fn logical_xor(&mut self, reg_x: u8, reg_y: u8) {
        self.regs[reg_x as usize] = self.regs[reg_x as usize] ^ self.regs[reg_y as usize];
    }

    fn logical_add(&mut self, reg_x: u8, reg_y: u8) {
        let add_value =
                    (self.regs[reg_x as usize] as u16) + self.regs[reg_y as usize] as u16;

        if add_value > 0xFF {
            /* Overflow occurred, set the flag register */
            self.regs[0x0F as usize] = 1;
        } else {
            self.regs[0x0F as usize] = 0;
        }

        self.regs[reg_x as usize] = (add_value & 0x00FF) as u8;
    }

    fn logical_sub(&mut self, reg_x: u8, reg_y: u8) {
        if self.regs[reg_x as usize] > self.regs[reg_y as usize] {
            self.regs[0x0F as usize] = 1;
            self.regs[reg_x as usize] =
                self.regs[reg_x as usize] - self.regs[reg_y as usize];
        } else {
            self.regs[0x0F as usize] = 0;
            /* From the specification, this instruction should result in the rolling of the uint */
            self.regs[reg_x as usize] = ((0x100 - (self.regs[reg_y as usize] as u16)
                + (self.regs[reg_x as usize] as u16))
                & 0xFF) as u8;
        }
    }

    fn logical_right_shift(&mut self, reg_x: u8) {
        /* TODO: this should be made optional, since some implementation (like CHIP-48 or SUPER-CHIP)
         * did not apply this instruction */
        //self.regs[reg_x as usize] = self.regs[reg_y as usize];

        /* Set the flag register to 1 if the shifted bit was 1 */
        if (self.regs[reg_x as usize] & 0x01) == 0x01 {
            self.regs[0x0F as usize] = 1;
        } else {
            self.regs[0x0F as usize] = 0;
        }

        self.regs[reg_x as usize] = self.regs[reg_x as usize] >> 1;
    }

    fn logical_sub_inv(&mut self, reg_x: u8, reg_y: u8) {
        if self.regs[reg_y as usize] >= self.regs[reg_x as usize] {
            self.regs[reg_x as usize] =
                self.regs[reg_y as usize] - self.regs[reg_x as usize];

            /* Set VF after the operation so that VF can be used in subtractions; this should not break compatibility anyway */
            self.regs[0x0F as usize] = 1;
        } else {
            /* Since the operation would underflow, let's multiply by -1 by swapping the operands */
            self.regs[reg_x as usize] =
                self.regs[reg_x as usize] - self.regs[reg_y as usize];

            /* Set VF after the operation so that VF can be used in subtractions; this should not break compatibility anyway */
            self.regs[0x0F as usize] = 0;
        }
    }

    fn logical_left_shift(&mut self, reg_x: u8) {
        /* TODO: this should be made optional, since some implementation (like CHIP-48 or SUPER-CHIP)
         * did not apply this instruction */
        //self.regs[reg_x as usize] = self.regs[reg_y as usize];

        /* Set the flag register to 1 if the shifted bit was 1 */
        if (self.regs[reg_x as usize] & 0x80) == 0x80 {
            self.regs[0x0F as usize] = 1;
        } else {
            self.regs[0x0F as usize] = 0;
        }

        self.regs[reg_x as usize] = self.regs[reg_x as usize] << 1;
    }

    fn skip_if_content_diff(&mut self, reg_x: u8, reg_y: u8) {
        if self.regs[reg_x as usize] != self.regs[reg_y as usize] {
            self.pc += 2;
        }
    }

    fn set_index(&mut self, value: u16) {
        self.i = value;
    }

    fn jump_with_reg(&mut self, value: u16) {
        /* TODO : this should be made configurable, as some implementations interpret this like a "BXNN" */
        self.pc = value + (self.regs[0x00 as usize] as u16);
    }

    fn random(&mut self, reg: u8, value: u8) {
        let mut rng = rand::thread_rng();
        let random: u8 = rng.gen();
        self.regs[reg as usize] = random & value;
    }

    /* TODO : this should be moved to another entity */
    fn draw_sprite(&mut self, x: u8, y: u8, n: u8) {
        /* Get X and Y coordinates from the registers */
        let x = self.regs[x as usize] % (DISPLAY_WIDTH as u8);
        let y = self.regs[y as usize] % (DISPLAY_HEIGHT as u8);

        /* Set VF to 0 as default; it will be set to 1 if any pixel is turned off */
        self.regs[0x0F as usize] = 0;

        for sprite_row in 0..n {
            let y_pos = (y + sprite_row) as usize;
            if y_pos < DISPLAY_HEIGHT {
                let sprite_data = self.memory[(self.i + (sprite_row as u16)) as usize];

                for sprite_bit_i in 0..8 {
                    let x_pos = (x + sprite_bit_i) as usize;
                    if x_pos < DISPLAY_WIDTH {
                        /* The bits must be read from MIB to LIB */
                        let bit_index = 7 - sprite_bit_i;
                        let bit_value = (sprite_data & (0x1 << bit_index)) >> bit_index;
                        let led_status = self.display.lock().unwrap().is_on(x_pos, y_pos);

                        /* If current pixel is on and bit is high, flip the led */
                        if (bit_value != 0) && led_status {
                            self.display.lock().unwrap().led_off(x_pos, y_pos);

                            /* Set VF to 1 since a led has been changed */
                            self.regs[0x0F as usize] = 1;
                        } else if (bit_value != 0) && !led_status {
                            self.display.lock().unwrap().led_on(x_pos, y_pos);
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

    fn skip_if_pressed(&mut self, reg: u8) {
        let key = self.regs[reg as usize];
        if self.keypad.lock().unwrap().get_is_pressed(key as usize) {
            self.pc += 2;
        }
    }

    fn skip_if_not_pressed(&mut self, reg: u8) {
        let key = self.regs[reg as usize];
        if !self.keypad.lock().unwrap().get_is_pressed(key as usize) {
            self.pc += 2;
        }
    }

    fn copy_delay_timer(&mut self, reg: u8) {
        let timer = self.delay_timer.lock().unwrap();
        self.regs[reg as usize] = *timer;
    }

    fn wait_for_key(&mut self, reg: u8) {
        let keypad = self.keypad.lock().unwrap();
        let mut key = 0;

        while key < 0x10 {
            if keypad.get_is_pressed(key as usize) {
                self.regs[reg as usize] = key;
                break;
            } else {
                key += 1;
            }
        }

        /* If not key was pressed, decrement the PC so that this instruction is executed again */
        if key >= 0x10 {
            self.pc -= 2;
        }
    }

    fn set_delay_timer(&mut self, reg: u8) {
        let mut timer = self.delay_timer.lock().unwrap();
        *timer = self.regs[reg as usize];
    }

    fn set_sound_timer(&mut self, reg: u8) {
        let mut timer = self.sound_timer.lock().unwrap();
        *timer = self.regs[reg as usize];
    }

    fn add_to_index(&mut self, reg: u8) {
        let reg_value = self.regs[reg as usize];

        let mut temp_add = self.i as u32;
        temp_add += reg_value as u32;

        /* The original interpreter doesn't seem to need the overflow check and flag register set, but it seems
         * the Amiga interpreter for CHIP-8 did, so let's check it here */
        if temp_add > 0xFFFF {
            self.regs[0x0F as usize] = 1;
        } else {
            self.regs[0x0F as usize] = 0;
        }

        self.i = (temp_add & 0x0000FFFF) as u16;
    }

    fn set_index_to_font(&mut self, reg: u8) {
        let reg_value = self.regs[reg as usize];
        self.i = FONT_START + ((FONT_SIZE as u16) * (reg_value as u16));
    }

    fn binary_conversion(&mut self, reg: u8) {
        let mut reg_value = self.regs[reg as usize];

        /* For example, if the value was "156" ->
         * memory[i] = 1
         * memory[i + 1] = 5
         * memory[i + 2] = 6
        */
        self.memory[(self.i + 2) as usize] = reg_value % 10;
        reg_value /= 10;
        self.memory[(self.i + 1) as usize] = reg_value % 10;
        reg_value /= 10;
        self.memory[(self.i + 0) as usize] = reg_value % 10;
    }

    fn store(&mut self, reg_max: u8) {
        /* TODO : this should be made configurable as the original CHIP-8 interpreter incremented the I register
         * while executing the instruction; more moderns ROMs do not expect this */
        /* The range uses reg_max + 1 since reg_max must be included */
        for reg_i in 0..(reg_max + 1) {
            self.memory[(self.i + (reg_i as u16)) as usize] = self.regs[reg_i as usize];
        }
    }

    fn load(&mut self, reg_max: u8) {
        /* TODO : this should be made configurable as the original CHIP-8 interpreter incremented the I register
         * while executing the instruction; more moderns ROMs do not expect this */
        /* The range uses reg_max + 1 since reg_max must be included */
        for reg_i in 0..(reg_max + 1) {
            self.regs[reg_i as usize] = self.memory[(self.i + (reg_i as u16)) as usize];
        }
    }

    fn execute(&mut self, instr: Instruction) {
        match instr {
            Instruction::ClearScreen => self.clear_screen(),
            Instruction::Return => self.return_subroutine(),
            Instruction::Jump(i) => self.jump(i),
            Instruction::Call(i) => self.call_subroutine(i),
            Instruction::SkipIfEqual(r, v) => self.skip_if_eq(r, v),
            Instruction::SkipIfDifferent(r, v) => self.skip_if_diff(r, v),
            Instruction::SkipIfContentEqual(x, y) => self.skip_if_content_eq(x, y),
            Instruction::SetRegister(r, v) => self.set_register(r, v),
            Instruction::AddToRegister(r, v) => self.add_to_reg(r, v),
            Instruction::LogicalCopy(x, y) => self.logical_copy(x, y),
            Instruction::LogicalOr(x, y) => self.logical_or(x, y),
            Instruction::LogicalAnd(x, y) => self.logical_and(x, y),
            Instruction::LogicalXor(x, y) => self.logical_xor(x, y),
            Instruction::LogicalAdd(x, y) => self.logical_add(x, y),
            Instruction::LogicalSubtract(x, y) => self.logical_sub(x, y),
            Instruction::LogicalRightShift(x) => self.logical_right_shift(x),
            Instruction::LogicalSubtractInverse(x, y) => self.logical_sub_inv(x, y),
            Instruction::LogicalLeftShift(x) => self.logical_left_shift(x),
            Instruction::SkipIfContentDifferent(x,y) => self.skip_if_content_diff(x, y),
            Instruction::SetIndex(v) => self.set_index(v),
            Instruction::JumpWithRegister(i) => self.jump_with_reg(i),
            Instruction::Random(r, v) => self.random(r, v),
            Instruction::Display(x, y, n) => self.draw_sprite(x, y, n),
            Instruction::SkipIfPressed(r) => self.skip_if_pressed(r),
            Instruction::SkipIfNotPressed(r) => self.skip_if_not_pressed(r),
            Instruction::CopyDelayTimer(r) => self.copy_delay_timer(r),
            Instruction::WaitForKey(r) => self.wait_for_key(r),
            Instruction::SetDelayTimer(r) => self.set_delay_timer(r),
            Instruction::SetSoundTimer(r) => self.set_sound_timer(r),
            Instruction::AddToIndex(r) => self.add_to_index(r),
            Instruction::SetIndexToFont(r) => self.set_index_to_font(r),
            Instruction::BinaryConversion(r) => self.binary_conversion(r),
            Instruction::Store(v) => self.store(v),
            Instruction::Load(v) => self.load(v),
        }
    }

    fn start_timers(&self) {
        let delay_timer = self.delay_timer.clone();

        thread::spawn(move || {
            loop {
                /* Cycle at ~60Hz */
                let millis = time::Duration::from_millis(16);
                thread::sleep(millis);
                let mut timer = delay_timer.lock().unwrap();
                if *timer > 0 {
                    *timer -= 1;
                }
            }
        });

        let sound_timer = self.sound_timer.clone();

        thread::spawn(move || {
            /* Create the stream handle here so that it doesn't go out of scope after playing a sound */
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            /* Save the value with which the timer was loaded; play a tune only when is loaded with a higher value */
            let mut playing_timer = 0;

            loop {
                /* Cycle at ~60Hz */
                let millis = time::Duration::from_millis(16);
                thread::sleep(millis);
                let mut timer = sound_timer.lock().unwrap();

                if *timer > 0 && *timer > playing_timer {
                    playing_timer = *timer;
                    let source = SineWave::new(440)
                        .take_duration(time::Duration::from_millis((playing_timer as u64) * 16))
                        .amplify(1.0);
                    stream_handle.play_raw(source).unwrap();
                }

                if *timer > 0 {
                    *timer -= 1;
                } else if *timer == 0 {
                    playing_timer = 0;
                }
            }
        });
    }

    pub fn run(&mut self, rom_path: &str) {
        self.load_fonts();
        self.load_rom(rom_path);
        self.start_timers();

        self.pc = ROM_START;

        loop {
            let instr = Instruction::from(self.fetch());
            self.execute(instr);

            /* TODO : timing can be implemented better; but supposing that the fetch/execution times
             * are negligible, a 1429us sleep will make the emulator execute ~700 instruction per seconds,
             * which seems like a speed which fits well enough for most games */
            let millis = time::Duration::from_micros(1429);
            thread::sleep(millis);
        }
    }
}
