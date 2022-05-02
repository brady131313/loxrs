use crate::chunk::Chunk;

pub fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

pub fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.get_byte(offset + 1);
    print!("{name:<16} {constant:4} ");

    let value = chunk.get_constant(constant as usize);
    println!("'{value}'");

    offset + 2
}

pub fn constant_long_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let c1 = chunk.get_byte(offset + 1);
    let c2 = chunk.get_byte(offset + 2);
    let constant = ((c1 as u16) << 8) | c2 as u16;
    print!("{name:<16} {constant:4} ");

    let value = chunk.get_constant(constant as usize);
    println!("'{value}'");

    offset + 3
}

