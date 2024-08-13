use crate::parser::ast::{self, DataType};
use crate::xxh3_hasher::Xxh3HasherBuilder;
use clap::Parser;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, PartialEq)]
struct DbMetadata {
    table_pointers: HashMap<String, String, Xxh3HasherBuilder>,
    table_schemas: HashMap<String, TableSchema, Xxh3HasherBuilder>,
}

#[derive(Debug, PartialEq)]
struct TableSchema {
    row_definitions: HashMap<String, ast::DataType, Xxh3HasherBuilder>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct DbMetadataJson {
    table_pointers: Vec<TablePointerJson>,
    table_schemas: Vec<TableSchemaJson>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TablePointerJson {
    table_name: String,
    file_path: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TableSchemaJson {
    table_name: String,
    definitions: Vec<CollumnDefintionJson>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CollumnDefintionJson {
    collumn_name: String,
    data_type: String,
}

impl DbMetadata {
    pub fn create(dir_path: &Path) -> Result<Self, String> {
        let file_path = dir_path.join("db_metadata.ss");
        if let Err(_) = create_dir_all(dir_path) {
            return Err("Cant create db files directory".to_string());
        }
        if file_path.exists() {
            let file =
                File::open(&file_path).map_err(|e| format!("Cant open db metadata file {}", e))?;
            let json = serde_json::from_reader(file)
                .map_err(|e| format!("Error parsing db metadata file, {}", e))?;
            DbMetadata::from(json)
        } else {
            let mut file = File::create(&file_path)
                .map_err(|e| format!("Cant create db metadata file: {}", e))?;
            let empty_metadata_json = DbMetadataJson {
                table_pointers: vec![],
                table_schemas: vec![],
            };
            let json_string = serde_json::to_string(&empty_metadata_json)
                .map_err(|e| format!("Failed to serialize DbMetadataJson: {}", e))?;
            file.write_all(json_string.as_bytes())
                .map_err(|e| format!("Failed to write to file: {}", e))?;

            Ok(DbMetadata {
                table_pointers: HashMap::with_hasher(Xxh3HasherBuilder),
                table_schemas: HashMap::with_hasher(Xxh3HasherBuilder),
            })
        }
    }

    fn from(json: DbMetadataJson) -> Result<Self, String> {
        let mut table_pointers = HashMap::with_hasher(Xxh3HasherBuilder);
        for pointer in json.table_pointers {
            table_pointers.insert(pointer.table_name, pointer.file_path);
        }
        let mut table_schemas = HashMap::with_hasher(Xxh3HasherBuilder);
        for schema in json.table_schemas {
            let mut column_definition = HashMap::with_hasher(Xxh3HasherBuilder);
            for definition in schema.definitions {
                let data_type = match definition.data_type.as_str() {
                    dt if dt == crate::parser::token::DataType::Integer32.value() => {
                        Ok(DataType::Int32)
                    }
                    dt if dt == crate::parser::token::DataType::Varchar256.value() => {
                        Ok(DataType::VarChar256)
                    }
                    _ => Err(format!(
                        "Error maping json to db metadata, unknown data type: {}",
                        definition.data_type
                    )),
                }?;
                column_definition.insert(definition.collumn_name, data_type);
            }
            table_schemas.insert(
                schema.table_name,
                TableSchema {
                    row_definitions: column_definition,
                },
            );
        }
        Ok(DbMetadata {
            table_pointers,
            table_schemas,
        })
    }

    fn save(&self, dir_path: &Path) -> Result<(), String> {
        let db_metadata_json = DbMetadataJson::from(self);
        let json_data =
            serde_json::to_string_pretty(&db_metadata_json).expect("Failed to serialize to JSON");
        let full_path = Path::new(dir_path).join("db_metadata.json");
        let mut file = File::create(&full_path).map_err(|e| format!("Cant open file, {}", e))?;
        file.write_all(json_data.as_bytes())
            .map_err(|e| format!("Failed writing to file {}", e))
    }
}

impl DbMetadataJson {
    fn from(metadata: &DbMetadata) -> Self {
        let table_pointers = metadata
            .table_pointers
            .iter()
            .map(|(table_name, file_path)| TablePointerJson {
                table_name: table_name.clone(),
                file_path: file_path.clone(),
            })
            .collect();

        let table_schemas = metadata
            .table_schemas
            .iter()
            .map(|(table_name, table_schema)| {
                let definitions: Vec<CollumnDefintionJson> = table_schema
                    .row_definitions
                    .iter()
                    .map(|(collumn_name, data_type)| {
                        let data_type = match data_type {
                            ast::DataType::Int32 => crate::parser::token::DataType::Integer32
                                .value()
                                .to_string(),
                            ast::DataType::VarChar256 => crate::parser::token::DataType::Varchar256
                                .value()
                                .to_string(),
                        };
                        CollumnDefintionJson {
                            collumn_name: collumn_name.clone(),
                            data_type,
                        }
                    })
                    .collect();
                TableSchemaJson {
                    table_name: table_name.clone(),
                    definitions,
                }
            })
            .collect();
        DbMetadataJson {
            table_pointers,
            table_schemas,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::DataType;
    use serde_json::to_writer_pretty;
    use tempfile::TempDir;
    #[test]
    fn when_calling_create_and_there_is_no_metadata_file_create_file_and_return_empty_metadata() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("db_metadata.ss");
        let empty_metadata_json = DbMetadataJson {
            table_pointers: vec![],
            table_schemas: vec![],
        };

        let metadata = DbMetadata::create(temp_dir.path());

        assert_eq!(
            metadata,
            Ok(DbMetadata {
                table_pointers: HashMap::with_hasher(Xxh3HasherBuilder),
                table_schemas: HashMap::with_hasher(Xxh3HasherBuilder),
            })
        );
        assert!(
            file_path.exists(),
            "The db_metadata.ss file should have been created"
        );
        assert_eq!(
            std::fs::read_to_string(&file_path).expect("Failed to read the file"),
            serde_json::to_string(&empty_metadata_json).unwrap(),
            "The db_metadata.ss file content is incorrect"
        );
    }

    #[test]
    fn when_calling_create_and_there_is_metadata_file_return_valid_metadata() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("db_metadata.ss");
        let metadata_json = DbMetadataJson {
            table_pointers: vec![TablePointerJson {
                table_name: "user".to_string(),
                file_path: "user.ss".to_string(),
            }],
            table_schemas: vec![TableSchemaJson {
                table_name: "user".to_string(),
                definitions: vec![
                    CollumnDefintionJson {
                        collumn_name: "id".to_string(),
                        data_type: crate::parser::token::DataType::Integer32
                            .value()
                            .to_string(),
                    },
                    CollumnDefintionJson {
                        collumn_name: "name".to_string(),
                        data_type: crate::parser::token::DataType::Varchar256
                            .value()
                            .to_string(),
                    },
                ],
            }],
        };
        let file = File::create(&file_path).unwrap();
        to_writer_pretty(file, &metadata_json).expect("Unable to write data to file");
        let mut expected_table_pointers = HashMap::with_hasher(Xxh3HasherBuilder);
        expected_table_pointers.insert("user".to_string(), "user.ss".to_string());
        let mut expected_row_definitions = HashMap::with_hasher(Xxh3HasherBuilder);
        expected_row_definitions.insert("id".to_string(), DataType::Int32);
        expected_row_definitions.insert("name".to_string(), DataType::VarChar256);
        let mut expected_table_schemas = HashMap::with_hasher(Xxh3HasherBuilder);
        expected_table_schemas.insert(
            "user".to_string(),
            TableSchema {
                row_definitions: expected_row_definitions,
            },
        );

        let result = DbMetadata::create(temp_dir.path());

        assert_eq!(
            result,
            Ok(DbMetadata {
                table_pointers: expected_table_pointers,
                table_schemas: expected_table_schemas,
            })
        );
    }

    #[test]
    fn when_db_metadada_is_given_map_it_to_valid_db_metadada_json() {
        let mut table_pointers = HashMap::with_hasher(Xxh3HasherBuilder);
        table_pointers.insert("users".to_string(), "user.ss".to_string());
        let mut row_definitions = HashMap::with_hasher(Xxh3HasherBuilder);
        row_definitions.insert("id".to_string(), DataType::Int32);
        row_definitions.insert("name".to_string(), DataType::VarChar256);
        let mut table_schemas = HashMap::with_hasher(Xxh3HasherBuilder);
        table_schemas.insert("users".to_string(), TableSchema { row_definitions });
        let expected_table_pointers = vec![TablePointerJson {
            table_name: "users".to_string(),
            file_path: "user.ss".to_string(),
        }];
        let expected_table_schemas = vec![TableSchemaJson {
            table_name: "users".to_string(),
            definitions: vec![
                CollumnDefintionJson {
                    collumn_name: "id".to_string(),
                    data_type: crate::parser::token::DataType::Integer32
                        .value()
                        .to_string(),
                },
                CollumnDefintionJson {
                    collumn_name: "name".to_string(),
                    data_type: crate::parser::token::DataType::Varchar256
                        .value()
                        .to_string(),
                },
            ],
        }];
        let expected_db_metadata_json = DbMetadataJson {
            table_pointers: expected_table_pointers,
            table_schemas: expected_table_schemas,
        };
        let db_metadata = DbMetadata {
            table_pointers,
            table_schemas,
        };

        let db_metadata_json = DbMetadataJson::from(&db_metadata);

        assert_eq!(db_metadata_json, expected_db_metadata_json);
    }
}
