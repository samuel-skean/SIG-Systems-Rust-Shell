mod lexer;
mod parser;
mod safe_wrappers;

use safe_wrappers::{fork, exec, wait, ForkReturn, WaitReturn};

#[cfg(test)]
mod tests;

use std::io::{self, Write};

use parser::{Arg, Command};

fn main() {
    // Input REPL
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }

        let command = Command::parse(input).unwrap();
        run_command(&command).expect("run_command shouldn't return error");
        println!("{command:#?}");
    }
}

fn run_command(cmd: &Command) -> io::Result<()> {
    match fork()? {
        ForkReturn::Child => {
            let args = cmd.argv.iter().filter_map(|arg| {
                if let Arg::Word(w) = arg { Some(w) } else { None }
            }).collect::<Vec<_>>();

            if args.len() == 0 {
                return Ok(())
            }

            if let Err(e) = exec(args[0], args.as_slice()) {
                eprintln!("Error running {}: {e}", args[0]);
            } else {
                unsafe { std::hint::unreachable_unchecked() };
            }
        },
        ForkReturn::Parent(_) => {
            wait();
        }
    }
    Ok(())
}
