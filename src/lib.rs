pub mod parse {
    #[derive(Debug, PartialEq, Clone)]
    enum JsonToken {
        LeftBrace,
        RightBrace,
        LeftBracket,
        RightBracket,
        Colon,
        Comma,
        String(String),
        Number(f64),
        True,
        False,
        Null,
    }
    
    #[derive(Debug)]
    enum TokenizeError {
        UnexpectedCharacter(char, u32),
        // Add more error variants as needed
    }
    
    struct JsonTokenizer<'a> {
        input: &'a str,
        position: usize,
    }
    
    impl<'a> JsonTokenizer<'a> {
        fn new(input: &'a str) -> Self {
            JsonTokenizer { input, position: 0 }
        }
    
        fn next(&mut self) -> Option<char> {
            self.position += 1;
            self.input.chars().nth(self.position - 1)
        }
    
        fn parse_int(&mut self) -> Result<JsonToken, TokenizeError> {
            let start_position = self.position;
            while let Some(ch) = self.next() {
                if !ch.is_ascii_digit() && ch != '.' {
                    self.position -= 1; // Move the position back for the next token to start at the non-numeric character
                    let number_str = &self.input[start_position..(self.position)];
                    if let Ok(number) = number_str.parse::<f64>() {
                        return Ok(JsonToken::Number(number));
                    } else {
                        return Err(TokenizeError::UnexpectedCharacter(ch, self.position.try_into().unwrap()));
                    }
                }
            }
    
            Err(TokenizeError::UnexpectedCharacter('\0', self.position.try_into().unwrap()))
        }
    
    
        fn parse_string(&mut self) -> Result<JsonToken, TokenizeError> {
            let mut string = String::new();
            while let Some(ch) = self.next() {
                // TODO: sanitize escaped characters
                match ch {
                    '"' => return Ok(JsonToken::String(string)),
                    _ => string.push(ch)
                }
            }
            // TODO: add different error for unclosed strings
            Err(TokenizeError::UnexpectedCharacter('"', self.position.try_into().unwrap()))
        }
    
    
        fn parse_keyword(&mut self, keyword: &'static str, token: JsonToken) -> Result<JsonToken, TokenizeError> {
            // tokenize keywords (true, false, null)
            self.position -= 1;
            let start_position = self.position;
    
            for expected_ch in keyword.chars() {
                if let Some(ch) = self.next() {
                    if ch != expected_ch {
                        return Err(TokenizeError::UnexpectedCharacter(ch, self.position.try_into().unwrap()));
                    }
                } else {
                    return Err(TokenizeError::UnexpectedCharacter('\0', self.position.try_into().unwrap()));
                }
            }
    
            Ok(token)
        }
    
        fn tokenize(&mut self) -> Result<Vec<JsonToken>, TokenizeError>{
            let mut tokens: Vec<JsonToken> = Vec::new();
            while let Some(ch) = self.next() {
                match ch {
                    '{' => tokens.push(JsonToken::LeftBrace),
                    '}' => tokens.push(JsonToken::RightBrace),
                    ',' => tokens.push(JsonToken::Comma),
                    ':' => tokens.push(JsonToken::Colon),
                    '[' => tokens.push(JsonToken::LeftBracket),
                    ']' => tokens.push(JsonToken::RightBracket),
                    '0'..='9' => {
                        self.position -= 1; // parse_int jumping back to the first character of number;
                        tokens.push(self.parse_int()?)
                    },
                    '"' => tokens.push(self.parse_string()?),
                    't' => tokens.push(self.parse_keyword("true", JsonToken::True)?),
                    'f' => tokens.push(self.parse_keyword("false", JsonToken::False)?),
                    'n' => tokens.push(self.parse_keyword("null", JsonToken::Null)?),
                    _ => {}
                }
            }
            Ok(tokens)
        }
    }
    
    
    #[derive(Debug)]
    pub enum JsonValue {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<JsonValue>),
        Object(Vec<(String, JsonValue)>)
    }
    
    #[derive(Debug)]
    enum ParseError {
        UnexpectedToken(JsonToken),
        UnexpectedEnd,
    }
    
    struct JsonParser<'a> {
        tokens: &'a [JsonToken],
        position: usize,
    }
    
    impl<'a> JsonParser<'a> {
        fn new(tokens: &'a [JsonToken]) -> Self {
            JsonParser { tokens, position: 0 }
        }
    
        fn next(&mut self) -> Option<&'a JsonToken> {
            let token = self.tokens.get(self.position);
            self.position += 1;
            token
        }
    
        fn parse(&mut self) -> Result<JsonValue, ParseError> {
            if let Some(token) = self.next() {
                match token {
                    JsonToken::Null => Ok(JsonValue::Null),
                    JsonToken::True => Ok(JsonValue::Bool(true)),
                    JsonToken::False => Ok(JsonValue::Bool(false)),
                    JsonToken::Number(num) => Ok(JsonValue::Number(*num)),
                    JsonToken::String(s) => Ok(JsonValue::String(s.clone())),
                    JsonToken::LeftBrace => self.parse_object(),
                    JsonToken::LeftBracket => self.parse_array(),
                    _ => Err(ParseError::UnexpectedToken(token.clone())),
                }
            } else {
                Err(ParseError::UnexpectedEnd)
            }
        }
    
        fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
            let mut object = Vec::new();
    
            loop {
                if let Some(token) = self.next() {
                    match token {
                        JsonToken::RightBrace => return Ok(JsonValue::Object(object)),
                        JsonToken::String(key) => {
                            if let Some(JsonToken::Colon) = self.next() {
                                let value = self.parse()?;
                                object.push((key.clone(), value));
    
                                match self.next() {
                                    Some(JsonToken::Comma) => continue,
                                    Some(JsonToken::RightBrace) => return Ok(JsonValue::Object(object)),
                                    _ => return Err(ParseError::UnexpectedToken(token.clone())),
                                }
                            } else {
                                return Err(ParseError::UnexpectedToken(token.clone()));
                            }
                        }
                        _ => return Err(ParseError::UnexpectedToken(token.clone())),
                    }
                } else {
                    return Err(ParseError::UnexpectedEnd);
                }
            }
        }
    
        fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
            let mut array = Vec::new();
    
            loop {
                if let Some(token) = self.next() {
                    match token {
                        JsonToken::RightBracket => return Ok(JsonValue::Array(array)),
                        _ => {
                            self.position -= 1; // Move the position back for the next token to start at the array element
                            let value = self.parse()?;
                            array.push(value);
    
                            match self.next() {
                                Some(JsonToken::Comma) => continue,
                                Some(JsonToken::RightBracket) => return Ok(JsonValue::Array(array)),
                                _ => return Err(ParseError::UnexpectedToken(token.clone())),
                            }
                        }
                    }
                } else {
                    return Err(ParseError::UnexpectedEnd);
                }
            }
        }
    }


    use std::fs::read_to_string;
    pub fn load_from_file(path: &str) -> JsonValue {
        let file_data = read_to_string(path).unwrap();
        let mut tokenizer = JsonTokenizer::new(&file_data);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = JsonParser::new(&tokens);
        parser.parse().unwrap()
    }
}
