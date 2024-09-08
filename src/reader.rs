use crate::types::MalData;

pub struct Reader {
    tokens: Vec<String>,
    i: usize,
}

impl Reader {
    pub fn new(input: String) -> Self {
        Self {
            tokens: Self::tokenize(input),
            i: 0,
        }
    }

    // TODO: benchmark this and implement a better tokenizer than regex
    fn tokenize(input: String) -> Vec<String> {
        let re = regex::Regex::new(
            r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#,
        )
        .unwrap();
        re.captures_iter(input.as_str())
            .map(|cap| cap[1].to_string())
            .collect()
    }

    pub fn read_form(&mut self) -> MalData {
        if self.tokens[self.i] == "(" {
            self.i += 1;
            self.read_list()
        } else {
            self.read_atom()
        }
    }

    fn read_list(&mut self) -> MalData {
        let mut list = Vec::new();
        while self.tokens[self.i] != ")" {
            list.push(self.read_form());
        }
        self.i += 1;
        MalData::List(list)
    }

    fn read_atom(&mut self) -> MalData {
        let token = self.tokens[self.i].clone();
        self.i += 1;
        match token.parse::<i64>() {
            Ok(integer) => MalData::Integer(integer),
            Err(_) => MalData::Symbol(token),
        }
    }
}
