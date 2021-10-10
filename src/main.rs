mod app;
mod chip8;
mod display;
mod fonts;
mod hsl;
mod keypad;
mod logger;

use clap;

use app::*;

fn main() {
    let matches = clap::App::new("chip8rs")
        .version("0.0.1")
        .author("Lorenzo A.")
        .about("CHIP-8 emulator written in Rust")
        .arg(
            clap::Arg::with_name("rom")
                .short("r")
                .long("rom")
                .value_name("FILE")
                .help("Path to the CHIP-8 ROM file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("nyan")
                .long("nyan")
                .help("Enter \"Nyan Cat\" mode")
                .takes_value(false),
        )
        .get_matches();

    let rom_path = matches.value_of("rom").unwrap();
    let nyan_mode = matches.is_present("nyan");

    let mut app = App::new(nyan_mode);
    app.run(rom_path.to_string());
}
