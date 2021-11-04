pub enum Instruction {
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