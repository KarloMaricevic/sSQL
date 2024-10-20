use std::{cmp::Ordering, fs::{self, File}, io::Read};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)] 
struct FileFormat {
    table_name: String,
    file_name: String,
    columns: Vec<ColumnDefinition>
}

#[derive(Debug)] 
struct ColumnDefinition {
    name: String,
    is_index: bool,
    data_type: ColumnType,
}

#[derive(Debug)] 
enum ColumnType {
    INT,
    STRING,
}

#[derive(Debug,PartialEq, PartialOrd, Eq)] 

pub enum SType {
    INT,
    STRING,
}
pub enum SData {
    INT(i32),
    STRING(String),
}

impl Ord for SData {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (SData::INT(a), SData::INT(b)) => a.cmp(b),
            (SData::STRING(a), SData::STRING(b)) => a.cmp(b),
            _ => panic!("Tried to compere 2 different types")
        }
    }
}

impl SData {
    pub fn serialized_size(&self) -> u32 {
        match self {
            SData::INT(_) => 2,
            SData::STRING(s) => 2 + s.len() as u32,
        }
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) {
        match self {
            SData::INT(value) => {
                buffer.extend(&value.to_le_bytes());
            },
            SData::STRING(value) => {
                let len = value.len() as u32;
                buffer.extend(&len.to_le_bytes());
                buffer.extend(value.as_bytes());
            },
        }
    }
}

const FILE_FORMAT_FILE_NAME: &str  = "ff.sl";

/* impl InformationSchema {
    pub fn create() -> Result<InformationSchema, String> {
        if !fs::metadata(FILE_FORMAT_FILE_NAME).is_ok() {
            File::create(FILE_FORMAT_FILE_NAME).map_err(|e| e.to_string())?;
        }
        let mut file = File::options()
            .read(true)
            .open(FILE_FORMAT_FILE_NAME)
            .map_err(|e: std::io::Error| {
                println!("{}", e.to_string());
                e.to_string()
            }
            )?;    
        let table_count = {
            let mut buff = [0u8; 2];
            file.read_exact(&mut buff).map_err(|e| e.to_string())?;
            LittleEndian::read_u16(&buff)
        };
        let mut formats : Vec<FileFormat> = Vec::with_capacity(table_count.into());
        for _ in 0..table_count {
            let table_name = {
                let size = {
                    let mut buffer= [0u8; 2];
                    file.read_exact(&mut buffer).map_err(|e| e.to_string())?;
                    LittleEndian::read_u16(&buffer)
                };
                let mut neme_buffer = vec![0u8; size.into()];
                file.read_exact(&mut neme_buffer).map_err(|e| e.to_string())?;
                String::from_utf8(neme_buffer)
            }.map_err(|_| "Error parsing table name")?;
            let file_name = {
                let size = {
                    let mut buffer = [0u8; 2];
                    file.read_exact(&mut buffer).map_err(|e| e.to_string())?;
                    LittleEndian::read_u16(&buffer)
                };
                let mut file_neme_buffer: Vec<u8> = vec![0u8; size.into()];
                file.read_exact(&mut file_neme_buffer).map_err(|e| e.to_string())?;
                String::from_utf8(file_neme_buffer)
            }.map_err(|_| "Error parsing table file name")?;
            let number_of_collumns = {
                let mut number_of_collumns_buffer = [0u8; 2];
                file.read_exact(&mut number_of_collumns_buffer).map_err(|e| e.to_string())?;
                LittleEndian::read_u16(&number_of_collumns_buffer)
            };
            let columns = { 
                let mut columns = Vec::with_capacity(number_of_collumns.into());
                for _ in 0..number_of_collumns {
                    let collumn_name = {
                        let size = {
                            let mut buffer = [0u8; 2];
                            file.read_exact(&mut buffer).map_err(|e| e.to_string())?;
                            LittleEndian::read_u16(&buffer)
                        };
                        let mut collumn_name_buffer: Vec<u8> = vec![0u8; size.into()];
                        file.read_exact(&mut collumn_name_buffer).map_err(|e| e.to_string())?;
                        String::from_utf8(collumn_name_buffer)
                    }.map_err(|_| "Error parsing collumn name")?;
                    let column_type = {
                        let mut buff =  [0u8; 1];
                        file.read_exact(&mut buff).map_err(|e| e.to_string())?;
                        match buff[0] {
                            0x00 => Ok(ColumnType::INT), 
                            0x01 => Ok(ColumnType::STRING),
                            _ => Err("Unknown type ")
                        }
                    }?;
                    let is_index = {
                        let mut buff = [0u8; 1];
                        file.read_exact(&mut buff).map_err(|e| e.to_string())?;
                        match buff[0] {
                            0x00 => Ok(false), 
                            0x01 => Ok(true),
                            _ => Err("Error parsing collumn index, unknown type")
                        }
                    }?;
                    columns.push(
                    ColumnDefinition {
                        name: collumn_name,
                        data_type: column_type,
                        is_index,
                    }
                    );
                }
                columns
            };
            formats.push(
                FileFormat {
                    table_name,
                    file_name,
                    columns,
                }
            );
        }
        let mut tables = HashMap::with_hasher(Xxh3HasherBuilder);
        for format in formats {
            tables.insert(format.table_name.clone(), format);
        }
        let mut index_trees = HashMap::with_hasher(Xxh3HasherBuilder);
        Ok(
            InformationSchema {
                tables,
                index_trees,
            }
        )
      }

    pub fn save_to_persistant_storage() -> Result<(), String> {

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::{Seek, Write};
    use tempfile::NamedTempFile;

    impl PartialEq for FileFormat {
        fn eq(&self, other: &Self) -> bool {
            self.table_name == other.table_name
                && self.file_name == other.file_name
                && self.columns == other.columns
        }
    }

    impl PartialEq for ColumnDefinition {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
                && self.is_index == other.is_index
                && self.data_type == other.data_type
        }
    }

    impl PartialEq for ColumnType {
        fn eq(&self, other: &Self) -> bool {
            matches!((self, other), 
                (ColumnType::INT, ColumnType::INT) | 
                (ColumnType::STRING, ColumnType::STRING))
        }
    }

    #[test]
    fn when_file_format_file_is_valid_is_shoud_be_correctly_parsed_to_infromation_schema() {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        let number_of_table_definitions: u16 = 2;
        let table_name_1 = "Table1";
        let table_name_2 = "Table2";
        let table_file1 = "table1.sl";
        let table_file2 = "table2.sl";
        let table_1_columns_count : u16= 2;
        let table_2_columns_count : u16= 1;
        let is_table_1_collumn_1_index = false; 
        let table_1_collumn_1 = "table_1_collumn_1";
        let table_1_collumn_1_type: u8 = 0;
        let is_table_1_collumn_2_index = false; 
        let table_1_collumn_2 = "table_1_collumn_2";
        let table_1_collumn_2_type: u8 = 1;
        let is_table_2_collumn_1_index = true; 
        let table_2_collumn_1 = "table_2_collumn_1";
        let table_2_collumn_1_type: u8 = 0;
        file.write(&number_of_table_definitions.to_le_bytes()).unwrap();
        file.write(&(table_name_1.len() as u16).to_le_bytes()).unwrap();
        file.write_all(table_name_1.as_bytes()).unwrap();
        file.write(&(table_file1.len() as u16).to_le_bytes()).unwrap();
        file.write_all(table_file1.as_bytes()).unwrap();
        file.write_all(&table_1_columns_count.to_le_bytes()).unwrap();
        file.write(&(table_1_collumn_1.len() as u16).to_le_bytes()).unwrap();
        file.write_all(table_1_collumn_1.as_bytes()).unwrap();
        file.write_all(&table_1_collumn_1_type.to_le_bytes()).unwrap();
        file.write(&[is_table_1_collumn_1_index as u8]).unwrap();
        file.write(&(table_1_collumn_2.len() as u16).to_le_bytes()).unwrap();
        file.write_all(table_1_collumn_2.as_bytes()).unwrap();
        file.write_all(&table_1_collumn_2_type.to_le_bytes()).unwrap();
        file.write(&[is_table_1_collumn_2_index as u8]).unwrap();
        file.write(&(table_name_2.len() as u16).to_le_bytes()).unwrap();
        file.write_all(table_name_2.as_bytes()).unwrap();
        file.write(&(table_file2.len() as u16).to_le_bytes()).unwrap();
        file.write_all(table_file2.as_bytes()).unwrap();
        file.write_all(&table_2_columns_count.to_le_bytes()).unwrap();
        file.write(&(table_2_collumn_1.len() as u16).to_le_bytes()).unwrap();
        file.write_all(table_2_collumn_1.as_bytes()).unwrap();
        file.write_all(&table_2_collumn_1_type.to_le_bytes()).unwrap();
        file.write(&[is_table_2_collumn_1_index as u8]).unwrap();

        file.flush().expect("Failed to flush temp file");
        file.seek(std::io::SeekFrom::Start(0)).expect("Failed to rewind the file");

        let temp_path = file.into_temp_path();
        let new_path = std::path::Path::new(FILE_FORMAT_FILE_NAME);
        fs::rename(&temp_path, &new_path).expect("Failed to rename temp file to 'ff.sl'");

        let result = InformationSchema::create().unwrap();

        assert_eq!(result.tables.len(), 2, "Expected to entries for table");
        if let None = result.tables.get(table_name_1) {
            panic!("Expected entry with key {}", table_name_1);
        }
        if let None = result.tables.get(table_name_2) {
            panic!("Expected entrie with key {}", table_name_2);
        }
        assert_eq!(
            result.tables[table_name_1], 
            FileFormat {
                table_name: table_name_1.to_string(),
                file_name: table_file1.to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: table_1_collumn_1.to_string(),
                        is_index: is_table_1_collumn_1_index,
                        data_type: ColumnType::INT,
                    },
                    ColumnDefinition {
                        name: table_1_collumn_2.to_string(),
                        is_index: is_table_1_collumn_2_index,
                        data_type: ColumnType::STRING,
                    },
                ]
            },
        );
        assert_eq!(
            result.tables[table_name_2],
            FileFormat {
                table_name: table_name_2.to_string(),
                file_name: table_file2.to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: table_2_collumn_1.to_string(),
                        is_index: is_table_2_collumn_1_index,
                        data_type: ColumnType::INT,
                    },
                ]
            }
        )
    }
}

 */