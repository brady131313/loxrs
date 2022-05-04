use crate::scanner::{Scanner, TokenType};

pub fn compile(src: &str) {
    let mut scanner = Scanner::new(src);
    let mut line = usize::MAX;

    loop {
        let token = scanner.scan_token();
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("\t| ");
        }
        println!("{token}");

        if token.typ == TokenType::Eof {
            break;
        }
    }
}
