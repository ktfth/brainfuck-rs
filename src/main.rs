mod lib;

use std::{io};

use lib::{Lexer, Parser, Interpreter};

fn main() -> io::Result<()> {
    let complete_path = std::env::current_dir().unwrap();
    let file_path = std::env::args().nth(1).unwrap();
    let input = std::fs::read_to_string(complete_path.join(file_path)).unwrap();
    let sanitized_input = input.split("\n").into_iter().map(|line| line.trim()).collect::<Vec<&str>>().join("");
    let mut lexer = Lexer::new(&sanitized_input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut stdout = io::stdout();
    let mut interpreter = Interpreter::new(ast, &mut stdout);
    interpreter.interpret(None)?;
    Ok(())
}
