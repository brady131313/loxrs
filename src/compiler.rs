use crate::{
    chunk::{Chunk, OpCode},
    scanner::{Scanner, Token, TokenType},
    value::Value,
    vm::{InterpretError, InterpretResult},
};

#[derive(Clone, Copy)]
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
    Primary
}

#[derive(Default)]
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

        if self.parser.had_error {
            Err(InterpretError::Compile)
        } else {
            Ok(self.compiling_chunk)
        }
    }

    fn expression(&mut self) {
        unimplemented!()
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
        self.expression();

        match typ {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
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
