use std::process::ExitCode;

use rustyline::{DefaultEditor, error::ReadlineError};

fn main() -> ExitCode {
    let mut editor = DefaultEditor::new().unwrap();

    loop {
        match editor.readline("❯ ") {
            Ok(input) if !input.is_empty() => {
                editor.add_history_entry(&input).unwrap();
                println!("input: {input}");
            }
            Ok(_) => {}
            Err(ReadlineError::Eof) => {
                println!("Finished");
                break ExitCode::SUCCESS;
            }
            Err(error) => {
                eprintln!("Error: {error}");
                break ExitCode::FAILURE;
            }
        }
    }
}
