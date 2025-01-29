use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub enum Token {
    Word(String),
    Pipe,
    PipeBoth,
    RedirOut,
    RedirErr,
    RedirBoth,
    AndThen,
    AndThenIf,
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.chars.next_if(|c| c.is_whitespace()).is_some() {}
    }

    fn lex_word(&mut self) -> Option<Token> {
        let mut word = String::new();
        let mut in_single_quotes = false;
        let mut in_double_quotes = false;

        while let Some(&c) = self.chars.peek() {
            if in_single_quotes {
                self.chars.next();
                if c == '\'' {
                    in_single_quotes = false;
                } else {
                    word.push(c);
                }
            } else if in_double_quotes {
                self.chars.next();
                if c == '"' {
                    in_double_quotes = false;
                } else {
                    word.push(c);
                }
            } else if c.is_whitespace() || c == '|' {
                break;
            } else if c == '\'' {
                self.chars.next();
                in_single_quotes = true;
            } else if c == '"' {
                self.chars.next();
                in_double_quotes = true;
            } else if c == '>' || c == '&' {
                break;
            } else if c.is_ascii_digit()
                && self
                    .chars
                    .clone()
                    .next_if(|&next_c| next_c == '>')
                    .is_some()
            {
                // Only break on digits if they're followed by '>', like "2>"
                break;
            } else {
                self.chars.next();
                word.push(c);
            }
        }

        if !word.is_empty() {
            Some(Token::Word(word))
        } else {
            None
        }
    }

    fn lex_and_then(&mut self) -> Option<Token> {
        let mut iter = self.chars.clone();

        if let Some(&c) = iter.peek() {
            if c == '&' {
                iter.next();

                if let Some(&next_c) = iter.peek() {
                    if next_c == '&' {
                        self.chars.next();
                        self.chars.next();
                        return Some(Token::AndThenIf);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else if c == ';' {
                self.chars.next();
                return Some(Token::AndThen);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    fn lex_redirection(&mut self) -> Option<Token> {
        let mut iter = self.chars.clone();
        let mut redir = String::new();

        if let Some(&c) = iter.peek() {
            if c == '1' || c == '2' || c == '&' {
                redir.push(c);
                iter.next();
            }
        }

        if let Some(&c) = iter.peek() {
            if c == '>' {
                redir.push(c);
                iter.next();

                if let Some(&next_c) = iter.peek() {
                    if next_c == '>' {
                        redir.push(next_c);
                        iter.next();
                    }
                }
            } else {
                return None;
            }
        } else {
            return None;
        }

        match redir.as_str() {
            ">" | "1>" | ">>" | "1>>" => {
                for _ in 0..redir.len() {
                    self.chars.next();
                }
                Some(Token::RedirOut)
            }
            "2>" | "2>>" => {
                for _ in 0..redir.len() {
                    self.chars.next();
                }
                Some(Token::RedirErr)
            }
            "&>" | "&>>" => {
                for _ in 0..redir.len() {
                    self.chars.next();
                }
                Some(Token::RedirBoth)
            }
            _ => None,
        }
    }

    fn lex_pipe(&mut self) -> Option<Token> {
        let mut iter = self.chars.clone();

        if let Some(c) = iter.next() {
            if c == '|' {
                if let Some(&next_c) = iter.peek() {
                    if next_c == '&' {
                        self.chars.next();
                        self.chars.next();
                        return Some(Token::PipeBoth);
                    }
                }
                self.chars.next();
                return Some(Token::Pipe);
            }
        }
        None
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        if let Some(token) = self.lex_redirection() {
            return Some(token);
        }

        if let Some(token) = self.lex_pipe() {
            return Some(token);
        }

        if let Some(token) = self.lex_and_then() {
            return Some(token);
        }

        self.lex_word()
    }
}
