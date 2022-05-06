use crate::{
    chunk::{Chunk, OpCode},
    scanner::{Scanner, Token, TokenType},
    value::Value,
    vm::{InterpretError, InterpretResult},
};

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
}

type Rule<'a, 'b> = fn(&'a mut Compiler<'b>) -> ();

#[derive(Clone, Copy)]
enum RuleType {
    Prefix,
    Infix,
    Precedence,
}

enum ParseRule<'a, 'b> {
    Rule(Option<Rule<'a, 'b>>),
    Precedence(Precedence),
}

impl<'a, 'b> ParseRule<'a, 'b> {
    pub fn as_rule(self) -> Option<Rule<'a, 'b>> {
        if let ParseRule::Rule(rule) = self {
            rule
        } else {
            unimplemented!()
        }
    }

    pub fn as_precedence(self) -> Precedence {
        if let ParseRule::Precedence(prec) = self {
            prec
        } else {
            unimplemented!()
        }
    }
}

fn get_rule<'a, 'b>(typ: TokenType, rule_type: RuleType) -> ParseRule<'a, 'b> {
    macro_rules! rule {
        ($prefix:expr, $infix:expr, $precedence:expr) => {
            match rule_type {
                RuleType::Prefix => ParseRule::Rule($prefix),
                RuleType::Infix => ParseRule::Rule($infix),
                RuleType::Precedence => ParseRule::Precedence($precedence),
            }
        };
    }

    match typ {
        TokenType::LParen => rule!(Some(Compiler::grouping), None, Precedence::None),
        TokenType::RParen => rule!(None, None, Precedence::None),
        TokenType::LBrace => rule!(None, None, Precedence::None),
        TokenType::RBrace => rule!(None, None, Precedence::None),
        TokenType::Comma => rule!(None, None, Precedence::None),
        TokenType::Dot => rule!(None, None, Precedence::None),
        TokenType::Minus => rule!(
            Some(Compiler::unary),
            Some(Compiler::binary),
            Precedence::Term
        ),
        TokenType::Plus => rule!(None, Some(Compiler::binary), Precedence::Term),
        TokenType::Semicolon => rule!(None, None, Precedence::None),
        TokenType::Slash => rule!(None, Some(Compiler::binary), Precedence::Factor),
        TokenType::Star => rule!(None, Some(Compiler::binary), Precedence::Factor),
        TokenType::Bang => rule!(None, None, Precedence::None),
        TokenType::BangEqual => rule!(None, None, Precedence::None),
        TokenType::Equal => rule!(None, None, Precedence::None),
        TokenType::EqualEqual => rule!(None, None, Precedence::None),
        TokenType::Greater => rule!(None, None, Precedence::None),
        TokenType::GreaterEqual => rule!(None, None, Precedence::None),
        TokenType::Less => rule!(None, None, Precedence::None),
        TokenType::LessEqual => rule!(None, None, Precedence::None),
        TokenType::Identifier => rule!(None, None, Precedence::None),
        TokenType::String => rule!(None, None, Precedence::None),
        TokenType::Number => rule!(Some(Compiler::number), None, Precedence::None),
        TokenType::And => rule!(None, None, Precedence::None),
        TokenType::Class => rule!(None, None, Precedence::None),
        TokenType::Else => rule!(None, None, Precedence::None),
        TokenType::False => rule!(Some(Compiler::literal), None, Precedence::None),
        TokenType::For => rule!(None, None, Precedence::None),
        TokenType::Fun => rule!(None, None, Precedence::None),
        TokenType::If => rule!(None, None, Precedence::None),
        TokenType::Nil => rule!(Some(Compiler::literal), None, Precedence::None),
        TokenType::Or => rule!(None, None, Precedence::None),
        TokenType::Print => rule!(None, None, Precedence::None),
        TokenType::Return => rule!(None, None, Precedence::None),
        TokenType::Super => rule!(None, None, Precedence::None),
        TokenType::This => rule!(None, None, Precedence::None),
        TokenType::True => rule!(Some(Compiler::literal), None, Precedence::None),
        TokenType::Var => rule!(None, None, Precedence::None),
        TokenType::While => rule!(None, None, Precedence::None),
        TokenType::Error => rule!(None, None, Precedence::None),
        TokenType::Eof => rule!(None, None, Precedence::None),
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
        match get_rule(self.parser.previous.typ, RuleType::Prefix).as_rule() {
            Some(rule) => rule(self),
            None => {
                self.error("Expect expression.");
                return;
            }
        };

        while precedence <= get_rule(self.parser.current.typ, RuleType::Precedence).as_precedence()
        {
            self.advance();
            let rule = get_rule(self.parser.previous.typ, RuleType::Infix)
                .as_rule()
                .expect("an infix parse rule");

            rule(self)
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
        let precedence = get_rule(typ, RuleType::Precedence).as_precedence();
        self.parse_precedence(precedence.next());

        match typ {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self) {
        match self.parser.previous.typ {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => unreachable!()
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        #[cfg(feature = "debug_print_code")]
        if !self.parser.had_error {
            self.compiling_chunk.disassemble_chunk("code");
        }
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
