use crate::{
    parser::token::{DataType, Keyword, Punctuation, Token, Value},
    string_helpers::StringHelpers,
};

pub struct Tokenizer {
    input: String,
    position: usize,
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        Tokenizer { input, position: 0 }
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, String> {
        let token_option = self.peek()?;
        if let Some(ref token) = token_option {
            match token {
                Token::Value(Value::String(s)) => self.position += s.len() + 2, // +2 for the quotes
                Token::Value(Value::Integer(i)) => self.position += i.to_string().len(),
                Token::Keyword(keyword) => self.position += keyword.value().len(),
                Token::Indentifer(identifier) => self.position += identifier.len(),
                Token::DataType(data_type) => self.position += data_type.value().len(),
                Token::Punctuation(_) => self.position += 1,
                Token::Wildcard => self.position += 1,
            }
        }
        Ok(token_option)
    }

    pub fn peek(&mut self) -> Result<Option<Token>, String> {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return Ok(None);
        }
        let ch = self.current_char();
        match ch {
            '"' => {
                if let Some(string_end_index) = self.input[self.position + 1..]
                    .find('"')
                    .map(|i| self.position + i + 1)
                {
                    let string_start_index = self.position + 1;
                    Ok(Some(Token::Value(Value::String(
                        self.input[string_start_index..string_end_index].to_string(),
                    ))))
                } else {
                    Err(format!(
                        "Neverending string, starting at position {}",
                        self.position
                    ))
                }
            }
            _ if ch.is_numeric() => {
                if let Some(numeric) = (&self.input[self.position..]).extract_integer() {
                    if let Ok(parsed_int) = numeric.parse::<i32>() {
                        Ok(Some(Token::Value(Value::Integer(parsed_int))))
                    } else {
                        Err(format!(
                            "Cannot parse int, starting at position {}",
                            self.position
                        ))
                    }
                } else {
                    Err(format!(
                        "Internal error while parsing int, starting at position {}",
                        self.position
                    ))
                }
            }
            ch if ch == Punctuation::LeftParen.value() => {
                Ok(Some(Token::Punctuation(Punctuation::LeftParen)))
            }
            ch if ch == Punctuation::RightParen.value() => {
                Ok(Some(Token::Punctuation(Punctuation::RightParen)))
            }
            ch if ch == Punctuation::SemiColon.value() => {
                Ok(Some(Token::Punctuation(Punctuation::SemiColon)))
            }
            ch if ch == Punctuation::Comma.value() => {
                Ok(Some(Token::Punctuation(Punctuation::Comma)))
            }
            ch if ch == '*' => Ok(Some(Token::Wildcard)),
            _ => {
                let token_value = self.input[self.position..].take_until(&[
                    ' ',
                    Punctuation::Comma.value(),
                    Punctuation::LeftParen.value(),
                    Punctuation::RightParen.value(),
                    Punctuation::SemiColon.value(),
                ]);
                match token_value.as_str() {
                    kw if kw == Keyword::Create.value() => {
                        Ok(Some(Token::Keyword(Keyword::Create)))
                    }
                    kw if kw == Keyword::Table.value() => Ok(Some(Token::Keyword(Keyword::Table))),
                    kw if kw == Keyword::Insert.value() => {
                        Ok(Some(Token::Keyword(Keyword::Insert)))
                    }
                    kw if kw == Keyword::Into.value() => Ok(Some(Token::Keyword(Keyword::Into))),
                    kw if kw == Keyword::Values.value() => {
                        Ok(Some(Token::Keyword(Keyword::Values)))
                    }
                    kw if kw == Keyword::Primary.value() => {
                        Ok(Some(Token::Keyword(Keyword::Primary)))
                    }
                    kw if kw == Keyword::Key.value() => Ok(Some(Token::Keyword(Keyword::Key))),
                    kw if kw == Keyword::Select.value() => {
                        Ok(Some(Token::Keyword(Keyword::Select)))
                    }
                    kw if kw == Keyword::From.value() => Ok(Some(Token::Keyword(Keyword::From))),
                    dt if dt == DataType::Integer32.value() => {
                        Ok(Some(Token::DataType(DataType::Integer32)))
                    }
                    dt if dt == DataType::Varchar256.value() => {
                        Ok(Some(Token::DataType(DataType::Varchar256)))
                    }
                    _ => Ok(Some(Token::Indentifer(token_value.to_string()))),
                }
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.current_char().is_whitespace() {
            self.position += 1;
        }
    }

    fn current_char(&self) -> char {
        self.input[self.position..].chars().next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_next_token_is_string_value_type_return_value_token_type() {
        let mut tokeinzer = Tokenizer::new("\"StringValue\",  ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(tokeinzer.position, "\"StringValue\"".len());
        assert_eq!(
            next_token,
            Ok(Some(Token::Value(Value::String("StringValue".to_string()))))
        );
    }

    #[test]
    fn when_next_token_is_number_value_type_return_value_token_type() {
        let mut tokeinzer = Tokenizer::new("123631 ,".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Value(Value::Integer(123631)))));
        assert_eq!(tokeinzer.position, "123631".len());
    }

    #[test]
    fn when_next_token_is_left_paren_value_type_return_punctuation_left_paren_token_type() {
        let mut tokeinzer = Tokenizer::new("(collumn_name ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(
            next_token,
            Ok(Some(Token::Punctuation(Punctuation::LeftParen)))
        );
        assert_eq!(tokeinzer.position, "(".len());
    }

    #[test]
    fn when_next_token_is_right_paren_value_type_return_punctuation_right_paren_token_type() {
        let mut tokeinzer = Tokenizer::new("); ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(
            next_token,
            Ok(Some(Token::Punctuation(Punctuation::RightParen)))
        );
        assert_eq!(tokeinzer.position, ")".len());
    }

    #[test]
    fn when_next_token_value_is_comma_return_comma_token_type() {
        let mut tokeinzer = Tokenizer::new(",collumn1    ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Punctuation(Punctuation::Comma))));
        assert_eq!(tokeinzer.position, ",".len());
    }

    #[test]
    fn when_next_token_is_semicollon_value_type_return_punctuation_semicollon_token_type() {
        let mut tokeinzer = Tokenizer::new(";    ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(
            next_token,
            Ok(Some(Token::Punctuation(Punctuation::SemiColon)))
        );
        assert_eq!(tokeinzer.position, ";".len());
    }

    #[test]
    fn when_next_token_value_is_create_return_keyowrd_create_token_type() {
        let mut tokeinzer = Tokenizer::new("CREATE ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Keyword(Keyword::Create))));
        assert_eq!(tokeinzer.position, "CREATE".len());
    }

    #[test]
    fn when_next_token_value_is_table_return_keyowrd_table_token_type() {
        let mut tokeinzer = Tokenizer::new("TABLE ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Keyword(Keyword::Table))));
        assert_eq!(tokeinzer.position, "TABLE".len());
    }

    #[test]
    fn when_next_token_value_is_insert_return_keyword_insert_token_type() {
        let mut tokeinzer = Tokenizer::new("INSERT ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Keyword(Keyword::Insert))));
        assert_eq!(tokeinzer.position, "INSERT".len());
    }

    #[test]
    fn when_next_token_value_is_into_return_keyword_into_token_type() {
        let mut tokeinzer = Tokenizer::new("INTO ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Keyword(Keyword::Into))));
        assert_eq!(tokeinzer.position, "INTO".len());
    }

    #[test]
    fn when_next_token_value_is_values_return_keyword_values_token_type() {
        let mut tokeinzer = Tokenizer::new("VALUES (".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Keyword(Keyword::Values))));
        assert_eq!(tokeinzer.position, "VALUES".len());
    }

    #[test]
    fn when_next_token_value_is_primary_return_keyword_primary_token_type() {
        let mut tokeinzer = Tokenizer::new("PRIMARY ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Keyword(Keyword::Primary))));
        assert_eq!(tokeinzer.position, "PRIMARY".len());
    }

    #[test]
    fn when_next_token_value_is_key_return_keyword_key_token_type() {
        let mut tokeinzer = Tokenizer::new("KEY ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Keyword(Keyword::Key))));
        assert_eq!(tokeinzer.position, "KEY".len());
    }

    #[test]
    fn when_next_token_value_is_int_return_data_type_int_token_type() {
        let mut tokeinzer = Tokenizer::new("INT,".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::DataType(DataType::Integer32))));
        assert_eq!(tokeinzer.position, "INT".len());
    }

    #[test]
    fn when_next_token_value_is_varchar_return_keyword_varchar_token_type() {
        let mut tokeinzer = Tokenizer::new("VARCHAR ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::DataType(DataType::Varchar256))));
        assert_eq!(tokeinzer.position, "VARCHAR".len());
    }

    #[test]
    fn when_next_token_value_is_indentifer_return_indentifier_token_type() {
        let mut tokeinzer = Tokenizer::new("indentifer_1 ".to_string());
        let next_token = tokeinzer.next_token();

        assert_eq!(
            next_token,
            Ok(Some(Token::Indentifer("indentifer_1".to_string())))
        );
        assert_eq!(tokeinzer.position, "indentifer_1".len());
    }

    #[test]
    fn when_next_token_value_is_wildcard_return_wildcard_token_type() {
        let mut tokeinzer = Tokenizer::new("* ".to_string());

        let next_token = tokeinzer.next_token();

        assert_eq!(next_token, Ok(Some(Token::Wildcard)));
        assert_eq!(tokeinzer.position, "*".len());
    }

    #[test]
    fn when_create_command_is_inputed_return_correct_tokens() {
        let mut tokeinzer = Tokenizer::new(
            "CREATE TABLE table_name (collumn1 INT, collumn2 VARCHAR, PRIMARY KEY (collumn1));"
                .to_string(),
        );
        let mut result = Vec::new();

        while let Ok(Some(token)) = tokeinzer.next_token() {
            result.push(token)
        }

        assert_eq!(
            result,
            [
                Token::Keyword(Keyword::Create),
                Token::Keyword(Keyword::Table),
                Token::Indentifer("table_name".to_string()),
                Token::Punctuation(Punctuation::LeftParen),
                Token::Indentifer("collumn1".to_string()),
                Token::DataType(DataType::Integer32),
                Token::Punctuation(Punctuation::Comma),
                Token::Indentifer("collumn2".to_string()),
                Token::DataType(DataType::Varchar256),
                Token::Punctuation(Punctuation::Comma),
                Token::Keyword(Keyword::Primary),
                Token::Keyword(Keyword::Key),
                Token::Punctuation(Punctuation::LeftParen),
                Token::Indentifer("collumn1".to_string()),
                Token::Punctuation(Punctuation::RightParen),
                Token::Punctuation(Punctuation::RightParen),
                Token::Punctuation(Punctuation::SemiColon),
            ]
        )
    }

    #[test]
    fn when_insert_command_is_inputed_return_correct_tokens() {
        let mut tokenizer =
            Tokenizer::new("INSERT INTO table_name (123, \"stringValue\");".to_string());
        let mut result = Vec::new();

        while let Ok(Some(token)) = tokenizer.next_token() {
            result.push(token)
        }

        assert_eq!(
            result,
            [
                Token::Keyword(Keyword::Insert),
                Token::Keyword(Keyword::Into),
                Token::Indentifer("table_name".to_string()),
                Token::Punctuation(Punctuation::LeftParen),
                Token::Value(Value::Integer(123)),
                Token::Punctuation(Punctuation::Comma),
                Token::Value(Value::String("stringValue".to_string())),
                Token::Punctuation(Punctuation::RightParen),
                Token::Punctuation(Punctuation::SemiColon),
            ]
        )
    }

    #[test]
    fn when_select_command_is_inputed_return_correct_tokens() {
        let mut tokenizer = Tokenizer::new("SELECT * FROM Customers;".to_string());
        let mut result = Vec::new();

        while let Ok(Some(token)) = tokenizer.next_token() {
            result.push(token)
        }

        assert_eq!(
            result,
            [
                Token::Keyword(Keyword::Select),
                Token::Wildcard,
                Token::Keyword(Keyword::From),
                Token::Indentifer("Customers".to_string()),
                Token::Punctuation(Punctuation::SemiColon)
            ]
        )
    }
}
