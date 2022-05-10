use crate::{
    chunk::{Chunk, OpCode},
    compiler::Compiler,
    object::Object,
    stack::Stack,
    value::Value,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpretError {
    Compile,
    Runtime,
}

pub type InterpretResult<T = ()> = Result<T, InterpretError>;

const STACK_SIZE: usize = 256;
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Stack<Value, STACK_SIZE>,
    objects: Vec<Object>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Stack::new(),
            objects: Vec::new(),
        }
    }

    pub fn interpret(&mut self, src: &str) -> InterpretResult {
        let compiler = Compiler::new(src);

        self.chunk = compiler.compile()?;
        self.ip = 0;

        self.run()
    }

    fn read_byte(&mut self) -> Option<OpCode> {
        let instruction = self.chunk.get_op(self.ip);
        self.ip += 1;

        instruction
    }

    fn read_constant(&mut self) -> Option<&Value> {
        let byte = self.read_byte().and_then(|o| o.as_byte())?;
        self.chunk.get_constant(byte as usize)
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
                OpCode::Constant => {
                    let constant = *self.read_constant().ok_or(InterpretError::Compile)?;
                    self.stack.push(constant);
                }
                OpCode::ConstantLong => todo!(),
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Equal => {
                    let b = *self.stack.pop();
                    let a = *self.stack.pop();
                    self.stack.push(Value::Bool(a.eq(&b)));
                }
                OpCode::Greater => self.binary_op(|a, b| a > b)?,
                OpCode::Less => self.binary_op(|a, b| a < b)?,
                OpCode::Add => self.binary_op(|a, b| a + b)?,
                OpCode::Subtract => self.binary_op(|a, b| a - b)?,
                OpCode::Multiply => self.binary_op(|a, b| a * b)?,
                OpCode::Divide => self.binary_op(|a, b| a / b)?,
                OpCode::Not => {
                    let val = self.stack.pop().is_falsey();
                    self.stack.push(Value::Bool(val))
                }
                OpCode::Negate => {
                    if let Some(Value::Num(..)) = self.stack.peek(0) {
                        let constant = self.stack.pop().as_num().unwrap();
                        self.stack.push(Value::Num(-constant))
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return Err(InterpretError::Runtime);
                    }
                }
                OpCode::Return => {
                    let value = self.stack.pop();
                    println!("{value}");
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
            let b = self.stack.pop().as_num().unwrap();
            let a = self.stack.pop().as_num().unwrap();
            self.stack.push(f(a, b));
            Ok(())
        } else {
            self.runtime_error("Operands must be numbers.");
            return Err(InterpretError::Runtime);
        }
    }

    fn runtime_error(&mut self, msg: &str) {
        println!("{msg}");

        let line = self.chunk.get_line(self.ip - 1);
        eprintln!("[line {line}] in script");
        self.stack.reset()
    }
}
