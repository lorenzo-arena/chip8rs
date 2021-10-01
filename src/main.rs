mod chip8;
mod display;
mod fonts;
mod logger;
mod app;

use clap;

use app::*;

fn main() {
    let matches = clap::App::new("chip8rs")
                             .version("0.0.1")
                             .author("Lorenzo A.")
                             .about("CHIP-8 emulator written in Rust")
                             .arg(clap::Arg::with_name("rom")
                                  .short("r")
                                  .long("rom")
                                  .value_name("FILE")
                                  .help("Path to the CHIP-8 ROM file")
                                  .required(true)
                                  .takes_value(true))
                             .get_matches();

    let rom_path = matches.value_of("rom").unwrap();

    let mut app = App::new();
    app.run(rom_path.to_string());
}
