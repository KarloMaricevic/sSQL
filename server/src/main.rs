mod command_validator;
mod config;
mod parser;
mod string_helpers;
mod constants;
mod page;
mod information_schema;
mod bptree;
mod catalog;
mod executor;
mod buff;
pub mod new_page;
use crate::config::Config;
use parser::ast::{ColumnDefinition, SqlStatement};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> Result<(), String> {
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
                let received_data = &buffer[..bytes_read];
                let command = match String::from_utf8(received_data.to_vec()) {
                    Ok(s) => s,
                    Err(_) => {
                        continue;
                    }
                };
                let sql_statemant = match parser::parse(command) {
                    Ok(statemant) => statemant,
                    Err(_) => {
                        continue;
                    }
                };
                match sql_statemant {
                    SqlStatement::CreateTable {
                        table_name,
                        primary_key,
                        columns,
                    } => {}
                    SqlStatement::Select { 
                        columns, 
                        table 
                    } => {
                        // pogledaj koji od columnsa ima index
                        // nadi OID 
                    }
                    _other => {
                        continue;
                    }
                }
            }
            Err(e) => eprintln!("Failed to read from stream: {}", e),
        }
    }
}

fn create_table(
    table_name: String,
    primary_key: String,
    columns: Vec<ColumnDefinition>,
) -> Result<(), String> {
    if table_name.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }
    if primary_key.is_empty() {
        return Err("Primary key cannot be empty".into());
    }
    if columns.is_empty() {
        return Err("At least one column must be provided".into());
    }

    Err("STUB".to_string())
}

struct FileMetadata {
    table_name: String,
    primary_key: String,
    columns: Vec<ColumnDefinition>,
    page_offset: Vec<i32>,
}
