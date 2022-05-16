use std::{collections::HashMap, fmt::Display};

use crate::{
    chunk::{Chunk, OpCode, OpLen},
    compiler::Compiler,
    object::{IString, StringInterner},
    stack::Stack,
    util::join_u8s,
    value::Value,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpretError {
    Compile,
    Runtime,
}

pub type InterpretResult<T = ()> = Result<T, InterpretError>;

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Stack<Value>,
    interner: StringInterner,
    globals: HashMap<String, Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Stack::new(),
            interner: StringInterner::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, src: &str) -> InterpretResult {
        let compiler = Compiler::new(src, &mut self.interner);

        let chunk = compiler.compile()?;
        self.chunk = chunk;
        self.ip = 0;

        self.run()
    }

    fn read_byte(&mut self) -> Option<OpCode> {
        let instruction = self.chunk.get_op(self.ip);
        self.ip += 1;

        instruction
    }

    fn read_short(&mut self) -> Option<u16> {
        let b1 = self.read_byte().and_then(|o| o.as_byte())?;
        let b2 = self.read_byte().and_then(|o| o.as_byte())?;
        let idx = join_u8s(b1, b2);
        Some(idx)
    }

    fn read_idx<L: Into<OpLen>>(&mut self, len: L) -> Option<usize> {
        match len.into() {
            OpLen::Short => self
                .read_byte()
                .and_then(|o| o.as_byte())
                .map(|b| b as usize),
            OpLen::Long => self.read_short().map(|b| b as usize),
        }
    }

    fn read_constant<L: Into<OpLen>>(&mut self, len: L) -> Option<&Value> {
        let idx = self.read_idx(len)?;
        self.chunk.get_constant(idx)
    }

    /// This does not convert the IString with the interner because IString is copy
    /// while reference to str would be tied to mut self making it pain to use
    fn read_string<L: Into<OpLen>>(&mut self, len: L) -> Option<IString> {
        self.read_constant(len)?.as_str()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("\t\t");
                for val in &self.stack {
                    print!("[ {val} ]")
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }

            match self.read_byte().ok_or(InterpretError::Compile)? {
                code @ (OpCode::Constant | OpCode::ConstantLong) => {
                    let constant = *self.read_constant(code).ok_or(InterpretError::Compile)?;
                    self.stack.push(constant);
                }
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Pop => {
                    self.stack.pop();
                }
                code @ (OpCode::GetLocal | OpCode::GetLocalLong) => {
                    let slot = self.read_idx(code).expect("a slot idx");
                    let slot_val = *self.stack.get(slot).expect("invalid stack idx");
                    self.stack.push(slot_val);
                }
                code @ (OpCode::SetLocal | OpCode::SetLocalLong) => {
                    let slot = self.read_idx(code).expect("a slot idx");
                    let new_val = *self.stack.peek(0).expect("invalid stack idx");
                    self.stack
                        .set(slot, new_val)
                        .expect("failed to update slot");
                }
                code @ (OpCode::GetGlobal | OpCode::GetGlobalLong) => {
                    let iname = self.read_string(code).expect("expected string");
                    let name = self.interner.get(iname);

                    if let Some(&value) = self.globals.get(name) {
                        self.stack.push(value)
                    } else {
                        let name = name.to_owned();
                        self.runtime_error(format!("Undefined variable '{name}'"));
                        return Err(InterpretError::Runtime);
                    }
                }
                code @ (OpCode::DefineGlobal | OpCode::DefineGlobalLong) => {
                    let iname = self.read_string(code).expect("expected string");
                    let name = self.interner.get(iname).to_owned();
                    let value = *self.stack.peek(0).unwrap();

                    self.globals.insert(name, value);
                    self.stack.pop();
                }
                code @ (OpCode::SetGlobal | OpCode::SetGlobalLong) => {
                    let iname = self.read_string(code).expect("expected string");
                    let name = self.interner.get(iname);

                    if let Some(val) = self.globals.get_mut(name) {
                        let new_val = self.stack.peek(0).unwrap();
                        *val = *new_val
                    } else {
                        let name = name.to_owned();
                        self.runtime_error(format!("Undefined variable '{name}'"));
                        return Err(InterpretError::Runtime);
                    }
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a.eq(&b)));
                }
                OpCode::Greater => self.binary_op(|a, b| a > b)?,
                OpCode::Less => self.binary_op(|a, b| a < b)?,
                OpCode::Add => match (self.stack.peek(0), self.stack.peek(1)) {
                    (Some(Value::String(..)), Some(Value::String(..))) => {
                        let b_intered = self.stack.pop().unwrap().as_str().unwrap();
                        let b = self.interner.get(b_intered);

                        let a_intered = self.stack.pop().unwrap().as_str().unwrap();
                        let a = self.interner.get(a_intered);

                        let concated = format!("{a}{b}");
                        let res = self.interner.intern(concated);
                        self.stack.push(Value::String(res))
                    }
                    (Some(Value::Num(..)), Some(Value::Num(..))) => {
                        let b = self.stack.pop().unwrap().as_num().unwrap();
                        let a = self.stack.pop().unwrap().as_num().unwrap();
                        self.stack.push(Value::Num(a + b));
                    }
                    _ => {
                        self.runtime_error("Operands must be two numbers or two strings.");
                        return Err(InterpretError::Runtime);
                    }
                },
                OpCode::Subtract => self.binary_op(|a, b| a - b)?,
                OpCode::Multiply => self.binary_op(|a, b| a * b)?,
                OpCode::Divide => self.binary_op(|a, b| a / b)?,
                OpCode::Not => {
                    let val = self.stack.pop().unwrap().is_falsey();
                    self.stack.push(Value::Bool(val))
                }
                OpCode::Negate => {
                    if let Some(Value::Num(..)) = self.stack.peek(0) {
                        let constant = self.stack.pop().unwrap().as_num().unwrap();
                        self.stack.push(Value::Num(-constant))
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return Err(InterpretError::Runtime);
                    }
                }
                OpCode::Print => {
                    let value = self.stack.pop().unwrap();
                    self.print_val(value);
                }
                OpCode::Jump => {
                    let offset = self.read_short().expect("a short to jump to");
                    self.ip += offset as usize
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short().expect("a short to jump to");
                    let cond = self.stack.peek(0).expect("a test condition");
                    if cond.is_falsey() {
                        self.ip += offset as usize
                    }
                }
                OpCode::Loop => {
                    let offset = self.read_short().expect("a short to jump to");
                    self.ip -= offset as usize
                }
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Byte(b) => unimplemented!("unimplemented opcode {b}"),
            }
        }
    }

    fn binary_op<F, V>(&mut self, f: F) -> InterpretResult
    where
        F: Fn(f64, f64) -> V,
        V: Into<Value>,
    {
        if let (Some(Value::Num(..)), Some(Value::Num(..))) =
            (self.stack.peek(0), self.stack.peek(1))
        {
            let b = self.stack.pop().unwrap().as_num().unwrap();
            let a = self.stack.pop().unwrap().as_num().unwrap();
            self.stack.push(f(a, b));
            Ok(())
        } else {
            self.runtime_error("Operands must be numbers.");
            return Err(InterpretError::Runtime);
        }
    }

    fn runtime_error<D: Display>(&mut self, msg: D) {
        println!("{msg}");

        let line = self.chunk.get_line(self.ip - 1);
        eprintln!("[line {line}] in script");
        self.stack.reset()
    }

    fn print_val(&self, val: Value) {
        if let Value::String(istr) = val {
            let str = self.interner.get(istr);
            println!("\"{str}\"")
        } else {
            println!("{val}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm() {
        let test = (0..=(u8::MAX as usize + 1))
            .into_iter()
            .map(|i| format!("var a{i} = \"this is a test {i}\";"))
            .collect::<Vec<_>>()
            .join(" ");
        let test = format!("{test} print a256;");

        let mut vm = Vm::new();
        println!("{:?}", vm.interpret(&test));
        panic!()
    }
}
