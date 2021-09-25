use std::fs::File;
use std::fs::OpenOptions;
use std::fs;
use std::io::prelude::*;

pub trait Logger {
    fn log(&self, message: String);
}

pub struct FileLogger {
    file_path: String
}

impl FileLogger {
    pub fn new(file_path: String) -> FileLogger {
        /* Try to remove the file, ignore the error in case */
        let _ = fs::remove_file(&file_path);

        FileLogger {
            file_path: file_path
        }
    }
}

impl Logger for FileLogger {
    fn log(&self, message: String) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.file_path)
            .unwrap();
        file.write_all(message.as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
    }
}