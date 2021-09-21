mod chip8;

use chip8::Chip8;

fn main() {
    let chip = Chip8::new();

    println!("{:?}", chip);
}
