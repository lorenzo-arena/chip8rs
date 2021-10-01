extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::input::*;
use piston::event_loop::*;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::window::WindowSettings;

use std::thread;
use std::sync::{Arc, Mutex};

use crate::chip8::Chip8;
use crate::display::*;

const WINDOW_WIDTH: usize = 640;
const WINDOW_HEIGHT: usize = 320;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

const LED_WIDTH: f64 = 10.0;

pub struct App {
    display: Arc<Mutex<LedsDisplay>>,
    window: glutin_window::GlutinWindow,
    gl: GlGraphics,
    color: [f32; 4],
    background: [f32; 4],
}

impl App {
    pub fn new() -> App {
        let opengl = OpenGL::V3_2;

        App {
            display: Arc::new(Mutex::new(LedsDisplay::new(DISPLAY_WIDTH, DISPLAY_HEIGHT, false))),
            window: WindowSettings::new(
                "CHIP-8 RS",
                [WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32]
            )
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap(),
            gl: GlGraphics::new(opengl),
            color: [0.0, 0.0, 0.0, 1.0],
            background: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;
        let background = self.background;
        let color = self.color;
        let display = self.display.clone();

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(background, gl);

            for y in 0..DISPLAY_HEIGHT {
                for x in 0..DISPLAY_WIDTH {
                    if display.lock().unwrap().is_on(x, y) {
                        let square = rectangle::square((x as f64) * LED_WIDTH, (y as f64) * LED_WIDTH, LED_WIDTH);

                        /* TODO : empty transformation; is there a way to skip this? */
                        let transform = c.transform.trans(0.0, 0.0);
                        rectangle(color, square, transform, gl);
                    }
                }
            }
        });
    }

    pub fn run(&mut self, rom_path: String) {
        let mut events = Events::new(EventSettings::new());

        let display = self.display.clone();

        thread::spawn(move|| {
            let mut chip = Chip8::new(&display);
            chip.run(&rom_path);
        });

        while let Some(e) = events.next(&mut self.window) {
            if let Some(r) = e.render_args() {
                self.render(&r);
            }
        }
    }
}