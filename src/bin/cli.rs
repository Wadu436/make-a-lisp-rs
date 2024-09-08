use std::ops::DerefMut;

use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use make_a_lisp_rs::rep;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut rl = DefaultEditor::new()?;
    let environment = make_a_lisp_rs::Env::new(None);
    make_a_lisp_rs::load_builtins(environment.borrow_mut().deref_mut());

    loop {
        // Read the input
        let input = rl.readline("user> ");
        match input {
            Ok(line) => {
                // Save the input in the history
                rl.add_history_entry(line.as_str())?;

                // Process the line
                match rep(line, environment.clone()) {
                    Ok(output) => println!("{}", output),
                    Err(e) => eprintln!("{}", e.red()),
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
