use types::MalData;

mod reader;
mod types;

pub fn read(input: String) -> MalData {
    let mut reader = reader::Reader::new(input);
    let ast = reader.read_form();
    ast
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

#[cfg(test)]
mod tests {
    use std::path::{self, PathBuf};

    use super::*;

    fn run_mal_test(input: PathBuf) {
        
    }

    #[test]
    fn it_works() {
        run_mal_test("./mal_tests/step1_read_print.mal".into());
        // assert_eq!(result, 4);
    }
}
