use crate::{
    util::{join_u8s, split_u16},
    value::Value,
};

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant,
    ConstantLong,
    Nil,
    True,
    False,
    Pop,
    GetLocal,
    GetLocalLong,
    SetLocal,
    SetLocalLong,
    GetGlobal,
    GetGlobalLong,
    DefineGlobal,
    DefineGlobalLong,
    SetGlobal,
    SetGlobalLong,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Jump,
    JumpIfFalse,
    Return,
    Byte(u8),
}

impl OpCode {
    pub fn as_byte(&self) -> Option<u8> {
        match self {
            Self::Byte(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_byte_mut(&mut self) -> Option<&mut u8> {
        match self {
            Self::Byte(b) => Some(b),
            _ => None,
        }
    }
}

impl From<u8> for OpCode {
    fn from(val: u8) -> Self {
        Self::Byte(val)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpLen {
    Short,
    Long,
}

impl From<OpCode> for OpLen {
    fn from(code: OpCode) -> Self {
        match code {
            OpCode::ConstantLong
            | OpCode::DefineGlobalLong
            | OpCode::GetGlobalLong
            | OpCode::GetLocalLong
            | OpCode::SetLocalLong => OpLen::Long,
            _ => OpLen::Short,
        }
    }
}

#[derive(Debug)]
pub struct LineStart {
    offset: usize,
    line: usize,
}

#[derive(Debug)]
pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
    lines: Vec<LineStart>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write_chunk<B: Into<OpCode>>(&mut self, byte: B, line: usize) {
        self.code.push(byte.into());

        // See if we're still on the same line
        if self.lines.last().map(|l| l.line) != Some(line) {
            self.lines.push(LineStart {
                offset: self.code.len() - 1,
                line,
            })
        }
    }

    pub fn write_maybe_long(
        &mut self,
        pair: (OpCode, OpCode),
        byte: usize,
        line: usize,
    ) -> Option<usize> {
        if byte <= u8::MAX as usize {
            self.write_chunk(pair.0, line);
            self.write_chunk(byte as u8, line);
        } else if byte <= u16::MAX as usize {
            let (b1, b2) = split_u16(byte as u16);
            self.write_chunk(pair.1, line);
            self.write_chunk(b1, line);
            self.write_chunk(b2, line);
        } else {
            return None;
        }

        Some(byte)
    }

    pub fn get_byte(&self, offset: usize) -> Option<u8> {
        self.get_op(offset).and_then(|o| o.as_byte())
    }

    pub fn get_byte_mut(&mut self, offset: usize) -> Option<&mut u8> {
        self.code.get_mut(offset).and_then(|o| o.as_byte_mut())
    }

    pub fn get_op(&self, offset: usize) -> Option<OpCode> {
        self.code.get(offset).copied()
    }

    pub fn add_constant<V: Into<Value>>(&mut self, value: V) -> usize {
        self.constants.push(value.into());
        self.constants.len() - 1
    }

    pub fn get_constant(&self, offset: usize) -> Option<&Value> {
        self.constants.get(offset)
    }

    pub fn get_line(&self, instruction: usize) -> usize {
        let mut start = 0;
        let mut end = self.lines.len();

        loop {
            let mid = (start + end) / 2;
            let line = &self.lines[mid];
            if instruction < line.offset {
                end = mid - 1;
            } else if mid == self.lines.len() - 1 || instruction < self.lines[mid + 1].offset {
                return line.line;
            } else {
                start = mid + 1;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn disassemble_chunk(&self, name: &str) {
        println!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:04} ");

        let line = self.get_line(offset);
        if offset > 0 && line == self.get_line(offset - 1) {
            print!("\t| ")
        } else {
            print!("{:4} ", line)
        }

        match self.code[offset] {
            OpCode::Return => self.simple_instruction("RETURN", offset),
            OpCode::Constant => self.constant_instruction("CONSTANT", offset),
            OpCode::ConstantLong => self.constant_long_instruction("CONSTANT_LONG", offset),
            OpCode::Nil => self.simple_instruction("NIL", offset),
            OpCode::True => self.simple_instruction("TRUE", offset),
            OpCode::False => self.simple_instruction("FALSE", offset),
            OpCode::Pop => self.simple_instruction("POP", offset),
            OpCode::GetLocal => self.byte_instruction("GET_LOCAL", offset),
            OpCode::GetLocalLong => self.byte_long_instruction("GET_LOCAL_LONG", offset),
            OpCode::SetLocal => self.byte_instruction("SET_LOCAL", offset),
            OpCode::SetLocalLong => self.byte_long_instruction("SET_LOCAL_LONG", offset),
            OpCode::GetGlobal => self.constant_instruction("GET_GLOBAL", offset),
            OpCode::GetGlobalLong => self.constant_long_instruction("GET_GLOBAL_LONG", offset),
            OpCode::DefineGlobal => self.constant_instruction("DEFINE_GLOBAL", offset),
            OpCode::DefineGlobalLong => {
                self.constant_long_instruction("DEFINE_GLOBAL_LONG", offset)
            }
            OpCode::SetGlobal => self.constant_instruction("SET_GLOBAL", offset),
            OpCode::SetGlobalLong => self.constant_long_instruction("SET_GLOBAL_LONG", offset),
            OpCode::Equal => self.simple_instruction("EQUAL", offset),
            OpCode::Greater => self.simple_instruction("GREATER", offset),
            OpCode::Less => self.simple_instruction("LESS", offset),
            OpCode::Add => self.simple_instruction("ADD", offset),
            OpCode::Subtract => self.simple_instruction("SUBTRACT", offset),
            OpCode::Multiply => self.simple_instruction("MULTIPLY", offset),
            OpCode::Divide => self.simple_instruction("DIVIDE", offset),
            OpCode::Not => self.simple_instruction("NOT", offset),
            OpCode::Negate => self.simple_instruction("NEGATE", offset),
            OpCode::Print => self.simple_instruction("PRINT", offset),
            OpCode::Jump => self.jump_instruction("JUMP", 1, offset),
            OpCode::JumpIfFalse => self.jump_instruction("JUMP_IF_FALSE", 1, offset),
            OpCode::Byte(b) => {
                println!("Unknown opcode {b}");
                offset + 1
            }
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        let slot = self.get_byte(offset + 1).unwrap();
        println!("{name:<16} {slot:4}");
        offset + 2
    }

    fn byte_long_instruction(&self, name: &str, offset: usize) -> usize {
        let s1 = self.get_byte(offset + 1).unwrap();
        let s2 = self.get_byte(offset + 2).unwrap();
        let slot = join_u8s(s1, s2);
        println!("{name:<16} {slot:4}");
        offset + 3
    }

    fn jump_instruction(&self, name: &str, sign: isize, offset: usize) -> usize {
        let s1 = self.get_byte(offset + 1).unwrap();
        let s2 = self.get_byte(offset + 2).unwrap();
        let jump = join_u8s(s1, s2);
        let to: isize = (offset as isize) + 3 + (sign * jump as isize);
        println!("{name:<16} {offset:4} -> {to}");

        offset + 3
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.get_byte(offset + 1).unwrap();
        print!("{name:<16} {constant:4} ");

        let value = self.get_constant(constant as usize).unwrap();
        println!("'{value}'");

        offset + 2
    }

    fn constant_long_instruction(&self, name: &str, offset: usize) -> usize {
        let c1 = self.get_byte(offset + 1).unwrap();
        let c2 = self.get_byte(offset + 2).unwrap();
        let constant = join_u8s(c1, c2);
        print!("{name:<16} {constant:4} ");

        let value = self.get_constant(constant as usize).unwrap();
        println!("'{value}'");

        offset + 3
    }
}
