use std::{collections::hash_map, fmt::Display};

use crate::error::MalError;

#[derive(Debug, Clone)]
pub enum MalData {
    List(Vec<MalData>),
    Vector(Vec<MalData>),
    HashMap(hash_map::HashMap<MalHashMapKey, MalData>),
    Integer(i64),
    Symbol(String),
    Nil,
    True,
    False,
    String(String),
    Keyword(String),
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum MalHashMapKey {
    String(String),
    Keyword(String),
}

impl From<MalHashMapKey> for MalData {
    fn from(value: MalHashMapKey) -> Self {
        match value {
            MalHashMapKey::String(string) => MalData::String(string),
            MalHashMapKey::Keyword(keyword) => MalData::Keyword(keyword),
        }
    }
}

impl TryFrom<MalData> for MalHashMapKey {
    type Error = MalError;

    fn try_from(value: MalData) -> Result<Self, Self::Error> {
        match value {
            MalData::String(string) => Ok(MalHashMapKey::String(string)),
            MalData::Keyword(keyword) => Ok(MalHashMapKey::Keyword(keyword)),
            _ => Err(MalError::InvalidHashMapKey),
        }
    }
}

fn escape_mal_string(string: &str) -> String {
    string
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("\n", "\\n")
}

// TODO: improve performance for the Display impl for List
impl Display for MalData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalData::List(list) => {
                write!(
                    f,
                    "({})",
                    list.iter()
                        .map(|el| format!("{}", el))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            MalData::Vector(vector) => {
                write!(
                    f,
                    "[{}]",
                    vector
                        .iter()
                        .map(|el| format!("{}", el))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            MalData::HashMap(hash_map) => {
                write!(
                    f,
                    "{{{}}}",
                    hash_map
                        .iter()
                        .map(|(key, value)| format!(
                            "{} {}",
                            MalData::from(key.clone()),
                            value.clone()
                        ))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            MalData::Integer(integer) => write!(f, "{}", integer),
            MalData::Symbol(symbol) => write!(f, "{}", symbol),
            MalData::Nil => write!(f, "nil"),
            MalData::True => write!(f, "true"),
            MalData::False => write!(f, "false"),
            MalData::String(string) => write!(f, "\"{}\"", escape_mal_string(string)),
            MalData::Keyword(keyword) => write!(f, ":{}", keyword),
        }
    }
}
