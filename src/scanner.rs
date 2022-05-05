use std::fmt::Display;

macro_rules! alpha {
    () => {
        'a'..='z' | 'A'..='Z' | '_'
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Single character
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

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
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

    Error,
    Eof,
}

impl Default for TokenType {
    fn default() -> Self {
        Self::Eof
    }
}

#[derive(Default, Clone, Copy)]
pub struct Token<'input> {
    pub typ: TokenType,
    pub src: &'input str,
    pub line: usize,
}

pub struct Scanner<'input> {
    src: &'input str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'input> Scanner<'input> {
    pub fn new(src: &'input str) -> Self {
        Self {
            src,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token<'input> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        match self.advance() {
            '(' => self.make_token(TokenType::LParen),
            ')' => self.make_token(TokenType::RParen),
            '{' => self.make_token(TokenType::LBrace),
            '}' => self.make_token(TokenType::RBrace),
            ';' => self.make_token(TokenType::Semicolon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '!' => {
                let typ = if self.matchh('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.make_token(typ)
            }
            '=' => {
                let typ = if self.matchh('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.make_token(typ)
            }
            '<' => {
                let typ = if self.matchh('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.make_token(typ)
            }
            '>' => {
                let typ = if self.matchh('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.make_token(typ)
            }
            '"' => self.string(),
            '0'..='9' => self.number(),
            alpha!() => self.identifier(),
            _ => self.error_token("Unexpected character."),
        }
    }

    fn string(&mut self) -> Token<'input> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error_token("Unterminated string.")
        } else {
            self.advance();
            self.make_token(TokenType::String)
        }
    }

    fn number(&mut self) -> Token<'input> {
        while is_digit(self.peek()) {
            self.advance();
        }

        // Look for fractional part
        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance(); // consume .

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token<'input> {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }

        let typ = self.identifier_type();
        self.make_token(typ)
    }

    fn identifier_type(&self) -> TokenType {
        match self.char_at(self.start) {
            'a' => self.check_keyword(1, "nd", TokenType::And),
            'c' => self.check_keyword(1, "lass", TokenType::Class),
            'e' => self.check_keyword(1, "lse", TokenType::Else),
            'f' => match self.char_at(self.start + 1) {
                'a' => self.check_keyword(2, "lse", TokenType::False),
                'o' => self.check_keyword(2, "r", TokenType::For),
                'u' => self.check_keyword(2, "n", TokenType::Fun),
                _ => TokenType::Identifier,
            },
            'i' => self.check_keyword(1, "f", TokenType::If),
            'n' => self.check_keyword(1, "il", TokenType::Nil),
            'o' => self.check_keyword(1, "r", TokenType::Or),
            'p' => self.check_keyword(1, "rint", TokenType::Print),
            'r' => self.check_keyword(1, "eturn", TokenType::Return),
            's' => self.check_keyword(1, "uper", TokenType::Super),
            't' => match self.char_at(self.start + 1) {
                'h' => self.check_keyword(2, "is", TokenType::This),
                'r' => self.check_keyword(2, "ue", TokenType::True),
                _ => TokenType::Identifier,
            },
            'v' => self.check_keyword(1, "ar", TokenType::Var),
            'w' => self.check_keyword(1, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, rest: &str, typ: TokenType) -> TokenType {
        let start = self.start + start;
        if &self.src[start..self.current] == rest {
            typ
        } else {
            TokenType::Identifier
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn char_at(&self, idx: usize) -> char {
        self.src[idx..].chars().next().unwrap_or('\0')
    }

    fn peek(&self) -> char {
        self.char_at(self.current)
    }

    fn peek_next(&self) -> char {
        self.char_at(self.current + 1)
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.char_at(self.current - 1)
    }

    fn is_at_end(&self) -> bool {
        self.peek() == '\0'
    }

    fn matchh(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn make_token(&self, typ: TokenType) -> Token<'input> {
        Token {
            typ,
            src: &self.src[self.start..self.current],
            line: self.line,
        }
    }

    fn error_token(&'input self, msg: &'static str) -> Token<'static> {
        Token {
            typ: TokenType::Error,
            src: msg,
            line: self.line,
        }
    }
}

fn is_digit(c: char) -> bool {
    matches!(c, '0'..='9')
}

fn is_alpha(c: char) -> bool {
    matches!(c, alpha!())
}

impl<'input> Display for Token<'input> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<10?} {}", self.typ, self.src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner() {
        let src = r#"(){},.-+;/*   ! != == = > >= < <= test "string" 5.0
                     and class else false for fun if nil or print return
                     super this true var while"#;
        let mut scanner = Scanner::new(&src);

        let tokens = [
            // Single character
            TokenType::LParen,
            TokenType::RParen,
            TokenType::LBrace,
            TokenType::RBrace,
            TokenType::Comma,
            TokenType::Dot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Semicolon,
            TokenType::Slash,
            TokenType::Star,
            // One or two character tokens
            TokenType::Bang,
            TokenType::BangEqual,
            TokenType::EqualEqual,
            TokenType::Equal,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
            // Literals
            TokenType::Identifier,
            TokenType::String,
            TokenType::Number,
            // Keywords
            TokenType::And,
            TokenType::Class,
            TokenType::Else,
            TokenType::False,
            TokenType::For,
            TokenType::Fun,
            TokenType::If,
            TokenType::Nil,
            TokenType::Or,
            TokenType::Print,
            TokenType::Return,
            TokenType::Super,
            TokenType::This,
            TokenType::True,
            TokenType::Var,
            TokenType::While,
        ];

        for typ in tokens {
            assert_eq!(scanner.scan_token().typ, typ);
        }
    }
}
