use ncurses::*;
use std::thread;
use std::time::Duration;

pub trait Display {
    fn open(&self);
    fn close(&self);
    fn refresh(&self);
    fn led_on(& mut self, x: u8, y: u8);
    fn led_off(& mut self, x: u8, y: u8);
}

pub struct NCursesDisplay {
    x_len: u8,
    y_len: u8,
    display_string: String
}

impl NCursesDisplay {
    pub fn new(x_len: u8, y_len: u8) -> NCursesDisplay {
        let mut display_string = String::new();
        let mut y = 0;

        while y < y_len {
            let mut x = 0;
            while x < x_len {
                display_string.push_str("#");
                x += 1;
            }
    
            display_string.push_str("\n");
            y += 1;
        }

        NCursesDisplay {
            x_len: x_len,
            y_len: y_len,
            display_string: display_string
        }
    }

    fn get_pos(&self, x: u8, y: u8) -> Result<usize, &str> {
        /* TODO : manage pos outsize limits? */
        if x >= self.x_len {
            Err("X coordinate is too large")
        } else if y >= self.y_len {
            Err("Y coordinate is too large")
        } else {
            /* Here 1 is added for the "\n" */
            Ok(x as usize + (y as usize * ((self.x_len as usize) + 1)))
        }
    }
}

impl Display for NCursesDisplay {
    fn open(&self) {
        /* Start ncurses. */
        initscr();

        /* cbreak and noecho can be used to hide the user input */
        cbreak();
        noecho();

        self.refresh();
    }

    fn refresh(&self) {
        clear();
        addstr(&self.display_string);
        refresh();
    }

    fn close(&self) {
        /* Terminate ncurses. */
        endwin();
    }

    fn led_on(& mut self, x: u8, y: u8) {
        /* TODO : better result management? */
        let pos = self.get_pos(x, y).unwrap();
        self.display_string.replace_range(pos..(pos + 1), "#");
    }

    fn led_off(& mut self, x: u8, y: u8) {
        /* TODO : better result management? */
        let pos = self.get_pos(x, y).unwrap();
        self.display_string.replace_range(pos..(pos + 1), " ");
    }
}