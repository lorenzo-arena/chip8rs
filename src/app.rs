extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

use std::panic;
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::chip8::Chip8;
use crate::display::*;
use crate::hsl::*;
use crate::keypad::*;

const WINDOW_WIDTH: usize = 640;
const WINDOW_HEIGHT: usize = 320;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

const LED_WIDTH: f64 = 10.0;

const KEYPAD_SIZE: usize = 0x10;

pub struct App {
    display: Arc<Mutex<LedsDisplay>>,
    keypad: Arc<Mutex<KeyboardKeypad>>,
    window: glutin_window::GlutinWindow,
    gl: GlGraphics,
    color: RGBPixel,
    background: [f32; 4],
    nyan_mode: bool,
}

impl App {
    pub fn new(nyan_mode: bool) -> App {
        let opengl = OpenGL::V3_2;

        let mut starting_color = RGBPixel {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };

        if nyan_mode {
            starting_color = RGBPixel {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            };
        }

        App {
            display: Arc::new(Mutex::new(LedsDisplay::new(
                DISPLAY_WIDTH,
                DISPLAY_HEIGHT,
                false,
            ))),
            keypad: Arc::new(Mutex::new(KeyboardKeypad::new(KEYPAD_SIZE))),
            window: WindowSettings::new("CHIP-8 RS", [WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32])
                .opengl(opengl)
                .exit_on_esc(true)
                .build()
                .unwrap(),
            gl: GlGraphics::new(opengl),
            color: starting_color,
            background: [1.0, 1.0, 1.0, 1.0],
            nyan_mode: nyan_mode,
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;
        let background = self.background;

        if self.nyan_mode {
            let mut hsl = rgb_to_hsl(&self.color);

            if hsl.h >= 360 {
                hsl.h = 1;
            } else {
                hsl.h += 1;
            }

            self.color = hsl_to_rgb(&hsl);
        }

        let color = self.color;
        let display = self.display.clone();

        self.gl.draw(args.viewport(), |c, gl| {
            /* Clear the screen. */
            clear(background, gl);

            for y in 0..DISPLAY_HEIGHT {
                for x in 0..DISPLAY_WIDTH {
                    if display.lock().unwrap().is_on(x, y) {
                        let square = rectangle::square(
                            (x as f64) * LED_WIDTH,
                            (y as f64) * LED_WIDTH,
                            LED_WIDTH,
                        );

                        /* TODO : empty transformation; is there a way to skip this? */
                        let transform = c.transform.trans(0.0, 0.0);
                        rectangle([color.r, color.g, color.b, 1.0], square, transform, gl);
                    }
                }
            }
        });
    }

    pub fn run(&mut self, rom_path: String) {
        /* Set a hook on panic so that panics on the CHIP-8 thread cause the program to exit */
        let orig_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            orig_hook(panic_info);
            process::exit(1);
        }));

        let mut events = Events::new(EventSettings::new());

        let display = self.display.clone();
        let keypad = self.keypad.clone();

        thread::spawn(move || {
            let mut chip = Chip8::new(&display, &keypad);
            chip.run(&rom_path);
        });

        while let Some(e) = events.next(&mut self.window) {
            if let Some(r) = e.render_args() {
                self.render(&r);
            }

            if let Some(Button::Keyboard(key)) = e.press_args() {
                match key {
                    /* TODO : add ASCII art for keypad */
                    /* TODO : move this logic in the keypad struct */
                    Key::D1 => self.keypad.lock().unwrap().set_is_pressed(0x01, true),
                    Key::D2 => self.keypad.lock().unwrap().set_is_pressed(0x02, true),
                    Key::D3 => self.keypad.lock().unwrap().set_is_pressed(0x03, true),
                    Key::D4 => self.keypad.lock().unwrap().set_is_pressed(0x0C, true),
                    Key::Q => self.keypad.lock().unwrap().set_is_pressed(0x04, true),
                    Key::W => self.keypad.lock().unwrap().set_is_pressed(0x05, true),
                    Key::E => self.keypad.lock().unwrap().set_is_pressed(0x06, true),
                    Key::R => self.keypad.lock().unwrap().set_is_pressed(0x0D, true),
                    Key::A => self.keypad.lock().unwrap().set_is_pressed(0x07, true),
                    Key::S => self.keypad.lock().unwrap().set_is_pressed(0x08, true),
                    Key::D => self.keypad.lock().unwrap().set_is_pressed(0x09, true),
                    Key::F => self.keypad.lock().unwrap().set_is_pressed(0x0E, true),
                    Key::Z => self.keypad.lock().unwrap().set_is_pressed(0x0A, true),
                    Key::X => self.keypad.lock().unwrap().set_is_pressed(0x00, true),
                    Key::C => self.keypad.lock().unwrap().set_is_pressed(0x0B, true),
                    Key::V => self.keypad.lock().unwrap().set_is_pressed(0x0F, true),
                    _ => {}
                }
            }

            if let Some(Button::Keyboard(key)) = e.release_args() {
                match key {
                    /* TODO : add ASCII art for keypad */
                    /* TODO : move this logic in the keypad struct */
                    Key::D1 => self.keypad.lock().unwrap().set_is_pressed(0x01, false),
                    Key::D2 => self.keypad.lock().unwrap().set_is_pressed(0x02, false),
                    Key::D3 => self.keypad.lock().unwrap().set_is_pressed(0x03, false),
                    Key::D4 => self.keypad.lock().unwrap().set_is_pressed(0x0C, false),
                    Key::Q => self.keypad.lock().unwrap().set_is_pressed(0x04, false),
                    Key::W => self.keypad.lock().unwrap().set_is_pressed(0x05, false),
                    Key::E => self.keypad.lock().unwrap().set_is_pressed(0x06, false),
                    Key::R => self.keypad.lock().unwrap().set_is_pressed(0x0D, false),
                    Key::A => self.keypad.lock().unwrap().set_is_pressed(0x07, false),
                    Key::S => self.keypad.lock().unwrap().set_is_pressed(0x08, false),
                    Key::D => self.keypad.lock().unwrap().set_is_pressed(0x09, false),
                    Key::F => self.keypad.lock().unwrap().set_is_pressed(0x0E, false),
                    Key::Z => self.keypad.lock().unwrap().set_is_pressed(0x0A, false),
                    Key::X => self.keypad.lock().unwrap().set_is_pressed(0x00, false),
                    Key::C => self.keypad.lock().unwrap().set_is_pressed(0x0B, false),
                    Key::V => self.keypad.lock().unwrap().set_is_pressed(0x0F, false),
                    _ => {}
                }
            }
        }
    }
}
