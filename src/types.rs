use std::fmt::Display;

#[derive(Debug)]
pub enum MalData {
    List(Vec<MalData>),
    Integer(i64),
    Symbol(String),
}

// TODO: improve performance for the Display impl for List
impl Display for MalData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalData::List(list) => {
                write!(
                    f,
                    "({})",
                    list.iter().map(|el| format!("{}", el)).collect::<Vec<String>>().join(" ")
                )
            }
            MalData::Integer(integer) => write!(f, "{}", integer),
            MalData::Symbol(symbol) => write!(f, "{}", symbol),
        }
    }
}
