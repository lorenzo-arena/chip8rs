# CHIP-8 RS

A [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) emulator written in Rust.

Some ROMs can be found on the internet, for example [here](https://github.com/kripod/chip8-roms) or [here](https://johnearnest.github.io/chip8Archive/).

## Build and run

To build and test the projects, run:

```bash
$ cargo build
$ cargo run -- --rom /path/to/rom.ch8
```

For more informations about available options, run:

```bash
$ cargo run -- --help
```

Current implementation uses [piston](https://www.piston.rs/) engine for drawing and input events.
