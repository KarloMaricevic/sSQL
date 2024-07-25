mod command_file_reader;
mod config;
mod s_sql_server;
mod std_in_utils;

use crate::config::Config;
use crate::std_in_utils::read_line;
use s_sql_server::SsqlServer;
use std::io::{self, Write};

fn main() {
    let config = Config::build().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let server_address = format!("{}:{}", config.host, config.port);
    let server = SsqlServer::new(&server_address).unwrap_or_else(|e| {
        eprintln!("Failed to connect to the server: {e}");
        std::process::exit(1);
    });
    loop {
        print!("sSQL> ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("Failed to flush stdout: {e}");
            std::process::exit(1);
        }
        match read_line() {
            Ok(input) => {
                if input == "quit" {
                    break;
                } else if input.starts_with("source") {
                    if let Some(file_path) = extract_file_path_from_command(&input) {
                        if let Err(e) = server.send_commands_from_file(&file_path) {
                            eprintln!("Error while sending command {e}");
                        }
                    } else {
                        eprintln!("Error while extracting file path from command");
                    }
                } else if !input.is_empty() {
                    if let Err(e) = server.send_command(&input) {
                        eprintln!("Error while sendong command from file {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input {e}");
            }
        }
    }
    println!("Closing sSQL CLI");
}

fn extract_file_path_from_command(command: &str) -> Option<String> {
    command.strip_prefix("source ").map(|path| path.to_string())
}
