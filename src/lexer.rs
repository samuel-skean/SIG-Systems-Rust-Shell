#[derive(Debug, PartialEq, PartialOrd)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    RightBracket,
    LeftBracket,

    Comma,           // x
    Dot,             // x
    DotDot,          // x
    Minus,           // x
    Plus,            // x
    Semicolon,       // x
    Glob,            // x
    Pound,           // x
    Ampersand,       // x
    DoubleAmpersand, // x
    Pipe,            // x
    Shebang,         // x
    Dollar,          // x
    Backslash,       // x
    Forwardslash,    // x
    OutputRedirect,  // x
    Append,          // x
    InputRedirect,   // x

    Bang,       // x
    BangEqual,  // x
    Equal,      // x
    EqualEqual, // x

    GlobbedWord(String),        // x
    Word(String),               // x
    DoubleQuotedString(String), // x
    SingleQuotedString(String), // x
    Integer(i64),               // x
    Float(f32),                 // x

    EOF, //x
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Token {
    token_type: TokenType,
}

impl Token {
    fn new(token_type: TokenType) -> Token {
        Token { token_type }
    }
}

pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    had_error: bool,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            start: 0,
            current: 0,
            had_error: false,
            tokens: Vec::new(),
        }
    }

    pub fn get_tokens(mut self) -> Option<Vec<Token>> {
        self.scan_tokens();
        if self.had_error {
            None
        } else {
            Some(self.tokens)
        }
    }
    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()
        }
        self.tokens.push(Token::new(TokenType::EOF));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    fn scan_token(&mut self) {
        if let Some(c) = self.next_char() {
            match c {
                '(' => self.add_token(TokenType::LeftParen),
                ')' => self.add_token(TokenType::RightParen),
                '[' => self.add_token(TokenType::LeftBracket),
                ']' => self.add_token(TokenType::RightBracket),
                '{' => self.add_token(TokenType::LeftBrace),
                '}' => self.add_token(TokenType::RightBrace),
                ',' => self.add_token(TokenType::Comma),
                '+' => self.add_token(TokenType::Plus),
                ';' => self.add_token(TokenType::Semicolon),
                '|' => self.add_token(TokenType::Pipe),
                '$' => self.add_token(TokenType::Dollar),
                '<' => self.add_token(TokenType::InputRedirect),
                '\\' => self.add_token(TokenType::Backslash),
                '/' => self.add_token(TokenType::Forwardslash),
                '\t' | '\n' | 'r' | ' ' => return,
                '"' => {
                    while self.peek().is_some_and(|c| c != '"') {
                        self.increment_current();
                        if self.peek().is_none() {
                            self.emit_error("Unterminated string literal");
                        }
                    }
                    self.add_token(TokenType::DoubleQuotedString(
                        self.source[self.start + 1..self.current].to_string(),
                    ));
                    self.increment_current();
                }
                '\'' => {
                    while self.peek().is_some_and(|c| c != '\'') {
                        self.increment_current();
                        if self.peek().is_none() {
                            self.emit_error("Unterminated string literal");
                        }
                    }
                    self.add_token(TokenType::SingleQuotedString(
                        self.source[self.start + 1..self.current].to_string(),
                    ));
                    self.increment_current();
                }
                '*' => {
                    if self.peek().is_some_and(|c| c == ' ') {
                        // stand alone glob
                        self.add_token(TokenType::Glob);
                    } else {
                        self.parse_word()
                    }
                }
                '#' => {
                    if self.peek().is_some_and(|c| c == '!') {
                        self.add_token(TokenType::Shebang);
                        self.increment_current();
                    } else {
                        self.add_token(TokenType::Pound);
                    }
                }
                '!' => {
                    if self.peek().is_some_and(|c| c == '=') {
                        self.add_token(TokenType::BangEqual);
                        self.increment_current();
                    } else {
                        self.add_token(TokenType::Bang);
                    }
                }
                '&' => {
                    if self.peek().is_some_and(|c| c == '&') {
                        self.add_token(TokenType::DoubleAmpersand);
                        self.increment_current();
                    } else {
                        self.add_token(TokenType::Ampersand);
                    }
                }

                '.' => {
                    if self.peek().is_some_and(|c| c == '.') {
                        self.add_token(TokenType::DotDot);
                        self.increment_current();
                    } else {
                        self.add_token(TokenType::Dot);
                    }
                }
                '=' => {
                    if self.peek().is_some_and(|c| c == '=') {
                        self.add_token(TokenType::EqualEqual);
                        self.increment_current();
                    } else {
                        self.add_token(TokenType::Equal);
                    }
                }
                '>' => {
                    if self.peek().is_some_and(|c| c == '>') {
                        self.add_token(TokenType::Append);
                        self.increment_current();
                    } else {
                        self.add_token(TokenType::OutputRedirect);
                    }
                }
                '-' => {
                    let next = self.peek();
                    if next.is_some_and(|c| c.is_ascii_alphabetic() || c == '-') {
                        self.parse_word();
                    } else if next.is_some_and(|c| c.is_numeric()) {
                        self.parse_number()
                    } else {
                        self.add_token(TokenType::Minus)
                    }
                }
                default => {
                    if default.is_numeric() {
                        self.parse_number()
                    } else if default.is_ascii_alphabetic() {
                        self.parse_word()
                    } else {
                        self.emit_error(&format!("invalid character: \'{}\'", default));
                    }
                }
            }
        }
    }
    fn emit_error(&mut self, message: &str) {
        self.had_error = true;
        let space = " ".repeat(self.current - 1);
        eprint!("{}", self.source);
        eprintln!("{}\x1b[;31m^{}\x1b[;37m", space, message);
    }
    pub fn next_char(&mut self) -> Option<char> {
        let ret = self.source.chars().nth(self.current);
        self.increment_current();
        ret
    }
    pub fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.current)
    }
    pub fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }
    fn parse_word(&mut self) {
        while self.peek().is_some_and(|c| !c.is_whitespace()) {
            self.increment_current()
        }
        if self.source[self.start..self.current].contains('*') {
            self.add_token(TokenType::GlobbedWord(
                self.source[self.start..self.current].to_string(),
            ));
        } else {
            self.add_token(TokenType::Word(
                self.source[self.start..self.current].to_string(),
            ));
        }
    }
    fn parse_number(&mut self) {
        while self.peek().is_some_and(|c| c.is_numeric()) {
            self.increment_current()
        }

        if self
            .peek()
            .is_some_and(|c| c == '.' && self.peek_next().is_some_and(|c| c.is_numeric()))
        {
            self.increment_current();
            while self.peek().is_some_and(|c| c.is_numeric()) {
                self.increment_current()
            }
            let num = self.source[self.start..self.current]
                .parse::<f32>()
                .unwrap_or(0.0);

            self.add_token(TokenType::Float(num));
        } else {
            let num: i64 = self.source[self.start..self.current].parse().unwrap_or(0);
            self.add_token(TokenType::Integer(num));
        }
    }
    fn add_token(&mut self, tok_type: TokenType) {
        self.tokens.push(Token::new(tok_type));
    }
    fn increment_current(&mut self) {
        self.current = self.current + 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn get_char() {
        let mut scan = Scanner::new("abcdef".to_string());
        assert_eq!(scan.next_char(), Some('a'));
        assert_eq!(scan.next_char(), Some('b'));
        assert_eq!(scan.next_char(), Some('c'));
        assert_eq!(scan.next_char(), Some('d'));
        assert_eq!(scan.next_char(), Some('e'));
        assert_eq!(scan.next_char(), Some('f'));
    }
    #[test]
    fn single_char_tokens() {
        let scan = Scanner::new("(".to_string());
        let expected = vec![Token::new(TokenType::LeftParen), Token::new(TokenType::EOF)];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("()()=".to_string());
        let expected = vec![
            Token::new(TokenType::LeftParen),
            Token::new(TokenType::RightParen),
            Token::new(TokenType::LeftParen),
            Token::new(TokenType::RightParen),
            Token::new(TokenType::Equal),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());
    }
    #[test]
    fn two_char_tokens() {
        let scan = Scanner::new("==".to_string());
        let expected = vec![
            Token::new(TokenType::EqualEqual),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());
    }

    #[test]
    fn float() {
        let scan = Scanner::new("1.23".to_string());
        let expected = vec![
            Token::new(TokenType::Float(1.23)),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("1.23 0.5 0.75 0.111".to_string());
        let expected = vec![
            Token::new(TokenType::Float(1.23)),
            Token::new(TokenType::Float(0.5)),
            Token::new(TokenType::Float(0.75)),
            Token::new(TokenType::Float(0.111)),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());
    }

    #[test]
    fn integer() {
        let scan = Scanner::new("123".to_string());
        let expected = vec![
            Token::new(TokenType::Integer(123)),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("1 2".to_string());
        let expected = vec![
            Token::new(TokenType::Integer(1)),
            Token::new(TokenType::Integer(2)),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("1 2 3 4 567".to_string());
        let expected = vec![
            Token::new(TokenType::Integer(1)),
            Token::new(TokenType::Integer(2)),
            Token::new(TokenType::Integer(3)),
            Token::new(TokenType::Integer(4)),
            Token::new(TokenType::Integer(567)),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());
    }
    #[test]
    fn commands() {
        let scan = Scanner::new("cd ..".to_string());
        let expected = vec![
            Token::new(TokenType::Word("cd".to_string())),
            Token::new(TokenType::DotDot),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("ls -a | grep file.txt".to_string());
        let expected = vec![
            Token::new(TokenType::Word("ls".to_string())),
            Token::new(TokenType::Word("-a".to_string())),
            Token::new(TokenType::Pipe),
            Token::new(TokenType::Word("grep".to_string())),
            Token::new(TokenType::Word("file.txt".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("ls *.csv | grep mnist".to_string());
        let expected = vec![
            Token::new(TokenType::Word("ls".to_string())),
            Token::new(TokenType::GlobbedWord("*.csv".to_string())),
            Token::new(TokenType::Pipe),
            Token::new(TokenType::Word("grep".to_string())),
            Token::new(TokenType::Word("mnist".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());
    }

    #[test]
    fn expression() {
        let scan = Scanner::new("123 < 10".to_string());
        let expected = vec![
            Token::new(TokenType::Integer(123)),
            Token::new(TokenType::InputRedirect),
            Token::new(TokenType::Integer(10)),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("123 < 10 && 20 < 30".to_string());
        let expected = vec![
            Token::new(TokenType::Integer(123)),
            Token::new(TokenType::InputRedirect),
            Token::new(TokenType::Integer(10)),
            Token::new(TokenType::DoubleAmpersand),
            Token::new(TokenType::Integer(20)),
            Token::new(TokenType::InputRedirect),
            Token::new(TokenType::Integer(30)),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("123 < 10 && 20 < 30".to_string());
        let expected = vec![
            Token::new(TokenType::Integer(123)),
            Token::new(TokenType::InputRedirect),
            Token::new(TokenType::Integer(10)),
            Token::new(TokenType::DoubleAmpersand),
            Token::new(TokenType::Integer(20)),
            Token::new(TokenType::InputRedirect),
            Token::new(TokenType::Integer(30)),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());
    }
    #[test]
    fn words() {
        let scan = Scanner::new("abc abc abc".to_string());
        let expected = vec![
            Token::new(TokenType::Word("abc".to_string())),
            Token::new(TokenType::Word("abc".to_string())),
            Token::new(TokenType::Word("abc".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());
        let scan = Scanner::new("a_bc abc abc".to_string());
        let expected = vec![
            Token::new(TokenType::Word("a_bc".to_string())),
            Token::new(TokenType::Word("abc".to_string())),
            Token::new(TokenType::Word("abc".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("a_bc a123_bc abc".to_string());
        let expected = vec![
            Token::new(TokenType::Word("a_bc".to_string())),
            Token::new(TokenType::Word("a123_bc".to_string())),
            Token::new(TokenType::Word("abc".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("x y z a b c".to_string());
        let expected = vec![
            Token::new(TokenType::Word("x".to_string())),
            Token::new(TokenType::Word("y".to_string())),
            Token::new(TokenType::Word("z".to_string())),
            Token::new(TokenType::Word("a".to_string())),
            Token::new(TokenType::Word("b".to_string())),
            Token::new(TokenType::Word("c".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("x y z a b c".to_string());
        let expected = vec![
            Token::new(TokenType::Word("x".to_string())),
            Token::new(TokenType::Word("y".to_string())),
            Token::new(TokenType::Word("z".to_string())),
            Token::new(TokenType::Word("a".to_string())),
            Token::new(TokenType::Word("b".to_string())),
            Token::new(TokenType::Word("c".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("\"a\"".to_string());
        let expected = vec![
            Token::new(TokenType::DoubleQuotedString("a".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("\"\"".to_string());
        let expected = vec![
            Token::new(TokenType::DoubleQuotedString("".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("\"a big boy\"".to_string());
        let expected = vec![
            Token::new(TokenType::DoubleQuotedString("a big boy".to_string())),
            Token::new(TokenType::EOF),
        ];
        assert_eq!(Some(expected), scan.get_tokens());

        let scan = Scanner::new("'a big boy".to_string());
        assert_eq!(None, scan.get_tokens());
    }
}
