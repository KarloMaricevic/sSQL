#[derive(Debug, PartialEq)]
pub enum SqlStatement {
    CreateTable {
        table_name: String,
        primary_key: String,
        columns: Vec<ColumnDefinition>,
    },
    InsertInto {
        table_name: String,
        column_names: Vec<String>,
        values: Vec<Value>,
    },
    Select {
        columns: Columns,
        table: String,
    },
}

#[derive(Debug, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq)]
pub enum DataType {
    VarChar256,
    Int32,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    VarChar256(String),
    Int32(i32),
}

#[derive(Debug, PartialEq)]
pub enum Columns {
    All,
    Specific(Vec<String>),
}
