use types::MalData;

mod reader;
mod types;

pub fn read(input: String) -> MalData {
    let mut reader = reader::Reader::new(input);
    
    reader.read_form()
}

pub fn eval(input: MalData) -> MalData {
    input
}

pub fn print(input: MalData) -> String {
    format!("{}", input)
}

pub fn rep(input: String) -> String {
    print(eval(read(input)))
}
