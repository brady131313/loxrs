use crate::{
    chunk::{Chunk, OpCode},
    object::StringInterner,
    scanner::{Scanner, Token, TokenType},
    util::split_u16,
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

type Rule<'a, 'input, 'vm> = fn(&'a mut Compiler<'input, 'vm>, bool) -> ();

#[derive(Clone, Copy)]
enum RuleType {
    Prefix,
    Infix,
    Precedence,
}

enum ParseRule<'a, 'input, 'vm> {
    Rule(Option<Rule<'a, 'input, 'vm>>),
    Precedence(Precedence),
}

impl<'a, 'input, 'vm> ParseRule<'a, 'input, 'vm> {
    pub fn as_rule(self) -> Option<Rule<'a, 'input, 'vm>> {
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

fn get_rule<'a, 'input, 'vm>(typ: TokenType, rule_type: RuleType) -> ParseRule<'a, 'input, 'vm> {
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
        TokenType::Bang => rule!(Some(Compiler::unary), None, Precedence::None),
        TokenType::BangEqual => rule!(None, Some(Compiler::binary), Precedence::Equality),
        TokenType::Equal => rule!(None, None, Precedence::None),
        TokenType::EqualEqual => rule!(None, Some(Compiler::binary), Precedence::Equality),
        TokenType::Greater => rule!(None, Some(Compiler::binary), Precedence::Comparison),
        TokenType::GreaterEqual => rule!(None, Some(Compiler::binary), Precedence::Comparison),
        TokenType::Less => rule!(None, Some(Compiler::binary), Precedence::Comparison),
        TokenType::LessEqual => rule!(None, Some(Compiler::binary), Precedence::Comparison),
        TokenType::Identifier => rule!(Some(Compiler::variable), None, Precedence::None),
        TokenType::String => rule!(Some(Compiler::string), None, Precedence::None),
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

impl<'input> Parser<'input> {
    fn error_at_current(&mut self, msg: &str) {
        self.error_at(self.current, msg)
    }

    fn error(&mut self, msg: &str) {
        self.error_at(self.previous, msg)
    }

    fn error_at(&mut self, token: Token, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        match token.typ {
            TokenType::Eof => eprint!(" at end"),
            TokenType::Error => {}
            _ => eprint!(" at {}", token.src),
        }

        eprintln!(": {msg}");
        self.had_error = true
    }
}

#[derive(Debug)]
pub struct Local<'input> {
    name: Token<'input>,
    depth: Option<usize>,
}

pub struct Compiler<'input, 'vm> {
    scanner: Scanner<'input>,
    parser: Parser<'input>,
    interner: &'vm mut StringInterner,
    compiling_chunk: Chunk,
    locals: Vec<Local<'input>>,
    scope_depth: usize,
}

impl<'input, 'vm> Compiler<'input, 'vm> {
    pub fn new(src: &'input str, interner: &'vm mut StringInterner) -> Self {
        Self {
            scanner: Scanner::new(src),
            parser: Parser::default(),
            compiling_chunk: Chunk::new(),
            locals: Vec::with_capacity(u8::MAX as usize),
            scope_depth: 0,
            interner,
        }
    }

    pub fn compile(mut self) -> InterpretResult<Chunk> {
        self.advance();
        while !self.matches(TokenType::Eof) {
            self.declaration()
        }
        self.end_compiler();

        if self.parser.had_error {
            Err(InterpretError::Compile)
        } else {
            Ok(self.compiling_chunk)
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let can_assign = precedence <= Precedence::Assignment;
        match get_rule(self.parser.previous.typ, RuleType::Prefix).as_rule() {
            Some(rule) => rule(self, can_assign),
            None => {
                self.parser.error("Expect expression.");
                return;
            }
        };

        while precedence <= get_rule(self.parser.current.typ, RuleType::Precedence).as_precedence()
        {
            self.advance();
            let rule = get_rule(self.parser.previous.typ, RuleType::Infix)
                .as_rule()
                .expect("an infix parse rule");

            rule(self, can_assign)
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.parser.error("Invalid assignment target.")
        }
    }

    fn declaration(&mut self) {
        if self.matches(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.synchronize()
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(global);
    }

    /// Consume identifier token and add its lexeme to chunk's
    /// constant table if in global scope returning its index
    fn parse_variable(&mut self, msg: &str) -> usize {
        self.consume(TokenType::Identifier, msg);

        self.declare_variable();
        if self.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(&self.parser.previous.src)
    }

    /// Variable is now ready for use
    fn define_variable(&mut self, global: usize) {
        // locals behave like stack
        if self.scope_depth == 0 {
            self.emit_long((OpCode::DefineGlobal, OpCode::DefineGlobalLong), global)
        } else {
            // Variable initializer is complete
            self.mark_initialized()
        }
    }

    /// Mark last local as initialized by setting current depth. Panics if no locals
    fn mark_initialized(&mut self) {
        let last_local = self.locals.last_mut().expect("At least one local");
        last_local.depth = Some(self.scope_depth)
    }

    /// Intern string and insert into constant table
    fn identifier_constant(&mut self, token: &str) -> usize {
        let istr = self.interner.intern(token);
        self.make_constant(Value::String(istr))
    }

    /// Add local variable to locals. Variable is added to scope
    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        let name = self.parser.previous;
        for local in self.locals.iter().rev() {
            if let Some(depth) = local.depth {
                if depth < self.scope_depth {
                    break;
                }
            }

            if name.src == local.name.src {
                self.parser
                    .error("Already a variable with this name in this scope.")
            }
        }

        self.add_local(name)
    }

    /// Locals refer to variables by slot index which is limited to u16
    fn add_local(&mut self, name: Token<'input>) {
        if self.locals.len() > u16::MAX as usize {
            self.parser
                .error("Too many local variables in one function.");
        } else {
            self.locals.push(Local { name, depth: None })
        }
    }

    fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement()
        } else if self.matches(TokenType::If) {
            self.if_statement()
        } else if self.matches(TokenType::LBrace) {
            self.begin_scope();
            self.block();
            self.end_scope()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print)
    }

    fn block(&mut self) {
        while !self.check(TokenType::RBrace) && !self.check(TokenType::Eof) {
            self.declaration()
        }

        self.consume(TokenType::RBrace, "Expect '}' after block.")
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop)
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop); // Pop cond from stack if true
        self.statement(); // statement if cond true

        // Skip else clause if cond was true
        let else_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop); // Pop cond from stack if false

        if self.matches(TokenType::Else) {
            self.statement(); // statement if cond false
        }
        self.patch_jump(else_jump)
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn number(&mut self, _can_assign: bool) {
        let value: f64 = self.parser.previous.src.parse().expect("a number");
        self.emit_constant(Value::Num(value))
    }

    fn string(&mut self, _can_assign: bool) {
        let str = self.parser.previous.src;
        let istr = self.interner.intern(&str[1..str.len() - 1]);
        self.emit_constant(Value::String(istr))
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.parser.previous.src, can_assign)
    }

    fn named_variable(&mut self, token: &str, can_assign: bool) {
        let (arg, get_ops, set_ops) = if let Some(arg) = self.resolve_local(token) {
            (
                arg,
                (OpCode::GetLocal, OpCode::GetLocalLong),
                (OpCode::SetLocal, OpCode::SetLocalLong),
            )
        } else {
            let arg = self.identifier_constant(token);
            (
                arg,
                (OpCode::GetGlobal, OpCode::GetGlobalLong),
                (OpCode::SetGlobal, OpCode::SetGlobalLong),
            )
        };

        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            self.emit_long(set_ops, arg)
        } else {
            self.emit_long(get_ops, arg)
        }
    }

    fn resolve_local(&mut self, name: &str) -> Option<usize> {
        for (idx, local) in self.locals.iter().enumerate().rev() {
            if local.name.src == name {
                if local.depth.is_none() {
                    self.parser
                        .error("Can't read local variable in its own initializer.")
                }

                return Some(idx);
            }
        }
        None
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RParen, "Expect ')' after expression.")
    }

    fn unary(&mut self, _can_assign: bool) {
        let typ = self.parser.previous.typ;
        self.parse_precedence(Precedence::Unary);

        match typ {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let typ = self.parser.previous.typ;
        let precedence = get_rule(typ, RuleType::Precedence).as_precedence();
        self.parse_precedence(precedence.next());

        match typ {
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.parser.previous.typ {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;

        while self.parser.current.typ != TokenType::Eof {
            if self.parser.previous.typ == TokenType::Semicolon {
                return;
            } else {
                match self.parser.current.typ {
                    TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return => return,
                    _ => {}
                }
            }

            self.advance()
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        #[cfg(feature = "debug_print_code")]
        if !self.parser.had_error {
            self.compiling_chunk.disassemble_chunk("code");
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1
    }

    /// Look for variables at scope just left and discard. At runtime
    /// locals occupy slot on stack so when they go out of scope, must pop
    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while self
            .locals
            .last()
            .map(|l| l.depth.expect("initialized local") > self.scope_depth)
            .unwrap_or(false)
        {
            self.emit_byte(OpCode::Pop);
            self.locals.pop();
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

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(OpCode::Byte(u8::MAX));
        self.emit_byte(OpCode::Byte(u8::MAX));

        self.compiling_chunk.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.compiling_chunk.len() - offset - 2;
        if jump > u16::MAX as usize {
            self.parser.error("Too much code to jump over.")
        }

        let (j1, j2) = split_u16(jump as u16);

        let old_j1 = self
            .compiling_chunk
            .get_byte_mut(offset)
            .expect("jump byte");
        *old_j1 = j1;

        let old_j2 = self
            .compiling_chunk
            .get_byte_mut(offset + 1)
            .expect("jump byte");
        *old_j2 = j2;
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.compiling_chunk.write_maybe_long(
            (OpCode::Constant, OpCode::ConstantLong),
            constant,
            self.parser.previous.line,
        );
    }

    fn emit_long(&mut self, pair: (OpCode, OpCode), byte: usize) {
        self.compiling_chunk
            .write_maybe_long(pair, byte, self.parser.previous.line);
    }

    /// Insert constant into chunk, erroring if too many in table
    fn make_constant(&mut self, value: Value) -> usize {
        let constant = self.compiling_chunk.add_constant(value);
        if constant > u16::MAX as usize {
            self.parser.error("Too many constants in one chunk.");
            0
        } else {
            constant
        }
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
                self.parser.error_at_current(self.parser.current.src)
            }
        }
    }

    fn consume(&mut self, typ: TokenType, msg: &str) {
        if self.parser.current.typ == typ {
            self.advance()
        } else {
            self.parser.error_at_current(msg)
        }
    }

    fn matches(&mut self, typ: TokenType) -> bool {
        if !self.check(typ) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn check(&self, typ: TokenType) -> bool {
        self.parser.current.typ == typ
    }
}
