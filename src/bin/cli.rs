use color_eyre::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut rl = DefaultEditor::new()?;

    loop {
        // Read the input
        let input = rl.readline("user> ");
        match input {
            Ok(line) => {
                // Save the input in the history
                rl.add_history_entry(line.as_str())?;

                // Process the line
                let output = make_a_lisp_rs::rep(line);

                // Print the output
                println!("{}", output);
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
