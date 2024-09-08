use error::MalError;
use types::MalData;

mod error;
mod reader;
mod types;

pub fn read(input: String) -> Result<MalData, MalError> {
    let mut reader = reader::Reader::new(input);
    reader.read_input()
}

pub fn eval(input: MalData) -> MalData {
    input
}

pub fn print(input: MalData) -> String {
    format!("{}", input)
}

pub fn rep(input: String) -> String {
    let ast = read(input);
    match ast {
        Ok(ast) => print(eval(ast)),
        Err(e) => format!("{}", e),
    }
}
