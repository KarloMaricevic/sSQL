mod config;
use crate::config::Config;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> std::io::Result<()> {
    let config = Config::build().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let listeniing_address = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(listeniing_address).unwrap_or_else(|e| {
        eprintln!("Couldt start listening for tcp connection, {e}");
        std::process::exit(1);
    });
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                println!("Received data: {:?}", &buffer[..bytes_read]);
                if let Err(e) = stream.write_all(&buffer[..bytes_read]) {
                    eprintln!("Failed to write to stream: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to read from stream: {}", e),
        }
    }
}
