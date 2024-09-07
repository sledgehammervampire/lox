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
        while let Some(b) = self.advance() {
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
                    // is comment
                    if self.advance_if_match(b'/') {
                        loop {
                            match self.peek() {
                                Some(c) if c != b'\n' => {}
                                _ => break,
                            }
                        }
                    } else {
                        tokens.push(self.make_token(TokenType::Slash));
                    }
                }
                b' ' | b'\r' | b'\t' => {}
                b'\n' => {
                    self.line += 1;
                }
                b'"' => {
                    if let Some(token) = self.scan_str() {
                        tokens.push(token);
                    }
                }
                b if b.is_ascii_digit() => {
                    if let Some(token) = self.scan_num() {
                        tokens.push(token);
                    }
                }
                _ => {
                    self.errors.push(ScanError {
                        line: self.line,
                        message: "Unexpected character.".to_string(),
                    });
                }
            }

            self.start = self.curr;
        }
        tokens.push(self.make_token(TokenType::Eof));
        tokens
    }

    fn scan_str<'a>(&'a mut self) -> Option<Token<'src>> {
        loop {
            match self.advance() {
                None => {
                    self.errors.push(ScanError {
                        line: self.line,
                        message: "Unterminated string.".to_string(),
                    });
                    break None;
                }
                Some(b'\n') => {
                    self.line += 1;
                }
                Some(b'"') => {
                    break Some(
                        self.make_token(TokenType::Str(&self.src[self.start + 1..self.curr - 1])),
                    );
                }
                _ => {}
            }
        }
    }

    fn scan_num<'a>(&'a mut self) -> Option<Token<'src>> {
        loop {
            match self.peek() {
                Some(c) if c.is_ascii_digit() => {
                    self.advance();
                }
                _ => break,
            }
        }
        if self.peek() == Some(b'.') && self.peek_next().map_or(false, |b| b.is_ascii_digit()) {
            self.advance();
            loop {
                match self.peek() {
                    Some(c) if c.is_ascii_digit() => {
                        self.advance();
                    }
                    _ => break,
                }
            }
        }
        Some(self.make_token(TokenType::Number(
            self.src[self.start..self.curr].parse().ok()?,
        )))
    }

    fn peek(&self) -> Option<u8> {
        self.src.bytes().nth(self.curr)
    }

    fn peek_next(&self) -> Option<u8> {
        self.src.bytes().nth(self.curr + 1)
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.src.bytes().nth(self.curr)?;
        self.curr += 1;
        Some(b)
    }

    fn advance_if_match(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.curr += 1;
            true
        } else {
            false
        }
    }

    fn make_token<'a>(&'a self, typ: TokenType<'src>) -> Token<'src> {
        Token {
            typ,
            lexeme: &self.src[self.start..self.curr],
            line: self.line,
        }
    }
}

#[derive(Debug)]
struct Token<'src> {
    typ: TokenType<'src>,
    lexeme: &'src str,
    line: usize,
}

#[derive(Debug)]
enum TokenType<'src> {
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
    Str(&'src str),
    Number(f64),

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
