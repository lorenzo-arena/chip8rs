mod chip8;
mod display;

use chip8::Chip8;

fn main() {
    let mut chip = Chip8::new();

    chip.run();
}
