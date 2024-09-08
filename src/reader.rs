use std::collections::hash_map;

use crate::{
    error::MalError,
    types::{MalData, MalHashMapKey},
};

pub struct Reader {
    tokens: Vec<String>,
    i: usize,
}

impl Reader {
    pub fn new(input: String) -> Self {
        let tokens = Self::tokenize(input);
        Self { tokens, i: 0 }
    }

    // TODO: benchmark this and implement a better tokenizer than regex
    fn tokenize(input: String) -> Vec<String> {
        let re = regex::Regex::new(
            r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"#,
        )
        .unwrap();
        re.captures_iter(input.as_str())
            .map(|cap| cap[1].to_string())
            .filter(|token| !token.starts_with(";"))
            .collect()
    }

    pub fn read_input(&mut self) -> Result<MalData, MalError> {
        if self.tokens.is_empty() {
            return Ok(MalData::Nil);
        }
        let data = self.read_form()?;
        if self.i < self.tokens.len() {
            return Err(MalError::ExpectedEOF {
                found: self.tokens[self.i].clone(),
            });
        }
        Ok(data)
    }

    fn read_form(&mut self) -> Result<MalData, MalError> {
        if self.i >= self.tokens.len() {
            return Err(MalError::Unexpected {
                found: "EOF".to_string(),
            });
        }
        if self.tokens[self.i] == "(" {
            self.i += 1;
            self.read_list()
        } else if self.tokens[self.i] == "[" {
            self.i += 1;
            self.read_vector()
        } else if self.tokens[self.i] == "{" {
            self.i += 1;
            self.read_hash_map()
        } else if let Some(stripped_token) = self.tokens[self.i].strip_prefix(":") {
            // keyword
            self.i += 1;
            if stripped_token.is_empty() {
                return Err(MalError::InvalidToken);
            }
            Ok(MalData::Keyword(stripped_token.to_string()))
        } else if self.tokens[self.i] == "'" {
            // quotes reader macro
            self.i += 1;
            let parsed = self.read_form()?;
            Ok(MalData::List(vec![
                MalData::Symbol("quote".to_string()),
                parsed,
            ]))
        } else if self.tokens[self.i].starts_with("`") {
            // quasiquote reader macro
            self.i += 1;
            let parsed = self.read_form()?;
            Ok(MalData::List(vec![
                MalData::Symbol("quasiquote".to_string()),
                parsed,
            ]))
        } else if self.tokens[self.i].starts_with("~@") {
            // splice-unquote reader macro
            self.i += 1;
            let parsed = self.read_form()?;
            Ok(MalData::List(vec![
                MalData::Symbol("splice-unquote".to_string()),
                parsed,
            ]))
        } else if self.tokens[self.i].starts_with("~") {
            // unquote reader macro
            self.i += 1;
            let parsed = self.read_form()?;
            Ok(MalData::List(vec![
                MalData::Symbol("unquote".to_string()),
                parsed,
            ]))
        } else if self.tokens[self.i].starts_with("@") {
            // deref reader macro
            self.i += 1;
            let parsed = self.read_form()?;
            Ok(MalData::List(vec![
                MalData::Symbol("deref".to_string()),
                parsed,
            ]))
        } else if self.tokens[self.i].starts_with("^") {
            // deref reader macro
            self.i += 1;
            let parsed_meta = self.read_form()?;
            let parsed = self.read_form()?;
            Ok(MalData::List(vec![
                MalData::Symbol("with-meta".to_string()),
                parsed,
                parsed_meta,
            ]))
        } else {
            self.read_atom()
        }
    }

    fn read_list(&mut self) -> Result<MalData, MalError> {
        self.read_sequence(")")
            .map(MalData::List)
    }

    fn read_vector(&mut self) -> Result<MalData, MalError> {
        self.read_sequence("]")
            .map(MalData::Vector)
    }

    fn read_hash_map(&mut self) -> Result<MalData, MalError> {
        let sequence = self.read_sequence("}")?;

        let kv_pairs = sequence.chunks_exact(2);
        if !kv_pairs.remainder().is_empty() {
            return Err(MalError::UnbalancedHashMap);
        }

        let mut hash_map = hash_map::HashMap::new();

        for kv_pair in kv_pairs {
            let key = MalHashMapKey::try_from(kv_pair[0].clone())?;
            hash_map.insert(key, kv_pair[1].clone());
        }

        Ok(MalData::HashMap(hash_map))
    }

    fn read_sequence(&mut self, ending_token: &str) -> Result<Vec<MalData>, MalError> {
        let mut sequence = Vec::new();
        loop {
            if self.i >= self.tokens.len() {
                return Err(MalError::UnbalancedBrackets);
            }
            let token = self.tokens[self.i].clone();
            if token == ending_token {
                break;
            } else {
                sequence.push(self.read_form()?);
            }
        }
        self.i += 1;
        Ok(sequence)
    }

    fn read_atom(&mut self) -> Result<MalData, MalError> {
        let token = self.tokens[self.i].clone();
        let parsed = match token.parse::<i64>() {
            Ok(integer) => MalData::Integer(integer),
            Err(_) => {
                if token.starts_with("\"") {
                    // Parse escape sequences
                    parse_string(token)?
                } else {
                    match token.as_str() {
                        "nil" => MalData::Nil,
                        "true" => MalData::True,
                        "false" => MalData::False,
                        _ => MalData::Symbol(token),
                    }
                }
            }
        };
        self.i += 1;
        Ok(parsed)
    }
}

// Parse a string token
fn parse_string(token: String) -> Result<MalData, MalError> {
    if !token.starts_with('"') {
        panic!("Expected string token to start with '\"'");
    }

    // Start parsing the string
    let mut i = 1;
    let mut parsed_string = String::new();
    let chars: Vec<_> = token.chars().collect();
    let token_len = chars.len();

    while i < token_len {
        // Handle escape sequences
        if chars[i] == '\\' {
            // Check if the next character is a valid escape sequence
            if i + 1 >= token_len {
                return Err(MalError::UnbalancedDoubleQuotes);
            }

            if chars[i + 1] == 'n' {
                // Newline escape sequence
                parsed_string.push('\n');
                i += 2;
            } else if chars[i + 1] == '\\' {
                // Backslash escape sequence
                parsed_string.push('\\');
                i += 2;
            } else if chars[i + 1] == '"' {
                // Double quote escape sequence
                parsed_string.push('"');
                i += 2;
            } else {
                return Err(MalError::InvalidEscapeSequence);
            }
            continue;
        }

        if chars[i] == '"' {
            // End of string
            if i < token_len - 1 {
                // We encountered an unescaped double quote
                return Err(MalError::UnbalancedDoubleQuotes);
            } else {
                return Ok(MalData::String(parsed_string));
            }
        }

        parsed_string.push(chars[i]);
        i += 1;
    }

    Err(MalError::UnbalancedDoubleQuotes)
}
