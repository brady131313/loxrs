use crate::{
    chunk::{Chunk, OpCode},
    compiler::Compiler,
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
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Stack::new(),
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
                OpCode::Add => self.binary_op(|a, b| a + b),
                OpCode::Subtract => self.binary_op(|a, b| a - b),
                OpCode::Multiply => self.binary_op(|a, b| a * b),
                OpCode::Divide => self.binary_op(|a, b| a / b),
                OpCode::Negate => {
                    let constant = self.stack.pop().as_num().expect("num operator");
                    self.stack.push(Value::Num(-constant));
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

    fn binary_op<F: Fn(f64, f64) -> f64>(&mut self, f: F) {
        let b = self.stack.pop().as_num().expect("num operator");
        let a = self.stack.pop().as_num().expect("num operator");
        self.stack.push(Value::Num(f(a, b)))
    }

    fn runtime_error(&mut self, msg: &str) {
        println!("{msg}");

        let line = self.chunk.get_line(self.ip - 1).expect("expected line");
        eprintln!("[line {line}] in script");
        self.stack.reset()

    }
}
