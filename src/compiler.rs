use crate::{
    chunk::{Chunk, OpCode},
    scanner::{Scanner, Token, TokenType},
    value::Value,
    vm::{InterpretError, InterpretResult},
};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    pub fn next(&self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => unreachable!(),
        }
    }

    pub fn from_token(typ: TokenType) -> Self {
        match typ {
            TokenType::LParen => Precedence::None,
            TokenType::RParen => Precedence::None,
            TokenType::LBrace => Precedence::None,
            TokenType::RBrace => Precedence::None,
            TokenType::Comma => Precedence::None,
            TokenType::Dot => Precedence::None,
            TokenType::Minus => Precedence::Term,
            TokenType::Plus => Precedence::Term,
            TokenType::Semicolon => Precedence::None,
            TokenType::Slash => Precedence::Factor,
            TokenType::Star => Precedence::Factor,
            TokenType::Bang => Precedence::None,
            TokenType::BangEqual => Precedence::None,
            TokenType::Equal => Precedence::None,
            TokenType::EqualEqual => Precedence::None,
            TokenType::Greater => Precedence::None,
            TokenType::GreaterEqual => Precedence::None,
            TokenType::Less => Precedence::None,
            TokenType::LessEqual => Precedence::None,
            TokenType::Identifier => Precedence::None,
            TokenType::String => Precedence::None,
            TokenType::Number => Precedence::None,
            TokenType::And => Precedence::None,
            TokenType::Class => Precedence::None,
            TokenType::Else => Precedence::None,
            TokenType::False => Precedence::None,
            TokenType::For => Precedence::None,
            TokenType::Fun => Precedence::None,
            TokenType::If => Precedence::None,
            TokenType::Nil => Precedence::None,
            TokenType::Or => Precedence::None,
            TokenType::Print => Precedence::None,
            TokenType::Return => Precedence::None,
            TokenType::Super => Precedence::None,
            TokenType::This => Precedence::None,
            TokenType::True => Precedence::None,
            TokenType::Var => Precedence::None,
            TokenType::While => Precedence::None,
            TokenType::Error => Precedence::None,
            TokenType::Eof => Precedence::None,
        }
    }
}

#[derive(Default, Debug)]
pub struct Parser<'input> {
    pub current: Token<'input>,
    pub previous: Token<'input>,
    pub had_error: bool,
    pub panic_mode: bool,
}

pub struct Compiler<'input> {
    scanner: Scanner<'input>,
    parser: Parser<'input>,
    compiling_chunk: Chunk,
}

impl<'input> Compiler<'input> {
    pub fn new(src: &'input str) -> Self {
        Self {
            scanner: Scanner::new(src),
            parser: Parser::default(),
            compiling_chunk: Chunk::new(),
        }
    }

    pub fn compile(mut self) -> InterpretResult<Chunk> {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();

        if self.parser.had_error {
            Err(InterpretError::Compile)
        } else {
            Ok(self.compiling_chunk)
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        match self.prefix_rule(self.parser.previous.typ) {
            Some(rule) => rule(self),
            None => {
                self.error("Expect expression.");
                return;
            }
        }

        while precedence <= Precedence::from_token(self.parser.current.typ) {
            self.advance();
            let rule = self.infix_rule(self.parser.previous.typ).expect("infix op");
            rule(self);
        }
    }

    fn prefix_rule(&self, typ: TokenType) -> Option<fn(&mut Self) -> ()> {
        match typ {
            TokenType::LParen => Some(Compiler::grouping),
            TokenType::RParen => None,
            TokenType::LBrace => None,
            TokenType::RBrace => None,
            TokenType::Comma => None,
            TokenType::Dot => None,
            TokenType::Minus => Some(Compiler::unary),
            TokenType::Plus => None,
            TokenType::Semicolon => None,
            TokenType::Slash => None,
            TokenType::Star => None,
            TokenType::Bang => None,
            TokenType::BangEqual => None,
            TokenType::Equal => None,
            TokenType::EqualEqual => None,
            TokenType::Greater => None,
            TokenType::GreaterEqual => None,
            TokenType::Less => None,
            TokenType::LessEqual => None,
            TokenType::Identifier => None,
            TokenType::String => None,
            TokenType::Number => Some(Compiler::number),
            TokenType::And => None,
            TokenType::Class => None,
            TokenType::Else => None,
            TokenType::False => None,
            TokenType::For => None,
            TokenType::Fun => None,
            TokenType::If => None,
            TokenType::Nil => None,
            TokenType::Or => None,
            TokenType::Print => None,
            TokenType::Return => None,
            TokenType::Super => None,
            TokenType::This => None,
            TokenType::True => None,
            TokenType::Var => None,
            TokenType::While => None,
            TokenType::Error => None,
            TokenType::Eof => None,
        }
    }

    fn infix_rule(&self, typ: TokenType) -> Option<fn(&mut Self) -> ()> {
        match typ {
            TokenType::LParen => None,
            TokenType::RParen => None,
            TokenType::LBrace => None,
            TokenType::RBrace => None,
            TokenType::Comma => None,
            TokenType::Dot => None,
            TokenType::Minus => Some(Compiler::binary),
            TokenType::Plus => Some(Compiler::binary),
            TokenType::Semicolon => None,
            TokenType::Slash => Some(Compiler::binary),
            TokenType::Star => Some(Compiler::binary),
            TokenType::Bang => None,
            TokenType::BangEqual => None,
            TokenType::Equal => None,
            TokenType::EqualEqual => None,
            TokenType::Greater => None,
            TokenType::GreaterEqual => None,
            TokenType::Less => None,
            TokenType::LessEqual => None,
            TokenType::Identifier => None,
            TokenType::String => None,
            TokenType::Number => None,
            TokenType::And => None,
            TokenType::Class => None,
            TokenType::Else => None,
            TokenType::False => None,
            TokenType::For => None,
            TokenType::Fun => None,
            TokenType::If => None,
            TokenType::Nil => None,
            TokenType::Or => None,
            TokenType::Print => None,
            TokenType::Return => None,
            TokenType::Super => None,
            TokenType::This => None,
            TokenType::True => None,
            TokenType::Var => None,
            TokenType::While => None,
            TokenType::Error => None,
            TokenType::Eof => None,
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn number(&mut self) {
        let value: f64 = self.parser.previous.src.parse().expect("a number");
        self.emit_constant(Value::Num(value))
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RParen, "Expect ')' after expression.")
    }

    fn unary(&mut self) {
        let typ = self.parser.previous.typ;
        self.parse_precedence(Precedence::Unary);

        match typ {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        let typ = self.parser.previous.typ;
        let precedence = Precedence::from_token(typ);
        self.parse_precedence(precedence.next());

        match typ {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => unreachable!(),
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return()
    }

    fn emit_byte(&mut self, byte: OpCode) {
        self.compiling_chunk
            .write_chunk(byte, self.parser.previous.line)
    }

    fn emit_bytes(&mut self, b1: OpCode, b2: OpCode) {
        self.emit_byte(b1);
        self.emit_byte(b2)
    }

    fn emit_constant(&mut self, value: Value) {
        self.compiling_chunk
            .write_constant(value, self.parser.previous.line)
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return)
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current;

        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.typ != TokenType::Error {
                break;
            } else {
                self.error_at_current(self.parser.current.src)
            }
        }
    }

    fn consume(&mut self, typ: TokenType, msg: &str) {
        if self.parser.current.typ == typ {
            self.advance()
        } else {
            self.error_at_current(msg)
        }
    }

    fn error_at_current(&mut self, msg: &str) {
        self.error_at(self.parser.current, msg)
    }

    fn error(&mut self, msg: &str) {
        self.error_at(self.parser.previous, msg)
    }

    fn error_at(&mut self, token: Token, msg: &str) {
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        match token.typ {
            TokenType::Eof => eprint!(" at end"),
            TokenType::Error => {}
            _ => eprint!(" at {}", token.src),
        }

        eprintln!(": {msg}");
        self.parser.had_error = true
    }
}
