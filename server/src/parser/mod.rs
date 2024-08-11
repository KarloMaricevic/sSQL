pub mod ast;
pub mod token;
mod tokenizer;

use ast::{ColumnDefinition, Columns, SqlStatement};
use token::{DataType, Keyword, Punctuation, Token, Value};
use tokenizer::Tokenizer;

pub fn parse(input: String) -> Result<SqlStatement, String> {
    let mut tokenizer = Tokenizer::new(input);
    match tokenizer.peek()? {
        Some(Token::Keyword(Keyword::Create)) => parse_create_command(tokenizer),
        Some(Token::Keyword(Keyword::Insert)) => parse_insert_command(tokenizer),
        Some(Token::Keyword(Keyword::Select)) => parse_select_command(tokenizer),
        Some(_) => Err("First token error, no such command".to_string()),
        None => Err("Error reading first token".to_string()),
    }
}

fn parse_create_command(mut tokenizer: tokenizer::Tokenizer) -> Result<SqlStatement, String> {
    if let Err(_err) = expect_keyword(&mut tokenizer, Keyword::Create) {
        return Err("No keyword \"CREATE\"".to_string());
    }
    if let Err(_err) = expect_keyword(&mut tokenizer, Keyword::Table) {
        return Err("No keyword \"TABLE\" after CREATE".to_string());
    }
    let table_name = match tokenizer.next_token() {
        Ok(Some(Token::Indentifer(name))) => name,
        _ => return Err("Expected table name".to_string()),
    };
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::LeftParen) {
        return Err("Expected left parenthesis after table name".to_string());
    }
    let mut columns = vec![];
    loop {
        let name = match tokenizer.next_token() {
            Ok(Some(Token::Indentifer(name))) => name,
            Ok(None) => return Err("Unexpected end of input while parsing column name".to_string()),
            _ => return Err("Expected column name".to_string()),
        };
        let data_type = match tokenizer.next_token() {
            Ok(Some(Token::DataType(type_token))) => match type_token {
                DataType::Integer32 => ast::DataType::Int32,
                DataType::Varchar256 => ast::DataType::VarChar256,
            },
            Ok(None) => return Err("Unexpected end of input while parsing data type".to_string()),
            _ => return Err("Expected data type".to_string()),
        };
        columns.push(ColumnDefinition { name, data_type });
        if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::Comma) {
            return Err("Expected left parenthesis after collumn definition".to_string());
        }
        if let Ok(Some(token)) = tokenizer.peek() {
            match token {
                Token::Indentifer(_) => (),
                Token::Keyword(Keyword::Primary) => break,
                _ => {
                    return Err(
                        "Expected comma or closing parenthesis after column definition".to_string(),
                    )
                }
            }
        } else {
            return Err("Failed while parsing command".to_string());
        }
    }
    if let Err(_err) = expect_keyword(&mut tokenizer, Keyword::Primary) {
        return Err("No keyword \"PRIMARY\" after collumn definitions".to_string());
    }
    if let Err(_err) = expect_keyword(&mut tokenizer, Keyword::Key) {
        return Err("No keyword \"KEY\" after PRIMARY".to_string());
    }
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::LeftParen) {
        return Err("Expected left parenthesis after table name".to_string());
    }
    let primary_key_collumn = match tokenizer.next_token() {
        Ok(Some(token::Token::Indentifer(name))) => name,
        Ok(None) => {
            return Err("Unexpected end of input when tried to parse primary key".to_string())
        }
        _ => return Err("Expected name of primary key collumn".to_string()),
    };
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::LeftParen) {
        return Err("Expected right parenthesis after primary collumn key name".to_string());
    }
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::LeftParen) {
        return Err("Expected right parenthesis when closing collumn definitions".to_string());
    }

    Ok(SqlStatement::CreateTable {
        table_name,
        primary_key: primary_key_collumn,
        columns,
    })
}

fn parse_insert_command(mut tokenizer: tokenizer::Tokenizer) -> Result<SqlStatement, String> {
    if let Err(_err) = expect_keyword(&mut tokenizer, Keyword::Insert) {
        return Err("No keyword \"INSERT\"".to_string());
    }
    if let Err(_err) = expect_keyword(&mut tokenizer, Keyword::Into) {
        return Err("No keyword \"INTO\" after INSERT".to_string());
    }
    let table_name = match tokenizer.next_token() {
        Ok(Some(Token::Indentifer(name))) => name,
        _ => return Err("Expected table name after \"INTO\" keyword".to_string()),
    };
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::LeftParen) {
        return Err("Expected left parenthesis after table name".to_string());
    }
    let mut column_names = vec![];
    loop {
        let name = match tokenizer.next_token() {
            Ok(Some(Token::Indentifer(name))) => name,
            Ok(None) => return Err("Unexpected end of input while parsing column name".to_string()),
            _ => return Err("Expected column name".to_string()),
        };
        column_names.push(name);
        if let Ok(Some(token)) = tokenizer.peek() {
            match token {
                Token::Punctuation(Punctuation::Comma) => {
                    tokenizer.next_token()?;
                }
                Token::Punctuation(Punctuation::RightParen) => {
                    break;
                }
                _ => {
                    return Err(
                        "Expected comma or closing parenthesis after column name".to_string()
                    );
                }
            }
        }
    }
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::RightParen) {
        return Err("Expected left parenthesis after table name".to_string());
    }
    if let Err(_err) = expect_keyword(&mut tokenizer, token::Keyword::Values) {
        return Err("Expected keyword \"Values\"".to_string());
    }
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::LeftParen) {
        return Err("Expected left parenthesis after \"Values keyword\"".to_string());
    }
    let mut values = vec![];
    loop {
        let value = match tokenizer.next_token() {
            Ok(Some(Token::Value(Value::String(s)))) => ast::Value::VarChar256(s),
            Ok(Some(Token::Value(Value::Integer(integer)))) => ast::Value::Int32(integer),
            Ok(None) => {
                return Err("Unexpected end of input when tried to parse primary key".to_string())
            }
            _ => return Err("Expected value".to_string()),
        };
        values.push(value);
        if let Ok(Some(token)) = tokenizer.peek() {
            match token {
                token::Token::Punctuation(Punctuation::Comma) => {
                    tokenizer.next_token()?;
                }
                token::Token::Punctuation(Punctuation::RightParen) => {
                    break;
                }
                _ => {
                    return Err(
                        "Expected comma or closing parenthesis after column value".to_string()
                    );
                }
            }
        }
    }
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::LeftParen) {
        return Err("Expected left parenthesis after values".to_string());
    }
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::SemiColon) {
        return Err("Expected semicolon at the end of the command".to_string());
    }
    Ok(ast::SqlStatement::InsertInto {
        table_name,
        column_names,
        values,
    })
}

fn parse_select_command(mut tokenizer: tokenizer::Tokenizer) -> Result<SqlStatement, String> {
    if let Err(_err) = expect_keyword(&mut tokenizer, Keyword::Select) {
        return Err("No keyword \"SELECT\"".to_string());
    }
    let columns: Columns = match tokenizer.peek() {
        Ok(Some(Token::Wildcard)) => {
            tokenizer.next_token()?;
            Columns::All
        }
        Ok(Some(Token::Indentifer(_))) => {
            let mut collumns = vec![];
            loop {
                let name = match tokenizer.next_token() {
                    Ok(Some(Token::Indentifer(name))) => name,
                    Ok(None) => {
                        return Err("Unexpected end of input while parsing column name".to_string())
                    }
                    _ => return Err("Expected column name".to_string()),
                };
                collumns.push(name);
                if let Ok(Some(token)) = tokenizer.peek() {
                    match token {
                        token::Token::Punctuation(Punctuation::Comma) => {
                            tokenizer.next_token()?;
                        }
                        token::Token::Keyword(Keyword::From) => break,
                        _ => {
                            return Err("Expected comma or closing parenthesis after column value"
                                .to_string());
                        }
                    }
                }
            }
            Columns::Specific(collumns)
        }
        _ => return Err("Error parsing collumns to show".to_string()),
    };
    if let Err(_err) = expect_keyword(&mut tokenizer, Keyword::From) {
        return Err("No keyword \"FROM\" after SELECT".to_string());
    }
    let table = match tokenizer.next_token() {
        Ok(Some(Token::Indentifer(name))) => name,
        Ok(None) => return Err("Unexpected end of input while parsing table name".to_string()),
        _ => return Err("Expected table name".to_string()),
    };
    if let Err(_err) = expect_punctuation(&mut tokenizer, Punctuation::SemiColon) {
        return Err("Expected semicolon at the end of the command".to_string());
    }
    Ok(SqlStatement::Select { columns, table })
}

fn expect_keyword(
    tokenizer: &mut tokenizer::Tokenizer,
    _expected: token::Keyword,
) -> Result<(), ()> {
    if let Ok(Some(Token::Keyword(_expected))) = tokenizer.next_token() {
        Ok(())
    } else {
        Err(())
    }
}

fn expect_punctuation(
    tokenizer: &mut tokenizer::Tokenizer,
    _expected: token::Punctuation,
) -> Result<(), ()> {
    if let Ok(Some(Token::Punctuation(_expected))) = tokenizer.next_token() {
        Ok(())
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::ToSocketAddrs;

    use super::*;

    #[test]
    fn when_create_command_is_inputed_return_correct_ast() {
        let command =
            "CREATE TABLE table_name (collumn1 INT, collumn2 VARCHAR, PRIMARY KEY (collumn1));"
                .to_string();

        let result = parse(command);

        assert_eq!(
            result,
            Ok(SqlStatement::CreateTable {
                table_name: "table_name".to_string(),
                primary_key: "collumn1".to_string(),
                columns: vec![
                    ast::ColumnDefinition {
                        name: "collumn1".to_string(),
                        data_type: ast::DataType::Int32,
                    },
                    ast::ColumnDefinition {
                        name: "collumn2".to_string(),
                        data_type: ast::DataType::VarChar256,
                    }
                ]
            })
        );
    }

    #[test]
    fn when_insert_command_is_inputed_return_correct_ast() {
        let command =
            "INSERT INTO table_name (collumn1, collumn2) VALUES (12, \"value2\");".to_string();

        let result = parse(command);

        assert_eq!(
            result,
            Ok(ast::SqlStatement::InsertInto {
                table_name: "table_name".to_string(),
                column_names: vec!["collumn1".to_string(), "collumn2".to_string()],
                values: vec![
                    ast::Value::Int32(12),
                    ast::Value::VarChar256("value2".to_string()),
                ]
            })
        );
    }

    #[test]
    fn when_select_command_is_inputed_return_correct_ast() {
        let command = "SELECT collumn1, collumn2 FROM table_name;".to_string();

        let result = parse(command);

        assert_eq!(
            result,
            Ok(ast::SqlStatement::Select {
                columns: Columns::Specific(
                    ["collumn1".to_string(), "collumn2".to_string()].to_vec()
                ),
                table: "table_name".to_string()
            })
        )
    }

    #[test]
    fn when_select_command_with_wildcard_is_inputed_return_correct_ast() {
        let command = "SELECT * FROM table_name;".to_string();

        let result = parse(command);

        assert_eq!(
            result,
            Ok(ast::SqlStatement::Select {
                columns: Columns::All,
                table: "table_name".to_string()
            })
        )
    }
}
