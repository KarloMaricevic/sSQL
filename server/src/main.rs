mod b_plus_tree;
mod command_validator;
mod config;
mod db_metadata;
mod parser;
mod string_helpers;
mod xxh3_hasher;
mod buf;
use crate::config::Config;
use command_validator::validate_command;
use parser::ast::{ColumnDefinition, SqlStatement};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
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

    let metadata = db_metadata::DbMetadata::create(Path::new("db_files")).map_err(|e| {
        eprintln!("Failed to initialize database metadata: {}", e);
        e.to_string()
    })?;
    let thread_safe_metadata = Arc::new(Mutex::new(metadata));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let cloned_metadata = Arc::clone(&thread_safe_metadata);
                thread::spawn(move || handle_client(stream, cloned_metadata));
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, metadata: Arc<Mutex<db_metadata::DbMetadata>>) {
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
                let locked_metadata = metadata.lock().unwrap();
                if let Err(_) = validate_command(&sql_statemant, &locked_metadata) {
                    continue;
                }
                match sql_statemant {
                    SqlStatement::CreateTable {
                        table_name,
                        primary_key,
                        columns,
                    } => {}

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
