use crate::{
    debug::{constant_instruction, constant_long_instruction, simple_instruction},
    value::Value,
};

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant,
    ConstantLong,
    Return,
    Byte(u8),
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

    pub fn write_constant<V: Into<Value>>(&mut self, value: V, line: usize) {
        let idx = self.add_constant(value) as u16;

        if idx <= u8::MAX as u16 {
            self.write_chunk(OpCode::Constant, line);
            self.write_chunk(idx as u8, line);
        } else {
            self.write_chunk(OpCode::ConstantLong, line);
            self.write_chunk((idx >> 8) as u8, line);
            self.write_chunk(idx as u8, line);
        }
    }

    pub fn get_byte(&self, offset: usize) -> u8 {
        match self.code.get(offset) {
            Some(OpCode::Byte(b)) => *b,
            _ => panic!("Expected byte at offset: {offset}"),
        }
    }

    pub fn add_constant<V: Into<Value>>(&mut self, value: V) -> usize {
        self.constants.push(value.into());
        self.constants.len() - 1
    }

    pub fn get_constant(&self, offset: usize) -> &Value {
        self.constants
            .get(offset as usize)
            .expect("expected value at index")
    }

    pub fn get_line(&self, instruction: usize) -> Option<usize> {
        let idx = self
            .lines
            .binary_search_by(|l| l.offset.cmp(&instruction))
            .ok()?;

        Some(self.lines[idx].line)
    }

    pub fn disassemble_chunk(&self, name: &str) {
        // println!("{:?}", self.lines);
        // println!("{:?}", self.code);
        println!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:04} ");

        let line = self.get_line(offset);
        if offset > 0 && line == self.get_line(offset - 1) {
            print!("\t| ")
        } else {
            print!("{:4} ", line.expect("A line number from offset"))
        }

        match self.code[offset] {
            OpCode::Return => simple_instruction("RETURN", offset),
            OpCode::Constant => constant_instruction("CONSTANT", self, offset),
            OpCode::ConstantLong => constant_long_instruction("CONSTANT_LONG", self, offset),
            OpCode::Byte(b) => {
                println!("Unknown opcode {b}");
                offset + 1
            }
        }
    }
}
