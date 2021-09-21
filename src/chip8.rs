#[derive(Debug)]
pub struct Chip8 {
    test: String
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            test: "Test".to_string()
        }
    }
}