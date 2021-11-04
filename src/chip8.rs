use crate::display::*;
use crate::fonts::Fonts;
use crate::fonts::FONT_SIZE;
use crate::keypad::*;
use crate::logger::FileLogger;
use crate::logger::Logger;
use crate::instruction::Instruction;
use crate::timer::{Timer, DelayTimer, SoundTimer};

use rand::Rng;
use std::sync::{Arc, Mutex};
use std::{fs, thread, time};

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
    delay_timer: DelayTimer,
    sound_timer: SoundTimer,
    regs: [u8; REGISTERS_SIZE],
    fonts: Fonts,
    logger: FileLogger,
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
            delay_timer: DelayTimer::new(),
            sound_timer: SoundTimer::new(),
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
        self.regs[reg as usize] = self.delay_timer.get_timer_value();
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
        self.delay_timer.set_timer_value(self.regs[reg as usize]);
    }

    fn set_sound_timer(&mut self, reg: u8) {
        self.sound_timer.set_timer_value(self.regs[reg as usize]);
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

    pub fn run(&mut self, rom_path: &str) {
        self.load_fonts();
        self.load_rom(rom_path);

        self.delay_timer.start(60.0);
        self.sound_timer.start(60.0);

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
