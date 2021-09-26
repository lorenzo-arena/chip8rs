mod chip8;
mod display;
mod fonts;
mod logger;

use chip8::Chip8;
use clap::{Arg, App};

fn main() {
    let matches = App::new("chip8rs")
                          .version("0.0.1")
                          .author("Lorenzo A.")
                          .about("CHIP-8 emulator written in Rust")
                          .arg(Arg::with_name("rom")
                               .short("r")
                               .long("rom")
                               .value_name("FILE")
                               .help("Path to the CHIP-8 ROM file")
                               .required(true)
                               .takes_value(true))
                          .get_matches();

    let rom_path = matches.value_of("rom").unwrap();
    let mut chip = Chip8::new();
    chip.run(rom_path);
}
