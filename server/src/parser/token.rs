#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Indentifer(String),
    DataType(DataType),
    Value(Value),
    Wildcard,
    Punctuation(Punctuation),
}

#[derive(Debug, PartialEq)]
pub enum DataType {
    Integer32,
    Varchar256,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Integer(i32),
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Create,
    Table,
    Insert,
    Into,
    Values,
    Primary,
    Key,
    Select,
    From,
}

#[derive(Debug, PartialEq)]
pub enum Punctuation {
    LeftParen,
    RightParen,
    Comma,
    SemiColon,
}

impl Keyword {
    pub fn value(&self) -> &'static str {
        match self {
            Keyword::Create => "CREATE",
            Keyword::Table => "TABLE",
            Keyword::Insert => "INSERT",
            Keyword::Into => "INTO",
            Keyword::Values => "VALUES",
            Keyword::Primary => "PRIMARY",
            Keyword::Key => "KEY",
            Keyword::Select => "SELECT",
            Keyword::From => "FROM",
        }
    }
}

impl DataType {
    pub fn value(&self) -> &'static str {
        match self {
            DataType::Integer32 => "INT",
            DataType::Varchar256 => "VARCHAR",
        }
    }
}

impl Punctuation {
    pub fn value(&self) -> char {
        match self {
            Punctuation::LeftParen => '(',
            Punctuation::RightParen => ')',
            Punctuation::Comma => ',',
            Punctuation::SemiColon => ';',
        }
    }
}
