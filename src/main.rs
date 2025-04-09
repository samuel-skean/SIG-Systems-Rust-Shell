mod lexer;
mod parser;
mod safe_wrappers;

use safe_wrappers::{fork, exec, wait, ForkReturn, WaitReturn, pipe, close, dup2};
use parser::RedirType;

#[cfg(test)]
mod tests;

use std::{io::{self, Write}, os::fd::{AsRawFd, RawFd}};

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
        run_commands(&command).expect("run_commands shouldn't return error");
    }
}

fn run_command(cmd: &Command, read_from: Option<RawFd>) -> io::Result<()> {
    use ForkReturn as FR;
    use RedirType as RT;

    if let Some(pipe_to) = cmd.pipe_to.as_ref() {
        let pipe = pipe()?;

        match fork()? {
            FR::Parent(_) => {
                close(pipe.read_fd)?;

                let args = cmd.argv.iter().filter_map(|arg| {
                    if let Arg::Word(w) = arg { Some(w) } else { None }
                }).collect::<Vec<_>>();

                if args.is_empty() {
                    return Ok(())
                }

                if let Some(read_from) = read_from {
                    dup2(read_from, libc::STDIN_FILENO)?;
                }

                match pipe_to.pipe_type {
                    RT::Stdout => dup2(pipe.write_fd, libc::STDOUT_FILENO)?,
                    RT::Stderr => dup2(pipe.write_fd, libc::STDERR_FILENO)?,
                    RT::Both => {
                        dup2(pipe.write_fd, libc::STDOUT_FILENO)?;
                        dup2(pipe.write_fd, libc::STDERR_FILENO)?;
                    },
                }

                if let Err(e) = exec(args[0], args.as_slice()) {
                    eprintln!("Error running {}: {e}", args[0]);
                    Err(io::Error::last_os_error())
                } else {
                    unsafe { std::hint::unreachable_unchecked() };
                }
            }
            FR::Child => {
                close(pipe.write_fd)?;
                run_command(pipe_to.target.as_ref(), Some(pipe.read_fd))
            },
        }
    } else {
        let args = cmd.argv.iter().filter_map(|arg| {
            if let Arg::Word(w) = arg { Some(w) } else { None }
        }).collect::<Vec<_>>();

        if args.is_empty() {
            return Ok(())
        }

        if let Some(read_from) = read_from {
            dup2(read_from, libc::STDIN_FILENO)?;
        }

        if let Err(e) = exec(args[0], args.as_slice()) {
            eprintln!("Error running {}: {e}", args[0]);
            Err(io::Error::last_os_error())
        } else {
            unsafe { std::hint::unreachable_unchecked() };
        }
    }
}

fn run_commands(cmd: &Command) -> io::Result<()> {
    match fork()? {
        ForkReturn::Parent(_) => {
            wait()?;
            Ok(())
        }
        ForkReturn::Child => {
            run_command(cmd, None)
        }
    }
}
