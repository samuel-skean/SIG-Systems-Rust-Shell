use std::iter::Peekable;
use std::path::PathBuf;

use crate::lexer::{Lexer, Token};

#[derive(Debug)]
pub struct Parser<I: Iterator<Item = Token>> {
    tokens: Peekable<I>,
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct CommandGroup {
    pub commands: Vec<Command>,
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct Command {
    pub argv: Vec<String>,
    pub pipe_to: Option<PipeTo>,
    pub redirect_to: Vec<FileRedir>,
    pub and_then: Option<AndThen>,
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct PipeTo {
    pub pipe_type: RedirType,
    pub target: Box<Command>,
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct AndThen {
    pub target: CommandGroup,
    pub conditional: bool,
}

#[derive(Debug)]
pub enum RedirType {
    Stdout,
    Stderr,
    Both,
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct FileRedir {
    pub redirect_type: RedirType,
    pub target: PathBuf,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(tokens: I) -> Self {
        Parser {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse_command_group(&mut self) -> CommandGroup {
        self.parse_command()
            // TODO: Fix this nasty hack and actually support `&&` and `;` parsing
            .map(|cmd| CommandGroup {
                commands: vec![cmd],
            })
            .unwrap_or(CommandGroup { commands: vec![] })
    }

    fn parse_command(&mut self) -> Option<Command> {
        let mut argv = Vec::new();
        let mut pipe_to = None;
        let mut redirect_to = Vec::new();
        let mut and_then = None;

        while let Some(token) = self.tokens.peek().cloned() {
            match token {
                Token::Word(_) => {
                    if let Some(Token::Word(word)) = self.tokens.next() {
                        argv.push(word);
                    }
                }
                Token::RedirOut | Token::RedirErr | Token::RedirBoth => {
                    let redir_type = self.parse_redir_type()?;
                    if let Some(Token::Word(path)) = self.tokens.next() {
                        redirect_to.push(FileRedir {
                            redirect_type: redir_type,
                            target: PathBuf::from(path),
                        });
                    } else {
                        eprintln!("Error: Missing filename after redirection");
                        return None;
                    }
                }
                Token::Pipe | Token::PipeBoth => {
                    let pipe_token = self.tokens.next();
                    let pipe_type = match pipe_token {
                        Some(Token::Pipe) => RedirType::Stdout,
                        Some(Token::PipeBoth) => RedirType::Both,
                        _ => RedirType::Stdout,
                    };
                    if let Some(next_command) = self.parse_command() {
                        pipe_to = Some(PipeTo {
                            pipe_type,
                            target: Box::new(next_command),
                        });
                    }
                    break;
                }
                Token::AndThen => {
                    self.tokens.next();
                    let next_group = self.parse_command_group();
                    and_then = Some(AndThen {
                        target: next_group,
                        conditional: false,
                    });
                    break;
                }
                Token::AndThenIf => {
                    self.tokens.next();
                    let next_group = self.parse_command_group();
                    and_then = Some(AndThen {
                        target: next_group,
                        conditional: true,
                    });
                    break;
                }
            }
        }

        if !argv.is_empty() {
            Some(Command {
                argv,
                pipe_to,
                and_then,
                redirect_to,
            })
        } else {
            None
        }
    }

    fn parse_redir_type(&mut self) -> Option<RedirType> {
        match self.tokens.next()? {
            Token::RedirOut => Some(RedirType::Stdout),
            Token::RedirErr => Some(RedirType::Stderr),
            Token::RedirBoth => Some(RedirType::Both),
            _ => None,
        }
    }
}

impl CommandGroup {
    pub fn parse(input: impl AsRef<str>) -> Self {
        let lexer = Lexer::new(input.as_ref());
        let mut parser = Parser::new(lexer);
        parser.parse_command_group()
    }
}
