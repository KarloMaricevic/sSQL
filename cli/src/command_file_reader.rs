use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

pub struct CommandFileReader {
    file: File,
    position: u64,
}

impl CommandFileReader {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self { file, position: 0 })
    }

    pub fn read_next_command(&mut self) -> io::Result<String> {
        let mut buffer = String::new();
        let mut reader = io::BufReader::new(&mut self.file);
        reader.seek(SeekFrom::Start(self.position))?;
        let mut c = [0];
        let mut found_semicolon = false;
        while reader.read_exact(&mut c).is_ok() {
            let ch = c[0] as char;
            buffer.push(ch);
            self.position += 1;
            if ch == ';' {
                found_semicolon = true;
                break;
            }
        }
        self.position = reader.stream_position()?;
        if !found_semicolon && buffer.is_empty() {
            return Ok(String::new());
        }
        Ok(buffer)
    }
}
