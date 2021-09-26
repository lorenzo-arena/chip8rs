mod chip8;
mod display;
mod fonts;
mod logger;

use chip8::Chip8;

fn main() {
    let mut chip = Chip8::new();
    let rom_path = "test_opcode.ch8";

    chip.run(rom_path);
}
