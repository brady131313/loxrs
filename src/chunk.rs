use crate::{
    debug::{constant_instruction, constant_long_instruction, simple_instruction},
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
    DefineGlobal,
    DefineGlobalLong,
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
}

impl From<u8> for OpCode {
    fn from(val: u8) -> Self {
        Self::Byte(val)
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

    // pub fn write_constant<V: Into<Value>>(&mut self, value: V, line: usize) -> Option<usize> {
    //     let idx = self.add_constant(value);
    //     self.write_maybe_long((OpCode::Constant, OpCode::ConstantLong), idx, line)
    // }

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
            self.write_chunk(pair.1, line);
            self.write_chunk((byte >> 8) as u8, line);
            self.write_chunk(byte as u8, line);
        } else {
            return None;
        }

        Some(byte)
    }

    pub fn get_byte(&self, offset: usize) -> Option<u8> {
        self.get_op(offset).and_then(|o| o.as_byte())
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
            OpCode::Return => simple_instruction("RETURN", offset),
            OpCode::Constant => constant_instruction("CONSTANT", self, offset),
            OpCode::ConstantLong => constant_long_instruction("CONSTANT_LONG", self, offset),
            OpCode::Nil => simple_instruction("NIL", offset),
            OpCode::True => simple_instruction("TRUE", offset),
            OpCode::False => simple_instruction("FALSE", offset),
            OpCode::Pop => simple_instruction("POP", offset),
            OpCode::DefineGlobal => constant_instruction("DEFINE_GLOBAL", self, offset),
            OpCode::DefineGlobalLong => {
                constant_long_instruction("DEFINE_GLOBAL_LONG", self, offset)
            }
            OpCode::Equal => simple_instruction("EQUAL", offset),
            OpCode::Greater => simple_instruction("GREATER", offset),
            OpCode::Less => simple_instruction("LESS", offset),
            OpCode::Add => simple_instruction("ADD", offset),
            OpCode::Subtract => simple_instruction("SUBTRACT", offset),
            OpCode::Multiply => simple_instruction("MULTIPLY", offset),
            OpCode::Divide => simple_instruction("DIVIDE", offset),
            OpCode::Not => simple_instruction("NOT", offset),
            OpCode::Negate => simple_instruction("NEGATE", offset),
            OpCode::Print => simple_instruction("PRINT", offset),
            OpCode::Byte(b) => {
                println!("Unknown opcode {b}");
                offset + 1
            }
        }
    }
}
