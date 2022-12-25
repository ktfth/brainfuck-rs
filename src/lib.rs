const EMPTY: &'static str = "";

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
    self.input.split(EMPTY).into_iter().enumerate().filter(|(i, c)| {
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
        _ => panic!("Unexpected character: {} at {}", c, i + 1),
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
pub enum AstNodeType {
  CellIncrement,
  CellDecrement,
  PointerIncrement,
  PointerDecrement,
  Output,
  Input,
  Loop(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ast {
  pub body: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
  pub token: Token,
  pub kind: AstNodeType,
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
        kind: AstNodeType::CellIncrement,
        body: None,
      },
      TokenType::DecrementValue => Node {
        token,
        kind: AstNodeType::CellDecrement,
        body: None,
      },
      TokenType::IncrementPointer => Node {
        token,
        kind: AstNodeType::PointerIncrement,
        body: None,
      },
      TokenType::DecrementPointer => Node {
        token,
        kind: AstNodeType::PointerDecrement,
        body: None,
      },
      TokenType::Output => Node {
        token,
        kind: AstNodeType::Output,
        body: None,
      },
      TokenType::Input => Node {
        token,
        kind: AstNodeType::Input,
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
          kind: AstNodeType::Loop(false),
          body: Some(Box::new(ast)),
        }
      },
      TokenType::LoopEnd => {
        Node {
          token,
          kind: AstNodeType::Loop(true),
          body: None,
        }
      },
      TokenType::WhiteSpace => self.expression(),
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
  pub root_pointer: usize,
}

impl Interpreter {
  pub fn new(ast: Ast, cells: Option<Vec<u8>>, pointer: Option<usize>, root_pointer: Option<usize>) -> Interpreter {
    match cells {
      Some(cells) => Interpreter {
        ast,
        cells,
        pointer: pointer.unwrap_or(0),
        root_pointer: root_pointer.unwrap_or(0),
      },
      None => Interpreter {
        ast,
        cells: vec![0; 30000],
        pointer: pointer.unwrap_or(0),
        root_pointer: root_pointer.unwrap_or(0),
      },
    }
  }

  pub fn interpret(&mut self) {
    for node in self.ast.body.iter() {
      match node.kind {
        AstNodeType::CellIncrement => self.cells[self.pointer] += 1,
        AstNodeType::CellDecrement => self.cells[self.pointer] -= 1,
        AstNodeType::PointerIncrement => {
          self.root_pointer += 1;
          self.pointer += 1;
        },
        AstNodeType::PointerDecrement => {
          self.root_pointer -= 1;
          self.pointer -= 1;
        },
        AstNodeType::Output => {
          if self.cells[self.pointer] > 0 {
            print!("{}", self.cells[self.pointer] as char);
          } else {
            println!("");
          }
        },
        AstNodeType::Input => {
          let mut input = String::new();
          std::io::stdin().read_line(&mut input).unwrap();
          self.cells[self.pointer] = input.chars().next().unwrap() as u8;
        },
        AstNodeType::Loop(_) => {
          for sub_node in node.body.clone().unwrap().body {
            match sub_node.kind {
              AstNodeType::CellIncrement => self.cells[self.pointer] += self.cells[self.root_pointer],
              AstNodeType::CellDecrement => self.cells[self.pointer] -= self.cells[self.root_pointer],
              AstNodeType::PointerIncrement => self.pointer += 1,
              AstNodeType::PointerDecrement => self.pointer -= 1,
              AstNodeType::Output => {
                if self.cells[self.pointer] > 0 {
                  print!("{}", self.cells[self.pointer] as char);
                } else {
                  println!("");
                }
              },
              AstNodeType::Input => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                self.cells[self.pointer] = input.chars().next().unwrap() as u8;
              },
              AstNodeType::Loop(_) => {
                let mut interpreter = Interpreter::new(Ast {
                  body: vec![sub_node],
                }, Some(self.cells.clone()), Some(self.pointer), Some(self.root_pointer));
                interpreter.interpret();
              },
            }      
          }
        },
      }
    }
  }
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
        kind: AstNodeType::CellIncrement,
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
    let mut interpreter = Interpreter::new(ast, None, None, None);
    interpreter.interpret();
  }
}