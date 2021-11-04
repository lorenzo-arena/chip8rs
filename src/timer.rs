use std::sync::{Arc, Mutex};
use std::{thread, time};

use rodio::source::{SineWave, Source};
use rodio::OutputStream;

pub trait Timer<T> {
    fn get_timer_value(&mut self) -> T;
    fn set_timer_value(&mut self, value: T);
    fn start(&mut self, freq: f32);
}

pub struct DelayTimer {
    timer: Arc<Mutex<u8>>,
}

impl DelayTimer {
    pub fn new() -> DelayTimer {
        DelayTimer {
            timer: Arc::new(Mutex::new(0)),
        }
    }
}

impl Timer<u8> for DelayTimer {
    fn get_timer_value(&mut self) -> u8 {
        let timer = self.timer.lock().unwrap();
        *timer
    }

    fn set_timer_value(&mut self, value: u8) {
        let mut timer = self.timer.lock().unwrap();
        *timer = value;
    }

    fn start(&mut self, freq: f32) {
        let timer = self.timer.clone();

        thread::spawn(move || {
            loop {
                let period = time::Duration::from_secs_f32(1.0 / freq);
                thread::sleep(period);
                let mut timer = timer.lock().unwrap();
                if *timer > 0 {
                    *timer -= 1;
                }
            }
        });
    }
}

pub struct SoundTimer {
    timer: Arc<Mutex<u8>>,
}

impl SoundTimer {
    pub fn new() -> SoundTimer {
        SoundTimer {
            timer: Arc::new(Mutex::new(0)),
        }
    }
}

impl Timer<u8> for SoundTimer {
    fn get_timer_value(&mut self) -> u8 {
        let timer = self.timer.lock().unwrap();
        *timer
    }

    fn set_timer_value(&mut self, value: u8) {
        let mut timer = self.timer.lock().unwrap();
        *timer = value;
    }

    fn start(&mut self, freq: f32) {
        let timer = self.timer.clone();

        thread::spawn(move || {
            /* Create the stream handle here so that it doesn't go out of scope after playing a sound */
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            /* Save the value with which the timer was loaded; play a tune only when is loaded with a higher value */
            let mut playing_timer = 0;

            loop {
                let period = time::Duration::from_secs_f32(1.0 / freq);
                thread::sleep(period);
                let mut timer = timer.lock().unwrap();

                if *timer > 0 && *timer > playing_timer {
                    playing_timer = *timer;
                    let source = SineWave::new(440)
                        .take_duration(time::Duration::from_millis((playing_timer as u64) * 16))
                        .amplify(1.0);
                    stream_handle.play_raw(source).unwrap();
                }

                if *timer > 0 {
                    *timer -= 1;
                } else if *timer == 0 {
                    playing_timer = 0;
                }
            }
        });
    }
}

