use std::process::ExitCode;

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};
use chumsky::Parser;
use elaborator_kernel::{elab::elab_expr_syntax, kernel::infer_type, parser};
use parsers::{Error, Result};
use rustyline::{DefaultEditor, error::ReadlineError};

const REPL_ID: &str = "REPL";

fn print_errors(errors: &Vec<Error>, id: &str, src: &str) {
    for error in errors {
        Report::build(ReportKind::Error, (id, error.span().into_range()))
            .with_config(Config::new().with_index_type(IndexType::Byte))
            .with_message(error.to_string())
            .with_label(
                Label::new((id, error.span().into_range()))
                    .with_message(error.reason().to_string())
                    .with_color(Color::Red),
            )
            .finish()
            .print((id, Source::from(src)))
            .unwrap();
    }
}

fn repl_process<'src>(input: &'src str) -> Result<'src, ()> {
    let expr_syntax = parser::expr_syntax().parse(input).into_result()?;
    let expr = elab_expr_syntax(&expr_syntax);
    let typ = infer_type(&expr);
    println!("{typ}");
    Ok(())
}

fn main() -> ExitCode {
    let mut editor = DefaultEditor::new().unwrap();

    loop {
        match editor.readline("❯ ") {
            Ok(input) if !input.is_empty() => {
                editor.add_history_entry(&input).unwrap();

                if let Err(errors) = repl_process(&input) {
                    print_errors(&errors, REPL_ID, &input);
                }
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
