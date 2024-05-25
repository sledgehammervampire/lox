use std::{
    env,
    fmt::Display,
    fs,
    io::{self, Write},
    process,
};

fn main() -> Result<(), io::Error> {
    let args: Vec<_> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        process::exit(64);
    }
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    match args.get(1) {
        Some(f) => {
            let prog = fs::read_to_string(f)?;
            run(&prog);
        }
        None => {
            let mut line = String::new();
            loop {
                write!(&mut stdout, "> ")?;
                stdout.flush()?;
                let n = stdin.read_line(&mut line)?;
                if n == 0 {
                    break;
                }
                run(&line);
                line.clear();
            }
        }
    }
    Ok(())
}

fn run(src: &str) {
    let mut scanner = Scanner::new(src);
    let tokens = scanner.scan();
    dbg!(tokens);
}

struct Scanner<'src> {
    src: &'src str,
    start: usize,
    curr: usize,
    line: usize,
    errors: Vec<ScanError>,
}

impl<'src> Scanner<'src> {
    fn new(src: &str) -> Scanner {
        Scanner {
            src,
            start: 0,
            curr: 0,
            line: 1,
            errors: vec![],
        }
    }

    fn scan(&mut self) -> Vec<Token<'_>> {
        let mut tokens = vec![];
        while !self.is_at_end() {
            self.start = self.curr;

            let b = self.advance();
            match b {
                b'(' => tokens.push(self.make_token(TokenType::LParen)),
                b')' => tokens.push(self.make_token(TokenType::RParen)),
                b'{' => tokens.push(self.make_token(TokenType::LBrace)),
                b'}' => tokens.push(self.make_token(TokenType::RBrace)),
                b',' => tokens.push(self.make_token(TokenType::Comma)),
                b'.' => tokens.push(self.make_token(TokenType::Dot)),
                b'-' => tokens.push(self.make_token(TokenType::Minus)),
                b'+' => tokens.push(self.make_token(TokenType::Plus)),
                b';' => tokens.push(self.make_token(TokenType::Semicolon)),
                b'*' => tokens.push(self.make_token(TokenType::Star)),
                b'!' => {
                    let typ = if self.advance_if_match(b'=') {
                        TokenType::BangEqual
                    } else {
                        TokenType::Bang
                    };
                    tokens.push(self.make_token(typ))
                }
                b'=' => {
                    let typ = if self.advance_if_match(b'=') {
                        TokenType::EqualEqual
                    } else {
                        TokenType::Equal
                    };
                    tokens.push(self.make_token(typ))
                }
                b'<' => {
                    let typ = if self.advance_if_match(b'=') {
                        TokenType::LessEqual
                    } else {
                        TokenType::Less
                    };
                    tokens.push(self.make_token(typ))
                }
                b'>' => {
                    let typ = if self.advance_if_match(b'=') {
                        TokenType::GreaterEqual
                    } else {
                        TokenType::Greater
                    };
                    tokens.push(self.make_token(typ))
                }
                b'/' => {
                    if self.advance_if_match(b'/') {
                        while !self.is_at_end() && self.src.bytes().nth(self.curr).unwrap() != b'\n'
                        {
                            self.curr += 1;
                        }
                    } else {
                        tokens.push(self.make_token(TokenType::Slash));
                    }
                }
                b' ' | b'\r' | b'\t' => {}
                b'\n' => {
                    self.line += 1;
                }
                _ => {
                    self.errors.push(ScanError {
                        line: self.line,
                        message: "Unexpected character.".to_string(),
                    });
                }
            }
        }
        tokens.push(Token {
            typ: TokenType::Eof,
            lexeme: "",
            line: self.line,
        });
        tokens
    }

    fn advance(&mut self) -> u8 {
        let b = self.src.bytes().nth(self.curr).unwrap();
        self.curr += 1;
        b
    }

    fn advance_if_match(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.src.bytes().nth(self.curr).unwrap() != expected {
            false
        } else {
            self.curr += 1;
            true
        }
    }

    fn make_token<'a>(&'a self, typ: TokenType) -> Token<'src> {
        Token {
            typ,
            lexeme: &self.src[self.start..self.curr],
            line: self.line,
        }
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.src.len()
    }
}

#[derive(Debug)]
struct Token<'src> {
    typ: TokenType,
    lexeme: &'src str,
    line: usize,
}

#[derive(Debug)]
enum TokenType {
    // Single-character tokens.
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Ident,
    Str,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug)]
struct ScanError {
    line: usize,
    message: String,
}

impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.message)
    }
}
