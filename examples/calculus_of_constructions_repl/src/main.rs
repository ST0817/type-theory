mod parser;

use std::process::ExitCode;

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};
use calculus_of_constructions::check::{Context, check_axiom, check_def, check_term};
use chumsky::Parser;
use parsers::{Error, Result};
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::parser::ReplCmd;

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

fn repl_process<'src>(input: &'src str, context: &mut Context) -> Result<'src, ()> {
    match parser::repl_cmd().parse(input).into_result()? {
        ReplCmd::Def { name, term } => check_def(name, &term, context),
        ReplCmd::Axiom { name, term } => check_axiom(name, &term, context),
        ReplCmd::Term { term } => {
            let (checked_term, ty) = check_term(&term, context)?;
            println!("{checked_term}");
            println!("=> {ty}");
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    let mut editor = DefaultEditor::new().unwrap();
    let mut context = Context::new();

    loop {
        match editor.readline("❯ ") {
            Ok(input) if !input.is_empty() => {
                editor.add_history_entry(&input).unwrap();

                if let Err(errors) = repl_process(&input, &mut context) {
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
