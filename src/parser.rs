use std::path::PathBuf;
use std::{hint::unreachable_unchecked, iter::Peekable};
use bumpalo::boxed::Box as BumpaloBox;
use bumpalo::Bump;

use crate::lexer::{Lexer, Token};

#[derive(Debug)]
pub enum ParseError {
    Empty,
    MissingFileName,
    UnmatchedDelimiterError,
    InvalidVariable,
    UnterminatedStringLiteral,
    NonRedirTypeToken,
    NotFound,
}

#[derive(Debug)]
pub struct ParseErrors {
    errors: Vec<ParseError>,
}

impl ParseErrors {
    fn into_iter(self) -> impl Iterator<Item = ParseError> {
        self.errors.into_iter()
    }
}

#[derive(Debug, PartialEq)]
pub enum Arg<'bump> {
    Word(String),
    Variable(String),
    Subshell(Command<'bump>),
}

#[derive(Debug)]
pub struct Parser<I: Iterator<Item = Result<Token, ParseError>>> {
    tokens: Peekable<I>,
}

#[derive(Debug, PartialEq)]
pub struct Command<'bump> {
    pub argv: Vec<Arg<'bump>>,
    pub pipe_to: Option<PipeTo<'bump>>,
    pub redirect_to: Vec<FileRedir>,
    pub and_then: Option<AndThen<'bump>>,
}

#[derive(Debug, PartialEq)]
pub struct PipeTo<'bump> {
    pub pipe_type: RedirType,
    pub target: BumpaloBox<'bump, Command<'bump>>,
}

#[derive(Debug, PartialEq)]
pub struct AndThen<'bump> {
    pub conditional: bool,
    pub target: BumpaloBox<'bump, Command<'bump>>,
}

#[derive(Debug, PartialEq)]
pub enum RedirType {
    Stdout,
    Stderr,
    Both,
}

#[derive(Debug)]
pub struct NonRedirTypeToken {}
impl TryFrom<Token> for RedirType {
    type Error = ParseError;
    fn try_from(val: Token) -> Result<Self, Self::Error> {
        use Token as T;
        use RedirType as R;

        match val {
            T::RedirOut | T::Pipe => Ok(R::Stdout),
            T::RedirBoth | T::PipeBoth => Ok(R::Both),
            T::RedirErr => Ok(R::Stderr),
            _ => Err(ParseError::NonRedirTypeToken)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FileRedir {
    pub redirect_type: RedirType,
    pub target: PathBuf,
}

impl<I: Iterator<Item = Result<Token, ParseError>>> Parser<I> {
    pub fn new(tokens: I) -> Self {
        Parser {
            tokens: tokens.peekable(),
        }
    }

    fn parse_command<'bump>(&mut self, bump: &'bump Bump) -> Result<Command<'bump>, ParseErrors> {
        let mut errors = Vec::new();
        let mut argv = Vec::new();
        let mut pipe_to = None;
        let mut redirect_to = Vec::new();
        let mut and_then = None;

        while let Some(token_res) = self.tokens.next() {
            match token_res {
                Ok(tok) => match tok {
                    Token::Word(word) => argv.push(Arg::Word(word)),
                    tok if matches!(tok, Token::RedirOut | Token::RedirErr | Token::RedirBoth) => {
                        let redir_type = tok.try_into().unwrap();
                        if let Some(Ok(Token::Word(path))) = self.tokens.next() {
                            redirect_to.push(FileRedir {
                                redirect_type: redir_type,
                                target: PathBuf::from(path),
                            });
                        } else {
                            errors.push(ParseError::MissingFileName);
                        }
                    }
                    pipe_token if matches!(pipe_token, Token::Pipe | Token::PipeBoth) => {
                        let pipe_type: RedirType = pipe_token.try_into().unwrap();

                        match self.parse_command(bump) {
                            Ok(next_command) => {
                                pipe_to = Some(PipeTo {
                                    pipe_type,
                                    target: BumpaloBox::new_in(next_command, bump),
                                });
                            }
                            Err(errs) => {
                                errors.extend(errs.into_iter());
                            }
                        }
                        break;
                    }
                    Token::AndThen => {
                        match self.parse_command(bump) {
                            Ok(next_command) => {
                                and_then = Some(AndThen {
                                    target: BumpaloBox::new_in(next_command, bump),
                                    conditional: false,
                                });
                            }
                            Err(errs) => {
                                errors.extend(errs.into_iter());
                            }
                        }
                        break;
                    }
                    Token::AndThenIf => {
                        match self.parse_command(bump) {
                            Ok(next_command) => {
                                and_then = Some(AndThen {
                                    target: BumpaloBox::new_in(next_command, bump),
                                    conditional: true,
                                });
                            }
                            Err(errs) => {
                                errors.extend(errs.into_iter());
                            }
                        }
                        break;
                    }
                    Token::SubShell(command) => {
                        match Command::parse(command, bump) {
                            Ok(command) => argv.push(Arg::Subshell(command)),
                            Err(errs) => errors.extend(errs.into_iter()),
                        }
                    }
                    Token::Variable(s) => {
                        argv.push(Arg::Variable(s));
                    }
                    _ => {
                        // TODO: Re-evaluate this!
                        unsafe { unreachable_unchecked() }
                    }
                },
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() || argv.is_empty() {
            Err(ParseErrors { errors })
        } else {
            Ok(Command {
                argv,
                pipe_to,
                and_then,
                redirect_to,
            })
        }
    }

    fn parse_redir_type(&mut self) -> RedirType {
        match self.tokens.next() {
            Some(Ok(Token::RedirOut)) => RedirType::Stdout,
            Some(Ok(Token::RedirErr)) => RedirType::Stderr,
            Some(Ok(Token::RedirBoth)) => RedirType::Both,
            _ => panic!("peek is Token::Redir, but matched none of Redir Variants"),
        }
    }
}

impl<'bump> Command<'bump> {
    pub fn parse(input: impl AsRef<str>, bump: &'bump Bump) -> Result<Self, ParseErrors> {
        let lexer = Lexer::new(input.as_ref());
        let mut parser = Parser::new(lexer);
        parser.parse_command(bump)
    }
}
