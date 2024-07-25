use std::{
    cell::RefCell,
    io::{Result, Write},
    net::TcpStream,
};

use crate::command_file_reader::CommandFileReader;

pub struct SsqlServer {
    stream: RefCell<TcpStream>,
}

impl SsqlServer {
    pub fn new(server_address: &str) -> Result<Self> {
        let stream = TcpStream::connect(server_address)?;
        Ok(Self {
            stream: RefCell::new(stream),
        })
    }

    pub fn send_command(&self, command: &str) -> Result<()> {
        let mut stream = self.stream.borrow_mut();
        stream.write_all(command.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    // this could go to the main function but for simplicity leave it here
    pub fn send_commands_from_file(&self, file_path: &str) -> Result<()> {
        let mut command_reader = CommandFileReader::new(file_path)?;
        loop {
            let command = command_reader.read_next_command()?;
            if command.is_empty() {
                break;
            } else {
                self.send_command(&command)?;
            }
        }
        Ok(())
    }
}
