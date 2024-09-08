use std::collections::hash_map;

use error::MalError;
use types::{MalData, MalEnvironment};

mod error;
mod reader;
mod types;

pub struct Interpreter {
    env: MalEnvironment,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            env: MalEnvironment::new(),
        };

        interpreter.load_builtins();

        interpreter
    }

    fn load_builtins(&mut self) {
        self.env.insert(
            "+".to_owned(),
            Box::new(|args: &[MalData]| {
                if let MalData::Integer(arg0) = &args[0] {
                    if let MalData::Integer(arg1) = &args[1] {
                        Ok(MalData::Integer(arg0 + arg1))
                    } else {
                        Err(MalError::TypeError(args[1].clone()))
                    }
                } else {
                    Err(MalError::TypeError(args[0].clone()))
                }
            }),
        );

        self.env.insert(
            "-".to_owned(),
            Box::new(|args: &[MalData]| {
                if let MalData::Integer(arg0) = &args[0] {
                    if let MalData::Integer(arg1) = &args[1] {
                        Ok(MalData::Integer(arg0 - arg1))
                    } else {
                        Err(MalError::TypeError(args[1].clone()))
                    }
                } else {
                    Err(MalError::TypeError(args[0].clone()))
                }
            }),
        );

        self.env.insert(
            "*".to_owned(),
            Box::new(|args: &[MalData]| {
                if let MalData::Integer(arg0) = &args[0] {
                    if let MalData::Integer(arg1) = &args[1] {
                        Ok(MalData::Integer(arg0 * arg1))
                    } else {
                        Err(MalError::TypeError(args[1].clone()))
                    }
                } else {
                    Err(MalError::TypeError(args[0].clone()))
                }
            }),
        );

        self.env.insert(
            "/".to_owned(),
            Box::new(|args: &[MalData]| {
                if let MalData::Integer(arg0) = &args[0] {
                    if let MalData::Integer(arg1) = &args[1] {
                        Ok(MalData::Integer(arg0 / arg1))
                    } else {
                        Err(MalError::TypeError(args[1].clone()))
                    }
                } else {
                    Err(MalError::TypeError(args[0].clone()))
                }
            }),
        );
    }

    pub fn read(&self, input: String) -> Result<MalData, MalError> {
        let mut reader = reader::Reader::new(input);
        reader.read_input()
    }

    pub fn eval(&self, input: MalData) -> Result<MalData, MalError> {
        if self.env.contains_key("DEBUG-EVAL") {
            println!("EVAL: {}", input);
        }
        use MalData::*;
        match &input {
            Symbol(s) => {
                if !self.env.contains_key(s) {
                    Err(MalError::SymbolNotFound(s.clone()))
                } else {
                    Ok(input)
                }
            }
            List(list) => {
                let evaluated_list = list
                    .iter()
                    .map(|el| self.eval(el.clone()))
                    .collect::<Result<Vec<_>, _>>()?;
                if !evaluated_list.is_empty() {
                    if let MalData::Symbol(s) = &evaluated_list[0] {
                        let native_function = self.env.get(s).unwrap();
                        Ok(native_function(&evaluated_list[1..])?)
                    } else {
                        Err(MalError::TypeError(evaluated_list[0].clone()))
                    }
                } else {
                    Ok(MalData::List(evaluated_list))
                }
            }
            Vector(vector) => Ok(MalData::Vector(
                vector
                    .iter()
                    .map(|el| self.eval(el.clone()))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            HashMap(hash_map) => {
                // Evaluate the hashmap
                let evaluated_hash_map: Result<
                    hash_map::HashMap<types::MalHashMapKey, MalData>,
                    MalError,
                > = hash_map
                    .iter()
                    .map(|(key, value)| self.eval(value.clone()).map(|value| (key.clone(), value)))
                    .collect();
                Ok(MalData::HashMap(evaluated_hash_map?))
            }
            _ => Ok(input),
        }
    }

    pub fn print(&self, input: MalData) -> String {
        format!("{}", input)
    }

    pub fn rep(&self, input: String) -> String {
        let evaluated = self.read(input).and_then(|ast| self.eval(ast));
        match evaluated {
            Ok(evaluated) => self.print(evaluated),
            Err(e) => format!("{}", e),
        }
    }
}
