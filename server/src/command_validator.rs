/* use crate::db_metadata::DbMetadata;
use crate::parser::ast::{Columns, DataType, SqlStatement, Value};

pub fn validate_command(command: &SqlStatement, metadata: &DbMetadata) -> Result<(), String> {
    match command {
        SqlStatement::CreateTable {
            table_name,
            primary_key,
            columns,
        } => {
            if metadata.table_pointers.contains_key(table_name) {
                return Err(format!("Already have table with name: {}", table_name));
            }
            if let None = columns
                .iter()
                .find(|definition| *primary_key == definition.name)
            {
                return Err(format!(
                    "No collumnd definied with primary key name, {}",
                    primary_key
                ));
            }
        }
        SqlStatement::InsertInto {
            table_name,
            column_names,
            values,
        } => {
            let table_schema = metadata.table_schemas.get(table_name).ok_or_else(|| {
                format!("Db dosent containt specified table, name {}", table_name)
            })?;
            for (index, name) in column_names.iter().enumerate() {
                let column_data_type = table_schema
                    .row_definitions
                    .get(name)
                    .ok_or_else(|| format!("There is no column named {} in table", name))?;

                let value = values
                    .get(index)
                    .ok_or_else(|| format!("Missing value for column '{}'", name))?;
                match column_data_type {
                    DataType::Int32 => {
                        if let Value::Int32(value) = value {
                            if *value < i32::MIN || *value > i32::MAX {
                                return Err(format!(
                                    "Value {} for column '{}' is out of range for i32.",
                                    value, name
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Value for column '{}' is not of type integer.",
                                name
                            ));
                        }
                    }
                    DataType::VarChar256 => {
                        if let Value::VarChar256(ref s) = value {
                            if s.len() > 256 {
                                return Err(format!(
                                    "Value for column '{}' exceeds maximum length of 256 characters.",
                                    name
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Value for column '{}' is not of type VarChar.",
                                name
                            ));
                        }
                    }
                }
            }
            for column_name in table_schema.row_definitions.keys() {
                if !column_names.contains(column_name) {
                    return Err(format!(
                        "Error: Missing value for required column '{}'.",
                        column_name
                    ));
                }
            }
        }
        SqlStatement::Select { columns, table } => {
            let table_schema = metadata
                .table_schemas
                .get(table)
                .ok_or_else(|| format!("Db dosent containt specified table, name {}", table))?;
            if let Columns::Specific(specific_columns) = columns {
                for column in specific_columns {
                    if !table_schema.row_definitions.contains_key(column) {
                        return Err(format!("No column named '{}'", column));
                    }
                }
            }
        }
    }
    Ok(())
}
 */