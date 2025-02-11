mod lexer;
mod parser;

#[cfg(test)]
mod tests;

use std::io::{self, Write};

use bumpalo::Bump;
use parser::Command;

fn main() {
    // Input REPL loop
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut bump = Bump::new();
    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }

        let command = Command::parse(input, &bump);
        println!("{command:#?}");
        drop(command);
        bump.reset();
    }
}
