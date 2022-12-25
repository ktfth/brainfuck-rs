mod lib;

use lib::{Lexer, Parser, Interpreter};

fn main() {
    let input = std::fs::read_to_string("input.bf").unwrap();
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut interpreter = Interpreter::new(ast, None, None);
    interpreter.interpret();
}
