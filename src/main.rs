mod chip8;
mod display;
mod fonts;

use chip8::Chip8;

fn main() {
    let mut chip = Chip8::new();
    let rom_path = "particle_demo.ch8";

    chip.run(rom_path);
}
