use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
  IncrementPointer,
  DecrementPointer,
  IncrementValue,
  DecrementValue,
  Output,
  Input,
  LoopStart,
  LoopEnd,
  WhiteSpace,
  Ignore,
  EOF,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
  pub pos: usize,
  pub kind: TokenType,
  pub lexeme: char,
}

pub struct Lexer {
  pub input: String,
}

impl Lexer {
  pub fn new(input: &str) -> Lexer {
    Lexer {
      input: input.to_string(),
    }
  }

  pub fn eat_first_emptyspace(&mut self) -> Vec<&str> {
    self.input.split("").into_iter().enumerate().filter(|(i, c)| {
      let c = *c;
      !(*i == 0 && c.is_empty())
    }).map(|(_, c)| c).collect::<Vec<&str>>()
  }

  pub fn tokenize(&mut self) -> Vec<Token> {
    self.eat_first_emptyspace().into_iter().enumerate().map(|(i, c)| {
      let kind = match c {
        ">" => TokenType::IncrementPointer,
        "<" => TokenType::DecrementPointer,
        "+" => TokenType::IncrementValue,
        "-" => TokenType::DecrementValue,
        "." => TokenType::Output,
        "," => TokenType::Input,
        "[" => TokenType::LoopStart,
        "]" => TokenType::LoopEnd,
        "" => TokenType::EOF,
        " " => TokenType::WhiteSpace,
        _ => TokenType::Ignore,
      };

      match kind {
        TokenType::EOF => Token {
          pos: i + 1,
          kind,
          lexeme: '\0',
        },
        _ => Token {
          pos: i + 1,
          kind,
          lexeme: c.chars().next().unwrap(),
        },
      }
    }).collect::<Vec<Token>>()   
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
  CellIncrement,
  CellDecrement,
  PointerIncrement,
  PointerDecrement,
  Output,
  Input,
  #[allow(dead_code)]
  Ignore,
  #[allow(dead_code)]
  WhiteSpace,
  LoopStart,
  LoopEnd,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ast {
  pub body: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
  pub token: Token,
  pub kind: NodeType,
  pub body: Option<Box<Ast>>,
}

pub struct Parser {
  pub tokens: Vec<Token>,
  pub current: usize,
}

impl Parser {
  pub fn new(tokens: Vec<Token>) -> Parser {
    Parser {
      tokens,
      current: 0,
    }
  }

  pub fn parse(&mut self) -> Ast {
    let mut ast = Ast {
      body: vec![],
    };

    while !self.is_at_end() {
      ast.body.push(self.expression());
    }

    ast
  }

  fn expression(&mut self) -> Node {
    let token = self.advance();

    match token.kind {
      TokenType::IncrementValue => Node {
        token,
        kind: NodeType::CellIncrement,
        body: None,
      },
      TokenType::DecrementValue => Node {
        token,
        kind: NodeType::CellDecrement,
        body: None,
      },
      TokenType::IncrementPointer => Node {
        token,
        kind: NodeType::PointerIncrement,
        body: None,
      },
      TokenType::DecrementPointer => Node {
        token,
        kind: NodeType::PointerDecrement,
        body: None,
      },
      TokenType::Output => Node {
        token,
        kind: NodeType::Output,
        body: None,
      },
      TokenType::Input => Node {
        token,
        kind: NodeType::Input,
        body: None,
      },
      TokenType::LoopStart => {
        let mut ast = Ast {
          body: vec![],
        };

        while !self.is_at_end() && self.peek().kind != TokenType::LoopEnd {
          ast.body.push(self.expression());
        }

        self.advance();

        Node {
          token,
          kind: NodeType::LoopStart,
          body: Some(Box::new(ast)),
        }
      },
      TokenType::LoopEnd => {
        Node {
          token,
          kind: NodeType::LoopEnd,
          body: None,
        }
      },
      TokenType::WhiteSpace => {
        Node {
          token,
          kind: NodeType::WhiteSpace,
          body: None,
        }
      },
      TokenType::Ignore => {
        Node {
          token,
          kind: NodeType::Ignore,
          body: None,
        }
      },
      _ => panic!("Unexpected token: {:?}", token),
    }
  }

  fn advance(&mut self) -> Token {
    if !self.is_at_end() {
      self.current += 1;
    }

    self.previous()
  }

  fn is_at_end(&self) -> bool {
    self.peek().kind == TokenType::EOF
  }

  fn peek(&self) -> Token {
    self.tokens[self.current]
  }

  fn previous(&self) -> Token {
    self.tokens[self.current - 1]
  }
}

pub struct Interpreter {
  pub ast: Ast,
  pub cells: Vec<u8>,
  pub pointer: usize,
}

impl Interpreter {
  pub fn new(ast: Ast) -> Interpreter {
    Interpreter {
      ast,
      cells: vec![0; 30_000],
      pointer: 0,
    }
  }

  pub fn interpret(&mut self, nodes: Option<&Vec<Node>>) {
    match nodes {
      Some(body) => {
        for node in body.iter() {
          match node.kind {
            NodeType::Ignore | NodeType::WhiteSpace | NodeType::LoopEnd => {},
            NodeType::CellIncrement => self.cells[self.pointer] += 1,
            NodeType::CellDecrement => self.cells[self.pointer] -= 1,
            NodeType::PointerIncrement => {
              self.pointer += 1;
              if self.pointer >= self.cells.len() {
                self.pointer = 0;
              }
            },
            NodeType::PointerDecrement => {
              if self.pointer == 0 {
                self.pointer = self.cells.len() - 1;
              } else {
                self.pointer -= 1;
              }
            },
            NodeType::Output => {
              if self.cells[self.pointer] != 0 {
                print!("{}", self.cells[self.pointer] as char);
              } else {
                println!("");
              }
            },
            NodeType::Input => {
              let mut input = String::new();
              std::io::stdin().read_line(&mut input).unwrap();
              self.cells[self.pointer] = input.chars().next().unwrap() as u8;
            },
            NodeType::LoopStart => {
              self.interpret_loop(&node.body.as_ref().unwrap().body);
            },
          }
        }
      },
      None => {
        self.interpret(Some(&self.ast.body.clone()));
      },
    }
  }

  pub fn interpret_web(&mut self, nodes: Option<&Vec<Node>>) -> String {
    let mut out = String::new();

    match nodes {
      Some(body) => {
        for node in body.iter() {
          match node.kind {
            NodeType::Ignore | NodeType::WhiteSpace | NodeType::LoopEnd => {},
            NodeType::CellIncrement => self.cells[self.pointer] += 1,
            NodeType::CellDecrement => self.cells[self.pointer] -= 1,
            NodeType::PointerIncrement => {
              self.pointer += 1;
              if self.pointer >= self.cells.len() {
                self.pointer = 0;
              }
            },
            NodeType::PointerDecrement => {
              if self.pointer == 0 {
                self.pointer = self.cells.len() - 1;
              } else {
                self.pointer -= 1;
              }
            },
            NodeType::Output => {
              if self.cells[self.pointer] != 0 {
                out.push(self.cells[self.pointer] as char);
              } else {
                out.push('\n');
              }
            },
            NodeType::Input => {},
            NodeType::LoopStart => {
              out.push_str(self.interpret_web_loop(&node.body.as_ref().unwrap().body).as_str());
            },
          }
        }
      },
      None => {
        out.push_str(self.interpret_web(Some(&self.ast.body.clone())).as_str());
      },
    }

    out
  }

  fn interpret_loop(&mut self, nodes: &Vec<Node>) {
    while self.cells[self.pointer] != 0 {
      self.interpret(Some(nodes));
    }
  }

  fn interpret_web_loop(&mut self, nodes: &Vec<Node>) -> String {
    let mut out = String::new();

    while self.cells[self.pointer] != 0 {
      out.push_str(self.interpret_web(Some(nodes)).as_str());
    }

    out
  }
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run(code: &str) -> String {
  let mut lexer = Lexer::new(code);
  let tokens = lexer.tokenize();
  let mut parser = Parser::new(tokens);
  let ast = parser.parse();
  let mut interpreter = Interpreter::new(ast);
  interpreter.interpret_web(None)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tokenize() {
    let mut lexer = Lexer::new("++++++++++[>++++++++>+++++++++++>++++++++++>++++>+++>++++++++>++++++++++++>+++++++++++>++++++++++>+++++++++++>+++>+<<<<<<<<<<<<-]>-.>--.>---.>++++.>++.>---.>---.>.>.>+.>+++.>.");
    let tokens = lexer.tokenize();
    assert_eq!(tokens.len(), 176);
    assert_eq!(tokens[0].kind, TokenType::IncrementValue);
    assert_eq!(tokens[0].lexeme, '+');
  }

  #[test]
  fn test_parser() {
    let mut parser = Parser::new(vec![Token {
      pos: 0,
      kind: TokenType::IncrementValue,
      lexeme: '+',
    }, Token {
      pos: 1,
      kind: TokenType::EOF,
      lexeme: '\0',
    }]);
    let ast = parser.parse();
    assert_eq!(ast, Ast {
      body: vec![Node {
        token: Token {
          pos: 0,
          kind: TokenType::IncrementValue,
          lexeme: '+',
        },
        kind: NodeType::CellIncrement,
        body: None,
      }],
    });
  }

  #[test]
  fn test_interpreter() {
    let mut lexer = Lexer::new("++++++++++[>++++++++>+++++++++++>++++++++++>++++>+++>++++++++>++++++++++++>+++++++++++>++++++++++>+++++++++++>+++>+<<<<<<<<<<<<-]>-.>--.>---.>++++.>++.>---.>---.>.>.>+.>+++.>.");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut interpreter = Interpreter::new(ast);
    interpreter.interpret(None);
  }

  #[test]
  fn test_run() {
    run("++++++++++[>++++++++>+++++++++++>++++++++++>++++>+++>++++++++>++++++++++++>+++++++++++>++++++++++>+++++++++++>+++>+<<<<<<<<<<<<-]>-.>--.>---.>++++.>++.>---.>---.>.>.>+.>+++.>.");
  }
}